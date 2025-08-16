use std::fs;
use std::path::Path;
use std::process::Command;

fn get_executable_path() -> String {
    let path = format!(
        "{}/target/debug/auto-parallel-wasm",
        env!("CARGO_MANIFEST_DIR")
    );
    if !Path::new(&path).exists() {
        panic!("Executable not found at: {}", path);
    }
    path
}

fn create_test_wasm_file() -> String {
    let wat_content = r#"
(module
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func $_start
    i32.const 10
    i32.const 5
    i32.add
    i32.const 2
    i32.mul
    i32.const 3
    i32.div_s
    i32.const 1
    i32.sub
    drop
  )
  (start 1)
)
"#;

    let wat_file = format!("/tmp/test_wasm_{:?}.wat", std::thread::current().id());
    let wasm_file = format!("/tmp/test_wasm_{:?}.wasm", std::thread::current().id());

    fs::write(&wat_file, wat_content).unwrap();

    let output = Command::new("wat2wasm")
        .args([&wat_file, "-o", &wasm_file])
        .output()
        .expect("wat2wasm command failed - make sure wabt tools are installed");

    if !output.status.success() {
        panic!(
            "wat2wasm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::remove_file(&wat_file).ok();

    wasm_file
}

#[test]
fn test_exec_command() {
    let wasm_file = create_test_wasm_file();
    let executable = get_executable_path();

    let output = Command::new(&executable)
        .args(["exec", &wasm_file])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Command failed with exit code: {:?}", output.status.code());
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "exec command should succeed");

    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_compile_command() {
    let wasm_file = create_test_wasm_file();
    let output_file = format!("/tmp/test_wasm_output_{:?}.o", std::thread::current().id());
    let executable = get_executable_path();

    let output = Command::new(&executable)
        .args(["compile", &wasm_file, &output_file])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!(
            "Compile command failed with exit code: {:?}",
            output.status.code()
        );
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "compile command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Compiled to:"),
        "Should show compilation message"
    );
    assert!(
        stdout.contains(&output_file),
        "Should show output file path"
    );

    assert!(Path::new(&output_file).exists(), "Output file should exist");

    let metadata = fs::metadata(&output_file).unwrap();
    assert!(metadata.len() > 0, "Output file should not be empty");

    fs::remove_file(&wasm_file).ok();
    fs::remove_file(&output_file).ok();
}

#[test]
fn test_ir_command() {
    let wasm_file = create_test_wasm_file();
    let ir_output_file = format!("/tmp/test_wasm_ir_{:?}.ll", std::thread::current().id());
    let executable = get_executable_path();

    let expected_ir = "; ModuleID = 'wasm_aot'
source_filename = \"wasm_aot\"
target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"

define i32 @func_0(i32 %0, i32 %1) {
entry:
  %add = add i32 %0, %1
  ret i32 %add
}

define void @_start() {
entry:
  ret void
}

define i32 @main() {
entry:
  ret i32 0
}
";

    let output = Command::new(&executable)
        .args(["ir", &wasm_file])
        .output()
        .expect("Failed to execute ir stdout command");

    assert!(output.status.success(), "ir stdout command should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.trim(),
        expected_ir.trim(),
        "IR stdout content should match exactly"
    );

    let output = Command::new(&executable)
        .args(["ir", &wasm_file, &ir_output_file])
        .output()
        .expect("Failed to execute ir file command");

    assert!(output.status.success(), "ir file command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("LLVM IR written to:"),
        "Should show IR output message"
    );
    assert!(
        stdout.contains(&ir_output_file),
        "Should show output file path"
    );

    assert!(
        Path::new(&ir_output_file).exists(),
        "IR output file should exist"
    );

    let ir_content = fs::read_to_string(&ir_output_file).unwrap();
    assert_eq!(
        ir_content.trim(),
        expected_ir.trim(),
        "IR file content should match exactly"
    );

    fs::remove_file(&wasm_file).ok();
    fs::remove_file(&ir_output_file).ok();
}
