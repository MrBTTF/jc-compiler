use std::collections::{BTreeMap, HashMap};

use super::data::{Data, DataLocation};

#[derive(Debug, Clone, Copy)]
pub enum SymbolType {
    Data(DataSymbol),
    Text,
}

#[derive(Debug, Clone, Copy)]
pub enum DataSymbol {
    Comptime,
    Runtime,
}

#[derive(Debug, Clone, Copy)]
pub enum Section {
    Undefined,
    Text,
    Data,
    Absolute,
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolScope {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,
    offset: usize,
    section: Section,
    _type: SymbolType,
    scope: SymbolScope,
}
impl Symbol {
    fn new(
        name: String,
        offset: usize,
        section: Section,
        _type: SymbolType,
        scope: SymbolScope,
    ) -> Self {
        Self {
            name,
            offset,
            section,
            _type,
            scope,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn get_section(&self) -> Section {
        self.section
    }

    pub fn get_type(&self) -> SymbolType {
        self._type
    }

    pub fn get_scope(&self) -> SymbolScope {
        self.scope
    }
}

#[derive(Debug, Clone)]
pub struct Relocation {
    symbol: String,
    offset: usize,
    _type: SymbolType,
}

impl Relocation {
    pub fn new(symbol: String, offset: usize, _type: SymbolType) -> Self {
        Self {
            symbol,
            offset,
            _type,
        }
    }

    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn get_type(&self) -> SymbolType {
        self._type
    }
}

pub struct SymbolResolver {}
impl SymbolResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn resolve(
        &self,
        symbol_data: &BTreeMap<String, Data>,
        labels: &BTreeMap<String, usize>,
    ) -> Vec<Symbol> {
        let mut symbols = vec![];

        for (id, data) in symbol_data {
            let data_loc = match data.data_loc {
                DataLocation::Stack(_) => continue,
                DataLocation::DataSection(data_loc) => data_loc,
            };
            symbols.push(Symbol::new(
                id.clone(),
                data_loc as usize,
                Section::Data,
                SymbolType::Data(DataSymbol::Comptime),
                SymbolScope::Local,
            ));
        }

        for (label, offset) in labels {
            symbols.push(Symbol::new(
                label.clone(),
                *offset,
                Section::Text,
                SymbolType::Text,
                SymbolScope::Local,
            ));
        }

        symbols
    }
}
