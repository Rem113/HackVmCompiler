use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

// TODO: Comment code generation
fn compile_file(input_file_path: &Path, output_file_path: &Path) {
    let input_file = match File::open(input_file_path) {
        Ok(file) => file,
        Err(err) => panic!("Couldn't open file: {}", err.to_string()),
    };

    let output_file = match File::create(output_file_path) {
        Ok(file) => file,
        Err(err) => panic!("Couldn't create file: {}", err.to_string()),
    };

    let input_file_name = match input_file_path.file_name() {
        Some(name) => String::from(name.to_str().unwrap()),
        None => panic!("An error has occured"),
    };

    let reader = io::BufReader::new(input_file);
    let mut writer = io::BufWriter::new(output_file);

    let mut errors: Vec<String> = Vec::new();

    for (index, line) in reader.lines().map(|l| l.unwrap()).enumerate() {
        let comment_line = format!("// {}\n", line.clone());
        let compiled_line = compile_line(line.clone(), input_file_name.clone());

        let compiled_line = match compiled_line {
            Ok(val) => val,
            Err(err) => {
                errors.push(format!("Error on line {}:\n{}", index + 1, err));
                continue;
            }
        };

        let text_to_write = format!("{}{}\n", comment_line, compiled_line);
        match writer.write(text_to_write.as_bytes()) {
            Ok(_) => {}
            Err(err) => panic!("An error has occured: {}", err.to_string()),
        };
    }

    for error in errors {
        println!("{}", error);
    }
}

fn compile_line(line: String, file_name: String) -> Result<String, String> {
    let trimmed = line.trim();

    if trimmed.len() == 0 {
        return Ok(String::new());
    }

    let fragments: Vec<&str> = trimmed.split(" ").collect();

    match fragments[0] {
        "push" => compile_push(&fragments[1..], file_name),
        "pop" => compile_pop(&fragments[1..]),
        "add" => compile_add(),
        "sub" => compile_sub(),
        "neg" => compile_neg(),
        "eq" => compile_eq(),
        "gt" => compile_gt(),
        "lt" => compile_lt(),
        "and" => compile_and(),
        "or" => compile_or(),
        "not" => compile_not(),
        _ => Ok(line),
    }
}

fn compile_push(args: &[&str], file_name: String) -> Result<String, String> {
    if args.len() != 2 {
        return Err(format!("Push takes two arguments, received {}", args.len()));
    };

    match args[0] {
        "local" | "argument" | "this" | "that" => {
            let address = String::from(match args[0] {
                "local" => "LCL",
                "argument" => "ARG",
                "this" => "THIS",
                "that" => "THAT",
                _ => panic!("An error has occured"),
            });
            let arg = match args[1].parse::<u8>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(String::from(
                        "Invalid syntax: push takes an integer as a second argument",
                    ))
                }
            };
            let mut result = String::new();
            result.push_str(&format!("@{}\n", arg));
            result.push_str("D=A\n");
            result.push_str(&format!("@{}\n", address));
            result.push_str("A=M+D\n");
            result.push_str("D=M\n");
            result.push_str("@SP\n");
            result.push_str("A=M\n");
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M+1\n");

            Ok(result)
        }
        "temp" => {
            let arg = args[1];

            let value = match arg.parse::<u8>() {
                Ok(val) => val + 5,
                Err(err) => return Err(format!("Syntax error: {}", err.to_string())),
            };

            if value < 5 || value > 12 {
                return Err(String::from("Syntax error: temp must be between 0 and 7"));
            };

            let mut result = String::new();
            result.push_str(&format!("@{}\n", value));
            result.push_str("D=A\n");
            result.push_str("@THAT\n");
            result.push_str("A=M+D\n");
            result.push_str("D=M\n");
            result.push_str("@SP\n");
            result.push_str("A=M\n");
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M+1\n");

            Ok(result)
        }
        "static" => {
            let arg = args[1];
            let mut class_name = file_name.clone();
            class_name.truncate(file_name.len() - 3);
            let mut result = String::new();
            result.push_str(&format!("@{}.{}\n", class_name, arg));
            result.push_str("D=M\n");
            result.push_str("@SP\n");
            result.push_str("A=M\n");
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M+1\n");

            Ok(result)
        }
        "pointer" => {
            let arg = args[1];
            let this_or_that = match arg.parse() {
                Ok(val) => match val {
                    0 => "THIS",
                    1 => "THAT",
                    _ => return Err(String::from("Syntax error: pointer must be 0 or 1")),
                },
                Err(_) => return Err(String::from("Syntax error: push pointer argument must be 0 or 1")),
            };
            let mut result = String::new();
            result.push_str(&format!("@{}\n", this_or_that));
            result.push_str("D=M\n");
            result.push_str("@SP\n");
            result.push_str("A=M\n");
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M+1\n");

            Ok(result)
        }
        "constant" => {
            let arg = args[1];
            let mut result = String::new();
            result.push_str(&format!("@{}\n", arg));
            result.push_str("D=A\n");
            result.push_str("@SP\n");
            result.push_str("A=M\n");
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M+1\n");

            Ok(result)
        }
        _ => Err(String::from("Syntax error: push first argument must be {local, argument, this, that, temp, static, pointer, constant}")),
    }
}

fn compile_pop(args: &[&str]) -> Result<String, String> {
    if args.len() != 2 {
        return Err(format!("Push takes two arguments, received {}", args.len()));
    };

    match args[0] {
        "local" | "argument" | "this" | "that" => {
            let address = String::from(match args[0] {
                "local" => "LCL",
                "argument" => "ARG",
                "this" => "THIS",
                "that" => "THAT",
                _ => return Err(String::from("An error has occured")),
            });
            let arg = match args[1].parse() {
                Ok(val) => val,
                Err(_) => {
                    return Err(String::from(
                        "Invalid syntax: pop second operand must be an integer",
                    ))
                }
            };

            let mut result = String::new();
            result.push_str("@SP\n");
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");
            result.push_str(&format!("@{}\n", address));
            result.push_str("A=M\n");

            for _ in 0..arg {
                result.push_str("A=A+1\n");
            }

            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

            Ok(result)
        }
        "temp" => {
            let arg = args[1];

            let value = match arg.parse::<u8>() {
                Ok(val) => val + 5,
                Err(err) => return Err(format!("Syntax error: {}", err.to_string())),
            };

            if value < 5 || value > 12 {
                return Err(String::from("Syntax error: temp must be between 0 and 7"));
            };

            let mut result = String::new();
            result.push_str("@SP\n");
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");
            result.push_str(&format!("@{}\n", value));
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

            Ok(result)
        }
        "static" => {
            let arg = args[1];
            let mut result = String::new();
            result.push_str(&format!("@STATIC\n"));
            result.push_str("D=A\n");
            result.push_str(&format!("@{}\n", arg));
            result.push_str("D=D+A\n");
            result.push_str("@TEMP\n");
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");
            result.push_str("A=M\n");
            result.push_str("D=M\n");
            result.push_str("@TEMP\n");
            result.push_str("A=M\n");
            result.push_str("M=D\n");

            Ok(result)
        }
        "pointer" => {
            let arg = args[1];
            let this_or_that = match arg.parse() {
                Ok(val) => match val {
                    0 => "THIS",
                    1 => "THAT",
                    _ => return Err(String::from("Invalid syntax: pointer must be 0 or 1")),
                },
                Err(_) => return Err(String::from("An error has occured")),
            };
            let mut result = String::new();
            result.push_str("@SP\n");
            result.push_str("A=M-1\n");
            result.push_str("D=M\n");
            result.push_str(&format!("@{}\n", this_or_that));
            result.push_str("M=D\n");
            result.push_str("@SP\n");
            result.push_str("M=M-1\n");

            Ok(result)
        }
        _ => return Err(String::from("Syntax error: pop first argument must be {local, argument, this, that, temp, static, pointer}")),
    }
}

// TODO: Refactor add, sub, and, or
fn compile_add() -> Result<String, String> {
    let mut result = String::new();
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("M=M+D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

fn compile_sub() -> Result<String, String> {
    let mut result = String::new();
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("M=M-D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

fn compile_neg() -> Result<String, String> {
    let mut result = String::new();
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    result.push_str("@0\n");
    result.push_str("D=A-D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");
    result.push_str("A=M\n");
    result.push_str("M=D\n");
    result.push_str("@SP\n");
    result.push_str("M=M+1\n");

    Ok(result)
}

fn compile_eq() -> Result<String, String> {
    let mut result = String::new();
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("D=D-M\n");
    result.push_str("@IF_TRUE0\n");
    result.push_str(";JEQ\n");
    result.push_str("D=0\n");
    result.push_str("@IF_FALSE0\n");
    result.push_str(";JMP\n");
    result.push_str("(IF_TRUE0)\n");
    result.push_str("D=-1\n");
    result.push_str("(IF_FALSE0)\n");
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("A=A-1\n");
    result.push_str("M=D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

// TODO: Refactor gt and lt
fn compile_gt() -> Result<String, String> {
    let mut result = String::new();

    result.push_str("@SP\n");
    result.push_str("A=M\n");
    result.push_str("A=A-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("D=M-D\n");
    result.push_str("@GREATER\n");
    result.push_str("D;JGT\n");
    result.push_str("D=0\n");
    result.push_str("@ELSE\n");
    result.push_str("0;JMP\n");
    result.push_str("(GREATER)\n");
    result.push_str("D=-1\n");
    result.push_str("(ELSE)\n");
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("A=A-1\n");
    result.push_str("M=D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

fn compile_lt() -> Result<String, String> {
    let mut result = String::new();

    result.push_str("@SP\n");
    result.push_str("A=M\n");
    result.push_str("A=A-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("D=M-D\n");
    result.push_str("@LESSER\n");
    result.push_str("D;JLT\n");
    result.push_str("D=0\n");
    result.push_str("@ELSE\n");
    result.push_str("0;JMP\n");
    result.push_str("(LESSER)\n");
    result.push_str("D=-1\n");
    result.push_str("(ELSE)\n");
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("A=A-1\n");
    result.push_str("M=D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

fn compile_and() -> Result<String, String> {
    let mut result = String::new();

    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("M=M&D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

fn compile_or() -> Result<String, String> {
    let mut result = String::new();

    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("D=M\n");
    result.push_str("A=A-1\n");
    result.push_str("M=M|D\n");
    result.push_str("@SP\n");
    result.push_str("M=M-1\n");

    Ok(result)
}

fn compile_not() -> Result<String, String> {
    let mut result = String::new();
    result.push_str("@SP\n");
    result.push_str("A=M-1\n");
    result.push_str("M=!M\n");

    Ok(result)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: vmcomp <path>");
    };

    let input_file_path = Path::new(&args[1]);

    let output_file_name = format!("{}.asm", &args[1]);
    let output_file_path = Path::new(&output_file_name);

    compile_file(input_file_path, output_file_path);
}
