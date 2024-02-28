use std::env;
use std::io::Write;

enum BaseConversionError {
    ParseIntError,
    InvalidInputFormat,
}

impl From<std::num::ParseIntError> for BaseConversionError {
    fn from(_: std::num::ParseIntError) -> Self {
        BaseConversionError::ParseIntError
    }
}

#[derive(Debug, PartialEq)]    
enum Token {
    Number(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen
}

fn parse_num(input: &str) -> Result<String, BaseConversionError> {
    if input.starts_with("0x") {
        i64::from_str_radix(&input[2..], 16)
            .map(|num| num.to_string())
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.starts_with("b") {
        i64::from_str_radix(&input[1..], 10)
            .map(|num| format!("{:b}b", num))
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.starts_with("Fx") {
        u64::from_str_radix(&input[2..], 16)
            .map(f64::from_bits)
            .map(|float| float.to_string())
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.starts_with("Bx") {
        i64::from_str_radix(&input[2..], 16)
            .map(|num| format!("{:b}", num))
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.starts_with("Ox") {
        i64::from_str_radix(&input[2..], 16)
            .map(|num| format!("{:o}", num))
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.ends_with("d") {
        i64::from_str_radix(&input[..input.len() - 1], 2)
            .map(|num| num.to_string())
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.ends_with("f") {
        input[..input.len() - 1].parse::<f64>()
            .map(|num| format!("0x{:x}", num.to_bits()))
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.ends_with("o") {
        i64::from_str_radix(&input[..input.len() - 1], 8)
            .map(|num| format!("0x{:x}", num))
            .map_err(|_| BaseConversionError::ParseIntError)
    } else if input.ends_with("b") {
        i64::from_str_radix(&input[..input.len() - 1], 2)
            .map(|num| format!("0x{:x}", num))
            .map_err(|_| BaseConversionError::ParseIntError)
    } else {
        i64::from_str_radix(input, 10)
            .map(|num| format!("0x{:x}", num))
            .map_err(|_| BaseConversionError::ParseIntError)
    }
}

// fn read_input() -> Vec<String> {
//     print!("> ");
//     std::io::stdout().flush().unwrap();
//     let mut input = String::new();
//     std::io::stdin()
//         .read_line(&mut input)
//         .expect("Cannot read input");
//     let args = input
//         .split_whitespace()
//         .map(|s| s.to_string())
//         .collect::<Vec<String>>();
//     args
// }

fn infix_to_postfix(tokens: Vec<Token>) -> Vec<Token> {
    // implements shunting yard algorithm to convert Vec<Token>
    // to reverse polish notation

    let mut output_queue: Vec<Token> = Vec::new();
    let mut operator_stack: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(_) => output_queue.push(token),
            Token::Plus | Token::Minus | Token::Star | Token::Slash => {
                while let Some(op) = operator_stack.last() {
                    if op != &Token::LParen {
                        output_queue.push(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                operator_stack.push(token);
            }
            Token::LParen => operator_stack.push(token),
            Token::RParen => {
                while let Some(op) = operator_stack.pop() {
                    if op == Token::LParen {
                        break;
                    } else {
                        output_queue.push(op);
                    }
                }
            }
        }
    }

    while let Some(op) = operator_stack.pop() {
        output_queue.push(op);
    }

    output_queue
}


fn parse_expr() -> Vec<Token> {
    print!("> ");
    let mut input = String::new();
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Cannot parse input expression");
    input = input.trim_end().to_string();

    // Tokenize
    let mut tokens = Vec::<Token>::new();
    let mut curr = Vec::<char>::new();
    let mut chars = input.as_str().chars().peekable();
    
    while let Some(&c) = chars.peek() {
        // println!("tokens: {:?}", tokens);
        // println!("curr: {:?}", curr);
        match c {
            ' ' => { chars.next(); }, // Skip spaces
            '+' | '-' | '/' | '*' | '(' | ')' => {
                if !curr.is_empty() {
                    let string: String = curr.iter().collect();
                    match parse_num(&string) {
                        Ok(num) => { tokens.push(Token::Number(num)); curr.clear(); },
                        Err(e) => println!("Could not convert number {}", string),
                    }
                }

                tokens.push(match c {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '/' => Token::Slash,
                    '*' => Token::Star,
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    _ => unreachable!(), // We've checked all cases
                });
                chars.next();
            },
            _ => { 
                curr.push(c);
                chars.next();
            }
        }
    }
    if !curr.is_empty() {
        let string: String = curr.into_iter().collect();
        match parse_num(&string) {
            Ok(num) => tokens.push(Token::Number(num)),
            Err(e) => println!("Could not parse number {}", string),
        }
    }
    // println!("{:?}", tokens);
    tokens
}

fn eval_expr(tokens: Vec<Token>) -> Result<i64, &'static str>{
    let mut iter = tokens.iter();
    let mut stack: Vec<i64> = Vec::new();

    while let Some(token) = iter.next() {
            // println!("stack: {:?}", stack);
        match token {
            Token::Number(num) => {
                // keep converting it until it's an int
                let mut temp_num = num.clone();
                while !temp_num.chars().all(|c| (c.is_numeric() || c == '.' || c == '-')) {
                    let parse_result = parse_num(&temp_num);
                    match parse_result {
                        Ok(result_num) => {
                            temp_num = result_num;
                        },
                        Err(e) => println!("Could not parse number")
                    }
                }
                stack.push(i64::from_str_radix(temp_num.as_str(), 10).unwrap());
            },

            Token::Plus => {
                let (a, b) = (stack.pop().ok_or("Invalid expression")?, stack.pop().ok_or("Invalid expression")?);
                stack.push(b + a);
            },

            Token::Minus => {
                let (a, b) = (stack.pop().ok_or("Invalid expression")?, stack.pop().ok_or("Invalid expression")?);
                stack.push(b - a);
            },

            Token::Star => {
                let (a, b) = (stack.pop().ok_or("Invalid expression")?, stack.pop().ok_or("Invalid expression")?);
                stack.push(b * a);
            },

            Token::Slash => {
                let (a, b) = (stack.pop().ok_or("Invalid expression")?, stack.pop().ok_or("Invalid expression")?);
                if a == 0 {
                    return Err("Division by zero");
                }
                stack.push(b / a);
            },
            _ => return Err("Unexpected token")
        }
    }
    stack.pop().ok_or("Invalid expression")
}

fn check_force_output(args: &Vec<String>) -> (Option<&'static str>) {
    let bases: [&'static str; 5] = ["f", "2", "8", "10", "16"];
    for arg in args {
        let arg_str = arg.as_str();
        if let Some(c) = arg_str.chars().next(){
            if c == '=' {
                let parts: Vec<&str> = arg_str.split('=').collect();
                if bases.contains(&parts[1]) {
                    for base in bases {
                        if base == parts[1] {
                            return Some(base)
                        }
                    }
                }
            }
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() == 1 {
        loop {
            let tokens = parse_expr();
            let result = eval_expr(infix_to_postfix(tokens));
            match result {
                Ok(r) => println!("{}", r),
                Err(e) => println!("{}", e)
            }
        }
    }
    else {
        let base = check_force_output(&args);
        let mut starting_index = 1;
        if base.is_some() {
            starting_index = 2;
        }
        for input in &args[starting_index..] {
            match parse_num(input) {
                Ok(result) => {
                    let mut temp_num = 0;
                    let mut temp_num_str = result.clone();
                    while !temp_num_str.chars().all(|c| (c.is_numeric() || c == '.' || c == '-')) {
                        let parse_result = parse_num(&temp_num_str);
                        match parse_result {
                            Ok(result_num) => {
                                temp_num_str = result_num;
                            },
                            Err(e) => println!("Could not parse number")
                        }
                    }
                    match i64::from_str_radix(&temp_num_str, 10) {
                        Ok(num) => { temp_num = num; },
                        Err(_) => println!("Failed to convert expression result")
                    }
                    if base.is_some() {
                        match base {
                            Some("f") => println!("{:.5}", temp_num),
                            Some("2") => println!("b{:b}", temp_num),
                            Some("8") => println!("Ox{:o}", temp_num),
                            Some("10") => println!("{}", temp_num),
                            Some("16") => println!("0x{:x}", temp_num),
                            _ => println!("{}", result)
                        }
                    }
                },
                Err(BaseConversionError::ParseIntError) => println!("Error: Failed to parse input"),
                Err(BaseConversionError::InvalidInputFormat) => println!("Error: Invalid input format"),
            }
        }
    }
}