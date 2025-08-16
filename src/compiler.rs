use anyhow::{Result, anyhow};
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue};
use wasmparser::{Operator, ValType};

use crate::wasm_parser::{Function, WasmModule};

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| anyhow!("Failed to initialize native target: {}", e))?;

        let module = context.create_module(module_name);
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| anyhow!("Failed to create execution engine: {}", e))?;

        Ok(Self {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
        })
    }

    pub fn compile_module(&self, wasm_module: &WasmModule) -> Result<()> {
        for function in &wasm_module.functions {
            self.compile_function(function)?;
        }

        if let Some(start_idx) = wasm_module.start_func_idx {
            self.create_main(start_idx)?;
        }

        Ok(())
    }

    fn compile_function(&self, function: &Function) -> Result<FunctionValue<'ctx>> {
        let param_types: Vec<BasicMetadataTypeEnum> = function
            .func_type
            .params()
            .iter()
            .map(|vt| self.val_type_to_llvm_type(*vt).into())
            .collect();

        let return_type = if function.func_type.results().is_empty() {
            None
        } else if function.func_type.results().len() == 1 {
            Some(self.val_type_to_llvm_type(function.func_type.results()[0]))
        } else {
            return Err(anyhow!("Multiple return values not supported"));
        };

        let fn_type = match return_type {
            Some(ret_type) => ret_type.fn_type(&param_types, false),
            None => self.context.void_type().fn_type(&param_types, false),
        };

        let default_name = format!("func_{}", function.idx);
        let func_name = function.name.as_ref().unwrap_or(&default_name);
        let llvm_func = self.module.add_function(func_name, fn_type, None);

        let entry_block = self.context.append_basic_block(llvm_func, "entry");
        self.builder.position_at_end(entry_block);

        let mut value_stack: Vec<BasicValueEnum<'ctx>> = Vec::new();
        let mut locals: Vec<BasicValueEnum<'ctx>> = Vec::new();

        for (i, _) in function.func_type.params().iter().enumerate() {
            locals.push(llvm_func.get_nth_param(i as u32).unwrap());
        }

        for local_type in &function.body.locals {
            let alloca = match local_type {
                ValType::I32 => self
                    .builder
                    .build_alloca(self.context.i32_type(), "local")
                    .unwrap(),
                ValType::I64 => self
                    .builder
                    .build_alloca(self.context.i64_type(), "local")
                    .unwrap(),
                _ => return Err(anyhow!("Unsupported local type: {:?}", local_type)),
            };
            locals.push(alloca.as_basic_value_enum());
        }

        for operator in &function.body.operators {
            match operator {
                Operator::I32Const { value } => {
                    value_stack.push(
                        self.context
                            .i32_type()
                            .const_int(*value as u64, false)
                            .into(),
                    );
                }
                Operator::I64Const { value } => {
                    value_stack.push(
                        self.context
                            .i64_type()
                            .const_int(*value as u64, false)
                            .into(),
                    );
                }
                Operator::I32Add => {
                    let rhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let lhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let result = self.builder.build_int_add(lhs, rhs, "add").unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32Sub => {
                    let rhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let lhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let result = self.builder.build_int_sub(lhs, rhs, "sub").unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32Mul => {
                    let rhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let lhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let result = self.builder.build_int_mul(lhs, rhs, "mul").unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32DivS => {
                    let rhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let lhs = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow"))?
                        .into_int_value();
                    let result = self.builder.build_int_signed_div(lhs, rhs, "div").unwrap();
                    value_stack.push(result.into());
                }
                Operator::LocalGet { local_index } => {
                    let local_value = locals
                        .get(*local_index as usize)
                        .ok_or(anyhow!("Invalid local index: {}", local_index))?;
                    value_stack.push(*local_value);
                }
                Operator::Return => {
                    if function.func_type.results().is_empty() {
                        self.builder.build_return(None).unwrap();
                    } else {
                        let return_value = value_stack
                            .pop()
                            .ok_or(anyhow!("No return value on stack"))?;
                        self.builder.build_return(Some(&return_value)).unwrap();
                    }
                    return Ok(llvm_func);
                }
                Operator::Drop => {
                    value_stack.pop().ok_or(anyhow!("Stack underflow"))?;
                }
                Operator::End => {
                    if function.func_type.results().is_empty() {
                        self.builder.build_return(None).unwrap();
                    } else {
                        let return_value = value_stack
                            .pop()
                            .ok_or(anyhow!("No return value on stack"))?;
                        self.builder.build_return(Some(&return_value)).unwrap();
                    }
                }
                _ => return Err(anyhow!("Unsupported operator: {:?}", operator)),
            }
        }

        Ok(llvm_func)
    }

    fn create_main(&self, start_func_idx: u32) -> Result<()> {
        let i32_type = self.context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_func = self.module.add_function("main", main_fn_type, None);

        let entry_block = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry_block);

        let start_func_name = format!("func_{}", start_func_idx);
        if let Some(start_func) = self.module.get_function(&start_func_name) {
            self.builder
                .build_call(start_func, &[], "call_start")
                .unwrap();
        }

        let exit_code = self.context.i32_type().const_int(0, false);
        self.builder.build_return(Some(&exit_code)).unwrap();

        Ok(())
    }

    fn val_type_to_llvm_type(&self, val_type: ValType) -> BasicTypeEnum<'ctx> {
        match val_type {
            ValType::I32 => self.context.i32_type().into(),
            ValType::I64 => self.context.i64_type().into(),
            _ => panic!("Unsupported value type: {:?}", val_type),
        }
    }

    pub fn run_main(&self) -> Result<i32> {
        type MainFunc = unsafe extern "C" fn() -> i32;

        unsafe {
            let main_func: inkwell::execution_engine::JitFunction<MainFunc> = self
                .execution_engine
                .get_function("main")
                .map_err(|_| anyhow!("Failed to find main function"))?;

            Ok(main_func.call())
        }
    }

    pub fn print_ir_to_stdout(&self) {
        self.module.print_to_stderr();
    }

    pub fn write_ir_to_file(&self, output_path: &str) -> Result<()> {
        self.module
            .print_to_file(output_path)
            .map_err(|e| anyhow!("Failed to write IR to file: {}", e))?;
        Ok(())
    }

    pub fn write_object_file(&self, output_path: &str) -> Result<()> {
        use inkwell::targets::{CodeModel, FileType, RelocMode, TargetMachine};

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| anyhow!("Failed to get target: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| anyhow!("Failed to create target machine"))?;

        target_machine
            .write_to_file(&self.module, FileType::Object, output_path.as_ref())
            .map_err(|e| anyhow!("Failed to write object file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_parser::{Function, FunctionBody, WasmModule};
    use inkwell::context::Context;
    use wasmparser::{FuncType, ValType};

    fn create_simple_function(idx: u32, operators: Vec<Operator<'static>>) -> Function {
        Function {
            idx,
            name: None,
            func_type: FuncType::new([], []),
            body: FunctionBody {
                locals: vec![],
                operators,
            },
        }
    }

    #[test]
    fn test_compiler_creation() {
        let context = Context::create();
        let result = Compiler::new(&context, "test_module");
        assert!(result.is_ok());
    }

    #[test]
    fn test_val_type_conversion() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let i32_type = compiler.val_type_to_llvm_type(ValType::I32);
        assert!(i32_type.is_int_type());

        let i64_type = compiler.val_type_to_llvm_type(ValType::I64);
        assert!(i64_type.is_int_type());
    }

    #[test]
    fn test_compile_empty_function() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let function = create_simple_function(0, vec![Operator::End]);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_function_with_i32_const() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 42 },
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_function_with_arithmetic() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 10 },
            Operator::I32Const { value: 5 },
            Operator::I32Add,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_function_with_all_arithmetic_ops() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 20 },
            Operator::I32Const { value: 4 },
            Operator::I32Add,
            Operator::I32Const { value: 2 },
            Operator::I32Sub,
            Operator::I32Const { value: 3 },
            Operator::I32Mul,
            Operator::I32Const { value: 6 },
            Operator::I32DivS,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_empty_module() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let module = WasmModule {
            functions: vec![],
            start_func_idx: None,
        };

        let result = compiler.compile_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stack_underflow() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![Operator::I32Add, Operator::End];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_err());
    }
}
