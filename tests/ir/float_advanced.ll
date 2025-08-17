; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

define void @_start() {
entry:
  %fabs32 = call float @llvm.fabs.f32(float 0xC0091EB860000000)
  %fadd32 = fadd float %fabs32, 2.000000e+00
  %fsqrt32 = call float @llvm.sqrt.f32(float %fadd32)
  %fceil32 = call float @llvm.ceil.f32(float %fsqrt32)
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare float @llvm.fabs.f32(float %0) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare float @llvm.sqrt.f32(float %0) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare float @llvm.ceil.f32(float %0) #0

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}

attributes #0 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }