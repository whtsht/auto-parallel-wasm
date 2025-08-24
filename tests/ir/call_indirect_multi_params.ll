; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@table_0 = global [3 x ptr] [ptr @func_1, ptr @func_2, ptr @func_3]

declare void @assert_eq32(i32 %0, i32 %1)

declare void @assert_eq64(i64 %0, i64 %1)

define i32 @func_1(i32 %0, i32 %1) {
entry:
  %add = add i32 %0, %1
  ret i32 %add
}

define i32 @func_2(i32 %0, i32 %1, i32 %2) {
entry:
  %mul = mul i32 %0, %1
  %add = add i32 %mul, %2
  ret i32 %add
}

define void @func_3(i32 %0) {
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
  call void @assert_eq32(i32 %indirect_call, i32 30)
  br i1 true, label %valid_call1, label %trap2

do_call:                                          ; preds = %valid_call
  %indirect_call = call i32 %func_ptr(i32 10, i32 20)
  br label %after_call

valid_call1:                                      ; preds = %after_call
  %func_ptr4 = load ptr, ptr getelementptr inbounds ([3 x ptr], ptr @table_0, i32 0, i32 1), align 8
  %null_check5 = icmp eq ptr %func_ptr4, null
  br i1 %null_check5, label %trap2, label %do_call6

trap2:                                            ; preds = %valid_call1, %after_call
  unreachable

after_call3:                                      ; preds = %do_call6
  call void @assert_eq32(i32 %indirect_call7, i32 37)
  br i1 true, label %valid_call8, label %trap9

do_call6:                                         ; preds = %valid_call1
  %indirect_call7 = call i32 %func_ptr4(i32 5, i32 6, i32 7)
  br label %after_call3

valid_call8:                                      ; preds = %after_call3
  %func_ptr11 = load ptr, ptr getelementptr inbounds ([3 x ptr], ptr @table_0, i32 0, i32 2), align 8
  %null_check12 = icmp eq ptr %func_ptr11, null
  br i1 %null_check12, label %trap9, label %do_call13

trap9:                                            ; preds = %valid_call8, %after_call3
  unreachable

after_call10:                                     ; preds = %do_call13
  ret void

do_call13:                                        ; preds = %valid_call8
  call void %func_ptr11(i32 42)
  br label %after_call10
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}
