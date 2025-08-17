(module
  (func $putchar (import "" "putchar") (param i32) (result i32))
  (func $_start (export "_start")
    (local $counter i32)
    ;; Initialize counter to 0
    i32.const 0
    local.set $counter
    ;; Loop: for (counter = 0; counter < 3; counter++)
    loop
      ;; Print current counter value + '0' (ASCII)
      local.get $counter
      i32.const 48  ;; ASCII '0'
      i32.add
      call $putchar
      drop
      ;; Increment counter
      local.get $counter
      i32.const 1
      i32.add
      local.set $counter
      ;; Check if counter < 3, continue loop if true
      local.get $counter
      i32.const 3
      i32.lt_s
      br_if 0  ;; continue loop (jump to loop start)
    end
  )
  (start 1)
)
