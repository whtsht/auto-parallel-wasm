(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))

  (func $test_i64_comparisons
    ;; Test i64.eq equal values
    i64.const 1000000000000
    i64.const 1000000000000
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.ne different values
    i64.const 1000000000000
    i64.const 2000000000000
    i64.ne
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.eqz with zero
    i64.const 0
    i64.eqz
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.eqz with non-zero
    i64.const 1000000000000
    i64.eqz
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.lt_s signed comparison
    i64.const -1
    i64.const 1
    i64.lt_s
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.lt_u unsigned comparison (same values as above)
    i64.const -1
    i64.const 1
    i64.lt_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.le_s less than or equal
    i64.const 100
    i64.const 100
    i64.le_s
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.le_u less than or equal unsigned
    i64.const 100
    i64.const 200
    i64.le_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.gt_s greater than signed
    i64.const 200
    i64.const 100
    i64.gt_s
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.gt_u greater than unsigned with negative value
    i64.const -1
    i64.const 1
    i64.gt_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.ge_s greater than or equal signed
    i64.const 100
    i64.const 100
    i64.ge_s
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Test i64.ge_u greater than or equal unsigned
    i64.const 300
    i64.const 200
    i64.ge_u
    i32.const 48
    i32.add
    call $putchar
    drop

    ;; Newline
    i32.const 10
    call $putchar
    drop
  )

  (start $test_i64_comparisons)
)
