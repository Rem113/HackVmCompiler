// push constant 5
@5
D=A
@SP
A=M
M=D
@SP
M=M+1

// push constant 4
@4
D=A
@SP
A=M
M=D
@SP
M=M+1

// gt
@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@GREATER
D;JGT
D=0
@ELSE
0;JMP
(GREATER)
D=-1
(ELSE)
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1

// push constant 2
@2
D=A
@SP
A=M
M=D
@SP
M=M+1

// push constant 3
@3
D=A
@SP
A=M
M=D
@SP
M=M+1

// lt
@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@LESSER
D;JLT
D=0
@ELSE
0;JMP
(LESSER)
D=-1
(ELSE)
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

