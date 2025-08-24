; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@table_0 = global [1 x ptr] [ptr @func_1]

define i32 @func_1() {
entry:
  ret i32 42
}

define void @_start() {
entry:
  br i1 true, label %valid_call, label %trap

valid_call:                                       ; preds = %entry
  %func_ptr = load ptr, ptr @table_0, align 8
  %null_check = icmp eq ptr %func_ptr, null
  br i1 %null_check, label %trap, label %do_call

trap:                                             ; preds = %valid_call, %entry
  unreachable

after_call:                                       ; preds = %do_call
  ret void

do_call:                                          ; preds = %valid_call
  %indirect_call = call i32 %func_ptr()
  br label %after_call
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}
