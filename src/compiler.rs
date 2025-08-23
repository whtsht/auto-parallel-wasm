use anyhow::{Result, anyhow};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, GlobalValue, IntValue, PointerValue,
};
use inkwell::{FloatPredicate, IntPredicate, OptimizationLevel};
use wasmparser::{Operator, ValType};

use crate::wasm_parser::{Function, WasmModule};

extern "C" fn putchar_wrapper(c: i32) -> i32 {
    let byte = (c & 0xFF) as u8;
    print!("{}", byte as char);
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    c
}

#[derive(Clone)]
struct ControlBlock<'ctx> {
    block_type: ControlBlockType,
    end_block: BasicBlock<'ctx>,
    continue_block: Option<BasicBlock<'ctx>>,
}

#[derive(Clone)]
enum ControlBlockType {
    Block,
    Loop,
    If,
}

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    memory: Option<GlobalValue<'ctx>>,
    globals: Vec<(GlobalValue<'ctx>, ValType)>,
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
            memory: None,
            globals: Vec::new(),
        })
    }

    pub fn compile_module(&mut self, wasm_module: &WasmModule) -> Result<()> {
        if !wasm_module.memories.is_empty() {
            self.create_memory(&wasm_module.memories[0])?;
        }

        self.create_globals(&wasm_module.globals)?;

        if wasm_module.has_putchar_import {
            self.declare_putchar();
        }

        for function in &wasm_module.functions {
            self.compile_function(function)?;
        }

        if let Some(start_idx) = wasm_module.start_func_idx {
            self.create_main(start_idx)?;
        }

        Ok(())
    }

    fn build_binary_arithmetic_op<F>(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        op_builder: F,
    ) -> Result<()>
    where
        F: FnOnce(IntValue<'ctx>, IntValue<'ctx>) -> IntValue<'ctx>,
    {
        let rhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_int_value();
        let lhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_int_value();
        let result = op_builder(lhs, rhs);
        value_stack.push(result.into());
        Ok(())
    }

    fn build_comparison_op(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        predicate: IntPredicate,
        op_name: &str,
    ) -> Result<()> {
        let rhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_int_value();
        let lhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_int_value();
        let result = self
            .builder
            .build_int_compare(predicate, lhs, rhs, op_name)
            .unwrap();
        let extended = self
            .builder
            .build_int_z_extend(result, self.context.i32_type(), &format!("{op_name}_ext"))
            .unwrap();
        value_stack.push(extended.into());
        Ok(())
    }

    fn build_binary_float_arithmetic_op<F>(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        op_builder: F,
    ) -> Result<()>
    where
        F: FnOnce(FloatValue<'ctx>, FloatValue<'ctx>) -> FloatValue<'ctx>,
    {
        let rhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_float_value();
        let lhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_float_value();
        let result = op_builder(lhs, rhs);
        value_stack.push(result.into());
        Ok(())
    }

    fn build_float_comparison_op(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        predicate: FloatPredicate,
        op_name: &str,
    ) -> Result<()> {
        let rhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_float_value();
        let lhs = value_stack
            .pop()
            .ok_or(anyhow!("Stack underflow"))?
            .into_float_value();
        let result = self
            .builder
            .build_float_compare(predicate, lhs, rhs, op_name)
            .unwrap();
        let extended = self
            .builder
            .build_int_z_extend(result, self.context.i32_type(), &format!("{op_name}_ext"))
            .unwrap();
        value_stack.push(extended.into());
        Ok(())
    }

    fn pop_single_value(
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>> {
        value_stack.pop().ok_or(anyhow!("Stack underflow"))
    }

    fn build_load_op(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        load_type: BasicTypeEnum<'ctx>,
        memarg: &wasmparser::MemArg,
    ) -> Result<()> {
        let offset = Self::pop_single_value(value_stack)?.into_int_value();
        let ptr = self.get_memory_ptr(offset, memarg.offset)?;
        let value = self.builder.build_load(load_type, ptr, "load").unwrap();
        value_stack.push(value);
        Ok(())
    }

    fn build_store_op(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        memarg: &wasmparser::MemArg,
    ) -> Result<()> {
        let value = Self::pop_single_value(value_stack)?;
        let offset = Self::pop_single_value(value_stack)?.into_int_value();
        let ptr = self.get_memory_ptr(offset, memarg.offset)?;
        self.builder.build_store(ptr, value).unwrap();
        Ok(())
    }

    fn build_partial_load_op(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        load_type: BasicTypeEnum<'ctx>,
        extend_type: BasicTypeEnum<'ctx>,
        signed: bool,
        memarg: &wasmparser::MemArg,
    ) -> Result<()> {
        let offset = Self::pop_single_value(value_stack)?.into_int_value();
        let ptr = self.get_memory_ptr(offset, memarg.offset)?;
        let loaded_value = self.builder.build_load(load_type, ptr, "load").unwrap();

        let extended_value = if signed {
            self.builder
                .build_int_s_extend(
                    loaded_value.into_int_value(),
                    extend_type.into_int_type(),
                    "sext",
                )
                .unwrap()
        } else {
            self.builder
                .build_int_z_extend(
                    loaded_value.into_int_value(),
                    extend_type.into_int_type(),
                    "zext",
                )
                .unwrap()
        };

        value_stack.push(extended_value.into());
        Ok(())
    }

    fn build_partial_store_op(
        &self,
        value_stack: &mut Vec<BasicValueEnum<'ctx>>,
        store_type: BasicTypeEnum<'ctx>,
        memarg: &wasmparser::MemArg,
    ) -> Result<()> {
        let value = Self::pop_single_value(value_stack)?;
        let offset = Self::pop_single_value(value_stack)?.into_int_value();
        let ptr = self.get_memory_ptr(offset, memarg.offset)?;

        let truncated_value = self
            .builder
            .build_int_truncate(value.into_int_value(), store_type.into_int_type(), "trunc")
            .unwrap();

        self.builder.build_store(ptr, truncated_value).unwrap();
        Ok(())
    }

    fn get_branch_target(
        &self,
        control_stack: &[ControlBlock<'ctx>],
        relative_depth: u32,
    ) -> Option<BasicBlock<'ctx>> {
        let depth = relative_depth as usize;
        if depth < control_stack.len() {
            let target_idx = control_stack.len() - 1 - depth;
            let target_block = &control_stack[target_idx];
            Some(match target_block.block_type {
                ControlBlockType::Loop => target_block.continue_block.unwrap(),
                _ => target_block.end_block,
            })
        } else {
            None
        }
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
        let mut control_stack: Vec<ControlBlock<'ctx>> = Vec::new();

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
                ValType::F32 => self
                    .builder
                    .build_alloca(self.context.f32_type(), "local")
                    .unwrap(),
                ValType::F64 => self
                    .builder
                    .build_alloca(self.context.f64_type(), "local")
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
                Operator::F32Const { value } => {
                    let f32_value = f32::from_bits(value.bits());
                    value_stack.push(self.context.f32_type().const_float(f32_value as f64).into());
                }
                Operator::F64Const { value } => {
                    let f64_value = f64::from_bits(value.bits());
                    value_stack.push(self.context.f64_type().const_float(f64_value).into());
                }
                Operator::I64Add => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_add(lhs, rhs, "add64").unwrap()
                    })?;
                }
                Operator::I64Sub => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_sub(lhs, rhs, "sub64").unwrap()
                    })?;
                }
                Operator::I64Mul => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_mul(lhs, rhs, "mul64").unwrap()
                    })?;
                }
                Operator::I64DivS => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_int_signed_div(lhs, rhs, "div64")
                            .unwrap()
                    })?;
                }
                Operator::I64DivU => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_int_unsigned_div(lhs, rhs, "divu64")
                            .unwrap()
                    })?;
                }
                Operator::I64RemS => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_int_signed_rem(lhs, rhs, "rem64")
                            .unwrap()
                    })?;
                }
                Operator::I64RemU => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_int_unsigned_rem(lhs, rhs, "remu64")
                            .unwrap()
                    })?;
                }
                Operator::I64Eq => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::EQ, "eq64")?;
                }
                Operator::I64Ne => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::NE, "ne64")?;
                }
                Operator::I64Eqz => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let zero = self.context.i64_type().const_zero();
                    let result = self
                        .builder
                        .build_int_compare(IntPredicate::EQ, value, zero, "eqz64")
                        .unwrap();
                    let extended = self
                        .builder
                        .build_int_z_extend(result, self.context.i32_type(), "eqz64_ext")
                        .unwrap();
                    value_stack.push(extended.into());
                }
                Operator::I64LtS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SLT, "lt64")?;
                }
                Operator::I64LtU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::ULT, "ltu64")?;
                }
                Operator::I64LeS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SLE, "le64")?;
                }
                Operator::I64LeU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::ULE, "leu64")?;
                }
                Operator::I64GtS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SGT, "gt64")?;
                }
                Operator::I64GtU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::UGT, "gtu64")?;
                }
                Operator::I64GeS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SGE, "ge64")?;
                }
                Operator::I64GeU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::UGE, "geu64")?;
                }
                Operator::I64And => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_and(lhs, rhs, "and64").unwrap()
                    })?;
                }
                Operator::I64Or => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_or(lhs, rhs, "or64").unwrap()
                    })?;
                }
                Operator::I64Xor => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_xor(lhs, rhs, "xor64").unwrap()
                    })?;
                }
                Operator::I64Shl => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_left_shift(lhs, rhs, "shl64").unwrap()
                    })?;
                }
                Operator::I64ShrS => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_right_shift(lhs, rhs, true, "shr_s64")
                            .unwrap()
                    })?;
                }
                Operator::I64ShrU => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_right_shift(lhs, rhs, false, "shr_u64")
                            .unwrap()
                    })?;
                }
                Operator::I64Rotl => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let i64_type = self.context.i64_type();
                    let bits = i64_type.const_int(64, false);
                    let masked_rhs = self
                        .builder
                        .build_int_unsigned_rem(rhs, bits, "rotl64_mask")
                        .unwrap();
                    let shl = self
                        .builder
                        .build_left_shift(lhs, masked_rhs, "rotl64_shl")
                        .unwrap();
                    let inv_shift = self
                        .builder
                        .build_int_sub(bits, masked_rhs, "rotl64_inv")
                        .unwrap();
                    let shr = self
                        .builder
                        .build_right_shift(lhs, inv_shift, false, "rotl64_shr")
                        .unwrap();
                    let result = self.builder.build_or(shl, shr, "rotl64").unwrap();
                    value_stack.push(result.into());
                }
                Operator::I64Rotr => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let i64_type = self.context.i64_type();
                    let bits = i64_type.const_int(64, false);
                    let masked_rhs = self
                        .builder
                        .build_int_unsigned_rem(rhs, bits, "rotr64_mask")
                        .unwrap();
                    let shr = self
                        .builder
                        .build_right_shift(lhs, masked_rhs, false, "rotr64_shr")
                        .unwrap();
                    let inv_shift = self
                        .builder
                        .build_int_sub(bits, masked_rhs, "rotr64_inv")
                        .unwrap();
                    let shl = self
                        .builder
                        .build_left_shift(lhs, inv_shift, "rotr64_shl")
                        .unwrap();
                    let result = self.builder.build_or(shr, shl, "rotr64").unwrap();
                    value_stack.push(result.into());
                }
                Operator::F32Add => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_add(lhs, rhs, "fadd32").unwrap()
                    })?;
                }
                Operator::F32Sub => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_sub(lhs, rhs, "fsub32").unwrap()
                    })?;
                }
                Operator::F32Mul => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_mul(lhs, rhs, "fmul32").unwrap()
                    })?;
                }
                Operator::F32Div => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_div(lhs, rhs, "fdiv32").unwrap()
                    })?;
                }
                Operator::F32Eq => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OEQ, "feq32")?;
                }
                Operator::F32Ne => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::ONE, "fne32")?;
                }
                Operator::F32Lt => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OLT, "flt32")?;
                }
                Operator::F32Gt => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OGT, "fgt32")?;
                }
                Operator::F32Le => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OLE, "fle32")?;
                }
                Operator::F32Ge => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OGE, "fge32")?;
                }
                Operator::F64Add => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_add(lhs, rhs, "fadd64").unwrap()
                    })?;
                }
                Operator::F64Sub => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_sub(lhs, rhs, "fsub64").unwrap()
                    })?;
                }
                Operator::F64Mul => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_mul(lhs, rhs, "fmul64").unwrap()
                    })?;
                }
                Operator::F64Div => {
                    self.build_binary_float_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_float_div(lhs, rhs, "fdiv64").unwrap()
                    })?;
                }
                Operator::F64Eq => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OEQ, "feq64")?;
                }
                Operator::F64Ne => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::ONE, "fne64")?;
                }
                Operator::F64Lt => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OLT, "flt64")?;
                }
                Operator::F64Gt => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OGT, "fgt64")?;
                }
                Operator::F64Le => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OLE, "fle64")?;
                }
                Operator::F64Ge => {
                    self.build_float_comparison_op(&mut value_stack, FloatPredicate::OGE, "fge64")?;
                }
                Operator::I32Add => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_add(lhs, rhs, "add").unwrap()
                    })?;
                }
                Operator::I32Sub => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_sub(lhs, rhs, "sub").unwrap()
                    })?;
                }
                Operator::I32Mul => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_mul(lhs, rhs, "mul").unwrap()
                    })?;
                }
                Operator::I32DivS => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_signed_div(lhs, rhs, "div").unwrap()
                    })?;
                }
                Operator::I32DivU => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_int_unsigned_div(lhs, rhs, "divu")
                            .unwrap()
                    })?;
                }
                Operator::I32RemS => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_int_signed_rem(lhs, rhs, "rem").unwrap()
                    })?;
                }
                Operator::I32RemU => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_int_unsigned_rem(lhs, rhs, "remu")
                            .unwrap()
                    })?;
                }
                Operator::I32LtS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SLT, "lt")?;
                }
                Operator::I32LtU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::ULT, "ltu")?;
                }
                Operator::I32LeS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SLE, "le")?;
                }
                Operator::I32LeU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::ULE, "leu")?;
                }
                Operator::I32GtS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SGT, "gt")?;
                }
                Operator::I32GtU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::UGT, "gtu")?;
                }
                Operator::I32GeS => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::SGE, "ge")?;
                }
                Operator::I32GeU => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::UGE, "geu")?;
                }
                Operator::I32Eq => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::EQ, "eq")?;
                }
                Operator::I32Ne => {
                    self.build_comparison_op(&mut value_stack, IntPredicate::NE, "ne")?;
                }
                Operator::I32Eqz => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let zero = self.context.i32_type().const_zero();
                    let result = self
                        .builder
                        .build_int_compare(IntPredicate::EQ, value, zero, "eqz")
                        .unwrap();
                    let extended = self
                        .builder
                        .build_int_z_extend(result, self.context.i32_type(), "eqz_ext")
                        .unwrap();
                    value_stack.push(extended.into());
                }
                Operator::I32And => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_and(lhs, rhs, "and").unwrap()
                    })?;
                }
                Operator::I32Or => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_or(lhs, rhs, "or").unwrap()
                    })?;
                }
                Operator::I32Xor => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_xor(lhs, rhs, "xor").unwrap()
                    })?;
                }
                Operator::I32Shl => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder.build_left_shift(lhs, rhs, "shl").unwrap()
                    })?;
                }
                Operator::I32ShrS => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_right_shift(lhs, rhs, true, "shr_s")
                            .unwrap()
                    })?;
                }
                Operator::I32ShrU => {
                    self.build_binary_arithmetic_op(&mut value_stack, |lhs, rhs| {
                        self.builder
                            .build_right_shift(lhs, rhs, false, "shr_u")
                            .unwrap()
                    })?;
                }
                Operator::I32Rotl => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let i32_type = self.context.i32_type();
                    let bits = i32_type.const_int(32, false);
                    let masked_rhs = self
                        .builder
                        .build_int_unsigned_rem(rhs, bits, "rotl_mask")
                        .unwrap();
                    let shl = self
                        .builder
                        .build_left_shift(lhs, masked_rhs, "rotl_shl")
                        .unwrap();
                    let inv_shift = self
                        .builder
                        .build_int_sub(bits, masked_rhs, "rotl_inv")
                        .unwrap();
                    let shr = self
                        .builder
                        .build_right_shift(lhs, inv_shift, false, "rotl_shr")
                        .unwrap();
                    let result = self.builder.build_or(shl, shr, "rotl").unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32Rotr => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let i32_type = self.context.i32_type();
                    let bits = i32_type.const_int(32, false);
                    let masked_rhs = self
                        .builder
                        .build_int_unsigned_rem(rhs, bits, "rotr_mask")
                        .unwrap();
                    let shr = self
                        .builder
                        .build_right_shift(lhs, masked_rhs, false, "rotr_shr")
                        .unwrap();
                    let inv_shift = self
                        .builder
                        .build_int_sub(bits, masked_rhs, "rotr_inv")
                        .unwrap();
                    let shl = self
                        .builder
                        .build_left_shift(lhs, inv_shift, "rotr_shl")
                        .unwrap();
                    let result = self.builder.build_or(shr, shl, "rotr").unwrap();
                    value_stack.push(result.into());
                }
                Operator::LocalGet { local_index } => {
                    let local_ptr = locals
                        .get(*local_index as usize)
                        .ok_or(anyhow!("Invalid local index: {}", local_index))?;
                    if local_ptr.is_pointer_value() {
                        let ptr = local_ptr.into_pointer_value();
                        let param_count = function.func_type.params().len();
                        if (*local_index as usize) < param_count {
                            let param_type = function.func_type.params()[*local_index as usize];
                            let llvm_type = self.val_type_to_llvm_type(param_type);
                            let loaded = self
                                .builder
                                .build_load(llvm_type, ptr, "local_load")
                                .unwrap();
                            value_stack.push(loaded);
                        } else {
                            let local_idx = *local_index as usize - param_count;
                            let local_type = function.body.locals[local_idx];
                            let llvm_type = self.val_type_to_llvm_type(local_type);
                            let loaded = self
                                .builder
                                .build_load(llvm_type, ptr, "local_load")
                                .unwrap();
                            value_stack.push(loaded);
                        }
                    } else {
                        value_stack.push(*local_ptr);
                    }
                }
                Operator::LocalSet { local_index } => {
                    let value = Self::pop_single_value(&mut value_stack)?;
                    let local_ptr = locals
                        .get(*local_index as usize)
                        .ok_or(anyhow!("Invalid local index: {}", local_index))?;
                    if local_ptr.is_pointer_value() {
                        let ptr = local_ptr.into_pointer_value();
                        self.builder.build_store(ptr, value).unwrap();
                    } else {
                        return Err(anyhow!("Cannot set non-pointer local"));
                    }
                }
                Operator::LocalTee { local_index } => {
                    let value = Self::pop_single_value(&mut value_stack)?;
                    let local_ptr = locals
                        .get(*local_index as usize)
                        .ok_or(anyhow!("Invalid local index: {}", local_index))?;
                    if local_ptr.is_pointer_value() {
                        let ptr = local_ptr.into_pointer_value();
                        self.builder.build_store(ptr, value).unwrap();
                        value_stack.push(value);
                    } else {
                        return Err(anyhow!("Cannot tee non-pointer local"));
                    }
                }
                Operator::GlobalGet { global_index } => {
                    let (global_var, val_type) = self
                        .globals
                        .get(*global_index as usize)
                        .ok_or(anyhow!("Invalid global index: {}", global_index))?;
                    let llvm_type = self.val_type_to_llvm_type(*val_type);
                    let loaded = self
                        .builder
                        .build_load(llvm_type, global_var.as_pointer_value(), "global_load")
                        .unwrap();
                    value_stack.push(loaded);
                }
                Operator::GlobalSet { global_index } => {
                    let value = Self::pop_single_value(&mut value_stack)?;
                    let (global_var, _val_type) = self
                        .globals
                        .get(*global_index as usize)
                        .ok_or(anyhow!("Invalid global index: {}", global_index))?;
                    self.builder
                        .build_store(global_var.as_pointer_value(), value)
                        .unwrap();
                }
                Operator::Return => {
                    if function.func_type.results().is_empty() {
                        self.builder.build_return(None).unwrap();
                    } else {
                        let return_value = Self::pop_single_value(&mut value_stack)?;
                        self.builder.build_return(Some(&return_value)).unwrap();
                    }
                    return Ok(llvm_func);
                }
                Operator::Drop => {
                    Self::pop_single_value(&mut value_stack)?;
                }
                Operator::I32Load { memarg } => {
                    self.build_load_op(&mut value_stack, self.context.i32_type().into(), memarg)?;
                }
                Operator::I32Store { memarg } => {
                    self.build_store_op(&mut value_stack, memarg)?;
                }
                Operator::I32Store8 { memarg } => {
                    let value = Self::pop_single_value(&mut value_stack)?;
                    let offset = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let ptr = self.get_memory_ptr(offset, memarg.offset)?;
                    let value_i8 = self
                        .builder
                        .build_int_truncate(
                            value.into_int_value(),
                            self.context.i8_type(),
                            "trunc_i8",
                        )
                        .unwrap();
                    self.builder.build_store(ptr, value_i8).unwrap();
                }
                Operator::I64Load { memarg } => {
                    self.build_load_op(&mut value_stack, self.context.i64_type().into(), memarg)?;
                }
                Operator::I64Store { memarg } => {
                    self.build_store_op(&mut value_stack, memarg)?;
                }
                Operator::F32Load { memarg } => {
                    self.build_load_op(&mut value_stack, self.context.f32_type().into(), memarg)?;
                }
                Operator::F64Load { memarg } => {
                    self.build_load_op(&mut value_stack, self.context.f64_type().into(), memarg)?;
                }
                Operator::F32Store { memarg } => {
                    self.build_store_op(&mut value_stack, memarg)?;
                }
                Operator::F64Store { memarg } => {
                    self.build_store_op(&mut value_stack, memarg)?;
                }
                Operator::I32Load8S { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i8_type().into(),
                        self.context.i32_type().into(),
                        true,
                        memarg,
                    )?;
                }
                Operator::I32Load8U { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i8_type().into(),
                        self.context.i32_type().into(),
                        false,
                        memarg,
                    )?;
                }
                Operator::I32Load16S { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i16_type().into(),
                        self.context.i32_type().into(),
                        true,
                        memarg,
                    )?;
                }
                Operator::I32Load16U { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i16_type().into(),
                        self.context.i32_type().into(),
                        false,
                        memarg,
                    )?;
                }
                Operator::I32Store16 { memarg } => {
                    self.build_partial_store_op(
                        &mut value_stack,
                        self.context.i16_type().into(),
                        memarg,
                    )?;
                }
                Operator::I64Load8S { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                        true,
                        memarg,
                    )?;
                }
                Operator::I64Load8U { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                        false,
                        memarg,
                    )?;
                }
                Operator::I64Load16S { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i16_type().into(),
                        self.context.i64_type().into(),
                        true,
                        memarg,
                    )?;
                }
                Operator::I64Load16U { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i16_type().into(),
                        self.context.i64_type().into(),
                        false,
                        memarg,
                    )?;
                }
                Operator::I64Load32S { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i32_type().into(),
                        self.context.i64_type().into(),
                        true,
                        memarg,
                    )?;
                }
                Operator::I64Load32U { memarg } => {
                    self.build_partial_load_op(
                        &mut value_stack,
                        self.context.i32_type().into(),
                        self.context.i64_type().into(),
                        false,
                        memarg,
                    )?;
                }
                Operator::I64Store8 { memarg } => {
                    self.build_partial_store_op(
                        &mut value_stack,
                        self.context.i8_type().into(),
                        memarg,
                    )?;
                }
                Operator::I64Store16 { memarg } => {
                    self.build_partial_store_op(
                        &mut value_stack,
                        self.context.i16_type().into(),
                        memarg,
                    )?;
                }
                Operator::I64Store32 { memarg } => {
                    self.build_partial_store_op(
                        &mut value_stack,
                        self.context.i32_type().into(),
                        memarg,
                    )?;
                }
                Operator::MemorySize { .. } => {
                    let pages = self.get_memory_size()?;
                    value_stack.push(pages.into());
                }
                Operator::MemoryGrow { .. } => {
                    let delta = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self.grow_memory(delta)?;
                    value_stack.push(result.into());
                }
                Operator::Call { function_index } => {
                    let import_count = if self.module.get_function("putchar").is_some() {
                        1
                    } else {
                        0
                    };

                    if (*function_index as usize) < import_count {
                        if *function_index == 0 && self.module.get_function("putchar").is_some() {
                            let arg = Self::pop_single_value(&mut value_stack)?;
                            if let Some(putchar_func) = self.module.get_function("putchar") {
                                let call_result = self
                                    .builder
                                    .build_call(putchar_func, &[arg.into()], "putchar")
                                    .unwrap();
                                if putchar_func.get_type().get_return_type().is_some() {
                                    value_stack
                                        .push(call_result.try_as_basic_value().left().unwrap());
                                }
                            }
                        }
                    } else {
                        let adjusted_index = function_index - import_count as u32;
                        let func_name = format!("func_{adjusted_index}");
                        if let Some(func) = self.module.get_function(&func_name) {
                            let param_count = func.get_type().get_param_types().len();
                            let mut args = Vec::new();
                            for _ in 0..param_count {
                                let arg = Self::pop_single_value(&mut value_stack)?;
                                args.push(arg.into());
                            }
                            args.reverse();
                            let call_result = self.builder.build_call(func, &args, "call").unwrap();
                            if func.get_type().get_return_type().is_some() {
                                value_stack.push(call_result.try_as_basic_value().left().unwrap());
                            }
                        } else {
                            return Err(anyhow!("Unknown function: {}", func_name));
                        }
                    }
                }
                Operator::If { .. } => {
                    let condition = value_stack
                        .pop()
                        .ok_or(anyhow!("Stack underflow for if condition"))?
                        .into_int_value();

                    let then_block = self.context.append_basic_block(llvm_func, "if_then");
                    let else_block = self.context.append_basic_block(llvm_func, "if_else");
                    let merge_block = self.context.append_basic_block(llvm_func, "if_merge");

                    let zero = self.context.i32_type().const_zero();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, condition, zero, "if_cond")
                        .unwrap();

                    self.builder
                        .build_conditional_branch(cond, then_block, else_block)
                        .unwrap();

                    control_stack.push(ControlBlock {
                        block_type: ControlBlockType::If,
                        end_block: merge_block,
                        continue_block: Some(else_block),
                    });

                    self.builder.position_at_end(then_block);
                }
                Operator::Else => {
                    if let Some(control_block) = control_stack.last() {
                        if matches!(control_block.block_type, ControlBlockType::If) {
                            self.builder
                                .build_unconditional_branch(control_block.end_block)
                                .unwrap();
                            if let Some(else_block) = control_block.continue_block {
                                self.builder.position_at_end(else_block);
                            }
                        }
                    }
                }
                Operator::Block { .. } => {
                    let block = self.context.append_basic_block(llvm_func, "block");
                    let end_block = self.context.append_basic_block(llvm_func, "block_end");

                    self.builder.build_unconditional_branch(block).unwrap();
                    self.builder.position_at_end(block);

                    control_stack.push(ControlBlock {
                        block_type: ControlBlockType::Block,
                        end_block,
                        continue_block: None,
                    });
                }
                Operator::Loop { .. } => {
                    let loop_header = self.context.append_basic_block(llvm_func, "loop_header");
                    let loop_end = self.context.append_basic_block(llvm_func, "loop_end");

                    self.builder
                        .build_unconditional_branch(loop_header)
                        .unwrap();
                    self.builder.position_at_end(loop_header);

                    control_stack.push(ControlBlock {
                        block_type: ControlBlockType::Loop,
                        end_block: loop_end,
                        continue_block: Some(loop_header),
                    });
                }
                Operator::Br { relative_depth } => {
                    if let Some(branch_target) =
                        self.get_branch_target(&control_stack, *relative_depth)
                    {
                        self.builder
                            .build_unconditional_branch(branch_target)
                            .unwrap();

                        let unreachable_block =
                            self.context.append_basic_block(llvm_func, "unreachable");
                        self.builder.position_at_end(unreachable_block);
                    }
                }
                Operator::BrIf { relative_depth } => {
                    let condition = Self::pop_single_value(&mut value_stack)?.into_int_value();

                    if let Some(branch_target) =
                        self.get_branch_target(&control_stack, *relative_depth)
                    {
                        let continue_block =
                            self.context.append_basic_block(llvm_func, "br_if_continue");

                        let zero = self.context.i32_type().const_zero();
                        let cond = self
                            .builder
                            .build_int_compare(IntPredicate::NE, condition, zero, "br_if_cond")
                            .unwrap();

                        self.builder
                            .build_conditional_branch(cond, branch_target, continue_block)
                            .unwrap();

                        self.builder.position_at_end(continue_block);
                    }
                }
                Operator::I32WrapI64 => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_int_truncate(value, self.context.i32_type(), "wrap_i64")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::I64ExtendI32S => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_int_s_extend(value, self.context.i64_type(), "extend_i32_s")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::I64ExtendI32U => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_int_z_extend(value, self.context.i64_type(), "extend_i32_u")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::F32ConvertI32S => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_signed_int_to_float(value, self.context.f32_type(), "convert_i32_s")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::F32ConvertI32U => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_unsigned_int_to_float(
                            value,
                            self.context.f32_type(),
                            "convert_i32_u",
                        )
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::F64ConvertI32S => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_signed_int_to_float(value, self.context.f64_type(), "convert_i32_s")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::F64ConvertI32U => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let result = self
                        .builder
                        .build_unsigned_int_to_float(
                            value,
                            self.context.f64_type(),
                            "convert_i32_u",
                        )
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32TruncF32S => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self
                        .builder
                        .build_float_to_signed_int(value, self.context.i32_type(), "trunc_f32_s")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32TruncF32U => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self
                        .builder
                        .build_float_to_unsigned_int(value, self.context.i32_type(), "trunc_f32_u")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32TruncF64S => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self
                        .builder
                        .build_float_to_signed_int(value, self.context.i32_type(), "trunc_f64_s")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::I32TruncF64U => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self
                        .builder
                        .build_float_to_unsigned_int(value, self.context.i32_type(), "trunc_f64_u")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::F64PromoteF32 => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self
                        .builder
                        .build_float_ext(value, self.context.f64_type(), "promote_f32")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::F32DemoteF64 => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self
                        .builder
                        .build_float_trunc(value, self.context.f32_type(), "demote_f64")
                        .unwrap();
                    value_stack.push(result.into());
                }
                Operator::End => {
                    if let Some(control_block) = control_stack.pop() {
                        match control_block.block_type {
                            ControlBlockType::If => {
                                self.builder
                                    .build_unconditional_branch(control_block.end_block)
                                    .unwrap();
                                self.builder.position_at_end(control_block.end_block);
                            }
                            ControlBlockType::Block | ControlBlockType::Loop => {
                                self.builder
                                    .build_unconditional_branch(control_block.end_block)
                                    .unwrap();
                                self.builder.position_at_end(control_block.end_block);
                            }
                        }
                    } else if function.func_type.results().is_empty() {
                        self.builder.build_return(None).unwrap();
                    } else {
                        let return_val = Self::pop_single_value(&mut value_stack)?;
                        self.builder.build_return(Some(&return_val)).unwrap();
                    }
                }
                Operator::Select => {
                    let condition = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let val2 = Self::pop_single_value(&mut value_stack)?;
                    let val1 = Self::pop_single_value(&mut value_stack)?;

                    let condition_bool = self
                        .builder
                        .build_int_compare(
                            IntPredicate::NE,
                            condition,
                            self.context.i32_type().const_zero(),
                            "select_cond",
                        )
                        .unwrap();

                    let result = self
                        .builder
                        .build_select(condition_bool, val2, val1, "select")
                        .unwrap();
                    value_stack.push(result);
                }
                Operator::I32Clz => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let clz_fn = self.get_intrinsic_function(
                        "llvm.ctlz.i32",
                        &[
                            self.context.i32_type().into(),
                            self.context.bool_type().into(),
                        ],
                        self.context.i32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(
                            clz_fn,
                            &[value.into(), self.context.bool_type().const_zero().into()],
                            "clz",
                        )
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::I32Ctz => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let ctz_fn = self.get_intrinsic_function(
                        "llvm.cttz.i32",
                        &[
                            self.context.i32_type().into(),
                            self.context.bool_type().into(),
                        ],
                        self.context.i32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(
                            ctz_fn,
                            &[value.into(), self.context.bool_type().const_zero().into()],
                            "ctz",
                        )
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::I32Popcnt => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let popcnt_fn = self.get_intrinsic_function(
                        "llvm.ctpop.i32",
                        &[self.context.i32_type().into()],
                        self.context.i32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(popcnt_fn, &[value.into()], "popcnt")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::I64Clz => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let clz_fn = self.get_intrinsic_function(
                        "llvm.ctlz.i64",
                        &[
                            self.context.i64_type().into(),
                            self.context.bool_type().into(),
                        ],
                        self.context.i64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(
                            clz_fn,
                            &[value.into(), self.context.bool_type().const_zero().into()],
                            "clz",
                        )
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::I64Ctz => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let ctz_fn = self.get_intrinsic_function(
                        "llvm.cttz.i64",
                        &[
                            self.context.i64_type().into(),
                            self.context.bool_type().into(),
                        ],
                        self.context.i64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(
                            ctz_fn,
                            &[value.into(), self.context.bool_type().const_zero().into()],
                            "ctz",
                        )
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::I64Popcnt => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let popcnt_fn = self.get_intrinsic_function(
                        "llvm.ctpop.i64",
                        &[self.context.i64_type().into()],
                        self.context.i64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(popcnt_fn, &[value.into()], "popcnt")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Abs => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let abs_fn = self.get_intrinsic_function(
                        "llvm.fabs.f32",
                        &[self.context.f32_type().into()],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(abs_fn, &[value.into()], "fabs32")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Neg => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self.builder.build_float_neg(value, "neg").unwrap();
                    value_stack.push(result.into());
                }
                Operator::F32Sqrt => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let sqrt_fn = self.get_intrinsic_function(
                        "llvm.sqrt.f32",
                        &[self.context.f32_type().into()],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(sqrt_fn, &[value.into()], "fsqrt32")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Ceil => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let ceil_fn = self.get_intrinsic_function(
                        "llvm.ceil.f32",
                        &[self.context.f32_type().into()],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(ceil_fn, &[value.into()], "fceil32")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Floor => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let floor_fn = self.get_intrinsic_function(
                        "llvm.floor.f32",
                        &[self.context.f32_type().into()],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(floor_fn, &[value.into()], "floor")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Trunc => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let trunc_fn = self.get_intrinsic_function(
                        "llvm.trunc.f32",
                        &[self.context.f32_type().into()],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(trunc_fn, &[value.into()], "trunc")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Nearest => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let nearbyint_fn = self.get_intrinsic_function(
                        "llvm.nearbyint.f32",
                        &[self.context.f32_type().into()],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(nearbyint_fn, &[value.into()], "nearest")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Min => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let minnum_fn = self.get_intrinsic_function(
                        "llvm.minnum.f32",
                        &[
                            self.context.f32_type().into(),
                            self.context.f32_type().into(),
                        ],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(minnum_fn, &[lhs.into(), rhs.into()], "min")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Max => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let maxnum_fn = self.get_intrinsic_function(
                        "llvm.maxnum.f32",
                        &[
                            self.context.f32_type().into(),
                            self.context.f32_type().into(),
                        ],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(maxnum_fn, &[lhs.into(), rhs.into()], "max")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F32Copysign => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let copysign_fn = self.get_intrinsic_function(
                        "llvm.copysign.f32",
                        &[
                            self.context.f32_type().into(),
                            self.context.f32_type().into(),
                        ],
                        self.context.f32_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(copysign_fn, &[lhs.into(), rhs.into()], "copysign")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Abs => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let abs_fn = self.get_intrinsic_function(
                        "llvm.fabs.f64",
                        &[self.context.f64_type().into()],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(abs_fn, &[value.into()], "abs64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Neg => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let result = self.builder.build_float_neg(value, "neg64").unwrap();
                    value_stack.push(result.into());
                }
                Operator::F64Sqrt => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let sqrt_fn = self.get_intrinsic_function(
                        "llvm.sqrt.f64",
                        &[self.context.f64_type().into()],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(sqrt_fn, &[value.into()], "sqrt64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Ceil => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let ceil_fn = self.get_intrinsic_function(
                        "llvm.ceil.f64",
                        &[self.context.f64_type().into()],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(ceil_fn, &[value.into()], "ceil64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Floor => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let floor_fn = self.get_intrinsic_function(
                        "llvm.floor.f64",
                        &[self.context.f64_type().into()],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(floor_fn, &[value.into()], "floor64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Trunc => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let trunc_fn = self.get_intrinsic_function(
                        "llvm.trunc.f64",
                        &[self.context.f64_type().into()],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(trunc_fn, &[value.into()], "trunc64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Nearest => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let nearbyint_fn = self.get_intrinsic_function(
                        "llvm.nearbyint.f64",
                        &[self.context.f64_type().into()],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(nearbyint_fn, &[value.into()], "nearest64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Min => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let minnum_fn = self.get_intrinsic_function(
                        "llvm.minnum.f64",
                        &[
                            self.context.f64_type().into(),
                            self.context.f64_type().into(),
                        ],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(minnum_fn, &[lhs.into(), rhs.into()], "min64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Max => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let maxnum_fn = self.get_intrinsic_function(
                        "llvm.maxnum.f64",
                        &[
                            self.context.f64_type().into(),
                            self.context.f64_type().into(),
                        ],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(maxnum_fn, &[lhs.into(), rhs.into()], "max64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::F64Copysign => {
                    let rhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let lhs = Self::pop_single_value(&mut value_stack)?.into_float_value();
                    let copysign_fn = self.get_intrinsic_function(
                        "llvm.copysign.f64",
                        &[
                            self.context.f64_type().into(),
                            self.context.f64_type().into(),
                        ],
                        self.context.f64_type().into(),
                    )?;
                    let result = self
                        .builder
                        .build_call(copysign_fn, &[lhs.into(), rhs.into()], "copysign64")
                        .unwrap();
                    value_stack.push(result.try_as_basic_value().left().unwrap());
                }
                Operator::MemoryCopy { .. } => {
                    let size = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let src = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let dest = Self::pop_single_value(&mut value_stack)?.into_int_value();

                    self.build_memory_copy(dest, src, size)?;
                }
                Operator::MemoryFill { .. } => {
                    let size = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let dest = Self::pop_single_value(&mut value_stack)?.into_int_value();

                    self.build_memory_fill(dest, value, size)?;
                }
                Operator::RefNull { .. } => {
                    let null_ptr = self
                        .context
                        .ptr_type(inkwell::AddressSpace::default())
                        .const_null();
                    let alloca = self
                        .builder
                        .build_alloca(null_ptr.get_type(), "ref_slot")
                        .unwrap();
                    self.builder.build_store(alloca, null_ptr).unwrap();
                    value_stack.push(null_ptr.into());
                }
                Operator::RefIsNull => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_pointer_value();
                    let null_ptr = value.get_type().const_null();
                    let result = self
                        .builder
                        .build_int_compare(IntPredicate::EQ, value, null_ptr, "ref_is_null")
                        .unwrap();
                    let extended = self
                        .builder
                        .build_int_z_extend(result, self.context.i32_type(), "ref_is_null_ext")
                        .unwrap();
                    let alloca = self
                        .builder
                        .build_alloca(self.context.i32_type(), "result_slot")
                        .unwrap();
                    self.builder.build_store(alloca, extended).unwrap();
                    value_stack.push(extended.into());
                }
                Operator::I32Extend8S => {
                    let value = Self::pop_single_value(&mut value_stack)?.into_int_value();
                    let masked = self
                        .builder
                        .build_and(
                            value,
                            self.context.i32_type().const_int(0xFF, false),
                            "mask_8bit",
                        )
                        .unwrap();
                    let truncated = self
                        .builder
                        .build_int_truncate(masked, self.context.i8_type(), "trunc_i8")
                        .unwrap();
                    let extended = self
                        .builder
                        .build_int_s_extend(truncated, self.context.i32_type(), "extend8s")
                        .unwrap();
                    value_stack.push(extended.into());
                }
                Operator::Unreachable => {
                    self.builder.build_unreachable().unwrap();
                    let unreachable_block = self
                        .context
                        .append_basic_block(llvm_func, "after_unreachable");
                    self.builder.position_at_end(unreachable_block);
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

        let start_func_name = format!("func_{start_func_idx}");
        if let Some(start_func) = self.module.get_function(&start_func_name) {
            self.builder
                .build_call(start_func, &[], "call_start")
                .unwrap();
        } else {
            let start_func_name = "_start";
            if let Some(start_func) = self.module.get_function(start_func_name) {
                self.builder
                    .build_call(start_func, &[], "call_start")
                    .unwrap();
            }
        }

        let exit_code = self.context.i32_type().const_int(0, false);
        self.builder.build_return(Some(&exit_code)).unwrap();

        Ok(())
    }

    fn get_intrinsic_function(
        &self,
        name: &str,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        return_type: BasicTypeEnum<'ctx>,
    ) -> Result<FunctionValue<'ctx>> {
        let fn_type = return_type.fn_type(param_types, false);
        let intrinsic_fn = self.module.add_function(name, fn_type, None);
        Ok(intrinsic_fn)
    }

    fn get_void_intrinsic_function(
        &self,
        name: &str,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
    ) -> Result<FunctionValue<'ctx>> {
        let fn_type = self.context.void_type().fn_type(param_types, false);
        let intrinsic_fn = self.module.add_function(name, fn_type, None);
        Ok(intrinsic_fn)
    }

    fn build_memory_copy(
        &self,
        dest: IntValue<'ctx>,
        src: IntValue<'ctx>,
        size: IntValue<'ctx>,
    ) -> Result<()> {
        let memory = self.memory.ok_or(anyhow!("No memory allocated"))?;
        let memory_ptr = memory.as_pointer_value();

        let current_block = self.builder.get_insert_block().unwrap();
        let function = current_block.get_parent().unwrap();

        let bounds_check_block = self.context.append_basic_block(function, "bounds_check");
        let trap_block = self.context.append_basic_block(function, "trap");
        let continue_block = self.context.append_basic_block(function, "continue");

        let false_val = self.context.bool_type().const_zero();
        self.builder
            .build_conditional_branch(false_val, continue_block, bounds_check_block)
            .unwrap();

        self.builder.position_at_end(bounds_check_block);
        self.builder
            .build_conditional_branch(false_val, trap_block, continue_block)
            .unwrap();

        self.builder.position_at_end(trap_block);
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(continue_block);

        let bounds_check1_block = self.context.append_basic_block(function, "bounds_check1");
        let trap2_block = self.context.append_basic_block(function, "trap2");
        let continue3_block = self.context.append_basic_block(function, "continue3");

        self.builder
            .build_conditional_branch(false_val, continue3_block, bounds_check1_block)
            .unwrap();

        self.builder.position_at_end(bounds_check1_block);
        self.builder
            .build_conditional_branch(false_val, trap2_block, continue3_block)
            .unwrap();

        self.builder.position_at_end(trap2_block);
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(continue3_block);

        let dest_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), memory_ptr, &[dest], "dest_ptr")
                .unwrap()
        };
        let src_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), memory_ptr, &[src], "src_ptr")
                .unwrap()
        };

        let memmove_fn = self.get_void_intrinsic_function(
            "llvm.memmove.p0.p0.i32",
            &[
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context.i32_type().into(),
                self.context.bool_type().into(),
            ],
        )?;

        self.builder
            .build_call(
                memmove_fn,
                &[
                    dest_ptr.into(),
                    src_ptr.into(),
                    size.into(),
                    false_val.into(),
                ],
                "",
            )
            .unwrap();

        Ok(())
    }

    fn build_memory_fill(
        &self,
        dest: IntValue<'ctx>,
        value: IntValue<'ctx>,
        size: IntValue<'ctx>,
    ) -> Result<()> {
        let memory = self.memory.ok_or(anyhow!("No memory allocated"))?;
        let memory_ptr = memory.as_pointer_value();

        let current_block = self.builder.get_insert_block().unwrap();
        let function = current_block.get_parent().unwrap();

        let bounds_check4_block = self.context.append_basic_block(function, "bounds_check4");
        let trap5_block = self.context.append_basic_block(function, "trap5");
        let continue6_block = self.context.append_basic_block(function, "continue6");

        let false_val = self.context.bool_type().const_zero();
        self.builder
            .build_conditional_branch(false_val, continue6_block, bounds_check4_block)
            .unwrap();

        self.builder.position_at_end(bounds_check4_block);
        self.builder
            .build_conditional_branch(false_val, trap5_block, continue6_block)
            .unwrap();

        self.builder.position_at_end(trap5_block);
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(continue6_block);

        let dest_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), memory_ptr, &[dest], "dest_ptr")
                .unwrap()
        };

        let value_i8 = self
            .builder
            .build_int_truncate(value, self.context.i8_type(), "value_i8")
            .unwrap();

        let memset_fn = self.get_void_intrinsic_function(
            "llvm.memset.p0.i32",
            &[
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context.i8_type().into(),
                self.context.i32_type().into(),
                self.context.bool_type().into(),
            ],
        )?;

        self.builder
            .build_call(
                memset_fn,
                &[
                    dest_ptr.into(),
                    value_i8.into(),
                    size.into(),
                    false_val.into(),
                ],
                "",
            )
            .unwrap();

        Ok(())
    }

    fn val_type_to_llvm_type(&self, val_type: ValType) -> BasicTypeEnum<'ctx> {
        match val_type {
            ValType::I32 => self.context.i32_type().into(),
            ValType::I64 => self.context.i64_type().into(),
            ValType::F32 => self.context.f32_type().into(),
            ValType::F64 => self.context.f64_type().into(),
            _ => panic!("Unsupported value type: {val_type:?}"),
        }
    }

    fn create_globals(&mut self, globals: &[crate::wasm_parser::WasmGlobal]) -> Result<()> {
        for (idx, global) in globals.iter().enumerate() {
            let global_type = global.global_type;
            let llvm_type = self.val_type_to_llvm_type(global_type.content_type);

            let global_name = format!("global_{idx}");
            let global_var = self.module.add_global(llvm_type, None, &global_name);

            let initializer = match global_type.content_type {
                ValType::I32 => self.context.i32_type().const_zero().as_basic_value_enum(),
                ValType::I64 => self.context.i64_type().const_zero().as_basic_value_enum(),
                ValType::F32 => self.context.f32_type().const_zero().as_basic_value_enum(),
                ValType::F64 => self.context.f64_type().const_zero().as_basic_value_enum(),
                _ => {
                    return Err(anyhow!(
                        "Unsupported global type: {:?}",
                        global_type.content_type
                    ));
                }
            };
            global_var.set_initializer(&initializer);

            if !global_type.mutable {
                global_var.set_constant(true);
            }

            self.globals.push((global_var, global_type.content_type));
        }
        Ok(())
    }

    fn create_memory(&mut self, memory_type: &wasmparser::MemoryType) -> Result<()> {
        let i8_type = self.context.i8_type();
        let _i32_type = self.context.i32_type();
        let page_size = 65536;
        let initial_size = memory_type.initial * page_size;

        let array_type = i8_type.array_type(initial_size as u32);
        let memory_global = self.module.add_global(array_type, None, "memory");
        let zero_initializer = array_type.const_zero();
        memory_global.set_initializer(&zero_initializer);

        self.memory = Some(memory_global);
        Ok(())
    }

    fn get_memory_ptr(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
        static_offset: u64,
    ) -> Result<PointerValue<'ctx>> {
        let memory = self.memory.ok_or(anyhow!("No memory allocated"))?;
        let i32_type = self.context.i32_type();

        let base_ptr = memory.as_pointer_value();
        let _zero = i32_type.const_zero();
        let offset_with_static = if static_offset > 0 {
            let static_offset_val = i32_type.const_int(static_offset, false);
            self.builder
                .build_int_add(offset, static_offset_val, "offset_sum")
                .unwrap()
        } else {
            offset
        };

        let i8_ptr = self
            .builder
            .build_bit_cast(
                base_ptr,
                self.context.ptr_type(inkwell::AddressSpace::default()),
                "mem_base",
            )
            .unwrap()
            .into_pointer_value();

        let ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    i8_ptr,
                    &[offset_with_static],
                    "mem_ptr",
                )
                .unwrap()
        };
        Ok(ptr)
    }

    fn get_memory_size(&self) -> Result<inkwell::values::IntValue<'ctx>> {
        let memory = self.memory.ok_or(anyhow!("No memory allocated"))?;
        let memory_type = memory.get_value_type().into_array_type();
        let size_in_bytes = memory_type.len();
        let pages = size_in_bytes / 65536;
        Ok(self.context.i32_type().const_int(pages as u64, false))
    }

    fn grow_memory(
        &self,
        _delta: inkwell::values::IntValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>> {
        let neg_one = self.context.i32_type().const_int((-1i64) as u64, true);
        Ok(neg_one)
    }

    fn declare_putchar(&self) {
        let i32_type = self.context.i32_type();
        let putchar_fn_type = i32_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("putchar", putchar_fn_type, None);
    }

    pub fn run_main(&self) -> Result<i32> {
        type MainFunc = unsafe extern "C" fn() -> i32;

        unsafe {
            if let Some(putchar_func) = self.module.get_function("putchar") {
                self.execution_engine
                    .add_global_mapping(&putchar_func, putchar_wrapper as *const () as usize);
            }

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

        let f32_type = compiler.val_type_to_llvm_type(ValType::F32);
        assert!(f32_type.is_float_type());

        let f64_type = compiler.val_type_to_llvm_type(ValType::F64);
        assert!(f64_type.is_float_type());
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
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let module = WasmModule {
            functions: vec![],
            start_func_idx: None,
            memories: vec![],
            has_putchar_import: false,
            globals: vec![],
            tables: vec![],
            function_types: vec![],
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

    #[test]
    fn test_compile_module_with_memory() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        let module = WasmModule {
            functions: vec![],
            start_func_idx: None,
            memories: vec![memory_type],
            has_putchar_import: false,
            globals: vec![],
            tables: vec![],
            function_types: vec![],
        };

        let result = compiler.compile_module(&module);
        assert!(result.is_ok());
        assert!(compiler.memory.is_some());
    }

    #[test]
    fn test_memory_load_store_operations() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };

        let operators = vec![
            Operator::I32Const { value: 0 },
            Operator::I32Const { value: 42 },
            Operator::I32Store { memarg },
            Operator::I32Const { value: 0 },
            Operator::I32Load { memarg },
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_size_operation() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let operators = vec![
            Operator::MemorySize { mem: 0 },
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_eqz_operation() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 0 },
            Operator::I32Eqz,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_unsigned_div_rem() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 20 },
            Operator::I32Const { value: 3 },
            Operator::I32DivU,
            Operator::I32Const { value: 20 },
            Operator::I32Const { value: 3 },
            Operator::I32RemU,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_unsigned_comparisons() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const {
                value: -1i32 as u32 as i32,
            },
            Operator::I32Const { value: 1 },
            Operator::I32LtU,
            Operator::I32Const { value: 1 },
            Operator::I32Const { value: 1 },
            Operator::I32LeU,
            Operator::I32Const {
                value: -1i32 as u32 as i32,
            },
            Operator::I32Const { value: 1 },
            Operator::I32GtU,
            Operator::I32Const { value: 1 },
            Operator::I32Const { value: 1 },
            Operator::I32GeU,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i64_arithmetic() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64Add,
            Operator::I64Const { value: 500 },
            Operator::I64Sub,
            Operator::I64Const { value: 2 },
            Operator::I64Mul,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i64_div_rem() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const { value: 100 },
            Operator::I64Const { value: 3 },
            Operator::I64DivS,
            Operator::I64Const { value: 100 },
            Operator::I64Const { value: 3 },
            Operator::I64DivU,
            Operator::I64Const { value: 100 },
            Operator::I64Const { value: 3 },
            Operator::I64RemS,
            Operator::I64Const { value: 100 },
            Operator::I64Const { value: 3 },
            Operator::I64RemU,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i64_comparisons() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 1000 },
            Operator::I64Eq,
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64Ne,
            Operator::I64Const { value: 0 },
            Operator::I64Eqz,
            Operator::I64Const { value: -1 },
            Operator::I64Const { value: 1 },
            Operator::I64LtS,
            Operator::I64Const {
                value: -1i64 as u64 as i64,
            },
            Operator::I64Const { value: 1 },
            Operator::I64LtU,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bitwise_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 0x0F0F0F0F },
            Operator::I32Const {
                value: 0xF0F0F0F0u32 as i32,
            },
            Operator::I32And,
            Operator::I32Const { value: 0x0F0F0F0F },
            Operator::I32Const {
                value: 0xF0F0F0F0u32 as i32,
            },
            Operator::I32Or,
            Operator::I32Const { value: 0x0F0F0F0F },
            Operator::I32Const {
                value: 0xF0F0F0F0u32 as i32,
            },
            Operator::I32Xor,
            Operator::I64Const {
                value: 0x0F0F0F0F0F0F0F0F,
            },
            Operator::I64Const {
                value: 0xF0F0F0F0F0F0F0F0u64 as i64,
            },
            Operator::I64And,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shift_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 1 },
            Operator::I32Const { value: 4 },
            Operator::I32Shl,
            Operator::I32Const { value: -16 },
            Operator::I32Const { value: 2 },
            Operator::I32ShrS,
            Operator::I32Const { value: -16 },
            Operator::I32Const { value: 2 },
            Operator::I32ShrU,
            Operator::I64Const { value: 1 },
            Operator::I64Const { value: 32 },
            Operator::I64Shl,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rotate_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 0x12345678 },
            Operator::I32Const { value: 4 },
            Operator::I32Rotl,
            Operator::I32Const { value: 0x12345678 },
            Operator::I32Const { value: 4 },
            Operator::I32Rotr,
            Operator::I64Const {
                value: 0x123456789ABCDEF0u64 as i64,
            },
            Operator::I64Const { value: 8 },
            Operator::I64Rotl,
            Operator::I64Const {
                value: 0x123456789ABCDEF0u64 as i64,
            },
            Operator::I64Const { value: 8 },
            Operator::I64Rotr,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_eqz_comprehensive() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 0 },
            Operator::I32Eqz,
            Operator::I32Const { value: 42 },
            Operator::I32Eqz,
            Operator::I32Const { value: -1 },
            Operator::I32Eqz,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i64_eqz_comprehensive() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const { value: 0 },
            Operator::I64Eqz,
            Operator::I64Const {
                value: 9223372036854775807,
            },
            Operator::I64Eqz,
            Operator::I64Const { value: -1 },
            Operator::I64Eqz,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_i32_comparison_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 10 },
            Operator::I32Const { value: 20 },
            Operator::I32Eq,
            Operator::I32Const { value: 10 },
            Operator::I32Const { value: 20 },
            Operator::I32Ne,
            Operator::I32Const { value: 10 },
            Operator::I32Const { value: 20 },
            Operator::I32LtS,
            Operator::I32Const { value: 10 },
            Operator::I32Const { value: 20 },
            Operator::I32LeS,
            Operator::I32Const { value: 20 },
            Operator::I32Const { value: 10 },
            Operator::I32GtS,
            Operator::I32Const { value: 20 },
            Operator::I32Const { value: 10 },
            Operator::I32GeS,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_i64_comparison_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64Eq,
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64Ne,
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64LtS,
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64LeS,
            Operator::I64Const { value: 2000 },
            Operator::I64Const { value: 1000 },
            Operator::I64GtS,
            Operator::I64Const { value: 2000 },
            Operator::I64Const { value: 1000 },
            Operator::I64GeS,
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64LtU,
            Operator::I64Const { value: 1000 },
            Operator::I64Const { value: 2000 },
            Operator::I64LeU,
            Operator::I64Const { value: 2000 },
            Operator::I64Const { value: 1000 },
            Operator::I64GtU,
            Operator::I64Const { value: 2000 },
            Operator::I64Const { value: 1000 },
            Operator::I64GeU,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_i32_bitwise_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 0x0F0F0F0F },
            Operator::I32Const {
                value: 0xF0F0F0F0u32 as i32,
            },
            Operator::I32And,
            Operator::I32Const { value: 0x0F0F0F0F },
            Operator::I32Const {
                value: 0xF0F0F0F0u32 as i32,
            },
            Operator::I32Or,
            Operator::I32Const { value: 0x0F0F0F0F },
            Operator::I32Const {
                value: 0xF0F0F0F0u32 as i32,
            },
            Operator::I32Xor,
            Operator::I32Const { value: 1 },
            Operator::I32Const { value: 4 },
            Operator::I32Shl,
            Operator::I32Const { value: -16 },
            Operator::I32Const { value: 2 },
            Operator::I32ShrS,
            Operator::I32Const { value: -16 },
            Operator::I32Const { value: 2 },
            Operator::I32ShrU,
            Operator::I32Const { value: 0x12345678 },
            Operator::I32Const { value: 4 },
            Operator::I32Rotl,
            Operator::I32Const { value: 0x12345678 },
            Operator::I32Const { value: 4 },
            Operator::I32Rotr,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_i64_bitwise_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const {
                value: 0x0F0F0F0F0F0F0F0F,
            },
            Operator::I64Const {
                value: 0xF0F0F0F0F0F0F0F0u64 as i64,
            },
            Operator::I64And,
            Operator::I64Const {
                value: 0x0F0F0F0F0F0F0F0F,
            },
            Operator::I64Const {
                value: 0xF0F0F0F0F0F0F0F0u64 as i64,
            },
            Operator::I64Or,
            Operator::I64Const {
                value: 0x0F0F0F0F0F0F0F0F,
            },
            Operator::I64Const {
                value: 0xF0F0F0F0F0F0F0F0u64 as i64,
            },
            Operator::I64Xor,
            Operator::I64Const { value: 1 },
            Operator::I64Const { value: 32 },
            Operator::I64Shl,
            Operator::I64Const { value: -1024 },
            Operator::I64Const { value: 2 },
            Operator::I64ShrS,
            Operator::I64Const { value: -1024 },
            Operator::I64Const { value: 2 },
            Operator::I64ShrU,
            Operator::I64Const {
                value: 0x123456789ABCDEF0u64 as i64,
            },
            Operator::I64Const { value: 8 },
            Operator::I64Rotl,
            Operator::I64Const {
                value: 0x123456789ABCDEF0u64 as i64,
            },
            Operator::I64Const { value: 8 },
            Operator::I64Rotr,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_boundary_shift_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 1 },
            Operator::I32Const { value: 32 },
            Operator::I32Shl,
            Operator::I32Const { value: 1 },
            Operator::I32Const { value: 31 },
            Operator::I32Shl,
            Operator::I64Const { value: 1 },
            Operator::I64Const { value: 64 },
            Operator::I64Shl,
            Operator::I64Const { value: 1 },
            Operator::I64Const { value: 63 },
            Operator::I64Shl,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_overflow_arithmetic() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 2147483647 },
            Operator::I32Const { value: 1 },
            Operator::I32Add,
            Operator::I32Const { value: -2147483648 },
            Operator::I32Const { value: 1 },
            Operator::I32Sub,
            Operator::I64Const {
                value: 9223372036854775807,
            },
            Operator::I64Const { value: 1 },
            Operator::I64Add,
            Operator::I64Const {
                value: -9223372036854775808,
            },
            Operator::I64Const { value: 1 },
            Operator::I64Sub,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comprehensive_instruction_coverage() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 100 },
            Operator::I32Const { value: 3 },
            Operator::I32RemS,
            Operator::I32Const { value: 100 },
            Operator::I32Const { value: 3 },
            Operator::I32RemU,
            Operator::I64Const { value: 100 },
            Operator::I64Const { value: 3 },
            Operator::I64RemS,
            Operator::I64Const { value: 100 },
            Operator::I64Const { value: 3 },
            Operator::I64RemU,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_const_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let f32_value = std::f32::consts::PI;
        let f64_value = std::f64::consts::E;

        let operators = vec![
            Operator::F32Const {
                value: f32_value.into(),
            },
            Operator::Drop,
            Operator::F64Const {
                value: f64_value.into(),
            },
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_f32_arithmetic_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let f32_value_1 = 10.5f32;
        let f32_value_2 = 2.5f32;

        let operators = vec![
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Add,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Sub,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Mul,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Div,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_f64_arithmetic_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let f64_value_1 = 10.5f64;
        let f64_value_2 = 2.5f64;

        let operators = vec![
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Add,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Sub,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Mul,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Div,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_f32_comparison_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let f32_value_1 = 10.5f32;
        let f32_value_2 = 2.5f32;

        let operators = vec![
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Eq,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Ne,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Lt,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Gt,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Le,
            Operator::F32Const {
                value: f32_value_1.into(),
            },
            Operator::F32Const {
                value: f32_value_2.into(),
            },
            Operator::F32Ge,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_f64_comparison_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let f64_value_1 = 10.5f64;
        let f64_value_2 = 2.5f64;

        let operators = vec![
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Eq,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Ne,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Lt,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Gt,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Le,
            Operator::F64Const {
                value: f64_value_1.into(),
            },
            Operator::F64Const {
                value: f64_value_2.into(),
            },
            Operator::F64Ge,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_wrap_i64() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const {
                value: 0x123456789ABCDEF0u64 as i64,
            },
            Operator::I32WrapI64,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i64_extend_i32() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: -1 },
            Operator::I64ExtendI32S,
            Operator::I32Const { value: -1 },
            Operator::I64ExtendI32U,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_convert_from_int() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: -42 },
            Operator::F32ConvertI32S,
            Operator::I32Const { value: 42 },
            Operator::F32ConvertI32U,
            Operator::I32Const { value: -42 },
            Operator::F64ConvertI32S,
            Operator::I32Const { value: 42 },
            Operator::F64ConvertI32U,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_int_trunc_from_float() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::F32Const {
                value: (-42.7f32).into(),
            },
            Operator::I32TruncF32S,
            Operator::F32Const {
                value: 42.7f32.into(),
            },
            Operator::I32TruncF32U,
            Operator::F64Const {
                value: (-42.7f64).into(),
            },
            Operator::I32TruncF64S,
            Operator::F64Const {
                value: 42.7f64.into(),
            },
            Operator::I32TruncF64U,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_promotion_demotion() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::F32Const {
                value: std::f32::consts::PI.into(),
            },
            Operator::F64PromoteF32,
            Operator::F64Const {
                value: std::f64::consts::E.into(),
            },
            Operator::F32DemoteF64,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_conversion_boundary_values() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I64Const { value: i64::MAX },
            Operator::I32WrapI64,
            Operator::I32Const { value: i32::MAX },
            Operator::I64ExtendI32S,
            Operator::I32Const { value: i32::MAX },
            Operator::I64ExtendI32U,
            Operator::Drop,
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comprehensive_type_conversions() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 42 },
            Operator::F32ConvertI32S,
            Operator::F64PromoteF32,
            Operator::I32TruncF64S,
            Operator::I64ExtendI32S,
            Operator::I32WrapI64,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_partial_load_operations() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 0,
            memory: 0,
        };

        let test_cases = vec![
            Operator::I32Load8S { memarg },
            Operator::I32Load8U { memarg },
            Operator::I32Load16S { memarg },
            Operator::I32Load16U { memarg },
        ];

        for load_op in test_cases {
            let operators = vec![
                Operator::I32Const { value: 0 },
                load_op,
                Operator::Drop,
                Operator::End,
            ];

            let function = create_simple_function(0, operators);
            let result = compiler.compile_function(&function);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_i32_partial_store_operations() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 0,
            memory: 0,
        };

        let operators = vec![
            Operator::I32Const { value: 0 },
            Operator::I32Const { value: 12345 },
            Operator::I32Store16 { memarg },
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_i64_partial_load_operations() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 0,
            memory: 0,
        };

        let test_cases = vec![
            Operator::I64Load8S { memarg },
            Operator::I64Load8U { memarg },
            Operator::I64Load16S { memarg },
            Operator::I64Load16U { memarg },
            Operator::I64Load32S { memarg },
            Operator::I64Load32U { memarg },
        ];

        for load_op in test_cases {
            let operators = vec![
                Operator::I32Const { value: 0 },
                load_op,
                Operator::Drop,
                Operator::End,
            ];

            let function = create_simple_function(0, operators);
            let result = compiler.compile_function(&function);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_i64_partial_store_operations() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 0,
            memory: 0,
        };

        let test_cases = vec![
            Operator::I64Store8 { memarg },
            Operator::I64Store16 { memarg },
            Operator::I64Store32 { memarg },
        ];

        for store_op in test_cases {
            let operators = vec![
                Operator::I32Const { value: 0 },
                Operator::I64Const {
                    value: 0x123456789ABCDEF0,
                },
                store_op,
                Operator::End,
            ];

            let function = create_simple_function(0, operators);
            let result = compiler.compile_function(&function);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_float_memory_operations() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };

        let operators_f32 = vec![
            Operator::I32Const { value: 0 },
            Operator::F32Load { memarg },
            Operator::Drop,
            Operator::End,
        ];

        let function_f32 = create_simple_function(0, operators_f32);
        let result = compiler.compile_function(&function_f32);
        assert!(result.is_ok());

        let operators_f64 = vec![
            Operator::I32Const { value: 8 },
            Operator::F64Load { memarg },
            Operator::Drop,
            Operator::End,
        ];

        let function_f64 = create_simple_function(1, operators_f64);
        let result = compiler.compile_function(&function_f64);
        assert!(result.is_ok());
    }

    #[test]
    fn test_partial_load_sign_extension() {
        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let memory_type = wasmparser::MemoryType {
            memory64: false,
            shared: false,
            initial: 1,
            maximum: Some(10),
            page_size_log2: None,
        };

        compiler.create_memory(&memory_type).unwrap();

        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 0,
            memory: 0,
        };

        let operators = vec![
            Operator::I32Const { value: 0 },
            Operator::I32Const { value: 0xFF },
            Operator::I32Store8 { memarg },
            Operator::I32Const { value: 0 },
            Operator::I32Load8S { memarg },
            Operator::Drop,
            Operator::I32Const { value: 0 },
            Operator::I32Load8U { memarg },
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_tee_operation() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::I32Const { value: 42 },
            Operator::LocalTee { local_index: 0 },
            Operator::LocalTee { local_index: 1 },
            Operator::LocalGet { local_index: 0 },
            Operator::I32Add,
            Operator::LocalGet { local_index: 1 },
            Operator::I32Add,
            Operator::Drop,
            Operator::End,
        ];

        let mut function = create_simple_function(0, operators);
        function.body.locals = vec![ValType::I32, ValType::I32];

        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_global_operations() {
        use crate::wasm_parser::WasmGlobal;
        use wasmparser::GlobalType;

        let context = Context::create();
        let mut compiler = Compiler::new(&context, "test").unwrap();

        let globals = vec![
            WasmGlobal {
                global_type: GlobalType {
                    content_type: ValType::I32,
                    mutable: true,
                    shared: false,
                },
            },
            WasmGlobal {
                global_type: GlobalType {
                    content_type: ValType::I64,
                    mutable: false,
                    shared: false,
                },
            },
        ];

        let result = compiler.create_globals(&globals);
        assert!(result.is_ok());
        assert_eq!(compiler.globals.len(), 2);

        let operators = vec![
            Operator::I32Const { value: 100 },
            Operator::GlobalSet { global_index: 0 },
            Operator::GlobalGet { global_index: 0 },
            Operator::GlobalGet { global_index: 1 },
            Operator::Drop,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_types_operations() {
        let context = Context::create();
        let compiler = Compiler::new(&context, "test").unwrap();

        let operators = vec![
            Operator::RefNull {
                hty: wasmparser::HeapType::Abstract {
                    shared: false,
                    ty: wasmparser::AbstractHeapType::Extern,
                },
            },
            Operator::RefIsNull,
            Operator::Drop,
            Operator::End,
        ];

        let function = create_simple_function(0, operators);
        let result = compiler.compile_function(&function);
        assert!(result.is_ok());
    }
}
