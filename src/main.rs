mod program;
mod tokens;
use program::Program;

fn main() {
    let program_source = std::fs::read_to_string(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| String::from("1.rsl")),
    )
    .unwrap();
    let program = Program::parse(&program_source);
    program.interpret();
    println!()
}
