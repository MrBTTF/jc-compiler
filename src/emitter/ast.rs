use std::{fmt::Display, mem};

#[derive(Debug, Clone)]
pub struct StatementList(pub Vec<Statement>);

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Declaration(Declaration),
    Assignment(Assignment),
    FuncDefinition(FuncDefinition),
    ControlFlow(ControlFlow),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationType {
    Let,
    Const,
}
impl Display for DeclarationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclarationType::Let => write!(f, "let"),
            DeclarationType::Const => write!(f, "const"),
        }
    }
}

impl TryFrom<&str> for DeclarationType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s == "let" {
            Ok(DeclarationType::Let)
        } else if s == "const" {
            Ok(DeclarationType::Const)
        } else {
            Err(format!("invalid assignemnt type: {s}"))
        }
    }
}

#[derive(Debug, Clone)]

pub struct Declaration(pub Ident, pub Expression, pub DeclarationType);

#[derive(Debug, Clone)]
pub struct FuncDefinition(
    pub Ident,
    pub Vec<Arg>,
    pub Option<Ident>,
    pub StatementList,
);

#[derive(Debug, Clone)]
pub struct Arg(pub Ident, pub Ident);

#[derive(Debug, Clone)]

pub struct Assignment(pub Ident, pub Expression);

#[derive(Debug, Clone)]
pub enum Expression {
    Ident(Ident),
    Literal(Literal),
    Call(Ident, Box<Expression>),
    Loop(Loop),
}

#[derive(Debug, Clone)]
pub enum ControlFlow {
    Return,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(Number),
}

impl Literal {
    pub fn len(&self) -> usize {
        match self {
            Literal::String(s) => s.len(),
            Literal::Number(n) => mem::size_of_val(&n.value),
        }
    }
    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            Literal::String(s) => [s.as_bytes().to_vec(), vec![0]].concat(),
            Literal::Number(n) => n.value.to_le_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
pub struct Ident {
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Number {
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub var: Ident,
    pub start: u64,
    pub end: u64,
    pub body: Vec<Statement>,
}

pub trait Visitor<T> {
    fn visit_statement_list(&mut self, n: &StatementList) -> T;
    fn visit_statement(&mut self, s: &Statement) -> T;
    fn visit_expression(&mut self, s: &Expression) -> T;
    fn visit_literal(&mut self, e: &Literal) -> T;
    fn visit_ident(&mut self, e: &Ident) -> T;
    fn visit_number(&mut self, e: &Number) -> T;
}
