(module
  (func $test_f64_operations
    ;; Test f64 unary operations
    f64.const -2.718281828
    f64.abs
    f64.sqrt
    f64.neg
    drop
    
    ;; Test f64 rounding operations
    f64.const 3.14159265359
    f64.ceil
    f64.const 2.71828
    f64.floor
    f64.add
    f64.const 1.41421
    f64.trunc
    f64.add
    drop
    
    ;; Test f64 min/max operations
    f64.const 1.0e10
    f64.const 2.0e10
    f64.min
    f64.const 3.0e10
    f64.const 4.0e10
    f64.max
    f64.add
    drop
    
    ;; Test f64 copysign
    f64.const 42.0
    f64.const -1.0
    f64.copysign
    drop
  )
  (start $test_f64_operations)
)