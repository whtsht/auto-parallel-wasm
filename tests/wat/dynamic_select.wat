(module
  (func $test_select (param $cond i32) (result i32)
    i32.const 10
    i32.const 20
    local.get $cond
    select
  )
  
  (func $main
    i32.const 1
    call $test_select
    drop
  )
  
  (start $main)
)