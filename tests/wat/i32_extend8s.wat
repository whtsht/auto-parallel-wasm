(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (func $test_i32_extend8s
    ;; Test i32.extend8_s with positive value (0x7F -> 127)
    i32.const 0x7F
    i32.extend8_s
    i32.const 127
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.extend8_s with negative value (0x80 -> -128)
    i32.const 0x80
    i32.extend8_s
    i32.const -128
    i32.eq
    i32.const 49
    i32.add
    call $putchar
    drop

    ;; Test i32.extend8_s with value that has upper bits set (0x1FF -> -1)
    i32.const 0x1FF
    i32.extend8_s
    i32.const -1
    i32.eq
    i32.const 49
    i32.add
    call $putchar
    drop

    ;; Newline
    i32.const 10
    call $putchar
    drop
  )

  (start $test_i32_extend8s)
)