use std::{env, fs, path::Path};

use parser::ast_printer;

mod emitter;
mod lexer;
mod parser;

fn main() {
    let source_filename = env::args().nth(1).expect("Missing source filename");
    let source_code = fs::read_to_string(source_filename).unwrap();
    let output_filename =
        Path::new(&env::args().nth(2).unwrap_or("./hello.exe".to_owned())).to_path_buf();

    let tokens = lexer::scanner::scan(source_code);
    let ast = parser::parse(tokens).unwrap();
    let output = ast_printer::visit_block(&ast);
    println!("{output}");

    emitter::build_executable(&ast, output_filename);
}
