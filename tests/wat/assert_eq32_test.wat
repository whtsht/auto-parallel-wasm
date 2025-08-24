(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))

  (func $start_func
    ;; Test 1: Basic equality (42 == 42)
    i32.const 42
    i32.const 42
    call $assert_eq32

    ;; Test 2: Arithmetic result (10 + 5 == 15)
    i32.const 10
    i32.const 5
    i32.add
    i32.const 15
    call $assert_eq32

    ;; Test 3: Multiple operations (20 * 2 - 5 == 35)
    i32.const 20
    i32.const 2
    i32.mul
    i32.const 5
    i32.sub
    i32.const 35
    call $assert_eq32

    ;; Test 4: Zero values (0 == 0)
    i32.const 0
    i32.const 0
    call $assert_eq32

    ;; Test 5: Negative values (-10 == -10)
    i32.const -10
    i32.const -10
    call $assert_eq32
  )

  (start $start_func)
)