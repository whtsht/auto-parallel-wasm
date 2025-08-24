(module
  (type $test_type (func (result i32)))
  
  (func $test_func (result i32)
    i32.const 42
  )
  
  (table 1 funcref)
  (elem (i32.const 0) $test_func)
  
  (func $start_func
    ;; Valid call (index 0)
    i32.const 0
    call_indirect (type $test_type)
    drop
    
    ;; Invalid call (index 1, out of bounds) - this should trap
    i32.const 1
    call_indirect (type $test_type)
    drop
  )
  
  (start $start_func)
)