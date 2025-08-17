(module
  (func $putchar (import "" "putchar") (param i32) (result i32))
  (memory 1)
  (func $_start
    ;; Store 42 in memory
    i32.const 0
    i32.const 42
    i32.store

    i32.const 4
    i32.const 100
    i32.store
    ;; Load 42 from memory and display
    i32.const 0
    i32.load
    ;; Display tens digit (42 / 10 + '0' = 4 + 48 = 52 = '4')
    i32.const 10
    i32.div_s
    i32.const 48
    i32.add
    call $putchar
    drop
    ;; Display ones digit (42 % 10 + '0' = 2 + 48 = 50 = '2')
    i32.const 0
    i32.load
    i32.const 10
    i32.rem_s
    i32.const 48
    i32.add
    call $putchar
    drop
    memory.size
    drop
  )
  (start 1)
)
