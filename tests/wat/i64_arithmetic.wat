(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (func $test_i64_arithmetic
    ;; Test i64.add
    i64.const 1000000000
    i64.const 2000000000
    i64.add
    i64.const 3000000000
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.sub
    i64.const 5000000000
    i64.const 2000000000
    i64.sub
    i64.const 3000000000
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.mul
    i64.const 1000000
    i64.const 1000000
    i64.mul
    i64.const 1000000000000
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.div_s positive numbers
    i64.const 100
    i64.const 3
    i64.div_s
    i64.const 33
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.div_s with negative dividend
    i64.const -100
    i64.const 3
    i64.div_s
    i64.const -33
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.div_u with large numbers
    i64.const 18446744073709551615
    i64.const 2
    i64.div_u
    i64.const 9223372036854775807
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.rem_s positive remainder
    i64.const 100
    i64.const 3
    i64.rem_s
    i64.const 1
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.rem_s negative dividend
    i64.const -100
    i64.const 3
    i64.rem_s
    i64.const -1
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.rem_u basic case
    i64.const 100
    i64.const 3
    i64.rem_u
    i64.const 1
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test with maximum values
    i64.const 9223372036854775807
    i64.const 1
    i64.add
    i64.const -9223372036854775808
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Newline
    i32.const 10
    call $putchar
    drop
  )

  (start $test_i64_arithmetic)
)
