; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare void @assert_eq32(i32 %0, i32 %1)

declare void @assert_eq64(i64 %0, i64 %1)

define void @_start() {
entry:
  call void @assert_eq32(i32 42, i32 42)
  call void @assert_eq32(i32 15, i32 15)
  call void @assert_eq32(i32 35, i32 35)
  call void @assert_eq32(i32 0, i32 0)
  call void @assert_eq32(i32 -10, i32 -10)
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}