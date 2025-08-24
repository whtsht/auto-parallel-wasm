(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))

  (func $test_i32_extended
    ;; Test i32.eqz with zero (0 == 0 should be 1)
    i32.const 0
    i32.eqz
    i32.const 1
    call $assert_eq32

    ;; Test i32.eqz with non-zero (42 != 0 should be 0)
    i32.const 42
    i32.eqz
    i32.const 0
    call $assert_eq32

    ;; Test i32.div_u basic division (20 / 3 = 6)
    i32.const 20
    i32.const 3
    i32.div_u
    i32.const 6
    call $assert_eq32

    ;; Test i32.div_u with large numbers (4294967295 / 2 = 2147483647)
    i32.const 4294967295
    i32.const 2
    i32.div_u
    i32.const 2147483647
    call $assert_eq32

    ;; Test i32.rem_u basic remainder (20 % 3 = 2)
    i32.const 20
    i32.const 3
    i32.rem_u
    i32.const 2
    call $assert_eq32

    ;; Test i32.rem_u with large divisor (5 % 7 = 5)
    i32.const 5
    i32.const 7
    i32.rem_u
    i32.const 5
    call $assert_eq32

    ;; Test i32.lt_u: 4294967295 < 1 (should be 0)
    i32.const 4294967295
    i32.const 1
    i32.lt_u
    i32.const 0
    call $assert_eq32

    ;; Test i32.le_u: 10 <= 10 (should be 1)
    i32.const 10
    i32.const 10
    i32.le_u
    i32.const 1
    call $assert_eq32

    ;; Test i32.gt_u: 4294967295 > 1 (should be 1)
    i32.const 4294967295
    i32.const 1
    i32.gt_u
    i32.const 1
    call $assert_eq32

    ;; Test i32.ge_u: 5 >= 5 (should be 1)
    i32.const 5
    i32.const 5
    i32.ge_u
    i32.const 1
    call $assert_eq32

    ;; Test signed comparison: -1 < 1 (should be 1)
    i32.const -1
    i32.const 1
    i32.lt_s
    i32.const 1
    call $assert_eq32

    ;; Test unsigned comparison: -1 (4294967295) > 1 (should be 0 for lt_u)
    i32.const -1
    i32.const 1
    i32.lt_u
    i32.const 0
    call $assert_eq32

    ;; Test division by 1 (identity): 123456 / 1 = 123456
    i32.const 123456
    i32.const 1
    i32.div_s
    i32.const 123456
    call $assert_eq32
  )

  (start $test_i32_extended)
)
