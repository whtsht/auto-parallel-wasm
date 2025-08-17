(module
  (func $putchar (import "" "putchar") (param i32) (result i32))

  (func $_start (export "_start")
    ;; Test if/else: print 'Y' for true branch
    i32.const 1
    if
      i32.const 89  ;; ASCII 'Y'
      call $putchar
      drop
    else
      i32.const 78  ;; ASCII 'N'
      call $putchar
      drop
    end

    ;; Test block with br: print 'B' and skip rest
    block
      i32.const 66  ;; ASCII 'B'
      call $putchar
      drop
      br 0  ;; break out of block
      ;; This should not execute
      i32.const 88  ;; ASCII 'X'
      call $putchar
      drop
    end
  )

  (start 1)
)
