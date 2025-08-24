; ModuleID = 'wasm_aot'
source_filename = "wasm_aot"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

@table_0 = global [15 x ptr] [ptr @func_1, ptr @func_2, ptr @func_3, ptr @func_4, ptr @func_5, ptr @func_6, ptr @func_7, ptr @func_8, ptr @func_9, ptr @func_10, ptr @func_11, ptr @func_12, ptr @func_13, ptr @func_14, ptr @func_15]

declare void @assert_eq32(i32 %0, i32 %1)

declare void @assert_eq64(i64 %0, i64 %1)

define void @func_1() {
entry:
  ret void
}

define void @func_2(i32 %0) {
entry:
  ret void
}

define i32 @func_3(i32 %0, i32 %1) {
entry:
  %add = add i32 %0, %1
  ret i32 %add
}

define i32 @func_4(i32 %0) {
entry:
  %mul = mul i32 %0, 10
  ret i32 %mul
}

define void @func_5(i32 %0, i32 %1, i32 %2) {
entry:
  ret void
}

define i32 @func_6(i32 %0, i32 %1, i32 %2) {
entry:
  %add = add i32 %0, %1
  %add1 = add i32 %add, %2
  ret i32 %add1
}

define void @func_7(i32 %0, i32 %1) {
entry:
  ret void
}

define void @func_8(i32 %0, i32 %1, i32 %2, i32 %3) {
entry:
  ret void
}

define i32 @func_9(i32 %0, i32 %1, i32 %2, i32 %3) {
entry:
  %add = add i32 %0, %1
  %add1 = add i32 %add, %2
  %add2 = add i32 %add1, %3
  ret i32 %add2
}

define i32 @func_10() {
entry:
  ret i32 42
}

define void @func_11(i32 %0, i32 %1, i32 %2, i32 %3, i32 %4) {
entry:
  ret void
}

define i32 @func_12(i32 %0, i32 %1, i32 %2, i32 %3, i32 %4) {
entry:
  %add = add i32 %0, %1
  %add1 = add i32 %add, %2
  %add2 = add i32 %add1, %3
  %add3 = add i32 %add2, %4
  ret i32 %add3
}

define i32 @func_13(i32 %0, i32 %1, i32 %2, i32 %3, i32 %4, i32 %5) {
entry:
  %add = add i32 %0, %1
  %add1 = add i32 %add, %2
  %add2 = add i32 %add1, %3
  %add3 = add i32 %add2, %4
  %add4 = add i32 %add3, %5
  ret i32 %add4
}

define void @func_14(i32 %0, i32 %1, i32 %2, i32 %3, i32 %4, i32 %5, i32 %6) {
entry:
  ret void
}

define i32 @func_15(i32 %0, i32 %1, i32 %2, i32 %3, i32 %4, i32 %5, i32 %6, i32 %7, i32 %8, i32 %9, i32 %10) {
entry:
  %add = add i32 %0, %1
  %add1 = add i32 %add, %2
  %add2 = add i32 %add1, %3
  %add3 = add i32 %add2, %4
  %add4 = add i32 %add3, %5
  %add5 = add i32 %add4, %6
  %add6 = add i32 %add5, %7
  %add7 = add i32 %add6, %8
  %add8 = add i32 %add7, %9
  %add9 = add i32 %add8, %10
  ret i32 %add9
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
  br i1 true, label %valid_call1, label %trap2

do_call:                                          ; preds = %valid_call
  call void %func_ptr()
  br label %after_call

valid_call1:                                      ; preds = %after_call
  %func_ptr4 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 1), align 8
  %null_check5 = icmp eq ptr %func_ptr4, null
  br i1 %null_check5, label %trap2, label %do_call6

trap2:                                            ; preds = %valid_call1, %after_call
  unreachable

after_call3:                                      ; preds = %do_call6
  br i1 true, label %valid_call7, label %trap8

do_call6:                                         ; preds = %valid_call1
  call void %func_ptr4(i32 100)
  br label %after_call3

valid_call7:                                      ; preds = %after_call3
  %func_ptr10 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 2), align 8
  %null_check11 = icmp eq ptr %func_ptr10, null
  br i1 %null_check11, label %trap8, label %do_call12

trap8:                                            ; preds = %valid_call7, %after_call3
  unreachable

after_call9:                                      ; preds = %do_call12
  call void @assert_eq32(i32 %indirect_call, i32 12)
  br i1 true, label %valid_call13, label %trap14

do_call12:                                        ; preds = %valid_call7
  %indirect_call = call i32 %func_ptr10(i32 5, i32 7)
  br label %after_call9

valid_call13:                                     ; preds = %after_call9
  %func_ptr16 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 3), align 8
  %null_check17 = icmp eq ptr %func_ptr16, null
  br i1 %null_check17, label %trap14, label %do_call18

trap14:                                           ; preds = %valid_call13, %after_call9
  unreachable

after_call15:                                     ; preds = %do_call18
  call void @assert_eq32(i32 %indirect_call19, i32 50)
  br i1 true, label %valid_call20, label %trap21

do_call18:                                        ; preds = %valid_call13
  %indirect_call19 = call i32 %func_ptr16(i32 5)
  br label %after_call15

valid_call20:                                     ; preds = %after_call15
  %func_ptr23 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 4), align 8
  %null_check24 = icmp eq ptr %func_ptr23, null
  br i1 %null_check24, label %trap21, label %do_call25

trap21:                                           ; preds = %valid_call20, %after_call15
  unreachable

after_call22:                                     ; preds = %do_call25
  br i1 true, label %valid_call26, label %trap27

do_call25:                                        ; preds = %valid_call20
  call void %func_ptr23(i32 1, i32 2, i32 3)
  br label %after_call22

valid_call26:                                     ; preds = %after_call22
  %func_ptr29 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 5), align 8
  %null_check30 = icmp eq ptr %func_ptr29, null
  br i1 %null_check30, label %trap27, label %do_call31

trap27:                                           ; preds = %valid_call26, %after_call22
  unreachable

after_call28:                                     ; preds = %do_call31
  call void @assert_eq32(i32 %indirect_call32, i32 6)
  br i1 true, label %valid_call33, label %trap34

do_call31:                                        ; preds = %valid_call26
  %indirect_call32 = call i32 %func_ptr29(i32 1, i32 2, i32 3)
  br label %after_call28

valid_call33:                                     ; preds = %after_call28
  %func_ptr36 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 9), align 8
  %null_check37 = icmp eq ptr %func_ptr36, null
  br i1 %null_check37, label %trap34, label %do_call38

trap34:                                           ; preds = %valid_call33, %after_call28
  unreachable

after_call35:                                     ; preds = %do_call38
  call void @assert_eq32(i32 %indirect_call39, i32 42)
  br i1 true, label %valid_call40, label %trap41

do_call38:                                        ; preds = %valid_call33
  %indirect_call39 = call i32 %func_ptr36()
  br label %after_call35

valid_call40:                                     ; preds = %after_call35
  %func_ptr43 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 8), align 8
  %null_check44 = icmp eq ptr %func_ptr43, null
  br i1 %null_check44, label %trap41, label %do_call45

trap41:                                           ; preds = %valid_call40, %after_call35
  unreachable

after_call42:                                     ; preds = %do_call45
  call void @assert_eq32(i32 %indirect_call46, i32 10)
  br i1 true, label %valid_call47, label %trap48

do_call45:                                        ; preds = %valid_call40
  %indirect_call46 = call i32 %func_ptr43(i32 1, i32 2, i32 3, i32 4)
  br label %after_call42

valid_call47:                                     ; preds = %after_call42
  %func_ptr50 = load ptr, ptr getelementptr inbounds ([15 x ptr], ptr @table_0, i32 0, i32 11), align 8
  %null_check51 = icmp eq ptr %func_ptr50, null
  br i1 %null_check51, label %trap48, label %do_call52

trap48:                                           ; preds = %valid_call47, %after_call42
  unreachable

after_call49:                                     ; preds = %do_call52
  call void @assert_eq32(i32 %indirect_call53, i32 15)
  ret void

do_call52:                                        ; preds = %valid_call47
  %indirect_call53 = call i32 %func_ptr50(i32 1, i32 2, i32 3, i32 4, i32 5)
  br label %after_call49
}

define i32 @main() {
entry:
  call void @_start()
  ret i32 0
}
