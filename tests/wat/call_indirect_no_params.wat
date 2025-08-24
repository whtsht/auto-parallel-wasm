(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))
  
  (type $no_param_i32 (func (result i32)))
  (type $no_param_void (func))
  
  (func $get_42 (result i32)
    i32.const 42
  )
  
  (func $void_func
    ;; Does nothing
  )
  
  (table 2 funcref)
  (elem (i32.const 0) $get_42 $void_func)
  
  (func $start_func
    ;; Test no-param function returning i32
    i32.const 0
    call_indirect (type $no_param_i32)
    i32.const 42
    call $assert_eq32
    
    ;; Test no-param void function
    i32.const 1
    call_indirect (type $no_param_void)
  )
  
  (start $start_func)
)