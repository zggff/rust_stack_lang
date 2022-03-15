use std::{io::Write, mem::size_of};

#[derive(Debug)]
enum MathOperator {
    Add,
    Sub,
}

impl MathOperator {
    fn apply(&self, a: isize, b: isize) -> isize {
        match self {
            MathOperator::Add => a + b,
            MathOperator::Sub => a - b,
        }
    }
}

#[derive(Debug)]
enum CmpOperator {
    Less,
    Greater,
    Equal,
}

impl CmpOperator {
    fn apply(&self, a: isize, b: isize) -> bool {
        match self {
            CmpOperator::Less => a < b,
            CmpOperator::Greater => a > b,
            CmpOperator::Equal => a == b,
        }
    }
}

#[derive(Debug)]
enum StackOperation {
    Dup,
    Swap,
    Over,
    Rot,
    Drop,
}

impl StackOperation {
    fn apply(&self, stack: &mut Vec<isize>) {
        match self {
            Self::Dup => stack.push(*stack.last().unwrap()),
            Self::Swap => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a);
                stack.push(b);
            }
            Self::Over => {
                let a = *stack.get(stack.len() - 2).unwrap();
                stack.push(a);
            }
            Self::Rot => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let c = stack.pop().unwrap();
                stack.push(b);
                stack.push(a);
                stack.push(c);
            }
            Self::Drop => {
                stack.pop();
            }
        }
    }
}

#[derive(Debug)]
enum Token {
    Push(isize),           // push value onto stack
    Math(MathOperator), // operations taking two values from the stack and pushing result of math operation onto stack
    Cmp(CmpOperator),   // operations taking two values from the stack and pushing either 0 or 1
    Stack(StackOperation), // operation operating directly on stack

    // TODO: review control flow for the language
    If(Vec<Token>),   // if statement, consuming boolean value from stack
    Loop(Vec<Token>), // infinite loop. To exit loop use break
    Continue,
    Break, // exit the loop

    // TODO: this methods must be replaced by sane as soon as some type system is developed. This methods are absurd and only exist for the purpose of developing the basic language syntax
    Put,           // prints the top of the stack
    Print(String), // prints string literal
    Debug,         // prints the whole stack
}

fn parse(code: &mut impl Iterator<Item = char>) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_symbol = String::new();
    let mut comment = false;
    let mut current_symbol_end = false;
    while let Some(char) = code.next() {
        if comment {
            if char == '\n' || (char == '*' && code.next().unwrap() == '/') {
                comment = false;
                current_symbol_end = false;
            }
            continue;
        }
        match char {
            ' ' | '\t' | '\n' => {
                current_symbol_end = true;
            }
            char => {
                current_symbol.push(char);
            }
        }
        if current_symbol_end {
            current_symbol_end = false;
            match current_symbol.as_str() {
                "" => {}

                // math operations
                "+" => tokens.push(Token::Math(MathOperator::Add)),
                "-" => tokens.push(Token::Math(MathOperator::Sub)),

                // boolean operations
                "<" => tokens.push(Token::Cmp(CmpOperator::Less)),
                ">" => tokens.push(Token::Cmp(CmpOperator::Greater)),
                "=" => tokens.push(Token::Cmp(CmpOperator::Equal)),

                // stack operations
                "dup" => tokens.push(Token::Stack(StackOperation::Dup)),
                "swap" => tokens.push(Token::Stack(StackOperation::Swap)),
                "over" => tokens.push(Token::Stack(StackOperation::Over)),
                "rot" => tokens.push(Token::Stack(StackOperation::Rot)),
                "drop" => tokens.push(Token::Stack(StackOperation::Drop)),

                // control flow operations
                "break" => tokens.push(Token::Break),
                "continue" => tokens.push(Token::Continue),
                "end" => return tokens,
                "loop" => tokens.push(Token::Loop(parse(code))),
                "if" => tokens.push(Token::If(parse(code))),

                // general fuctions
                "print" => {
                    let mut quotes_count = 0;
                    let mut string_literal = String::new();
                    for char in code.by_ref() {
                        if char == '"' {
                            quotes_count += 1;
                        } else if quotes_count >= 2 {
                            break;
                        } else {
                            string_literal.push(char);
                        }
                    }
                    dbg!(&string_literal);
                    tokens.push(Token::Print(string_literal))
                }
                "put" => tokens.push(Token::Put),
                "dbg" => tokens.push(Token::Debug),

                // comments
                "//" | "/*" => comment = true,
                "/**/" => {}

                symbol => {
                    let value = symbol.parse::<isize>();
                    match value {
                        Ok(value) => {
                            tokens.push(Token::Push(value));
                        }
                        Err(_e) => {
                            dbg!(tokens);
                            panic!("Unknown symbol: {}", symbol)
                        }
                    };
                }
            }
            current_symbol.clear();
        }
    }
    tokens
}

#[derive(Debug)]
enum InterpretationStatus {
    Exit,
    Break,
    Continue,
    None,
}

fn interpret(tokens: &[Token], stack: &mut Vec<isize>, status: &mut InterpretationStatus) {
    for token in tokens {
        match token {
            Token::Push(value) => {
                stack.push(*value);
            }
            Token::Math(operand) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(operand.apply(a, b));
            }
            Token::Cmp(operand) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(operand.apply(a, b) as isize);
            }
            Token::Stack(operand) => {
                operand.apply(stack);
            }
            Token::Print(value) => {
                print!("{}", value);
                std::io::stdout().flush().unwrap();
            }
            Token::Put => {
                print!("{}", stack.pop().unwrap());
                std::io::stdout().flush().unwrap();
            }
            Token::Debug => {
                println!("{:?}", stack);
            }
            Token::If(subprogram) => {
                if stack.pop().unwrap() != 0 {
                    interpret(subprogram, stack, status);
                    match status {
                        InterpretationStatus::None => {}
                        _ => return,
                    }
                }
            }
            Token::Loop(subprogram) => loop {
                interpret(subprogram, stack, status);
                match status {
                    InterpretationStatus::Continue => {
                        *status = InterpretationStatus::None;
                        continue;
                    }
                    InterpretationStatus::Break => {
                        break;
                    }
                    _ => {}
                }
            },
            Token::Break => {
                *status = InterpretationStatus::Break;
                return;
            }

            Token::Continue => {
                *status = InterpretationStatus::Continue;
                return;
            }

            // TODO:remove this unreachable arm after most tokens are filled int
            token => {
                unimplemented!("{:?}", token)
            }
        }
    }
}

fn main() {
    let program_source = std::fs::read_to_string(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| String::from("1.rsl")),
    )
    .unwrap();
    let mut program_chars = program_source.chars();
    let program = parse(&mut program_chars);
    dbg!(size_of::<Token>());
    dbg!(&program);
    interpret(
        &program,
        &mut Vec::with_capacity(1000),
        &mut InterpretationStatus::None,
    );
    println!()
}
