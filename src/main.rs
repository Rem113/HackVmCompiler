use std::env;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use regex;

enum BinaryArithmeticOperator {
    Add,
    Sub,
    And,
    Or,
}

enum BooleanOperator {
    GreaterThan,
    LesserThan,
    Equal,
}

const SP: usize = 256;
const LCL: usize = 512;
const ARG: usize = 768;
const THIS: usize = 1024;
const THAT: usize = 1280;

fn compile_file(file_path: &String) -> (Vec<String>, Vec<String>) {
    let file_path = Path::new(file_path);
    let file_name = file_path.file_name().unwrap().to_string_lossy().into_owned();

    let lines = read_lines(file_path);

    let mut output = Vec::new();
    let mut errors = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.len() == 0 || trimmed.starts_with("//") {
            continue;
        };

        output.push(format!("// {}\n", trimmed));

        let compiled_line = compile_line(index, &trimmed.to_string(), &file_name);

        match compiled_line {
            Ok(mut line) => {
                output.append(&mut line);
                output.push(format!("\n"));
            },
            Err(err) => errors.push(format!("Error on line {}:\n{}", index + 1, err)),
        };
    };

    (output, errors)
}

fn read_lines(file_path: &Path) -> Vec<String> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => panic!(format!("Couldn't open file: {}", err.to_string())),
    };

    let reader = io::BufReader::new(file);

    reader.lines().map(|l| l.unwrap()).collect()
}

fn write_lines(file_path: &Path, lines: &Vec<String>) {
    let file = match File::create(file_path) {
        Ok(file) => file,
        Err(err) => panic!(format!("Couldn't create file: {}", err.to_string())),
    };

    let mut writer = io::BufWriter::new(file);

    for line in lines {
        match writer.write(line.as_bytes()) {
            Ok(_) => (),
            Err(err) => panic!(format!("An error has occured: {}", err.to_string())),
        }
    };
}

fn compile_line(index: usize, line: &String, file_name: &String) -> Result<Vec<String>, String> {
    let fragments: Vec<&str> = regex::Regex::new(" +").unwrap()
        .split(line)
        .take_while(|arg| !arg.starts_with("//"))
        .map(|arg| arg.trim()).collect();

    let args = &fragments[1..];

    match fragments[0] {
        "push" => compile_push(args, file_name),
        "pop" => compile_pop(args, file_name),
        "add" => compile_add(args),
        "sub" => compile_sub(args),
        "neg" => compile_neg(args),
        "eq" => compile_eq(index, args),
        "gt" => compile_gt(index, args),
        "lt" => compile_lt(index, args),
        "and" => compile_and(args),
        "or" => compile_or(args),
        "not" => compile_not(args),
        "label" => compile_label(args, file_name),
        "goto" => compile_goto(args, file_name),
        "if-goto" => compile_if_goto(args, file_name),
        "function" => compile_function(args),
        "call" => compile_call(index, args),
        "return" => compile_return(),
        otherwise => Err(format!("Unsupported operation: {}", otherwise)),
    }
}

fn compile_push(args: &[&str], file_name: &String) -> Result<Vec<String>, String> {
    if args.len() != 2 {
        return Err(format!(
            "Syntax error: push takes two arguments, received {:?}",
            args
        ));
    };

    let mut result = Vec::new();

    match args[0] {
        "local" | "argument" | "this" | "that" => {
            let pointer = String::from(match args[0] {
                "local" => "LCL",
                "argument" => "ARG",
                "this" => "THIS",
                "that" => "THAT",
                _ => panic!("An error has occured"),
            });
            let arg = match args[1].parse::<u8>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(format!(
                        "Syntax error: push takes an integer as a second argument, received {}", args[1]
                    ))
                }
            };

            // Store offset in D
            result.push(format!("@{}\n", arg));
            result.push(format!("D=A\n"));
            // Get value (RAM[pointer + offset]) in D
            result.push(format!("@{}\n", pointer));
            result.push(format!("A=M+D\n"));
            result.push(format!("D=M\n"));

            result.append(&mut gen_push_to_sp_and_inc());

            Ok(result)
        }
        "temp" => {
            let arg = args[1];

            let value = match arg.parse::<u8>() {
                Ok(val) => val + 5,
                Err(_) => return Err(format!("Syntax error: push temp argument must be an integer, received {}", arg)),
            };

            if value < 5 || value > 12 {
                return Err(format!("Syntax error: push temp argument must be between 0 and 7, received {}", arg));
            };

            // Get value in D
            result.push(format!("@{}\n", value));
            result.push(format!("D=M\n"));
            
            result.append(&mut gen_push_to_sp_and_inc());

            Ok(result)
        }
        "static" => {
            let arg = args[1];
            let mut class_name = file_name.clone();
            class_name.truncate(file_name.len() - 3);

            
            // Get Class.Arg in D
            result.push(format!("@{}.{}\n", class_name, arg));
            result.push(format!("D=M\n"));
            
            result.append(&mut gen_push_to_sp_and_inc());

            Ok(result)
        }
        "pointer" => {
            let arg = args[1];
            let this_or_that = match arg.parse() {
                Ok(val) => match val {
                    0 => "THIS",
                    1 => "THAT",
                    _ => return Err(format!("Syntax error: pointer must be 0 or 1, received {}", arg)),
                },
                Err(_) => return Err(format!("Syntax error: push pointer argument must be 0 or 1, received {}", arg)),
            };

            // Get value in D
            result.push(format!("@{}\n", this_or_that));
            result.push(format!("D=M\n"));

            result.append(&mut gen_push_to_sp_and_inc());

            Ok(result)
        }
        "constant" => {
            let arg = args[1];

            // Get constant in D
            result.push(format!("@{}\n", arg));
            result.push(format!("D=A\n"));
            
            result.append(&mut gen_push_to_sp_and_inc());

            Ok(result)
        }
        _ => Err(String::from("Syntax error: push first argument must be {local, argument, this, that, temp, static, pointer, constant}")),
    }
}

fn compile_pop(args: &[&str], file_name: &String) -> Result<Vec<String>, String> {
    if args.len() != 2 {
        return Err(format!(
            "Syntax error: pop takes two arguments, received {:?}",
            args
        ));
    };

    let mut result = Vec::new();

    match args[0] {
        "local" | "argument" | "this" | "that" => {
            let pointer = String::from(match args[0] {
                "local" => "LCL",
                "argument" => "ARG",
                "this" => "THIS",
                "that" => "THAT",
                _ => return Err(String::from("An error has occured")),
            });
            let arg = match args[1].parse() {
                Ok(val) => val,
                Err(_) => {
                    return Err(format!(
                        "Invalid syntax: pop second operand must be an integer, received {}", args[1]
                    ))
                }
            };


            // Store value in D
            result.push(format!("@SP\n"));
            result.push(format!("A=M-1\n"));
            result.push(format!("D=M\n"));

            // Point M to correct memory location
            result.push(format!("@{}\n", pointer));
            result.push(format!("A=M\n"));

            // Offset the pointer
            (0..arg).for_each(|_| result.push(format!("A=A+1\n")));

            // Write to RAM[pointer + arg]
            result.push(format!("M=D\n"));
            // Decrement stack pointer
            result.push(format!("@SP\n"));
            result.push(format!("M=M-1\n"));

            Ok(result)
        }
        "temp" => {
            let arg = args[1];

            let value = match arg.parse::<u8>() {
                Ok(val) => val + 5,
                Err(_) => return Err(format!("Syntax error: push temp argument must be an integer, received {}", arg)),
            };

            if value < 5 || value > 12 {
                return Err(format!("Syntax error: pop temp must be between 0 and 7, received {}", arg));
            };

            // Store value in D
            result.push(format!("@SP\n"));
            result.push(format!("A=M-1\n"));
            result.push(format!("D=M\n"));
            // Write to RAM[value]
            result.push(format!("@{}\n", value));
            result.push(format!("M=D\n"));
            // Decrement stack pointer
            result.push(format!("@SP\n"));
            result.push(format!("M=M-1\n"));

            Ok(result)
        }
        "static" => {
            let arg = args[1];
            let mut class_name = file_name.clone();
            class_name.truncate(file_name.len() - 3);

            // Store value in D
            result.push(format!("@SP\n"));
            result.push(format!("A=M-1\n"));
            result.push(format!("D=M\n"));
            // Write D to Class.Arg
            result.push(format!("@{}.{}\n", class_name, arg));
            result.push(format!("M=D\n"));
            // Decrement stack pointer
            result.push(format!("@SP\n"));
            result.push(format!("M=M-1\n"));

            Ok(result)
        }
        "pointer" => {
            let arg = args[1];
            let this_or_that = match arg.parse() {
                Ok(val) => match val {
                    0 => "THIS",
                    1 => "THAT",
                    _ => return Err(format!("Syntax error: pop pointer must be 0 or 1, received {}", val)),
                },
                Err(_) => return Err(format!("Syntax error: pop pointer argument must be an integer, received {}", arg)),
            };
            
            // Store value in D
            result.push(format!("@SP\n"));
            result.push(format!("A=M-1\n"));
            result.push(format!("D=M\n"));
            // Write to correct memory location
            result.push(format!("@{}\n", this_or_that));
            result.push(format!("M=D\n"));
            // Decrement stack pointer
            result.push(format!("@SP\n"));
            result.push(format!("M=M-1\n"));

            Ok(result)
        }
        _ => return Err(format!("Syntax error: pop first argument must be {{local, argument, this, that, temp, static, pointer}}, received {}", args[0])),
    }
}

fn compile_add(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: add takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::Add))
}

fn compile_sub(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: sub takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::Sub))
}

fn compile_neg(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: neg takes no argument, received {:?}",
            args
        ));
    };

    let mut result = Vec::new();
    // Get value in D
    result.push(format!("@SP\n"));
    result.push(format!("A=M-1\n"));
    result.push(format!("D=M\n"));
    // D = 0 - D (2's complement)
    result.push(format!("@0\n"));
    result.push(format!("D=A-D\n"));
    // Store the result
    result.push(format!("@SP\n"));
    result.push(format!("A=M-1\n"));
    result.push(format!("M=D\n"));

    Ok(result)
}

fn compile_eq(index: usize, args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: eq takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_boolean_operation(index, BooleanOperator::Equal))
}

fn compile_gt(index: usize, args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: gt takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_boolean_operation(index, BooleanOperator::GreaterThan))
}

fn compile_lt(index: usize, args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: lt takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_boolean_operation(index, BooleanOperator::LesserThan))
}

fn compile_and(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: and takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::And))
}

fn compile_or(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: or takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::Or))
}

fn compile_not(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: not takes no argument, received {:?}",
            args
        ));
    };

    let mut result = Vec::new();
    // Point A to value
    result.push(format!("@SP\n"));
    result.push(format!("A=M-1\n"));
    // Value = not value
    result.push(format!("M=!M\n"));

    Ok(result)
}

fn compile_label(args: &[&str], file_name: &String) -> Result<Vec<String>, String> {
    if args.len() != 1 {
        return Err(format!("Syntax error: label takes one argument, received {:?}", args));
    };

    let mut class_name = file_name.clone();
    class_name.truncate(file_name.len() - 3);

    Ok(vec![format!("({}.{})\n", class_name, args[0])])
}

fn compile_goto(args: &[&str], file_name: &String) -> Result<Vec<String>, String> {
    if args.len() != 1 {
        return Err(format!("Syntax error: goto takes one argument, received {:?}", args));
    };

    let mut class_name = file_name.clone();
    class_name.truncate(file_name.len() - 3);

    let mut result = Vec::new();

    result.push(format!("@{}.{}\n", class_name, args[0]));
    // Unconditional jump
    result.push(format!("0;JMP\n"));

    Ok(result)
}

fn compile_if_goto(args: &[&str], file_name: &String) -> Result<Vec<String>, String> {
    if args.len() != 1 {
        return Err(format!("Syntax error: if-goto takes one argument, received {:?}", args));
    };

    let mut class_name = file_name.clone();
    class_name.truncate(file_name.len() - 3);

    let mut result = Vec::new();

    // Get the value on top of the stack in D
    result.push(format!("@SP\n"));
    result.push(format!("M=M-1\n"));
    result.push(format!("A=M\n"));
    result.push(format!("D=M\n"));
    // Jump to label if the value is not 0
    result.push(format!("@{}.{}\n", class_name, args[0]));
    result.push(format!("D;JNE\n"));

    Ok(result)
}

fn compile_call(index: usize, args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() != 2 {
        return Err(format!("Syntax error: call takes two arguments, received {:?}", args));
    };

    let func_name = args[0];
    let param_count = match args[1].parse::<u8>() {
        Ok(val) => val,
        Err(_) => return Err(format!("Syntax error: call second argument must be an integer, received {}", args[1])),
    };

    let mut result = Vec::new();

    result.push(format!("@{}{}.ReturnAddress\n", index, func_name));
    result.push(format!("D=A\n"));
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));

    result.push(format!("@LCL\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));

    result.push(format!("@ARG\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));

    result.push(format!("@THIS\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));

    result.push(format!("@THAT\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));

    result.push(format!("@SP\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@5\n"));
    result.push(format!("D=D-A\n"));
    result.push(format!("@{}\n", param_count));
    result.push(format!("D=D-A\n"));
    result.push(format!("@ARG\n"));
    result.push(format!("M=D\n"));
    
    result.push(format!("@SP\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@LCL\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@{}\n", func_name));
    result.push(format!("0;JMP\n"));
    result.push(format!("({}{}.ReturnAddress)\n", index, func_name));

    Ok(result)
}

fn compile_function(args: &[&str]) -> Result<Vec<String>, String> {
    if args.len() != 2 {
        return Err(format!("Syntax error: function takes two arguments, received {:?}", args));
    };

    if args[0].find(".").is_none() {
        return Err(format!("Syntax error: function first argument must be class.name, received {}", args[0]))
    }

    let class_name = args[0];
    let local_count = match args[1].parse::<u8>() {
        Ok(val) => val,
        Err(_) => return Err(format!("Syntax error: function second argument must be an integer, received {}", args[1]))
    };

    let mut result = Vec::new();

    // Define a label for the function
    result.push(format!("({})\n", class_name));
    // Store local variables count in D
    result.push(format!("@{}\n", local_count));
    result.push(format!("D=A\n"));
    // Set D values to 0 
    result.push(format!("@{}.End\n", class_name));
    result.push(format!("D;JEQ\n"));
    result.push(format!("({}.Loop)\n", class_name));
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=0\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));
    result.push(format!("@{}.Loop\n", class_name));
    result.push(format!("D=D-1;JNE\n"));
    result.push(format!("({}.End)\n", class_name));
    
    Ok(result)
}

fn compile_return() -> Result<Vec<String>, String> {
    let mut result = Vec::new();

    result.push(format!("@LCL\n"));
    result.push(format!("D=M\n"));

    result.push(format!("@5\n"));
    result.push(format!("A=D-A\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@13\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@SP\n"));
    result.push(format!("M=M-1\n"));
    result.push(format!("A=M\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@ARG\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@ARG\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=D+1\n"));

    result.push(format!("@LCL\n"));
    result.push(format!("M=M-1\n"));
    result.push(format!("A=M\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@THAT\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@LCL\n"));
    result.push(format!("M=M-1\n"));
    result.push(format!("A=M\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@THIS\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@LCL\n"));
    result.push(format!("M=M-1\n"));
    result.push(format!("A=M\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@ARG\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@LCL\n"));
    result.push(format!("M=M-1\n"));
    result.push(format!("A=M\n"));
    result.push(format!("D=M\n"));
    result.push(format!("@LCL\n"));
    result.push(format!("M=D\n"));

    result.push(format!("@13\n"));
    result.push(format!("A=M\n"));
    result.push(format!("0;JMP\n"));

    Ok(result)
}

fn compile_binary_operation(operator: BinaryArithmeticOperator) -> Vec<String> {
    let op = match operator {
        BinaryArithmeticOperator::Add => "+",
        BinaryArithmeticOperator::Sub => "-",
        BinaryArithmeticOperator::And => "&",
        BinaryArithmeticOperator::Or => "|",
    };

    let mut result = Vec::new();

    // Get y in D
    result.push(format!("@SP\n"));
    result.push(format!("A=M-1\n"));
    result.push(format!("D=M\n"));
    // Point M to x
    result.push(format!("A=A-1\n"));
    // Perform operation and store the result (x op y)
    result.push(format!("M=M{}D\n", op));
    // Decrement stack pointer
    result.push(format!("@SP\n"));
    result.push(format!("M=M-1\n"));

    result
}

fn compile_boolean_operation(index: usize, operator: BooleanOperator) -> Vec<String> {
    let op = match operator {
        BooleanOperator::GreaterThan => "JGT",
        BooleanOperator::LesserThan => "JLT",
        BooleanOperator::Equal => "JEQ",
    };

    let mut result = Vec::new();
    
    // Get y in D
    result.push(format!("@SP\n"));
    result.push(format!("A=M-1\n"));
    result.push(format!("D=M\n"));
    // M now points to x
    result.push(format!("A=A-1\n"));
    // Store diff in D (D = x - y)
    result.push(format!("D=M-D\n"));
    result.push(format!("@TRUE{}\n", index));
    // Jump to TRUE if x op y is true
    result.push(format!("D;{}\n", op));
    // Set result (D) to zero (false)
    result.push(format!("D=0\n"));
    result.push(format!("@ELSE{}\n", index));
    result.push(format!("0;JMP\n"));
    result.push(format!("(TRUE{})\n", index));
    // Set result (D) to minus one (true)
    result.push(format!("D=-1\n"));
    result.push(format!("(ELSE{})\n", index));
    // Save result in SP - 2 (overrides the first operand in the stack)
    result.push(format!("@SP\n"));
    result.push(format!("A=M-1\n"));
    result.push(format!("A=A-1\n"));
    result.push(format!("M=D\n"));
    // Decrement stack pointer
    result.push(format!("@SP\n"));
    result.push(format!("M=M-1\n"));
    
    result
}

fn gen_push_to_sp_and_inc() -> Vec<String> {
    let mut result = Vec::new();

    // Write to stack pointer
    result.push(format!("@SP\n"));
    result.push(format!("A=M\n"));
    result.push(format!("M=D\n"));
    // Increment stack pointer
    result.push(format!("@SP\n"));
    result.push(format!("M=M+1\n"));

    result
}

fn gen_init_code() -> Vec<String> {
    let mut result = Vec::new();

    result.push(format!("// Initialisation code\n"));

    // Set SP
    result.push(format!("@{}\n", SP));
    result.push(format!("D=A\n"));
    result.push(format!("@SP\n"));
    result.push(format!("M=D\n"));
    // Set LCL
    result.push(format!("@{}\n", LCL));
    result.push(format!("D=A\n"));
    result.push(format!("@LCL\n"));
    result.push(format!("M=D\n"));
    // Set ARG
    result.push(format!("@{}\n", ARG));
    result.push(format!("D=A\n"));
    result.push(format!("@ARG\n"));
    result.push(format!("M=D\n"));
    // Set THIS
    result.push(format!("@{}\n", THIS));
    result.push(format!("D=A\n"));
    result.push(format!("@THIS\n"));
    result.push(format!("M=D\n"));
    // Set THAT
    result.push(format!("@{}\n", THAT));
    result.push(format!("D=A\n"));
    result.push(format!("@THAT\n"));
    result.push(format!("M=D\n"));

    let mut sys_init_call = match compile_call(0, &["Sys.init", "0"]) {
        Ok(val) => val,
        Err(_) => panic!("An error has occured"),
    };

    result.append(&mut sys_init_call);

    result.push(format!("\n"));

    result
}

fn compile_dir(dir_name: &String) -> Vec<(String, (Vec<String>, Vec<String>))> {
    let read_dir = match fs::read_dir(dir_name) {
        Ok(dir) => dir,
        Err(_) => panic!("An error has occured")
    };

    let vm_files = read_dir.filter_map(|file| match file {
        Ok(file) => {
            let file_name = file.file_name().to_string_lossy().into_owned();
            if file_name.ends_with(".vm") {
                Some(file_name)
            } else {
                None
            }
        },
        Err(_) => None
    });
    
    vm_files.map(|file| (file.clone(), compile_file(&format!("{}/{}", dir_name, file)))).collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: vmcomp <path>");
    };
    
    let target = &args[1];

    let target_path = Path::new(target);

    if !target_path.exists() {
        panic!("Could not find specified path");
    };

    let output_name = format!("{}.asm", target);
    let output_path = Path::new(&output_name);

    let mut output = Vec::from(gen_init_code());

    if target_path.is_dir() {
        let compiled = compile_dir(target);

        compiled.iter().for_each(|(file, (lines, errors))| {
            // Print errors
            if !errors.is_empty() {
                println!("In file {}\n", file);
                errors.iter().for_each(|error| println!("{}\n", error));
            }

            // Add to output
            output.push(format!("// {}\n", file));
            output.append(&mut lines.clone());
        });
    } else {
        if args[1].find(".vm").is_none() {
            panic!("Please provide a .vm file or a directory");
        };
        
        let (lines, errors) = compile_file(target);

        errors.iter().for_each(|error| println!("{}", error));

        output.append(&mut lines.clone());
    };

    // Write output to file
    write_lines(output_path, &output);
}
