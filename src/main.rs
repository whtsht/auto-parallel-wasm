use anyhow::Result;
use auto_parallel_wasm::{Compiler, WasmModule};
use inkwell::context::Context;
use std::env;
use std::fs;
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "exec" => {
            if args.len() != 3 {
                eprintln!("Usage: auto-parallel-wasm exec <wasm-file>");
                process::exit(1);
            }
            exec_command(&args[2])
        }
        "compile" => {
            if args.len() != 4 {
                eprintln!("Usage: auto-parallel-wasm compile <wasm-file> <output-file>");
                process::exit(1);
            }
            compile_command(&args[2], &args[3])
        }
        "ir" => {
            if args.len() < 3 || args.len() > 4 {
                eprintln!("Usage: auto-parallel-wasm ir <wasm-file> [output-file]");
                process::exit(1);
            }
            let output_file = if args.len() == 4 {
                Some(args[3].as_str())
            } else {
                None
            };
            ir_command(&args[2], output_file)
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  auto-parallel-wasm exec <wasm-file>");
    eprintln!("  auto-parallel-wasm compile <wasm-file> <output-file>");
    eprintln!("  auto-parallel-wasm ir <wasm-file> [output-file]");
}

fn exec_command(wasm_file: &str) -> Result<()> {
    let wasm_bytes = fs::read(wasm_file)?;
    let wasm_module = WasmModule::parse(&wasm_bytes)?;

    let context = Context::create();
    let compiler = Compiler::new(&context, "wasm_aot")?;

    compiler.compile_module(&wasm_module)?;

    let exit_code = compiler.run_main()?;
    process::exit(exit_code);
}

fn compile_command(wasm_file: &str, output_file: &str) -> Result<()> {
    let wasm_bytes = fs::read(wasm_file)?;
    let wasm_module = WasmModule::parse(&wasm_bytes)?;

    let context = Context::create();
    let compiler = Compiler::new(&context, "wasm_aot")?;

    compiler.compile_module(&wasm_module)?;
    compiler.write_object_file(output_file)?;

    println!("Compiled to: {}", output_file);
    Ok(())
}

fn ir_command(wasm_file: &str, output_file: Option<&str>) -> Result<()> {
    let wasm_bytes = fs::read(wasm_file)?;
    let wasm_module = WasmModule::parse(&wasm_bytes)?;

    let context = Context::create();
    let compiler = Compiler::new(&context, "wasm_aot")?;

    compiler.compile_module(&wasm_module)?;

    match output_file {
        Some(file) => {
            compiler.write_ir_to_file(file)?;
            println!("LLVM IR written to: {}", file);
        }
        None => {
            compiler.print_ir_to_stdout();
        }
    }

    Ok(())
}
