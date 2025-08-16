# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Environment

This project uses Nix for development environment management. All commands must be run within the Nix development shell:

```bash
nix develop --command <command>
```

## Essential Commands

```bash
# Build the project
nix develop --command cargo build

# Execute WASM file with JIT compilation
nix develop --command cargo run exec <wasm-file>
nix develop --command cargo run <wasm-file>  # same as exec

# Compile WASM to native object file
nix develop --command cargo run compile <wasm-file> <output-file>

# Output LLVM IR
nix develop --command cargo run ir <wasm-file>  # to stdout
nix develop --command cargo run ir <wasm-file> <output-file>  # to file

# Run all tests
nix develop --command cargo test

# Run specific test
nix develop --command cargo test <test_name>

# Code quality checks (run all together)
nix develop --command bash -c "cargo fix --allow-dirty && cargo clippy --fix --allow-dirty && cargo fmt && cargo test"

# Clean build artifacts
nix develop --command cargo clean
```

## Architecture

This is a minimal WebAssembly AOT compiler that transforms WASM binaries into native code via LLVM IR:

```
WASM Binary → WasmParser → LLVM IR → JIT/Object/IR Output
```

### Core Components

**WasmModule (wasm_parser.rs)**: Parses WASM binaries using `wasmparser` crate, extracting function definitions, types, and operator sequences. Handles lifetime issues by converting borrowed operators to owned variants.

**Compiler (compiler.rs)**: Converts WASM stack-based operations to LLVM IR register-based operations. Uses a compile-time virtual stack (`Vec<BasicValueEnum>`) to track WASM stack state, but generates direct register operations in LLVM IR with no runtime stack manipulation.

**CLI (main.rs)**: Command-line interface supporting multiple output modes: JIT execution (`exec`), native object file compilation (`compile`), and LLVM IR output (`ir`).

### Stack-to-Register Translation

The compiler simulates WASM's stack machine during compilation but produces pure register-based LLVM IR:

- WASM operators push/pop from a compile-time virtual stack
- Each stack operation generates corresponding LLVM IR instructions
- Final output contains no stack operations - only SSA register assignments

### Supported Features

- i32 arithmetic operations (add, sub, mul, div_s)
- Constants and local variable access
- Start function (`_start`) detection and execution
- Basic control flow (drop, return, end)
- Multiple output modes:
  - JIT execution with immediate execution
  - Native object file generation
  - LLVM IR output (stdout or file)

### Dependencies

- **inkwell**: LLVM bindings for IR generation and JIT execution
- **wasmparser**: WASM binary parsing
- **anyhow**: Error handling

## Testing Structure

- **Unit tests**: Embedded in source files using `#[cfg(test)]`
- **Integration tests**: In `tests/` directory for end-to-end functionality
- **Test WASM data**: Embedded binary data in integration tests for reliable testing

## Code Standards

- No comments in code - write self-explanatory code instead
- Use descriptive variable and function names
- Prefer explicit error handling with `Result` types
- Follow Rust formatting conventions via `cargo fmt`