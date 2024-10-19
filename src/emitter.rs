use ::std::path::PathBuf;

use ast::*;
use data::DataBuilder;
#[cfg(target_os = "linux")]
use elf::build;
#[cfg(target_os = "linux")]
use elf::sections::VIRTUAL_ADDRESS_START as IMAGE_BASE;
use symbols::SymbolResolver;
use text::TextBuilder;

#[cfg(target_os = "windows")]
use exe::build;
#[cfg(target_os = "windows")]
use exe::sections::IMAGE_BASE;

pub mod ast;
mod data;
#[cfg(target_os = "linux")]
pub mod elf;
#[cfg(target_os = "windows")]
pub mod exe;
mod symbols;
mod text;
mod stack;

pub fn build_executable(ast: &ast::StatementList, output_path: PathBuf) {
    let mut data_builder = DataBuilder::default();
    data_builder.visit_ast(ast);
    dbg!(&data_builder.symbol_data);

    let code_context = text::build_code_context(ast, &data_builder, IMAGE_BASE);

    let symbol_resolver = SymbolResolver::new();
    let symbols = symbol_resolver.resolve(&data_builder.symbol_data, &code_context.get_labels());

    build(
        output_path,
        &code_context,
        &data_builder.symbol_data,
        symbols.as_slice(),
        code_context.get_relocations(),
    );
}
