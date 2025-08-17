(module
  (func $test_type_conversions
    ;; Test integer wrap and extend conversions
    i64.const 0x123456789ABCDEF0
    i32.wrap_i64                  ;; Should give 0x9ABCDEF0 (lower 32 bits)
    drop
    
    i32.const -1
    i64.extend_i32_s              ;; Should give 0xFFFFFFFFFFFFFFFF
    drop
    
    i32.const -1
    i64.extend_i32_u              ;; Should give 0x00000000FFFFFFFF
    drop
    
    ;; Test integer to float conversions
    i32.const -42
    f32.convert_i32_s             ;; Should give -42.0
    drop
    
    i32.const 42
    f32.convert_i32_u             ;; Should give 42.0
    drop
    
    i32.const -42
    f64.convert_i32_s             ;; Should give -42.0
    drop
    
    i32.const 42
    f64.convert_i32_u             ;; Should give 42.0
    drop
    
    ;; Test float to integer conversions
    f32.const -42.7
    i32.trunc_f32_s               ;; Should give -42
    drop
    
    f32.const 42.7
    i32.trunc_f32_u               ;; Should give 42
    drop
    
    f64.const -42.7
    i32.trunc_f64_s               ;; Should give -42
    drop
    
    f64.const 42.7
    i32.trunc_f64_u               ;; Should give 42
    drop
    
    ;; Test float promotion and demotion
    f32.const 3.14159
    f64.promote_f32               ;; Promote f32 to f64
    drop
    
    f64.const 2.718281828459045
    f32.demote_f64                ;; Demote f64 to f32 (may lose precision)
    drop
    
    ;; Test conversion chain
    i32.const 100
    f32.convert_i32_s             ;; 100 -> 100.0 (f32)
    f64.promote_f32               ;; 100.0 (f32) -> 100.0 (f64)
    i32.trunc_f64_s               ;; 100.0 (f64) -> 100 (i32)
    i64.extend_i32_s              ;; 100 (i32) -> 100 (i64)
    i32.wrap_i64                  ;; 100 (i64) -> 100 (i32)
    drop
  )
  
  (start $test_type_conversions)
)