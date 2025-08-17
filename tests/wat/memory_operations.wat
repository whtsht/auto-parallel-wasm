(module
  (memory 1)
  (func $_start
    i32.const 0
    i32.const 42
    i32.store
    
    i32.const 4
    i32.const 100
    i32.store
    
    i32.const 0
    i32.load
    drop
    
    memory.size
    drop
  )
  (start 0)
)