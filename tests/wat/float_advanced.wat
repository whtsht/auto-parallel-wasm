(module
  (func $test_float_ops
    f32.const -3.14
    f32.abs
    f32.const 2.0
    f32.add
    f32.sqrt
    f32.ceil
    drop
  )
  (start $test_float_ops)
)