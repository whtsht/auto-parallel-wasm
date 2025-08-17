(module
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func $_start
    i32.const 10
    i32.const 5
    i32.add
    i32.const 2
    i32.mul
    i32.const 3
    i32.div_s
    i32.const 1
    i32.sub
    drop
  )
  (start 1)
)