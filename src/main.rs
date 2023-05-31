use std::{env, fs};

mod emitter;
mod lexer;
mod parser;

// use emitter::build_elf;

fn main() {
    let source_filename = env::args().nth(1).expect("Missing source filename");
    let source_code = fs::read_to_string(source_filename).unwrap();

    let tokens = lexer::scanner::scan(source_code);
    let ast = parser::parse(tokens);

    // build_elf()
}
