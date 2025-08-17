; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i32 @putchar(i32 %0)

define void @_start() {
entry:
  br i1 true, label %if_then, label %if_else

if_then:                                          ; preds = %entry
  %putchar = call i32 @putchar(i32 89)
  br label %if_merge

if_else:                                          ; preds = %entry
  %putchar1 = call i32 @putchar(i32 78)
  br label %if_merge

if_merge:                                         ; preds = %if_else, %if_then
  br label %block

block:                                            ; preds = %if_merge
  %putchar2 = call i32 @putchar(i32 66)
  br label %block_end

block_end:                                        ; preds = %unreachable, %block
  ret void

unreachable:                                      ; No predecessors!
  %putchar3 = call i32 @putchar(i32 88)
  br label %block_end
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}