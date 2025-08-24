(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (type $type0 (func (result i32)))

  (func $func0 (result i32)
    i32.const 52  ;; ASCII '4'
    call $putchar
    drop
    i32.const 50  ;; ASCII '2'
    call $putchar
    drop
    i32.const 10  ;; ASCII '\n'
    call $putchar
    drop
    i32.const 42
  )

  (table 1 funcref)

  (elem (i32.const 0) $func0)

  (func $start_func
    ;; Call function at index 0
    i32.const 0
    call_indirect (type $type0)
    drop
  )

  (start $start_func)
)
