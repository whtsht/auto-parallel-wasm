; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@memory = global [65536 x i8] zeroinitializer

declare i32 @putchar(i32 %0)

define void @_start() {
entry:
  store i8 -1, ptr @memory, align 1
  %load = load i8, ptr @memory, align 1
  %sext = sext i8 %load to i32
  %load1 = load i8, ptr @memory, align 1
  %zext = zext i8 %load1 to i32
  store i16 4660, ptr getelementptr (i8, ptr @memory, i32 4), align 2
  %load2 = load i16, ptr getelementptr (i8, ptr @memory, i32 4), align 2
  %sext3 = sext i16 %load2 to i32
  %load4 = load i16, ptr getelementptr (i8, ptr @memory, i32 4), align 2
  %zext5 = zext i16 %load4 to i32
  store i8 -1, ptr getelementptr (i8, ptr @memory, i32 8), align 1
  %load6 = load i8, ptr getelementptr (i8, ptr @memory, i32 8), align 1
  %sext7 = sext i8 %load6 to i64
  %load8 = load i8, ptr getelementptr (i8, ptr @memory, i32 8), align 1
  %zext9 = zext i8 %load8 to i64
  store i32 305419896, ptr getelementptr (i8, ptr @memory, i32 16), align 4
  %load10 = load i32, ptr getelementptr (i8, ptr @memory, i32 16), align 4
  %sext11 = sext i32 %load10 to i64
  %load12 = load i32, ptr getelementptr (i8, ptr @memory, i32 16), align 4
  %zext13 = zext i32 %load12 to i64
  store i16 -26506, ptr getelementptr (i8, ptr @memory, i32 24), align 2
  %load14 = load i16, ptr getelementptr (i8, ptr @memory, i32 24), align 2
  %sext15 = sext i16 %load14 to i64
  %load16 = load i16, ptr getelementptr (i8, ptr @memory, i32 24), align 2
  %zext17 = zext i16 %load16 to i64
  %putchar = call i32 @putchar(i32 79)
  %putchar18 = call i32 @putchar(i32 75)
  ret void
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}
