(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))

  (type $type0 (func (result i32)))

  (func $func0 (result i32)
    i32.const 42
  )

  (func $func1 (result i32)
    i32.const 100
  )

  (table 2 funcref)

  (elem (i32.const 0) $func0 $func1)

  (func $start_func
    ;; Call function at index 0 (should return 42)
    i32.const 0
    call_indirect (type $type0)
    ;; Test that it returns 42
    i32.const 42
    call $assert_eq32

    ;; Call function at index 1 (should return 100)
    i32.const 1
    call_indirect (type $type0)
    ;; Test that it returns 100
    i32.const 100
    call $assert_eq32
  )

  (start $start_func)
)
