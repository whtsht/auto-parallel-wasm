; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

define void @_start() {
entry:
  %local = alloca i32, align 4
  ret void
}

define i32 @main() {
entry:
  ret i32 0
}