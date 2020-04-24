use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

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

fn compile_file(input_file_path: &Path, output_file_path: &Path) {
    let lines = read_lines(input_file_path);
    let input_file_name = match input_file_path.file_name() {
        Some(name) => String::from(name.to_str().unwrap()),
        None => panic!("Invalid file path"),
    };

    let mut output = Vec::new();
    let mut errors = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.len() == 0 {
            continue;
        };

        let comment_line = format!("// {}\n", trimmed.clone());
        let compiled_line = compile_line(index, trimmed.to_string(), input_file_name.clone());

        match compiled_line {
            Ok(val) => output.push(format!("{}{}\n", comment_line, val)),
            Err(err) => errors.push(format!("Error on line {}:\n{}", index + 1, err)),
        };
    }

    write_lines(output_file_path, &output);

    print_errors(&errors);
}

fn read_lines(file_path: &Path) -> Vec<String> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => panic!("Couldn't open file: {}", err.to_string()),
    };

    let reader = io::BufReader::new(file);

    reader.lines().map(|l| l.unwrap()).collect()
}

fn write_lines(file_path: &Path, lines: &Vec<String>) {
    let file = match File::create(file_path) {
        Ok(file) => file,
        Err(err) => panic!("Couldn't create file: {}", err.to_string()),
    };

    let mut writer = io::BufWriter::new(file);

    for line in lines {
        match writer.write(line.as_bytes()) {
            Ok(_) => (),
            Err(err) => panic!("An error has occured: {}", err.to_string()),
        }
    }
}

fn print_errors(errors: &Vec<String>) {
    for error in errors {
        println!("{}", error);
    }
}

fn compile_line(index: usize, line: String, file_name: String) -> Result<String, String> {
    let fragments: Vec<&str> = line.split(" ").collect();

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
        otherwise => Err(format!("Unsupported operation: {}", otherwise)),
    }
}

fn compile_push(args: &[&str], file_name: String) -> Result<String, String> {
    if args.len() != 2 {
        return Err(format!(
            "Syntax error: push takes two arguments, received {:?}",
            args
        ));
    };

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


            let mut result = String::new();

            // Store offset in D
            result.push_str(&format!("@{}\n", arg));
            result.push_str("D=A\n");
            // Get value (RAM[pointer + offset]) in D
            result.push_str(&format!("@{}\n", pointer));
            result.push_str("A=M+D\n");
            result.push_str("D=M\n");

            result.push_str(&push_to_sp_and_inc());

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

            let mut result = String::new();
            // Get value in D
            result.push_str(&format!("@{}\n", value));
            result.push_str("D=M\n");
            
            result.push_str(&push_to_sp_and_inc());

            Ok(result)
        }
        "static" => {
            let arg = args[1];
            let mut class_name = file_name.clone();
            class_name.truncate(file_name.len() - 3);

            
            let mut result = String::new();

            // Get Class.Arg in D
            result.push_str(&format!("@{}.{}\n", class_name, arg));
            result.push_str("D=M\n");
            
            result.push_str(&push_to_sp_and_inc());

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
            let mut result = String::new();

            // Get value in D
            result.push_str(&format!("@{}\n", this_or_that));
            result.push_str("D=M\n");

            result.push_str(&push_to_sp_and_inc());

            Ok(result)
        }
        "constant" => {
            let arg = args[1];
            let mut result = String::new();

            // Get constant in D
            result.push_str(&format!("@{}\n", arg));
            result.push_str("D=A\n");
            
            result.push_str(&push_to_sp_and_inc());

            Ok(result)
        }
        _ => Err(String::from("Syntax error: push first argument must be {local, argument, this, that, temp, static, pointer, constant}")),
    }
}

fn compile_pop(args: &[&str], file_name: String) -> Result<String, String> {
    if args.len() != 2 {
        return Err(format!(
            "Syntax error: pop takes two arguments, received {:?}",
            args
        ));
    };

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

            let mut result = String::new();

            // Store value in D
            result.push_str("@SP\n");
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");

            // Point M to correct memory location
            result.push_str(&format!("@{}\n", pointer));
            result.push_str("A=M\n");

            for _ in 0..arg {
                result.push_str("A=A+1\n");
            }

            // Write to RAM[pointer + arg]
            result.push_str("M=D\n");
            // Decrement stack pointer
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

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

            let mut result = String::new();

            // Store value in D
            result.push_str("@SP\n");
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");
            // Write to RAM[value]
            result.push_str(&format!("@{}\n", value));
            result.push_str("M=D\n");
            // Decrement stack pointer
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

            Ok(result)
        }
        "static" => {
            let arg = args[1];
            let mut class_name = file_name.clone();
            class_name.truncate(file_name.len() - 3);


            let mut result = String::new();

            // Store value in D
            result.push_str(&format!("@SP\n"));
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");
            // Write D to Class.Arg
            result.push_str(&format!("@{}.{}\n", class_name, arg));
            result.push_str("M=D\n");
            // Decrement stack pointer
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

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


            let mut result = String::new();
            
            // Store value in D
            result.push_str("@SP\n");
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");
            // Write to correct memory location
            result.push_str(&format!("@{}\n", this_or_that));
            result.push_str("M=D\n");
            // Decrement stack pointer
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

            Ok(result)
        }
        _ => return Err(format!("Syntax error: pop first argument must be {{local, argument, this, that, temp, static, pointer}}, received {}", args[0])),
    }
}

fn compile_add(args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: add takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::Add))
}

fn compile_sub(args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: sub takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::Sub))
}

fn compile_neg(args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: neg takes no argument, received {:?}",
            args
        ));
    };

    let mut result = String::new();
    // Get value in D
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    // D = 0 - D (2's complement)
    result.push_str("@0\n");
    result.push_str("D=A-D\n");
    // Store the result
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("M=D\n");

    Ok(result)
}

fn compile_eq(index: usize, args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: eq takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_boolean_operation(index, BooleanOperator::Equal))
}

fn compile_gt(index: usize, args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: gt takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_boolean_operation(index, BooleanOperator::GreaterThan))
}

fn compile_lt(index: usize, args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: lt takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_boolean_operation(index, BooleanOperator::LesserThan))
}

fn compile_and(args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: and takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::And))
}

fn compile_or(args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: or takes no argument, received {:?}",
            args
        ));
    };

    Ok(compile_binary_operation(BinaryArithmeticOperator::Or))
}

fn compile_not(args: &[&str]) -> Result<String, String> {
    if args.len() > 0 {
        return Err(format!(
            "Syntax error: not takes no argument, received {:?}",
            args
        ));
    };

    let mut result = String::new();
    // Point A to value
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    // Value = not value
    result.push_str("M=!M\n");

    Ok(result)
}

fn compile_binary_operation(operator: BinaryArithmeticOperator) -> String {
    let op = match operator {
        BinaryArithmeticOperator::Add => "+",
        BinaryArithmeticOperator::Sub => "-",
        BinaryArithmeticOperator::And => "&",
        BinaryArithmeticOperator::Or => "|",
    };

    let mut result = String::new();

    // Get y in D
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    // Point M to x
    result.push_str("A=A-1\n");
    // Perform operation and store the result (x op y)
    result.push_str(&format!("M=M{}D\n", op));
    // Decrement stack pointer
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    result
}

fn compile_boolean_operation(index: usize, operator: BooleanOperator) -> String {
    let op = match operator {
        BooleanOperator::GreaterThan => "JGT",
        BooleanOperator::LesserThan => "JLT",
        BooleanOperator::Equal => "JEQ",
    };

    let mut result = String::new();
    
    // Get y in D
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    // M now points to x
    result.push_str("A=A-1\n");
    // Store diff in D (D = x - y)
    result.push_str("D=M-D\n");
    result.push_str(&format!("@TRUE{}\n", index));
    // Jump to TRUE if x op y is true
    result.push_str(&format!("D;{}\n", op));
    // Set result (D) to zero (false)
    result.push_str("D=0\n");
    result.push_str(&format!("@ELSE{}\n", index));
    result.push_str("0;JMP\n");
    result.push_str(&format!("(TRUE{})\n", index));
    // Set result (D) to minus one (true)
    result.push_str("D=-1\n");
    result.push_str(&format!("(ELSE{})\n", index));
    // Save result in SP - 2 (overrides the first operand in the stack)
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("A=A-1\n");
    result.push_str("M=D\n");
    // Decrement stack pointer
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");
    
    result
}

fn push_to_sp_and_inc() -> String {
    let mut result = String::new();

    // Write to stack pointer
    result.push_str("@SP\n");
    result.push_str("A=M\n");
    result.push_str("M=D\n");
    // Increment stack pointer
    result.push_str("@SP\n");
    result.push_str("M=M+1\n");

    result
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: vmcomp <path>");
    };

    if args[1].find(".vm").is_none() {
        panic!("Please provide a .vm file");
    };

    let input_file_path = Path::new(&args[1]);

    let output_file_name = format!("{}.asm", &args[1]);
    let output_file_path = Path::new(&output_file_name);

    compile_file(input_file_path, output_file_path);
}
