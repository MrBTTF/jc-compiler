use std::{env, fs};

use emitter::ast::Visitor;

use parser::ast_printer::AstPrinter;

use crate::emitter::{elf::build_elf, exe::build_exe};

mod emitter;
mod lexer;
mod parser;


fn main() {
    let source_filename = env::args().nth(1).expect("Missing source filename");
    let source_code = fs::read_to_string(source_filename).unwrap();

    let tokens = lexer::scanner::scan(source_code);
    let ast = parser::parse(tokens);
    let output = AstPrinter {}.visit_statement_list(&ast);
    println!("{output}");

    build_exe(&ast);
}
