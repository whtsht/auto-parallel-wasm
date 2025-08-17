; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

define i32 @func_0() {
entry:
  ret i32 20
}

define i32 @func_1() {
entry:
  %clz = call i32 @llvm.ctlz.i32(i32 -2147483648, i1 false)
  %ctz = call i32 @llvm.cttz.i32(i32 1, i1 false)
  %add = add i32 %clz, %ctz
  %popcnt = call i32 @llvm.ctpop.i32(i32 305419896)
  %add1 = add i32 %add, %popcnt
  ret i32 %add1
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.ctlz.i32(i32 %0, i1 immarg %1) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.cttz.i32(i32 %0, i1 immarg %1) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.ctpop.i32(i32 %0) #0

define i64 @func_2() {
entry:
  %clz = call i64 @llvm.ctlz.i64(i64 -9223372036854775808, i1 false)
  %ctz = call i64 @llvm.cttz.i64(i64 1, i1 false)
  %add64 = add i64 %clz, %ctz
  %popcnt = call i64 @llvm.ctpop.i64(i64 1311768467463790320)
  %add641 = add i64 %add64, %popcnt
  ret i64 %add641
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.ctlz.i64(i64 %0, i1 immarg %1) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.cttz.i64(i64 %0, i1 immarg %1) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.ctpop.i64(i64 %0) #0

define void @_start() {
entry:
  %call = call i32 @func_0()
  %call1 = call i32 @func_1()
  %add = add i32 %call, %call1
  %call2 = call i64 @func_2()
  %wrap_i64 = trunc i64 %call2 to i32
  %add3 = add i32 %add, %wrap_i64
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}

attributes #0 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }