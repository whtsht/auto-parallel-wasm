(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))
  
  (func $_start
    ;; Test: 10 + 5 = 15
    i32.const 10
    i32.const 5
    i32.add
    i32.const 15
    call $assert_eq32
    
    ;; Test: multiplication (3 * 5 = 15)
    i32.const 3
    i32.const 5
    i32.mul
    i32.const 15
    call $assert_eq32
    
    ;; Test: division (30 / 2 = 15)
    i32.const 30
    i32.const 2
    i32.div_s
    i32.const 15
    call $assert_eq32
  )
  
  (start $_start)
)
