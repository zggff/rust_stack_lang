use stack_lang::io::Io;

fn main() {
    let program_source = std::fs::read_to_string(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| String::from("examples/hello_world.rsl")),
    )
    .unwrap();
    let program = stack_lang::program::Program::parse(&program_source);

    program.interpret(&mut Io::default());
    println!()
}
