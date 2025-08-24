(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))
  
  (type $add_type (func (param i32 i32) (result i32)))
  (type $void_type (func (param i32) (result)))
  (type $triple_type (func (param i32 i32 i32) (result i32)))
  
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  
  (func $multiply_triple (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.mul
    local.get 2
    i32.add
  )
  
  (func $void_func (param i32)
    ;; Does nothing, just for testing void return
  )
  
  (table 3 funcref)
  (elem (i32.const 0) $add $multiply_triple $void_func)
  
  (func $start_func
    ;; Test add function: 10 + 20 = 30
    i32.const 10
    i32.const 20
    i32.const 0
    call_indirect (type $add_type)
    i32.const 30
    call $assert_eq32
    
    ;; Test triple function: (5 * 6) + 7 = 37
    i32.const 5
    i32.const 6
    i32.const 7
    i32.const 1
    call_indirect (type $triple_type)
    i32.const 37
    call $assert_eq32
    
    ;; Test void function
    i32.const 42
    i32.const 2
    call_indirect (type $void_type)
  )
  
  (start $start_func)
)