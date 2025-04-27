use std::{fmt::Display, hash::Hash, mem};

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    FuncDeclaration,
    VarDeclaration,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub name: TypeName,
    pub modifiers: Vec<TypeModifer>,
}
impl Type {
    pub fn new(name: TypeName, modifiers: Vec<TypeModifer>) -> Self {
        Self { name, modifiers }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modifiers = self.modifiers.iter().fold(String::new(), |mut s, m| {
            s += &format!("{m}");
            s
        });
        f.write_fmt(format_args!("{} {}", modifiers, self.name))
    }
}

#[derive(Debug, Clone)]
pub enum TypeModifer {
    Ref,
}

impl Display for TypeModifer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeModifer::Ref => f.write_str("&"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeName {
    String,
    Int,
    Float,
    Bool,
    Unit,
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeName::String => f.write_str("String"),
            TypeName::Int => f.write_str("int"),
            TypeName::Float => f.write_str("float"),
            TypeName::Bool => f.write_str("bool"),
            TypeName::Unit => f.write_str(""),
        }
    }
}

impl From<&str> for TypeName {
    fn from(value: &str) -> Self {
        match value {
            "String" => TypeName::String,
            "int" => TypeName::Int,
            _ => panic!("invalid type: {}", value),
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
pub struct FuncDeclaration {
    pub name: Ident,
    pub args: Vec<Arg>,
    pub return_type: Type,
    pub body: Block,
}
impl FuncDeclaration {
    pub fn new(name: Ident, args: Vec<Arg>, return_type: Type, body: Block) -> Self {
        Self {
            name,
            args,
            return_type,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub scope: String,
    pub stmts: Vec<Statement>,
}

impl Block {
    pub fn new(scope: String, stmts: Vec<Statement>) -> Self {
        Self { scope, stmts }
    }
}

#[derive(Debug, Clone)]
pub enum ControlFlow {
    Return(Option<Expression>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDeclaration(VarDeclaration),
    FuncDeclaration(FuncDeclaration),
    Loop(Loop),
    Assignment(Assignment),
    Expression(Expression),
    ControlFlow(ControlFlow),
    Block(Block),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarDeclarationType {
    Let,
    Const,
}
impl Display for VarDeclarationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarDeclarationType::Let => write!(f, "let"),
            VarDeclarationType::Const => write!(f, "const"),
        }
    }
}

impl TryFrom<&str> for VarDeclarationType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s == "let" {
            Ok(VarDeclarationType::Let)
        } else if s == "const" {
            Ok(VarDeclarationType::Const)
        } else {
            Err(format!("invalid assignemnt type: {s}"))
        }
    }
}

#[derive(Debug, Clone)]

pub struct VarDeclaration {
    pub name: Ident,
    pub rhs: RhsExpression,
    pub declarion_type: VarDeclarationType,
}
impl VarDeclaration {
    pub fn new(id: Ident, expr: RhsExpression, decl_type: VarDeclarationType) -> Self {
        Self {
            name: id,
            rhs: expr,
            declarion_type: decl_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RhsExpression {
    Expression(Expression),
    Block(Block),
}

#[derive(Debug, Clone)]

pub struct Assignment {
    pub variable_name: Ident,
    pub rhs: RhsExpression,
}
impl Assignment {
    pub fn new(variable_name: Ident, rhs: RhsExpression) -> Self {
        Self { variable_name, rhs }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Unary(UnaryOperation),
    Binary(BinaryOperation),
    Ident(Ident),
    Literal(Literal),
    Call(Call),
}

#[derive(Debug, Clone)]
pub enum UnaryOperation {
    Minus(Box<Expression>),
    Not(Box<Expression>),
    Ref(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum BinaryOperation {
    Plus(Box<Expression>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func_name: Ident,
    pub args: Vec<Expression>,
}
impl Call {
    pub(crate) fn new(func_name: Ident, args: Vec<Expression>) -> Self {
        Self { func_name, args }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Integer(Integer),
    // Bool(Bool),
}

impl Literal {
    pub fn len(&self) -> usize {
        match self {
            Literal::String(s) => s.len(),
            Literal::Integer(n) => mem::size_of_val(&n.value),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
pub struct Ident {
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Integer {
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub var: Ident,
    pub start: u64,
    pub end: u64,
    pub body: Block,
}
