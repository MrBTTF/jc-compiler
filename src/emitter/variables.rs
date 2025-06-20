use crate::emitter::ast::Literal;
use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    mem,
};

use super::{ast, Integer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueType {
    String(String),
    Int(i64),
}

impl From<ast::Literal> for ValueType {
    fn from(value: ast::Literal) -> Self {
        match value {
            Literal::String(s) => ValueType::String(s),
            Literal::Integer(i) => ValueType::Int(i.value),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DataRef {
    // Used only for Windows target
    pub symbol: String,
    pub offset: usize,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum StackLocation {
    Block(u64),
    Function(u64),
}

impl From<StackLocation> for u64 {
    fn from(value: StackLocation) -> Self {
        match value {
            StackLocation::Block(loc) => loc,
            StackLocation::Function(loc) => loc,
        }
    }
}

impl From<StackLocation> for u32 {
    fn from(value: StackLocation) -> Self {
        u64::from(value) as u32
    }
}

impl From<&StackLocation> for u64 {
    fn from(value: &StackLocation) -> Self {
        value.clone().into()
    }
}

impl From<&StackLocation> for u32 {
    fn from(value: &StackLocation) -> Self {
        value.clone().into()
    }
}

#[derive(Clone, Debug)]
pub enum ValueLocation {
    Stack(StackLocation),
    DataSection(u64),
}

impl From<ValueLocation> for u64 {
    fn from(value: ValueLocation) -> Self {
        match value {
            ValueLocation::Stack(stack_loc) => stack_loc.into(),
            ValueLocation::DataSection(value_loc) => value_loc,
        }
    }
}

impl From<ValueLocation> for u32 {
    fn from(value: ValueLocation) -> Self {
        u64::from(value) as u32
    }
}

impl From<&ValueLocation> for u64 {
    fn from(value: &ValueLocation) -> Self {
        value.into()
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub value_type: ValueType,
    pub value_size: usize,
    pub value_loc: ValueLocation,
    pub reference: bool,
}

impl Variable {
    pub fn new(
        name: &str,
        value_type: ValueType,
        reference: bool,
        value_loc: ValueLocation,
    ) -> Variable {
        let value_size = match &value_type {
            ValueType::String(s) => match value_loc {
                ValueLocation::Stack(_) => s.len() + mem::size_of::<u64>(),
                ValueLocation::DataSection(_) => s.len(),
            },
            ValueType::Int(_) => mem::size_of::<i64>(),
        };

        Variable {
            name: name.to_string(),
            value_type,
            value_size,
            value_loc,
            reference,
        }
    }

    pub fn as_vec(&self) -> Vec<u8> {
        match &self.value_type {
            ValueType::String(s) => [s.as_bytes().to_vec(), vec![0]].concat(),
            ValueType::Int(i) => i.to_le_bytes().to_vec(),
        }
    }
}

pub fn build_variables(
    block: &ast::Block,
) -> (BTreeMap<String, Variable>, HashMap<String, Vec<String>>) {
    let mut variables_collector = VariablesCollector::default();
    variables_collector.visit_ast(block);
    (
        variables_collector.variables,
        variables_collector.scope_variable,
    )
}

#[derive(Default, Debug)]
pub struct VariablesCollector {
    pub variables: BTreeMap<String, Variable>,
    pub scope_variable: HashMap<String, Vec<String>>,
    data_section: Vec<usize>,
}

impl VariablesCollector {
    pub fn visit_ast(&mut self, block: &ast::Block) {
        let number_fmt = "%d\0".to_string();
        self.data_section.push(number_fmt.len());
        let value_type = ValueType::String(number_fmt);
        let printf_d_arg = "global::__printf_d_arg";
        self.variables.insert(
            printf_d_arg.to_string(),
            Variable::new(
                &printf_d_arg,
                value_type,
                false,
                ValueLocation::DataSection(0),
            ),
        );
        self.add_to_scope(&block.scope, vec![printf_d_arg.to_string()]);

        self.visit_block(block);
    }

    pub fn visit_block(&mut self, block: &ast::Block) {
        block.stmts.iter().for_each(|stmt| match stmt {
            ast::Statement::Expression(_) => (),
            ast::Statement::Loop(l) => self.visit_loop(l),
            ast::Statement::VarDeclaration(var_declaration) => {
                self.visit_var_declaration(var_declaration, &block.scope);
            }

            ast::Statement::FuncDeclaration(func_declaration) => {
                self.visit_func_declaration(func_declaration)
            }
            ast::Statement::Block(block) => self.visit_block(block),
            ast::Statement::Assignment(_) => (),
            ast::Statement::ControlFlow(_) => (),
        });
    }

    fn visit_var_declaration(&mut self, var_decl: &ast::VarDeclaration, scope: &str) {
        let expr = match &var_decl.rhs {
            ast::RhsExpression::Expression(expr) => expr,
            ast::RhsExpression::Block(block) => todo!(),
        };

        match expr {
            ast::Expression::Literal(lit) => {
                let value_loc = match var_decl.declarion_type {
                    ast::VarDeclarationType::Let => ValueLocation::Stack(StackLocation::Block(0)),
                    ast::VarDeclarationType::Const => ValueLocation::DataSection(0),
                };
                let id = format!("{}::{}", scope, &var_decl.name.value);
                self.variables.insert(
                    id.clone(),
                    Variable::new(&id, lit.clone().into(), false, value_loc),
                );
                self.add_to_scope(&scope, vec![id]);
            }
            _ => todo!(),
        }
    }

    fn visit_func_declaration(&mut self, func_decl: &ast::FuncDeclaration) {
        for arg in &func_decl.args {
            let has_ref = !arg._type.modifiers.is_empty();
            let lit = match arg._type.name {
                ast::TypeName::String => Literal::String("".to_string()),
                ast::TypeName::Int => Literal::Integer(Integer { value: 0 }),
                ast::TypeName::Float => todo!(),
                ast::TypeName::Bool => todo!(),
                ast::TypeName::Unit => todo!(),
            };

            let id = format!("{}::{}", func_decl.body.scope, &arg.name.value);

            self.variables.insert(
                id.clone(),
                Variable::new(
                    &id,
                    lit.into(),
                    has_ref,
                    ValueLocation::Stack(StackLocation::Function(0)),
                ),
            );
            self.add_to_scope(&func_decl.body.scope, vec![id.clone()]);
        }

        self.visit_block(&func_decl.body);
    }

    // fn visit_declaration(&mut self, statement: &ast::Statement) {}
    fn visit_loop(&mut self, l: &ast::Loop) {
        let id = &l.var;
        let lit = Literal::Integer(ast::Integer {
            value: l.start as i64,
        });

        let value_loc = ValueLocation::Stack(StackLocation::Block(0));

        let id = format!("{}::{}", l.body.scope, &id.value);
        self.variables
            .insert(id.clone(), Variable::new(&id, lit.into(), false, value_loc));

        self.visit_block(&l.body);
        self.add_to_scope(&l.body.scope, vec![id.clone()]);
    }

    fn add_to_scope(&mut self, parent: &str, symbols: Vec<String>) {
        match self.scope_variable.entry(parent.to_string()) {
            Entry::Vacant(v) => {
                v.insert(symbols);
            }
            Entry::Occupied(mut o) => {
                o.get_mut().extend(symbols);
            }
        }
    }
}
