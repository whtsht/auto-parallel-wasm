; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@memory = global [65536 x i8] zeroinitializer

define void @_start() {
entry:
  store i32 42, ptr @memory, align 4
  store i32 100, ptr getelementptr (i8, ptr @memory, i32 4), align 4
  %load = load i32, ptr @memory, align 4
  ret void
}

define i32 @main() {
entry:
  ret i32 0
}