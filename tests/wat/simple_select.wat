(module
  (func $test_select_true (result i32)
    i32.const 10
    i32.const 20
    i32.const 1
    select
  )
  
  (func $test_select_false (result i32)
    i32.const 10
    i32.const 20
    i32.const 0
    select
  )
  
  (func $main
    call $test_select_true
    drop
    call $test_select_false
    drop
  )
  
  (start $main)
)