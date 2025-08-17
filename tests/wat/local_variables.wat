(module
  (func $putchar (import "" "putchar") (param i32) (result i32))
  
  (func $_start (local i32)
    ;; ローカル変数に42を格納
    i32.const 42
    local.set 0
    
    ;; ローカル変数から読み取って表示
    local.get 0
    
    ;; 十の位を表示 (42 / 10 + '0' = 4 + 48 = 52 = '4')
    i32.const 10
    i32.div_s
    i32.const 48
    i32.add
    call $putchar
    drop
    
    ;; 一の位を表示 (42 % 10 + '0' = 2 + 48 = 50 = '2')
    local.get 0
    i32.const 10
    i32.rem_s
    i32.const 48
    i32.add
    call $putchar
    drop
  )
  
  (start 1)
)