(module
  (func $putchar (import "" "putchar") (param i32) (result i32))
  
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  
  (func $_start
    ;; 計算: 10 + 5 = 15
    i32.const 10
    i32.const 5
    i32.add
    
    ;; 十の位を表示 (15 / 10 + '0' = 1 + 48 = 49 = '1')
    i32.const 10
    i32.div_s
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; 一の位を表示 (15 % 10 + '0' = 5 + 48 = 53 = '5')
    i32.const 10
    i32.const 5
    i32.add
    i32.const 10
    i32.rem_s
    i32.const 48
    i32.add
    call $putchar
    drop
  )
  
  (start 2)
)