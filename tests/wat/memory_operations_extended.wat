(module
  (func $putchar (import "" "putchar") (param i32) (result i32))
  (memory 1)
  (func $_start
    ;; Test i32 partial loads/stores
    ;; Store -1 (0xFF) as 8-bit
    i32.const 0
    i32.const 255
    i32.store8
    
    ;; Load with sign extension (should get -1)
    i32.const 0
    i32.load8_s
    drop
    
    ;; Load with zero extension (should get 255)
    i32.const 0
    i32.load8_u
    drop
    
    ;; Store 0x1234 as 16-bit
    i32.const 4
    i32.const 0x1234
    i32.store16
    
    ;; Load 16-bit with sign extension
    i32.const 4
    i32.load16_s
    drop
    
    ;; Load 16-bit with zero extension
    i32.const 4
    i32.load16_u
    drop
    
    ;; Test i64 partial loads/stores
    ;; Store 0xFFFFFFFFFFFFFFFF as 8-bit
    i32.const 8
    i64.const 0xFF
    i64.store8
    
    ;; Load 8-bit to i64 with sign extension
    i32.const 8
    i64.load8_s
    drop
    
    ;; Load 8-bit to i64 with zero extension
    i32.const 8
    i64.load8_u
    drop
    
    ;; Store 0x12345678 as 32-bit
    i32.const 16
    i64.const 0x12345678
    i64.store32
    
    ;; Load 32-bit to i64 with sign extension
    i32.const 16
    i64.load32_s
    drop
    
    ;; Load 32-bit to i64 with zero extension
    i32.const 16
    i64.load32_u
    drop
    
    ;; Test i64 16-bit operations
    i32.const 24
    i64.const 0x9876
    i64.store16
    
    i32.const 24
    i64.load16_s
    drop
    
    i32.const 24
    i64.load16_u
    drop
    
    ;; Print success indicator
    i32.const 79  ;; 'O'
    call $putchar
    drop
    
    i32.const 75  ;; 'K'
    call $putchar
    drop
  )
  (start 1)
)