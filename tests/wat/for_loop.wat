(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))
  
  (func $_start (export "_start")
    (local $counter i32)
    (local $sum i32)
    
    ;; Initialize counter to 0
    i32.const 0
    local.set $counter
    i32.const 0
    local.set $sum
    
    ;; Loop: for (counter = 0; counter < 3; counter++)
    loop
      ;; Add counter to sum
      local.get $sum
      local.get $counter
      i32.add
      local.set $sum
      
      ;; Increment counter
      local.get $counter
      i32.const 1
      i32.add
      local.set $counter
      
      ;; Check if counter < 3, continue loop if true
      local.get $counter
      i32.const 3
      i32.lt_s
      br_if 0  ;; continue loop (jump to loop start)
    end
    
    ;; Test: sum should be 0+1+2 = 3
    local.get $sum
    i32.const 3
    call $assert_eq32
    
    ;; Test: final counter should be 3
    local.get $counter
    i32.const 3
    call $assert_eq32
  )
  
  (start 1)
)
