// push constant 5
@5
D=A
@SP
A=M
M=D
@SP
M=M+1

// push constant 6
@6
D=A
@SP
A=M
M=D
@SP
M=M+1

// eq
@SP
A=M-1
D=M
A=A-1
D=M-D
@TRUE2
D;JEQ
D=0
@ELSE2
0;JMP
(TRUE2)
D=-1
(ELSE2)
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1

// push constant 6
@6
D=A
@SP
A=M
M=D
@SP
M=M+1

// push constant 7
@7
D=A
@SP
A=M
M=D
@SP
M=M+1

// lt
@SP
A=M-1
D=M
A=A-1
D=M-D
@TRUE5
D;JLT
D=0
@ELSE5
0;JMP
(TRUE5)
D=-1
(ELSE5)
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1

// and
@SP
A=M-1
D=M
A=A-1
M=M&D
@SP
M=M-1

