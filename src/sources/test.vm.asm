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

// push constant 11
@11
D=A
@SP
A=M
M=D
@SP
M=M+1

// call test.fibonacci 1
@11test.fibonacci.ReturnAddress
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@5
D=D-A
@1
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@test.fibonacci
0;JMP
(11test.fibonacci.ReturnAddress)

// goto end
@test.end
0;JMP

// function test.fibonacci 0
(test.fibonacci)
@0
D=A
@test.fibonacci.End
D;JEQ
(test.fibonacci.Loop)
@SP
A=M
M=0
@SP
M=M+1
@test.fibonacci.Loop
D=D-1;JNE
(test.fibonacci.End)

// push argument 0
@0
D=A
@ARG
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1

// push constant 2
@2
D=A
@SP
A=M
M=D
@SP
M=M+1

// lt                     // checks if n<2
@SP
A=M-1
D=M
A=A-1
D=M-D
@TRUE17
D;JLT
D=0
@ELSE17
0;JMP
(TRUE17)
D=-1
(ELSE17)
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1

// if-goto IF_TRUE
@SP
M=M-1
A=M
D=M
@test.IF_TRUE
D;JNE

// goto IF_FALSE
@test.IF_FALSE
0;JMP

// label IF_TRUE          // if n<2, return n
(test.IF_TRUE)

// push argument 0
@0
D=A
@ARG
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1

// return
@LCL
D=M
@5
A=D-A
D=M
@13
M=D
@SP
M=M-1
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M
@SP
M=D+1
@LCL
M=M-1
A=M
D=M
@THAT
M=D
@LCL
M=M-1
A=M
D=M
@THIS
M=D
@LCL
M=M-1
A=M
D=M
@ARG
M=D
@LCL
M=M-1
A=M
D=M
@LCL
M=D
@13
A=M
0;JMP

// label IF_FALSE         // if n>=2, returns fib(n-2)+fib(n-1)
(test.IF_FALSE)

// push argument 0
@0
D=A
@ARG
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1

// push constant 2
@2
D=A
@SP
A=M
M=D
@SP
M=M+1

// sub
@SP
A=M-1
D=M
A=A-1
M=M-D
@SP
M=M-1

// call test.fibonacci 1  // computes fib(n-2)
@27test.fibonacci.ReturnAddress
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@5
D=D-A
@1
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@test.fibonacci
0;JMP
(27test.fibonacci.ReturnAddress)

// push argument 0
@0
D=A
@ARG
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1

// push constant 1
@1
D=A
@SP
A=M
M=D
@SP
M=M+1

// sub
@SP
A=M-1
D=M
A=A-1
M=M-D
@SP
M=M-1

// call test.fibonacci 1  // computes fib(n-1)
@31test.fibonacci.ReturnAddress
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@5
D=D-A
@1
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@test.fibonacci
0;JMP
(31test.fibonacci.ReturnAddress)

// add                    // returns fib(n-1) + fib(n-2)
@SP
A=M-1
D=M
A=A-1
M=M+D
@SP
M=M-1

// return
@LCL
D=M
@5
A=D-A
D=M
@13
M=D
@SP
M=M-1
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M
@SP
M=D+1
@LCL
M=M-1
A=M
D=M
@THAT
M=D
@LCL
M=M-1
A=M
D=M
@THIS
M=D
@LCL
M=M-1
A=M
D=M
@ARG
M=D
@LCL
M=M-1
A=M
D=M
@LCL
M=D
@13
A=M
0;JMP

// label end
(test.end)

