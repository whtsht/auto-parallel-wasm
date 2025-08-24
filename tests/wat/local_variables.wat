(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))
  
  (func $_start (local i32)
    ;; Test: Store 42 in local variable
    i32.const 42
    local.set 0
    
    ;; Test: Load from local variable (should be 42)
    local.get 0
    i32.const 42
    call $assert_eq32
    
    ;; Test: Modify local variable (42 + 8 = 50)
    local.get 0
    i32.const 8
    i32.add
    local.tee 0
    i32.const 50
    call $assert_eq32
    
    ;; Test: Local variable after modification (should be 50)
    local.get 0
    i32.const 50
    call $assert_eq32
  )
  
  (start 1)
)
