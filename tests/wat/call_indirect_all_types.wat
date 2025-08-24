(module
  (import "env" "assert_eq32" (func $assert_eq32 (param i32 i32)))
  
  ;; All 15 function types from hello_world.wasm requirements
  (type $type0 (func))                                          ;; () -> nil
  (type $type1 (func (param i32)))                              ;; (i32) -> nil  
  (type $type2 (func (param i32 i32) (result i32)))             ;; (i32, i32) -> i32
  (type $type3 (func (param i32) (result i32)))                 ;; (i32) -> i32
  (type $type4 (func (param i32 i32 i32)))                      ;; (i32, i32, i32) -> nil
  (type $type5 (func (param i32 i32 i32) (result i32)))         ;; (i32, i32, i32) -> i32
  (type $type6 (func (param i32 i32)))                          ;; (i32, i32) -> nil
  (type $type7 (func (param i32 i32 i32 i32)))                  ;; (i32, i32, i32, i32) -> nil
  (type $type8 (func (param i32 i32 i32 i32) (result i32)))     ;; (i32, i32, i32, i32) -> i32
  (type $type9 (func (result i32)))                             ;; () -> i32
  (type $type10 (func (param i32 i32 i32 i32 i32)))             ;; (i32, i32, i32, i32, i32) -> nil
  (type $type11 (func (param i32 i32 i32 i32 i32) (result i32))) ;; (i32, i32, i32, i32, i32) -> i32
  (type $type12 (func (param i32 i32 i32 i32 i32 i32) (result i32))) ;; (i32, i32, i32, i32, i32, i32) -> i32
  (type $type13 (func (param i32 i32 i32 i32 i32 i32 i32)))     ;; (i32, i32, i32, i32, i32, i32, i32) -> nil
  (type $type14 (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32))) ;; (i32 x 11) -> i32
  
  ;; Test functions for each type
  (func $func0 (type $type0))                                   ;; Does nothing
  (func $func1 (type $type1) (param i32))                       ;; Takes i32, does nothing
  (func $func2 (type $type2) (param i32 i32) (result i32)       
    local.get 0 
    local.get 1 
    i32.add)
  (func $func3 (type $type3) (param i32) (result i32)           
    local.get 0 
    i32.const 10 
    i32.mul)
  (func $func4 (type $type4) (param i32 i32 i32))               ;; Takes 3 i32s, does nothing
  (func $func5 (type $type5) (param i32 i32 i32) (result i32)   
    local.get 0 
    local.get 1 
    i32.add 
    local.get 2 
    i32.add)
  (func $func6 (type $type6) (param i32 i32))                   ;; Takes 2 i32s, does nothing
  (func $func7 (type $type7) (param i32 i32 i32 i32))           ;; Takes 4 i32s, does nothing
  (func $func8 (type $type8) (param i32 i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    local.get 2
    i32.add
    local.get 3
    i32.add)
  (func $func9 (type $type9) (result i32)                       
    i32.const 42)
  (func $func10 (type $type10) (param i32 i32 i32 i32 i32))     ;; Takes 5 i32s, does nothing
  (func $func11 (type $type11) (param i32 i32 i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    local.get 2
    i32.add
    local.get 3
    i32.add
    local.get 4
    i32.add)
  (func $func12 (type $type12) (param i32 i32 i32 i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    local.get 2
    i32.add
    local.get 3
    i32.add
    local.get 4
    i32.add
    local.get 5
    i32.add)
  (func $func13 (type $type13) (param i32 i32 i32 i32 i32 i32 i32)) ;; Takes 7 i32s, does nothing
  (func $func14 (type $type14) (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    local.get 2
    i32.add
    local.get 3
    i32.add
    local.get 4
    i32.add
    local.get 5
    i32.add
    local.get 6
    i32.add
    local.get 7
    i32.add
    local.get 8
    i32.add
    local.get 9
    i32.add
    local.get 10
    i32.add)
  
  (table 15 funcref)
  (elem (i32.const 0) 
    $func0 $func1 $func2 $func3 $func4 $func5 $func6 $func7 
    $func8 $func9 $func10 $func11 $func12 $func13 $func14)
  
  (func $start_func
    ;; Test type0: () -> nil
    i32.const 0
    call_indirect (type $type0)
    
    ;; Test type1: (i32) -> nil
    i32.const 100
    i32.const 1
    call_indirect (type $type1)
    
    ;; Test type2: (i32, i32) -> i32, expect 5+7=12
    i32.const 5
    i32.const 7
    i32.const 2
    call_indirect (type $type2)
    i32.const 12
    call $assert_eq32
    
    ;; Test type3: (i32) -> i32, expect 5*10=50
    i32.const 5
    i32.const 3
    call_indirect (type $type3)
    i32.const 50
    call $assert_eq32
    
    ;; Test type4: (i32, i32, i32) -> nil
    i32.const 1
    i32.const 2
    i32.const 3
    i32.const 4
    call_indirect (type $type4)
    
    ;; Test type5: (i32, i32, i32) -> i32, expect 1+2+3=6
    i32.const 1
    i32.const 2
    i32.const 3
    i32.const 5
    call_indirect (type $type5)
    i32.const 6
    call $assert_eq32
    
    ;; Test type9: () -> i32, expect 42
    i32.const 9
    call_indirect (type $type9)
    i32.const 42
    call $assert_eq32
    
    ;; Test type8: (i32, i32, i32, i32) -> i32, expect 1+2+3+4=10
    i32.const 1
    i32.const 2
    i32.const 3
    i32.const 4
    i32.const 8
    call_indirect (type $type8)
    i32.const 10
    call $assert_eq32
    
    ;; Test type11: (i32 x 5) -> i32, expect 1+2+3+4+5=15
    i32.const 1
    i32.const 2
    i32.const 3
    i32.const 4
    i32.const 5
    i32.const 11
    call_indirect (type $type11)
    i32.const 15
    call $assert_eq32
  )
  
  (start $start_func)
)