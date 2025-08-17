(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (func $test_i32_extended
    ;; Test i32.eqz with zero
    i32.const 0
    i32.eqz
    i32.const 49
    i32.add
    call $putchar
    drop

    ;; Test i32.eqz with non-zero
    i32.const 42
    i32.eqz
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.div_u basic division
    i32.const 20
    i32.const 3
    i32.div_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.div_u with large numbers
    i32.const 4294967295
    i32.const 2
    i32.div_u
    i32.const 2147483647
    i32.eq
    i32.const 49
    i32.add
    call $putchar
    drop

    ;; Test i32.rem_u basic remainder
    i32.const 20
    i32.const 3
    i32.rem_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.rem_u with large divisor
    i32.const 5
    i32.const 7
    i32.rem_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.lt_u: 4294967295 < 1 (should be 0)
    i32.const 4294967295
    i32.const 1
    i32.lt_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.le_u: 10 <= 10 (should be 1)
    i32.const 10
    i32.const 10
    i32.le_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.gt_u: 4294967295 > 1 (should be 1)
    i32.const 4294967295
    i32.const 1
    i32.gt_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i32.ge_u: 5 >= 5 (should be 1)
    i32.const 5
    i32.const 5
    i32.ge_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test signed/unsigned comparison differences with same bit pattern
    i32.const -1
    i32.const 1
    i32.lt_s
    i32.const 48
    i32.add
    call $putchar
    drop

    i32.const -1
    i32.const 1
    i32.lt_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test division by 1 (identity)
    i32.const 123456
    i32.const 1
    i32.div_s
    i32.const 123456
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

  (start $test_i32_extended)
)
