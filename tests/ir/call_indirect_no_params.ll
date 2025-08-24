; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@table_0 = global [2 x ptr] [ptr @func_1, ptr @func_2]

declare void @assert_eq32(i32 %0, i32 %1)

declare void @assert_eq64(i64 %0, i64 %1)

define i32 @func_1() {
entry:
  ret i32 42
}

define void @func_2() {
entry:
  ret void
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
  call void @assert_eq32(i32 %indirect_call, i32 42)
  br i1 true, label %valid_call1, label %trap2

do_call:                                          ; preds = %valid_call
  %indirect_call = call i32 %func_ptr()
  br label %after_call

valid_call1:                                      ; preds = %after_call
  %func_ptr4 = load ptr, ptr getelementptr inbounds ([2 x ptr], ptr @table_0, i32 0, i32 1), align 8
  %null_check5 = icmp eq ptr %func_ptr4, null
  br i1 %null_check5, label %trap2, label %do_call6

trap2:                                            ; preds = %valid_call1, %after_call
  unreachable

after_call3:                                      ; preds = %do_call6
  ret void

do_call6:                                         ; preds = %valid_call1
  call void %func_ptr4()
  br label %after_call3
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}
