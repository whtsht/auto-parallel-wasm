; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare void @assert_eq32(i32 %0, i32 %1)

declare void @assert_eq64(i64 %0, i64 %1)

define void @_start() {
entry:
  %local = alloca i32, align 4
  store i32 42, ptr %local, align 4
  %local_load = load i32, ptr %local, align 4
  call void @assert_eq32(i32 %local_load, i32 42)
  %local_load1 = load i32, ptr %local, align 4
  %add = add i32 %local_load1, 8
  store i32 %add, ptr %local, align 4
  call void @assert_eq32(i32 %add, i32 50)
  %local_load2 = load i32, ptr %local, align 4
  call void @assert_eq32(i32 %local_load2, i32 50)
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}