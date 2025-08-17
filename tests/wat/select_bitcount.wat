(module
  (func $test_select (result i32)
    i32.const 10
    i32.const 20
    i32.const 1
    select
  )
  
  (func $test_bitcount (result i32)
    i32.const 0x80000000
    i32.clz
    i32.const 0x00000001
    i32.ctz
    i32.add
    i32.const 0x12345678
    i32.popcnt
    i32.add
  )
  
  (func $test_i64_bitcount (result i64)
    i64.const 0x8000000000000000
    i64.clz
    i64.const 0x0000000000000001
    i64.ctz
    i64.add
    i64.const 0x123456789ABCDEF0
    i64.popcnt
    i64.add
  )
  
  (func $_start
    call $test_select
    call $test_bitcount
    i32.add
    call $test_i64_bitcount
    i32.wrap_i64
    i32.add
    drop
  )
  
  (start $_start)
)