use std::fs;
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to execute cargo run")
}

fn wat_to_wasm(wat_path: &str) -> String {
    let wasm_file = format!("/tmp/test_wasm_{:?}.wasm", std::thread::current().id());

    let output = Command::new("wat2wasm")
        .args([wat_path, "-o", &wasm_file])
        .output();

    let output = match output {
        Ok(out) => out,
        Err(_) => panic!("wat2wasm command failed - make sure wabt tools are installed"),
    };

    if !output.status.success() {
        panic!("wat2wasm failed with exit code: {:?}", output.status.code());
    }

    wasm_file
}

fn test_ir(wat_path: &str, expected_ir_path: &str) {
    let wasm_file = wat_to_wasm(wat_path);

    let output = run(&["ir", &wasm_file]);
    assert!(output.status.success(), "IR generation should succeed");

    let actual_ir = String::from_utf8_lossy(&output.stderr);
    let expected_ir =
        fs::read_to_string(expected_ir_path).expect("Failed to read expected IR file");

    assert_eq!(
        actual_ir.trim(),
        expected_ir.trim(),
        "IR should match expected output"
    );

    fs::remove_file(&wasm_file).ok();
}

fn test_compile(wat_path: &str) {
    let wasm_file = wat_to_wasm(wat_path);
    let output_file = format!("/tmp/test_output_{:?}.o", std::thread::current().id());

    let output = run(&["compile", &wasm_file, &output_file]);
    assert!(output.status.success(), "Compilation should succeed");
    assert!(
        std::path::Path::new(&output_file).exists(),
        "Output file should exist"
    );

    let metadata = fs::metadata(&output_file).unwrap();
    assert!(metadata.len() > 0, "Output file should not be empty");

    fs::remove_file(&wasm_file).ok();
    fs::remove_file(&output_file).ok();
}

fn test_jit(wat_path: &str) {
    let wasm_file = wat_to_wasm(wat_path);

    let output = run(&["exec", &wasm_file]);

    if wat_path.contains("empty_module") {
        assert!(
            output.status.success() || output.status.code() == Some(1),
            "Empty module should succeed or exit with code 1"
        );
    } else {
        assert!(output.status.success(), "JIT execution should succeed");
    }

    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_e2e() {
    let wat_files = [
        "tests/wat/basic_arithmetic.wat",
        "tests/wat/memory_operations.wat",
        "tests/wat/local_variables.wat",
    ];

    let ir_files = [
        "tests/ir/basic_arithmetic.ll",
        "tests/ir/memory_operations.ll",
        "tests/ir/local_variables.ll",
    ];

    for (wat_path, ir_path) in wat_files.iter().zip(ir_files.iter()) {
        test_ir(wat_path, ir_path);
        test_compile(wat_path);
        test_jit(wat_path);
    }
}
