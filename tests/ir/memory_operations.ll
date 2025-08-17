; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@memory = global [65536 x i8] zeroinitializer

declare i32 @putchar(i32 %0)

define void @_start() {
entry:
  store i32 42, ptr @memory, align 4
  store i32 100, ptr getelementptr (i8, ptr @memory, i32 4), align 4
  %load = load i32, ptr @memory, align 4
  %div = sdiv i32 %load, 10
  %add = add i32 %div, 48
  %putchar = call i32 @putchar(i32 %add)
  %load1 = load i32, ptr @memory, align 4
  %rem = srem i32 %load1, 10
  %add2 = add i32 %rem, 48
  %putchar3 = call i32 @putchar(i32 %add2)
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}