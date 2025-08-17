; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i32 @putchar(i32 %0)

define void @_start() {
entry:
  %local = alloca i32, align 4
  store i32 0, ptr %local, align 4
  br label %loop_header

loop_header:                                      ; preds = %loop_header, %entry
  %local_load = load i32, ptr %local, align 4
  %add = add i32 %local_load, 48
  %putchar = call i32 @putchar(i32 %add)
  %local_load1 = load i32, ptr %local, align 4
  %add2 = add i32 %local_load1, 1
  store i32 %add2, ptr %local, align 4
  %local_load3 = load i32, ptr %local, align 4
  %lt = icmp slt i32 %local_load3, 3
  %lt_ext = zext i1 %lt to i32
  %br_if_cond = icmp ne i32 %lt_ext, 0
  br i1 %br_if_cond, label %loop_header, label %br_if_continue

loop_end:                                         ; preds = %br_if_continue
  ret void

br_if_continue:                                   ; preds = %loop_header
  br label %loop_end
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}