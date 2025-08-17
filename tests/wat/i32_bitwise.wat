(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (func $test_i32_bitwise
    ;; Test i32.and: 0x0F & 0xF0 = 0x00
    i32.const 0x0F
    i32.const 0xF0
    i32.and
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.or: 0x0F | 0xF0 = 0xFF
    i32.const 0x0F
    i32.const 0xF0
    i32.or
    i32.const 255
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.xor: 0x0F ^ 0xF0 = 0xFF
    i32.const 0x0F
    i32.const 0xF0
    i32.xor
    i32.const 255
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.shl: 1 << 4 = 16
    i32.const 1
    i32.const 4
    i32.shl
    i32.const 16
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.shr_s: -16 >> 2 = -4
    i32.const -16
    i32.const 2
    i32.shr_s
    i32.const -4
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.shr_u: 0xFFFFFFFF >> 4 = 0x0FFFFFFF
    i32.const 0xFFFFFFFF
    i32.const 4
    i32.shr_u
    i32.const 0x0FFFFFFF
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.rotl: 0x12345678 rotl 4 = 0x23456781
    i32.const 0x12345678
    i32.const 4
    i32.rotl
    i32.const 0x23456781
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.rotr: 0x12345678 rotr 4 = 0x81234567
    i32.const 0x12345678
    i32.const 4
    i32.rotr
    i32.const 0x81234567
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test shift with large shift count (should be masked to 5 bits)
    i32.const 1
    i32.const 36
    i32.shl
    i32.const 16
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test rotate with large count (should be masked)
    i32.const 0x12345678
    i32.const 36
    i32.rotl
    i32.const 0x23456781
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Newline
    i32.const 10
    call $putchar
    drop
  )

  (start $test_i32_bitwise)
)
