use std::{
    fmt::Display,
    hash::Hash,
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Clone)]
pub struct StatementList {
    pub id: String,
    pub stmts: Vec<Statement>,
}

impl StatementList {
    pub fn new(id: String, stmts: Vec<Statement>) -> Self {
        Self { id, stmts }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Declaration(Declaration),
    Assignment(Assignment),
    FuncDefinition(FuncDefinition),
    Scope(StatementList),
    ControlFlow(ControlFlow),
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
pub enum Type {
    String,
    Number,
    Ref(Box<Type>),
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        match value {
            "String" => Type::String,
            "Number" => Type::Number,
            _ => panic!("inlaid type: {}", value),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => f.write_str("String"),
            Type::Number => f.write_str("Number"),
            Type::Ref(t) => f.write_fmt(format_args!("&{}", t)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Ident,
    pub _type: Type,
}

impl Arg {
    pub fn new(name: Ident, _type: Type) -> Self {
        Self { name, _type }
    }
}

#[derive(Debug, Clone)]

pub struct Assignment(pub Ident, pub Expression);

#[derive(Debug, Clone)]
pub enum Expression {
    Ident(Ident),
    Literal(Literal),
    Call(Ident, Vec<Expression>),
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
    pub body: StatementList,
}

pub trait Visitor<T> {
    fn visit_statement_list(&mut self, n: &StatementList) -> T;
    fn visit_statement(&mut self, s: &Statement) -> T;
    fn visit_expression(&mut self, s: &Expression) -> T;
    fn visit_literal(&mut self, e: &Literal) -> T;
    fn visit_ident(&mut self, e: &Ident) -> T;
    fn visit_number(&mut self, e: &Number) -> T;
}
