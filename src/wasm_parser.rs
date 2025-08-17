use anyhow::Result;
use wasmparser::{FuncType, GlobalType, MemoryType, Operator, Parser, Payload, TypeRef, ValType};

pub struct WasmModule {
    pub functions: Vec<Function>,
    pub start_func_idx: Option<u32>,
    pub memories: Vec<MemoryType>,
    pub has_putchar_import: bool,
    pub globals: Vec<WasmGlobal>,
}

pub struct WasmGlobal {
    pub global_type: GlobalType,
}

pub struct Function {
    pub idx: u32,
    pub name: Option<String>,
    pub func_type: FuncType,
    pub body: FunctionBody,
}

pub struct FunctionBody {
    pub locals: Vec<ValType>,
    pub operators: Vec<Operator<'static>>,
}

impl WasmModule {
    pub fn parse(wasm_bytes: &[u8]) -> Result<Self> {
        let mut functions = Vec::new();
        let mut start_func_idx = None;
        let mut func_types = Vec::new();
        let mut func_bodies = Vec::new();
        let mut import_count = 0;
        let mut memories = Vec::new();
        let mut has_putchar_import = false;
        let mut globals = Vec::new();

        for payload in Parser::new(0).parse_all(wasm_bytes) {
            match payload? {
                Payload::TypeSection(types) => {
                    for rec_group in types {
                        for sub_type in rec_group?.into_types() {
                            if let wasmparser::CompositeInnerType::Func(func_type) =
                                &sub_type.composite_type.inner
                            {
                                func_types.push(func_type.clone());
                            }
                        }
                    }
                }
                Payload::ImportSection(imports) => {
                    for import in imports {
                        let import = import?;
                        if matches!(import.ty, TypeRef::Func(_)) {
                            import_count += 1;
                            if import.name == "putchar" {
                                has_putchar_import = true;
                            }
                        }
                    }
                }
                Payload::FunctionSection(funcs) => {
                    for (idx, type_idx) in funcs.into_iter().enumerate() {
                        let type_idx = type_idx? as usize;
                        if type_idx < func_types.len() {
                            functions.push(Function {
                                idx: (import_count + idx) as u32,
                                name: None,
                                func_type: func_types[type_idx].clone(),
                                body: FunctionBody {
                                    locals: Vec::new(),
                                    operators: Vec::new(),
                                },
                            });
                        }
                    }
                }
                Payload::StartSection { func, .. } => {
                    start_func_idx = Some(func);
                }
                Payload::MemorySection(memory_section) => {
                    for memory in memory_section {
                        memories.push(memory?);
                    }
                }
                Payload::GlobalSection(global_section) => {
                    for global in global_section {
                        let global = global?;
                        globals.push(WasmGlobal {
                            global_type: global.ty,
                        });
                    }
                }
                Payload::CodeSectionEntry(body) => {
                    func_bodies.push(body);
                }
                _ => {}
            }
        }

        for (idx, body) in func_bodies.into_iter().enumerate() {
            if idx < functions.len() {
                let mut locals = Vec::new();
                let locals_reader = body.get_locals_reader()?;
                for local in locals_reader {
                    let (count, val_type) = local?;
                    for _ in 0..count {
                        locals.push(val_type);
                    }
                }

                let mut operators = Vec::new();
                let operators_reader = body.get_operators_reader()?;
                for op in operators_reader {
                    let op = op?;
                    let owned_op = match op {
                        Operator::I32Const { value } => Operator::I32Const { value },
                        Operator::I64Const { value } => Operator::I64Const { value },
                        Operator::F32Const { value } => Operator::F32Const { value },
                        Operator::F64Const { value } => Operator::F64Const { value },
                        Operator::I64Add => Operator::I64Add,
                        Operator::I64Sub => Operator::I64Sub,
                        Operator::I64Mul => Operator::I64Mul,
                        Operator::I64DivS => Operator::I64DivS,
                        Operator::I64DivU => Operator::I64DivU,
                        Operator::I64RemS => Operator::I64RemS,
                        Operator::I64RemU => Operator::I64RemU,
                        Operator::I64Eq => Operator::I64Eq,
                        Operator::I64Ne => Operator::I64Ne,
                        Operator::I64Eqz => Operator::I64Eqz,
                        Operator::I64LtS => Operator::I64LtS,
                        Operator::I64LtU => Operator::I64LtU,
                        Operator::I64LeS => Operator::I64LeS,
                        Operator::I64LeU => Operator::I64LeU,
                        Operator::I64GtS => Operator::I64GtS,
                        Operator::I64GtU => Operator::I64GtU,
                        Operator::I64GeS => Operator::I64GeS,
                        Operator::I64GeU => Operator::I64GeU,
                        Operator::I64And => Operator::I64And,
                        Operator::I64Or => Operator::I64Or,
                        Operator::I64Xor => Operator::I64Xor,
                        Operator::I64Shl => Operator::I64Shl,
                        Operator::I64ShrS => Operator::I64ShrS,
                        Operator::I64ShrU => Operator::I64ShrU,
                        Operator::I64Rotl => Operator::I64Rotl,
                        Operator::I64Rotr => Operator::I64Rotr,
                        Operator::F32Add => Operator::F32Add,
                        Operator::F32Sub => Operator::F32Sub,
                        Operator::F32Mul => Operator::F32Mul,
                        Operator::F32Div => Operator::F32Div,
                        Operator::F32Eq => Operator::F32Eq,
                        Operator::F32Ne => Operator::F32Ne,
                        Operator::F32Lt => Operator::F32Lt,
                        Operator::F32Gt => Operator::F32Gt,
                        Operator::F32Le => Operator::F32Le,
                        Operator::F32Ge => Operator::F32Ge,
                        Operator::F64Add => Operator::F64Add,
                        Operator::F64Sub => Operator::F64Sub,
                        Operator::F64Mul => Operator::F64Mul,
                        Operator::F64Div => Operator::F64Div,
                        Operator::F64Eq => Operator::F64Eq,
                        Operator::F64Ne => Operator::F64Ne,
                        Operator::F64Lt => Operator::F64Lt,
                        Operator::F64Gt => Operator::F64Gt,
                        Operator::F64Le => Operator::F64Le,
                        Operator::F64Ge => Operator::F64Ge,
                        Operator::I32Add => Operator::I32Add,
                        Operator::I32Sub => Operator::I32Sub,
                        Operator::I32Mul => Operator::I32Mul,
                        Operator::I32DivS => Operator::I32DivS,
                        Operator::I32DivU => Operator::I32DivU,
                        Operator::I32RemS => Operator::I32RemS,
                        Operator::I32RemU => Operator::I32RemU,
                        Operator::I32LtS => Operator::I32LtS,
                        Operator::I32LtU => Operator::I32LtU,
                        Operator::I32LeS => Operator::I32LeS,
                        Operator::I32LeU => Operator::I32LeU,
                        Operator::I32GtS => Operator::I32GtS,
                        Operator::I32GtU => Operator::I32GtU,
                        Operator::I32GeS => Operator::I32GeS,
                        Operator::I32GeU => Operator::I32GeU,
                        Operator::I32Eq => Operator::I32Eq,
                        Operator::I32Ne => Operator::I32Ne,
                        Operator::I32Eqz => Operator::I32Eqz,
                        Operator::I32And => Operator::I32And,
                        Operator::I32Or => Operator::I32Or,
                        Operator::I32Xor => Operator::I32Xor,
                        Operator::I32Shl => Operator::I32Shl,
                        Operator::I32ShrS => Operator::I32ShrS,
                        Operator::I32ShrU => Operator::I32ShrU,
                        Operator::I32Rotl => Operator::I32Rotl,
                        Operator::I32Rotr => Operator::I32Rotr,
                        Operator::LocalGet { local_index } => Operator::LocalGet { local_index },
                        Operator::LocalSet { local_index } => Operator::LocalSet { local_index },
                        Operator::LocalTee { local_index } => Operator::LocalTee { local_index },
                        Operator::GlobalGet { global_index } => {
                            Operator::GlobalGet { global_index }
                        }
                        Operator::GlobalSet { global_index } => {
                            Operator::GlobalSet { global_index }
                        }
                        Operator::F32Load { memarg } => Operator::F32Load { memarg },
                        Operator::F64Load { memarg } => Operator::F64Load { memarg },
                        Operator::F32Store { memarg } => Operator::F32Store { memarg },
                        Operator::F64Store { memarg } => Operator::F64Store { memarg },
                        Operator::I32Load { memarg } => Operator::I32Load { memarg },
                        Operator::I32Store { memarg } => Operator::I32Store { memarg },
                        Operator::I32Store8 { memarg } => Operator::I32Store8 { memarg },
                        Operator::I32Load8S { memarg } => Operator::I32Load8S { memarg },
                        Operator::I32Load8U { memarg } => Operator::I32Load8U { memarg },
                        Operator::I32Load16S { memarg } => Operator::I32Load16S { memarg },
                        Operator::I32Load16U { memarg } => Operator::I32Load16U { memarg },
                        Operator::I32Store16 { memarg } => Operator::I32Store16 { memarg },
                        Operator::I64Load { memarg } => Operator::I64Load { memarg },
                        Operator::I64Store { memarg } => Operator::I64Store { memarg },
                        Operator::I64Load8S { memarg } => Operator::I64Load8S { memarg },
                        Operator::I64Load8U { memarg } => Operator::I64Load8U { memarg },
                        Operator::I64Load16S { memarg } => Operator::I64Load16S { memarg },
                        Operator::I64Load16U { memarg } => Operator::I64Load16U { memarg },
                        Operator::I64Load32S { memarg } => Operator::I64Load32S { memarg },
                        Operator::I64Load32U { memarg } => Operator::I64Load32U { memarg },
                        Operator::I64Store8 { memarg } => Operator::I64Store8 { memarg },
                        Operator::I64Store16 { memarg } => Operator::I64Store16 { memarg },
                        Operator::I64Store32 { memarg } => Operator::I64Store32 { memarg },
                        Operator::MemorySize { mem, .. } => Operator::MemorySize { mem },
                        Operator::MemoryGrow { mem, .. } => Operator::MemoryGrow { mem },
                        Operator::Return => Operator::Return,
                        Operator::End => Operator::End,
                        Operator::Drop => Operator::Drop,
                        Operator::Call { function_index } => Operator::Call { function_index },
                        Operator::If { blockty } => Operator::If { blockty },
                        Operator::Else => Operator::Else,
                        Operator::Block { blockty } => Operator::Block { blockty },
                        Operator::Loop { blockty } => Operator::Loop { blockty },
                        Operator::Br { relative_depth } => Operator::Br { relative_depth },
                        Operator::BrIf { relative_depth } => Operator::BrIf { relative_depth },
                        Operator::I32WrapI64 => Operator::I32WrapI64,
                        Operator::I64ExtendI32S => Operator::I64ExtendI32S,
                        Operator::I64ExtendI32U => Operator::I64ExtendI32U,
                        Operator::F32ConvertI32S => Operator::F32ConvertI32S,
                        Operator::F32ConvertI32U => Operator::F32ConvertI32U,
                        Operator::F64ConvertI32S => Operator::F64ConvertI32S,
                        Operator::F64ConvertI32U => Operator::F64ConvertI32U,
                        Operator::I32TruncF32S => Operator::I32TruncF32S,
                        Operator::I32TruncF32U => Operator::I32TruncF32U,
                        Operator::I32TruncF64S => Operator::I32TruncF64S,
                        Operator::I32TruncF64U => Operator::I32TruncF64U,
                        Operator::F64PromoteF32 => Operator::F64PromoteF32,
                        Operator::F32DemoteF64 => Operator::F32DemoteF64,
                        // Phase 1: select + ビットカウント系命令
                        Operator::Select => Operator::Select,
                        Operator::I32Clz => Operator::I32Clz,
                        Operator::I32Ctz => Operator::I32Ctz,
                        Operator::I32Popcnt => Operator::I32Popcnt,
                        Operator::I64Clz => Operator::I64Clz,
                        Operator::I64Ctz => Operator::I64Ctz,
                        Operator::I64Popcnt => Operator::I64Popcnt,
                        // Phase 2: 浮動小数点高度演算
                        Operator::F32Abs => Operator::F32Abs,
                        Operator::F32Neg => Operator::F32Neg,
                        Operator::F32Sqrt => Operator::F32Sqrt,
                        Operator::F32Ceil => Operator::F32Ceil,
                        Operator::F32Floor => Operator::F32Floor,
                        Operator::F32Trunc => Operator::F32Trunc,
                        Operator::F32Nearest => Operator::F32Nearest,
                        Operator::F32Min => Operator::F32Min,
                        Operator::F32Max => Operator::F32Max,
                        Operator::F32Copysign => Operator::F32Copysign,
                        Operator::F64Abs => Operator::F64Abs,
                        Operator::F64Neg => Operator::F64Neg,
                        Operator::F64Sqrt => Operator::F64Sqrt,
                        Operator::F64Ceil => Operator::F64Ceil,
                        Operator::F64Floor => Operator::F64Floor,
                        Operator::F64Trunc => Operator::F64Trunc,
                        Operator::F64Nearest => Operator::F64Nearest,
                        Operator::F64Min => Operator::F64Min,
                        Operator::F64Max => Operator::F64Max,
                        Operator::F64Copysign => Operator::F64Copysign,
                        // Phase 3: バルクメモリ操作
                        Operator::MemoryCopy { src_mem, dst_mem } => {
                            Operator::MemoryCopy { src_mem, dst_mem }
                        }
                        Operator::MemoryFill { mem } => Operator::MemoryFill { mem },
                        _ => {
                            eprintln!("Unsupported operator: {op:?}");
                            continue;
                        }
                    };
                    operators.push(owned_op);
                }

                functions[idx].body.locals = locals;
                functions[idx].body.operators = operators;
            }
        }

        if let Some(start_idx) = start_func_idx {
            if let Some(func) = functions.iter_mut().find(|f| f.idx == start_idx) {
                func.name = Some("_start".to_string());
            }
        }

        Ok(WasmModule {
            functions,
            start_func_idx,
            memories,
            has_putchar_import,
            globals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_wasm() {
        let wasm_bytes = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        let result = WasmModule::parse(&wasm_bytes);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert!(module.functions.is_empty());
        assert!(module.start_func_idx.is_none());
        assert!(module.memories.is_empty());
        assert!(!module.has_putchar_import);
    }

    #[test]
    fn test_parse_function_with_i32_const() {
        let wasm_bytes = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0a, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x60, 0x00, 0x00, 0x03, 0x03, 0x02, 0x00, 0x01, 0x08, 0x01, 0x01,
            0x0a, 0x1b, 0x02, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, 0x11, 0x00, 0x41,
            0x0a, 0x41, 0x05, 0x6a, 0x41, 0x02, 0x6c, 0x41, 0x03, 0x6d, 0x41, 0x01, 0x6b, 0x1a,
            0x0b,
        ];

        let result = WasmModule::parse(&wasm_bytes);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert!(!module.functions.is_empty());
        assert!(module.start_func_idx.is_some());
    }

    #[test]
    fn test_function_body_operators() {
        let wasm_bytes = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0a, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x60, 0x00, 0x00, 0x03, 0x03, 0x02, 0x00, 0x01, 0x08, 0x01, 0x01,
            0x0a, 0x1b, 0x02, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, 0x11, 0x00, 0x41,
            0x0a, 0x41, 0x05, 0x6a, 0x41, 0x02, 0x6c, 0x41, 0x03, 0x6d, 0x41, 0x01, 0x6b, 0x1a,
            0x0b,
        ];
        let module = WasmModule::parse(&wasm_bytes).unwrap();

        if let Some(function) = module.functions.first() {
            assert!(!function.body.operators.is_empty());

            let has_const_or_add = function
                .body
                .operators
                .iter()
                .any(|op| matches!(op, Operator::I32Const { .. } | Operator::I32Add));
            assert!(has_const_or_add);
        }
    }

    #[test]
    fn test_start_function_detection() {
        let wasm_bytes = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0a, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x60, 0x00, 0x00, 0x03, 0x03, 0x02, 0x00, 0x01, 0x08, 0x01, 0x01,
            0x0a, 0x1b, 0x02, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, 0x11, 0x00, 0x41,
            0x0a, 0x41, 0x05, 0x6a, 0x41, 0x02, 0x6c, 0x41, 0x03, 0x6d, 0x41, 0x01, 0x6b, 0x1a,
            0x0b,
        ];
        let module = WasmModule::parse(&wasm_bytes).unwrap();

        assert!(module.start_func_idx.is_some());

        let start_function = module
            .functions
            .iter()
            .find(|f| f.name.as_ref() == Some(&"_start".to_string()));
        assert!(start_function.is_some());
    }
}
