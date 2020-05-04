// Initialisation code
@256
D=A
@SP
M=D
@512
D=A
@LCL
M=D
@768
D=A
@ARG
M=D
@1024
D=A
@THIS
M=D
@1280
D=A
@THAT
M=D

// label c
(test.c)

// push constant 1
@1
D=A
@SP
A=M
M=D
@SP
M=M+1

// if-goto c
@SP
M=M-1
A=M
D=M
@test.c
D;JNE

