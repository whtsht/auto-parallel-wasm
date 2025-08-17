(module
  (import "env" "putchar" (func $putchar (param i32) (result i32)))
  
  (func $test_i64_bitwise
    ;; Test i64.and
    i64.const 0x0F0F0F0F0F0F0F0F
    i64.const 0xF0F0F0F0F0F0F0F0
    i64.and
    i64.const 0
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.or
    i64.const 0x0F0F0F0F0F0F0F0F
    i64.const 0xF0F0F0F0F0F0F0F0
    i64.or
    i64.const 0xFFFFFFFFFFFFFFFF
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.xor
    i64.const 0x0F0F0F0F0F0F0F0F
    i64.const 0xF0F0F0F0F0F0F0F0
    i64.xor
    i64.const 0xFFFFFFFFFFFFFFFF
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.shl: 1 << 32 = 4294967296
    i64.const 1
    i64.const 32
    i64.shl
    i64.const 4294967296
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.shr_s with negative number
    i64.const -1024
    i64.const 2
    i64.shr_s
    i64.const -256
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.shr_u with negative number (unsigned shift)
    i64.const -1024
    i64.const 2
    i64.shr_u
    i64.const 4611686018427387648
    i64.gt_u
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.rotl
    i64.const 0x123456789ABCDEF0
    i64.const 8
    i64.rotl
    i64.const 0x3456789ABCDEF012
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test i64.rotr
    i64.const 0x123456789ABCDEF0
    i64.const 8
    i64.rotr
    i64.const 0xF0123456789ABCDE
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test shift with large count (masked to 6 bits for i64)
    i64.const 1
    i64.const 68
    i64.shl
    i64.const 16
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Test rotate with boundary values
    i64.const 0x8000000000000000
    i64.const 1
    i64.rotl
    i64.const 1
    i64.eq
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; Newline
    i32.const 10
    call $putchar
    drop
  )
  
  (start $test_i64_bitwise)
)