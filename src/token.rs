#[derive(Debug)]
pub enum MathOperator {
    Add,
    Sub,
    Mul,
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

// <- to load variable
// -> to store variable
#[derive(Debug)]
pub enum MemoryOperation {
    // PushByte,           // pushes single byte into the local memory, returning the address,
    PushBytes(Vec<u8>), // pushes a sequence of bytes into local memory, returning the address,
    StoreByte,          // takes the address from the stack and modifies the location in memory,
    LoadByte,
    Free, // takes the address and count from the stack and clears local memory
    Alloc,
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
    IfBlock(Vec<Token>, Vec<Token>), // if statement, consuming boolean value from stack
    // TODO: deprecate loop in favour of while 1
    LoopBlock(Vec<Token>), // infinite loop. To exit loop use break

    WhileBlock(Vec<Token>, Vec<Token>), // first is the condition, the second is the body of the loop
    Continue,
    Break,                             // exit the loop
    LetBlock(Vec<Token>, Vec<String>), // scope for the let bindings,
    Let(String),                       // get let binding

    // TODO: this methods must be replaced by sane as soon as some type system is developed. This methods are absurd and only exist for the purpose of developing the basic language syntax
    Putc, // prints the top of the stack
    Putu,
    Debug, // prints the whole stack
}
