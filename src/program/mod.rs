use crate::io::Io;
use crate::token::*;
use std::{collections::HashMap, io::Write, iter::Peekable, mem};

mod memory;
use memory::Memory;

#[derive(Debug)]
enum InterpretationStatus {
    Break,
    Continue,
    None,
}

#[derive(Debug)]
pub struct Program {
    functions: HashMap<String, Vec<Token>>,
}

impl Program {
    pub fn parse(code: &str) -> Self {
        let mut functions = HashMap::new();
        let mut code = Tokens::new(code).peekable();
        while let Some(token) = code.next() {
            match token.as_str() {
                "fn" => {
                    let function_name = code.next().unwrap();
                    match code.next().as_deref() {
                        Some("{") => {
                            let function = Self::parse_code_segment(&mut code, &functions, &vec![]);
                            functions.insert(function_name, function);
                        }
                        Some(token) => {
                            panic!("unsupported symbol: {token}, '{{' expected");
                        }
                        None => {
                            panic!("unexpected end of file");
                        }
                    }
                }

                symbol => {
                    panic!("umrecognised symbol on top level of program: {symbol}; Expected one of the following values: [fn]")
                }
            };
        }

        Self { functions }
    }

    // this function handles the parsing of funtion bodies
    fn parse_code_segment(
        code: &mut Peekable<Tokens>,
        functions: &HashMap<String, Vec<Token>>,
        lets: &Vec<String>,
    ) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = code.next() {
            match token.as_str() {
                // math operations
                "+" => tokens.push(Token::Math(MathOperator::Add)),
                "-" => tokens.push(Token::Math(MathOperator::Sub)),
                "*" => tokens.push(Token::Math(MathOperator::Mul)),

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
                "}" => return tokens,
                "loop" => match code.next().as_deref() {
                    Some("{") => {
                        tokens.push(Token::LoopBlock(Self::parse_code_segment(
                            code, functions, lets,
                        )));
                    }
                    Some(token) => {
                        panic!("unsupported symbol: {token}, '{{' expected",);
                    }
                    None => {
                        panic!("unexpected end of file");
                    }
                },
                "if" => match code.next().as_deref() {
                    Some("{") => {
                        let true_block = Self::parse_code_segment(code, functions, lets);
                        let false_block = if code.next_if(|token| token == "else").is_some() {
                            match code.next().as_deref() {
                                Some("{") => Self::parse_code_segment(code, functions, lets),
                                Some(token) => {
                                    panic!("unsupported symbol: {token}, '{{' expected");
                                }
                                None => {
                                    panic!("unexpected end of file");
                                }
                            }
                        } else {
                            vec![]
                        };
                        tokens.push(Token::IfBlock(true_block, false_block));
                    }
                    Some(token) => {
                        panic!("unsupported symbol: {token}, '{{' expected",);
                    }
                    None => {
                        panic!("unexpected end of file");
                    }
                },

                // TODO: replace this with proper output after access to memory and arrays are added to the language
                "putc" => tokens.push(Token::Putc),
                "putu" => tokens.push(Token::Putu),
                "???" => tokens.push(Token::Debug),
                "<-" => tokens.push(Token::Memory(MemoryOperation::LoadByte)),
                "->" => tokens.push(Token::Memory(MemoryOperation::StoreByte)),
                "alloc" => tokens.push(Token::Memory(MemoryOperation::Alloc)),
                "free" => tokens.push(Token::Memory(MemoryOperation::Free)),
                "let" => {
                    let mut let_bindings = Vec::new();
                    let mut new_lets = lets.clone();
                    while let Some(token) = code.next() {
                        if token == "{" {
                            new_lets.extend(let_bindings.clone());
                            tokens.push(Token::LetBlock(
                                Self::parse_code_segment(code, functions, &new_lets),
                                let_bindings,
                            ));
                            break;
                        } else {
                            let_bindings.push(token);
                        }
                    }
                }
                "while" => {
                    if let Some("{") = code.next().as_deref() {
                        let condition = Self::parse_code_segment(code, functions, lets);
                        if let Some("{") = code.next().as_deref() {
                            let loop_body = Self::parse_code_segment(code, functions, lets);
                            tokens.push(Token::WhileBlock(condition, loop_body));
                        }
                    }
                }

                token => {
                    if let Ok(value) = token.parse::<usize>() {
                        tokens.push(Token::Push(value));
                    } else if token.starts_with('"') && token.ends_with('"') {
                        let mut data = token[1..token.len() - 1].as_bytes().to_vec();
                        data.push(0);
                        tokens.push(Token::Memory(MemoryOperation::PushBytes(data)));
                    } else if let Some(_function) = functions.get(token) {
                        tokens.push(Token::FunctionCall(token.to_string()));
                    } else if lets.contains(&token.to_string()) {
                        tokens.push(Token::Let(token.to_string()))
                    } else {
                        panic!("Unknown token: {token}",);
                    }
                }
            }
        }
        tokens
    }

    pub fn interpret<W: Write>(&self, io: &mut Io<W>) {
        let main = self
            .functions
            .get("main")
            .expect("no main function provided");
        self.interpret_segment(
            main,
            &mut Vec::with_capacity(1000),
            &mut Memory::new(),
            &HashMap::new(),
            &mut InterpretationStatus::None,
            io,
        )
    }

    fn interpret_segment<W: Write>(
        &self,
        segment: &[Token],
        stack: &mut Vec<usize>,
        memory: &mut Memory,
        variables: &HashMap<String, usize>,
        status: &mut InterpretationStatus,
        io: &mut Io<W>,
    ) {
        for token in segment {
            match token {
                Token::Push(value) => {
                    stack.push(*value);
                }
                Token::Math(operand) => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = match operand {
                        MathOperator::Add => a + b,
                        MathOperator::Sub => a - b,
                        MathOperator::Mul => a * b,
                    };
                    stack.push(result);
                }
                Token::Cmp(operand) => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = match operand {
                        CmpOperator::Less => a < b,
                        CmpOperator::Greater => a > b,
                        CmpOperator::Equal => a == b,
                    };
                    stack.push(result as usize);
                }
                Token::Stack(operand) => match operand {
                    StackOperation::Dup => stack.push(*stack.last().unwrap()),
                    StackOperation::Swap => {
                        let a = stack.pop().unwrap();
                        let b = stack.pop().unwrap();
                        stack.push(a);
                        stack.push(b);
                    }
                    StackOperation::Over => {
                        let a = *stack.get(stack.len() - 2).unwrap();
                        stack.push(a);
                    }
                    StackOperation::Rot => {
                        let a = stack.pop().unwrap();
                        let b = stack.pop().unwrap();
                        let c = stack.pop().unwrap();
                        stack.push(b);
                        stack.push(a);
                        stack.push(c);
                    }
                    StackOperation::Drop => {
                        stack.pop();
                    }
                },
                Token::Memory(operand) => match operand {
                    MemoryOperation::PushBytes(data) => {
                        let address = memory.extend(data);
                        stack.push(address);
                    }
                    MemoryOperation::LoadByte => {
                        let address = stack.pop().unwrap();
                        let value = memory.get(address).unwrap();
                        stack.push(*value as usize);
                    }
                    MemoryOperation::StoreByte => {
                        let value = stack.pop().unwrap();
                        let address = stack.pop().unwrap();
                        memory.set(address, value as u8);
                    }
                    MemoryOperation::Free => {
                        let len = stack.pop().unwrap();
                        let address = stack.pop().unwrap();
                        memory.remove(address, len);
                    }
                    MemoryOperation::Alloc => {
                        let len = stack.pop().unwrap();
                        let address = memory.alloc(len);
                        stack.push(address);
                    }
                },
                Token::Putc => {
                    write!(
                        io,
                        "{}",
                        char::from_u32(stack.pop().unwrap() as u32).unwrap()
                    )
                    .unwrap();
                    io.flush().unwrap();
                }
                Token::Putu => {
                    write!(io, "{}", stack.pop().unwrap()).unwrap();
                    std::io::stdout().flush().unwrap();
                }
                Token::Debug => {
                    writeln!(io, "{stack:?} {memory:?}").unwrap();
                }
                Token::IfBlock(true_block, false_block) => {
                    let segment = if stack.pop().unwrap() != 0 {
                        true_block
                    } else {
                        false_block
                    };
                    self.interpret_segment(segment, stack, memory, variables, status, io);
                    match status {
                        InterpretationStatus::None => {}
                        _ => return,
                    }
                }
                Token::LoopBlock(segment) => loop {
                    self.interpret_segment(segment, stack, memory, variables, status, io);
                    match status {
                        InterpretationStatus::Continue => {
                            *status = InterpretationStatus::None;
                            continue;
                        }
                        InterpretationStatus::Break => {
                            *status = InterpretationStatus::None;
                            break;
                        }
                        _ => {}
                    }
                },
                Token::WhileBlock(condition, segment) => loop {
                    self.interpret_segment(condition, stack, memory, variables, status, io);
                    if stack.pop().unwrap() == 0 {
                        break;
                    }

                    self.interpret_segment(segment, stack, memory, variables, status, io);
                    match status {
                        InterpretationStatus::Continue => {
                            *status = InterpretationStatus::None;
                            continue;
                        }
                        InterpretationStatus::Break => {
                            *status = InterpretationStatus::None;
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

                Token::FunctionCall(function) => self.interpret_segment(
                    self.functions
                        .get(function)
                        .expect("no function with this name found"),
                    stack,
                    memory,
                    variables,
                    status,
                    io,
                ),
                Token::LetBlock(segment, let_bindings) => {
                    let mut new_variables = variables.clone();
                    for let_binding in let_bindings {
                        new_variables.insert(let_binding.clone(), stack.pop().unwrap());
                    }
                    self.interpret_segment(segment, stack, memory, &new_variables, status, io);
                    match status {
                        InterpretationStatus::None => {}
                        _ => return,
                    }
                }
                Token::Let(let_binding) => {
                    let value = variables.get(let_binding).unwrap();
                    stack.push(*value);
                }
            }
        }
    }
}

struct Tokens<'a> {
    code: std::str::Chars<'a>,
}

impl<'a> Tokens<'a> {
    pub fn new(code: &'a str) -> Self {
        Tokens { code: code.chars() }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut accumulator = String::new();
        let mut last_char = ' ';
        let mut is_comment = false;
        let separators = vec![' ', '\n', '\t'];

        while let Some(char) = self.code.next() {
            match char {
                '\n' if is_comment => is_comment = false,
                // this allows not to check for comments in the parsing function, as it consumes the iterator until the next buffer
                '/' if last_char == '/' => {
                    accumulator.pop();
                    is_comment = true
                }
                '"' if !is_comment => {
                    accumulator.push('"');
                    for char in self.code.by_ref() {
                        accumulator.push(char);
                        if char == '"' {
                            return Some(accumulator);
                        }
                    }
                }
                // WARNING: current next_token fails to parse code like: "fn main{}"; whitespace is required
                char if separators.contains(&char) => {
                    if !is_comment && !accumulator.is_empty() {
                        return Some(accumulator);
                    }
                }

                char if !is_comment => {
                    last_char = char;
                    accumulator.push(char)
                }
                _ => {}
            }
        }
        None
    }
}

fn next_token(chars: &mut impl Iterator<Item = char>) -> Option<String> {
    let mut accumulator = String::new();
    let mut last_char = ' ';
    let mut is_comment = false;
    let separators = vec![' ', '\n', '\t'];

    while let Some(char) = chars.next() {
        match char {
            '\n' if is_comment => is_comment = false,
            // this allows not to check for comments in the parsing function, as it consumes the iterator until the next buffer
            '/' if last_char == '/' => {
                accumulator.pop();
                is_comment = true
            }
            '"' if !is_comment => {
                accumulator.push('"');
                for char in chars.by_ref() {
                    accumulator.push(char);
                    if char == '"' {
                        return Some(accumulator);
                    }
                }
            }
            // WARNING: current next_token fails to parse code like: "fn main{}"; whitespace is required
            char if separators.contains(&char) => {
                if !is_comment && !accumulator.is_empty() {
                    return Some(accumulator);
                }
            }

            char if !is_comment => {
                last_char = char;
                accumulator.push(char)
            }
            _ => {}
        }
    }
    None
}

#[test]
fn test_next_token() {
    let string = r#"
        //test
        fn main {
            hello
            "test string"
        }
    "#;
    let code = &mut Tokens::new(string);
    assert_eq!(code.next(), Some(String::from("fn")));
    assert_eq!(code.next(), Some(String::from("main")));
    assert_eq!(code.next(), Some(String::from("{")));
    assert_eq!(code.next(), Some(String::from("hello")));
    assert_eq!(code.next(), Some(String::from("\"test string\"")));
    assert_eq!(code.next(), Some(String::from("}")));
    assert_eq!(code.next(), None);
}

macro_rules! test_program_output {
    ($code: expr, $output: expr) => {{
        let program = Program::parse($code);
        let mut writer = vec![];
        let mut io = Io::new(&mut writer);
        program.interpret(&mut io);
        assert_eq!(writer, $output);
    }};
}

#[test]
fn test_interpreter() {
    test_program_output!(
        r#"
        fn main {
            69 putu
            10 putc
        }
        "#,
        "69\n".as_bytes()
    );
}
