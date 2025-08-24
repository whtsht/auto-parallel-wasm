(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))

  (func $_start (export "_start")
    ;; Test if/else: true branch should return 89 ('Y')
    i32.const 1
    if (result i32)
      i32.const 89  ;; ASCII 'Y'
    else
      i32.const 78  ;; ASCII 'N'
    end
    i32.const 89
    call $assert_eq32

    ;; Test if/else: false branch should return 78 ('N')
    i32.const 0
    if (result i32)
      i32.const 89  ;; ASCII 'Y'
    else
      i32.const 78  ;; ASCII 'N'
    end
    i32.const 78
    call $assert_eq32

    ;; Test block with br: should return 66 ('B') and skip unreachable
    block (result i32)
      i32.const 66  ;; ASCII 'B'
      br 0  ;; break out of block with value
      ;; This should not execute
      i32.const 88  ;; ASCII 'X'
    end
    i32.const 66
    call $assert_eq32
  )

  (start 1)
)
