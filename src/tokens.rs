#[derive(Debug)]
pub enum MathOperator {
    Add,
    Sub,
}

#[derive(Debug)]
pub enum CmpOperator {
    Less,
    Greater,
    Equal,
}

#[derive(Debug)]
pub enum StackOperation {
    Dup,
    Swap,
    Over,
    Rot,
    Drop,
}

#[derive(Debug)]
pub enum MemoryOperation {
    PushByte,           // pushes single byte into the local memory, returning the address,
    PushBytes(Vec<u8>), // pushes a sequence of bytes into local memory, returning the address,
    ChangeByte,         // takes the address from the stack and modifies the location in memory,
    LoadByte,
    ClearMemory, // takes the address and count from the stack and clears local memory
}

#[derive(Debug)]
pub enum Token {
    Push(usize),           // push value onto stack
    Math(MathOperator), // operations taking two values from the stack and pushing result of math operation onto stack
    Cmp(CmpOperator),   // operations taking two values from the stack and pushing either 0 or 1
    Stack(StackOperation), // operation operating directly on stack
    Memory(MemoryOperation),
    FunctionCall(String),

    // TODO: review control flow for the language
    If(Vec<Token>),   // if statement, consuming boolean value from stack
    Loop(Vec<Token>), // infinite loop. To exit loop use break
    Continue,
    Break, // exit the loop

    // TODO: this methods must be replaced by sane as soon as some type system is developed. This methods are absurd and only exist for the purpose of developing the basic language syntax
    Putc, // prints the top of the stack
    Putu,
    Debug, // prints the whole stack
}
