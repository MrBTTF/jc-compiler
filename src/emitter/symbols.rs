use std::collections::BTreeMap;

use crate::emitter::variables::ValueType;

use super::variables::{ValueLocation, Variable};

// pub enum Symbol{
//     Variable(VariableSymbol),
//     Function,
// }

// #[derive(Debug, Clone)]
// pub struct VariableSymbol {
//     location: usize,
//     data_type: DataType,
// }

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
    data: Vec<u8>,
}
impl Symbol {
    fn new(
        name: String,
        offset: usize,
        section: Section,
        _type: SymbolType,
        scope: SymbolScope,
        data: Vec<u8>,
    ) -> Self {
        Self {
            name,
            offset,
            section,
            _type,
            scope,
            data,
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

    pub fn get_data(&self) -> &[u8] {
        &self.data
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
        variables: &BTreeMap<String, Variable>,
        labels: &BTreeMap<String, usize>,
    ) -> Vec<Symbol> {
        let mut symbols = vec![];

        let mut data_loc = 0;

        for (id, data) in variables {
            if matches!(data.value_loc, ValueLocation::Stack(_)) {
                continue;
            }

            let data_bytes = match &data.value_type {
                ValueType::String(string) => string.clone().into_bytes(),
                ValueType::Int(n) => n.to_le_bytes().to_vec(),
            };
            symbols.push(Symbol::new(
                id.clone(),
                data_loc as usize,
                Section::Data,
                SymbolType::Data(DataSymbol::Comptime),
                SymbolScope::Local,
                data_bytes,
            ));
            data_loc += data.value_size;
        }

        for (label, offset) in labels {
            symbols.push(Symbol::new(
                label.clone(),
                *offset,
                Section::Text,
                SymbolType::Text,
                SymbolScope::Local,
                vec![],
            ));
        }

        symbols
    }
}
