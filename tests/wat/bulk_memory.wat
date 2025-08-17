(module
  (memory 1)
  (func $test_bulk_memory
    ;; Initialize memory
    i32.const 0
    i32.const 0x12345678
    i32.store
    
    ;; Test memory.copy
    i32.const 4
    i32.const 0
    i32.const 4
    memory.copy
    
    ;; Test memory.fill
    i32.const 8
    i32.const 0xFF
    i32.const 4
    memory.fill
    
    ;; Load and check results
    i32.const 4
    i32.load
    drop
  )
  (start $test_bulk_memory)
)