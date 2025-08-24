(module
  (import "env" "assert_eq64" (func $assert_eq64 (param i64 i64)))

  (func $start_func
    ;; Test 1: Basic equality (42 == 42)
    i64.const 42
    i64.const 42
    call $assert_eq64

    ;; Test 2: Large number equality
    i64.const 9223372036854775807  ;; i64::MAX
    i64.const 9223372036854775807
    call $assert_eq64

    ;; Test 3: Arithmetic result (100 * 100 == 10000)
    i64.const 100
    i64.const 100
    i64.mul
    i64.const 10000
    call $assert_eq64

    ;; Test 4: Zero values (0 == 0)
    i64.const 0
    i64.const 0
    call $assert_eq64

    ;; Test 5: Negative values (-1000 == -1000)
    i64.const -1000
    i64.const -1000
    call $assert_eq64
  )

  (start $start_func)
)