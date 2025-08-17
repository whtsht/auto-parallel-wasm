; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i32 @putchar(i32 %0)

define i32 @func_1(i32 %0, i32 %1) {
entry:
  %add = add i32 %0, %1
  ret i32 %add
}

define void @_start() {
entry:
  %putchar = call i32 @putchar(i32 49)
  %putchar1 = call i32 @putchar(i32 53)
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}