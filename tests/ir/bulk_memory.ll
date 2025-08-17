; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@memory = global [65536 x i8] zeroinitializer

define void @_start() {
entry:
  store i32 305419896, ptr @memory, align 4
  br i1 false, label %continue, label %bounds_check

bounds_check:                                     ; preds = %entry
  br i1 false, label %trap, label %continue

trap:                                             ; preds = %bounds_check
  unreachable

continue:                                         ; preds = %bounds_check, %entry
  br i1 false, label %continue3, label %bounds_check1

bounds_check1:                                    ; preds = %continue
  br i1 false, label %trap2, label %continue3

trap2:                                            ; preds = %bounds_check1
  unreachable

continue3:                                        ; preds = %bounds_check1, %continue
  call void @llvm.memmove.p0.p0.i32(ptr getelementptr (i8, ptr @memory, i32 4), ptr @memory, i32 4, i1 false)
  br i1 false, label %continue6, label %bounds_check4

bounds_check4:                                    ; preds = %continue3
  br i1 false, label %trap5, label %continue6

trap5:                                            ; preds = %bounds_check4
  unreachable

continue6:                                        ; preds = %bounds_check4, %continue3
  call void @llvm.memset.p0.i32(ptr getelementptr (i8, ptr @memory, i32 8), i8 -1, i32 4, i1 false)
  %load = load i32, ptr getelementptr (i8, ptr @memory, i32 4), align 4
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memmove.p0.p0.i32(ptr nocapture writeonly %0, ptr nocapture readonly %1, i32 %2, i1 immarg %3) #0

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i32(ptr nocapture writeonly %0, i8 %1, i32 %2, i1 immarg %3) #1

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}

attributes #0 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #1 = { nocallback nofree nounwind willreturn memory(argmem: write) }