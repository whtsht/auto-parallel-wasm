(module
  (func $putchar (import "" "putchar") (param i32) (result i32))
  (func $_start (local i32)
    ;; Store 42 in local variable
    i32.const 42
    local.set 0
    ;; Load from local variable and display
    local.get 0
    ;; Display tens digit (42 / 10 + '0' = 4 + 48 = 52 = '4')
    i32.const 10
    i32.div_s
    i32.const 48
    i32.add
    call $putchar
    drop
    ;; Display ones digit (42 % 10 + '0' = 2 + 48 = 50 = '2')
    local.get 0
    i32.const 10
    i32.rem_s
    i32.const 48
    i32.add
    call $putchar
    drop
  )
  (start 1)
)
