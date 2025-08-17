use anyhow::Result;
use wasmparser::{FuncType, MemoryType, Operator, Parser, Payload, TypeRef, ValType};

pub struct WasmModule {
    pub functions: Vec<Function>,
    pub start_func_idx: Option<u32>,
    pub memories: Vec<MemoryType>,
    pub has_putchar_import: bool,
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
                        Operator::I32Add => Operator::I32Add,
                        Operator::I32Sub => Operator::I32Sub,
                        Operator::I32Mul => Operator::I32Mul,
                        Operator::I32DivS => Operator::I32DivS,
                        Operator::I32RemS => Operator::I32RemS,
                        Operator::LocalGet { local_index } => Operator::LocalGet { local_index },
                        Operator::LocalSet { local_index } => Operator::LocalSet { local_index },
                        Operator::I32Load { memarg } => Operator::I32Load { memarg },
                        Operator::I32Store { memarg } => Operator::I32Store { memarg },
                        Operator::I32Store8 { memarg } => Operator::I32Store8 { memarg },
                        Operator::I64Load { memarg } => Operator::I64Load { memarg },
                        Operator::I64Store { memarg } => Operator::I64Store { memarg },
                        Operator::MemorySize { mem, .. } => Operator::MemorySize { mem },
                        Operator::MemoryGrow { mem, .. } => Operator::MemoryGrow { mem },
                        Operator::Return => Operator::Return,
                        Operator::End => Operator::End,
                        Operator::Drop => Operator::Drop,
                        Operator::Call { function_index } => Operator::Call { function_index },
                        _ => continue,
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
