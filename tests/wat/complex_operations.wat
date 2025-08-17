(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (func $test_complex_operations
    ;; Bitwise operation with arithmetic: (a & b) + (a | b) = a + b
    i32.const 0x12345678
    i32.const 0x87654321
    i32.and
    i32.const 0x12345678
    i32.const 0x87654321
    i32.or
    i32.add
    i32.const 0x12345678
    i32.const 0x87654321
    i32.add
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Power of 2 detection using bitwise: (x & (x-1)) == 0 for powers of 2
    i32.const 64
    i32.const 64
    i32.const 1
    i32.sub
    i32.and
    i32.eqz
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Non-power of 2 test
    i32.const 63
    i32.const 63
    i32.const 1
    i32.sub
    i32.and
    i32.eqz
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Simple comparison test: 42 > 0
    i32.const 42
    i32.const 0
    i32.gt_s
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Simple arithmetic test: (-42) * (-1) = 42
    i32.const -42
    i32.const -1
    i32.mul
    i32.const 42
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Fast modulo by power of 2: x % 16 = x & 15
    i32.const 123
    i32.const 16
    i32.rem_u
    i32.const 123
    i32.const 15
    i32.and
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Bit reversal test (simple case: reverse low 4 bits)
    i32.const 9
    i32.const 1
    i32.and
    i32.const 3
    i32.shl
    i32.const 9
    i32.const 1
    i32.shr_u
    i32.const 1
    i32.and
    i32.const 2
    i32.shl
    i32.or
    i32.const 9
    i32.const 2
    i32.shr_u
    i32.const 1
    i32.and
    i32.const 1
    i32.shl
    i32.or
    i32.const 9
    i32.const 3
    i32.shr_u
    i32.const 1
    i32.and
    i32.or
    i32.const 9
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; 64-bit arithmetic combining with 32-bit result
    i64.const 1000000000
    i64.const 1000000000
    i64.mul
    i64.const 1000000000000000000
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Population count approximation for small numbers (count 1 bits in 7)
    i32.const 7
    i32.const 7
    i32.const 1
    i32.shr_u
    i32.const 0x55555555
    i32.and
    i32.sub
    i32.const 3
    i32.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Gray code conversion: binary to gray
    i32.const 6
    i32.const 6
    i32.const 1
    i32.shr_u
    i32.xor
    i32.const 5
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

  (start $test_complex_operations)
)
