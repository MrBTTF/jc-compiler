use ::std::path::PathBuf;

use ast::*;
#[cfg(target_os = "linux")]
use elf::build;
#[cfg(target_os = "linux")]
use elf::sections::VIRTUAL_ADDRESS_START as IMAGE_BASE;
use symbols::SymbolResolver;

#[cfg(target_os = "windows")]
use exe::build;
#[cfg(target_os = "windows")]
use exe::sections::IMAGE_BASE;

pub mod ast;
#[cfg(target_os = "linux")]
pub mod elf;
#[cfg(target_os = "windows")]
pub mod exe;
mod stack;
mod symbols;
mod text;
mod variables;

pub fn build_executable(ast: &ast::Block, output_path: PathBuf) {
    let (variables, scopes) = variables::build_variables(ast);
    dbg!(&variables);

    let code_context = text::build_code_context(ast, &variables, &scopes, IMAGE_BASE);

    let symbol_resolver = SymbolResolver::new();
    let symbols = symbol_resolver.resolve(&variables, &code_context.get_labels());

    build(
        output_path,
        &code_context,
        symbols.as_slice(),
        code_context.get_relocations(),
    );
}
