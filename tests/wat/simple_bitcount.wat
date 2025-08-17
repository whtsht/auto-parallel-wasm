(module
  (func $test_i32_clz (result i32)
    i32.const 0x80000000
    i32.clz
  )
  
  (func $test_i32_ctz (result i32)
    i32.const 0x00000001
    i32.ctz
  )
  
  (func $test_i32_popcnt (result i32)
    i32.const 0x12345678
    i32.popcnt
  )
  
  (func $main
    call $test_i32_clz
    drop
    call $test_i32_ctz
    drop
    call $test_i32_popcnt
    drop
  )
  
  (start $main)
)