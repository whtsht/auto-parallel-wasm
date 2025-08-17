; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i32 @putchar(i32 %0)

define void @_start() {
entry:
  %putchar = call i32 @putchar(i32 50)
  %putchar1 = call i32 @putchar(i32 48)
  %putchar2 = call i32 @putchar(i32 54)
  %putchar3 = call i32 @putchar(i32 50)
  %putchar4 = call i32 @putchar(i32 50)
  %putchar5 = call i32 @putchar(i32 53)
  %putchar6 = call i32 @putchar(i32 48)
  %putchar7 = call i32 @putchar(i32 49)
  %putchar8 = call i32 @putchar(i32 49)
  %putchar9 = call i32 @putchar(i32 49)
  %putchar10 = call i32 @putchar(i32 49)
  %putchar11 = call i32 @putchar(i32 48)
  %putchar12 = call i32 @putchar(i32 49)
  %putchar13 = call i32 @putchar(i32 10)
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}
