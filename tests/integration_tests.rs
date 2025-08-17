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

fn test_jit(wat_path: &str, expected_output_path: &str) {
    let wasm_file = wat_to_wasm(wat_path);

    let output = run(&["exec", &wasm_file]);

    if wat_path.contains("empty_module") {
        assert!(
            output.status.success() || output.status.code() == Some(1),
            "Empty module should succeed or exit with code 1"
        );
    } else {
        assert!(output.status.success(), "JIT execution should succeed");

        let actual_output = String::from_utf8_lossy(&output.stdout);
        let expected_output =
            fs::read_to_string(expected_output_path).expect("Failed to read expected output file");

        assert_eq!(
            actual_output.trim(),
            expected_output.trim(),
            "Output should match expected"
        );
    }

    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_e2e() {
    let test_cases = [
        (
            "tests/wat/basic_arithmetic.wat",
            "tests/ir/basic_arithmetic.ll",
            "tests/output/basic_arithmetic.txt",
        ),
        (
            "tests/wat/memory_operations.wat",
            "tests/ir/memory_operations.ll",
            "tests/output/memory_operations.txt",
        ),
        (
            "tests/wat/local_variables.wat",
            "tests/ir/local_variables.ll",
            "tests/output/local_variables.txt",
        ),
        (
            "tests/wat/control_flow.wat",
            "tests/ir/control_flow.ll",
            "tests/output/control_flow.txt",
        ),
        (
            "tests/wat/for_loop.wat",
            "tests/ir/for_loop.ll",
            "tests/output/for_loop.txt",
        ),
        (
            "tests/wat/i32_extended.wat",
            "tests/ir/i32_extended.ll",
            "tests/output/i32_extended.txt",
        ),
        (
            "tests/wat/i32_bitwise.wat",
            "tests/ir/i32_bitwise.ll",
            "tests/output/i32_bitwise.txt",
        ),
        (
            "tests/wat/i64_arithmetic.wat",
            "tests/ir/i64_arithmetic.ll",
            "tests/output/i64_arithmetic.txt",
        ),
        (
            "tests/wat/i64_comparisons.wat",
            "tests/ir/i64_comparisons.ll",
            "tests/output/i64_comparisons.txt",
        ),
        (
            "tests/wat/i64_bitwise.wat",
            "tests/ir/i64_bitwise.ll",
            "tests/output/i64_bitwise.txt",
        ),
        (
            "tests/wat/complex_operations.wat",
            "tests/ir/complex_operations.ll",
            "tests/output/complex_operations.txt",
        ),
    ];

    for (wat_path, ir_path, output_path) in test_cases.iter() {
        test_ir(wat_path, ir_path);
        test_compile(wat_path);
        test_jit(wat_path, output_path);
    }
}
