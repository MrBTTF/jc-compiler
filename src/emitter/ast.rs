pub struct StatementList(pub Vec<Statement>);

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Assignment(Ident, Expression),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Call(Ident, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Ident(Ident),
    String(String),
    Number(Number),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Ident {
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Number {
    pub value: i128,
}

pub trait Visitor<T> {
    fn visit_statement_list(&mut self, n: &StatementList) -> T;
    fn visit_statement(&mut self, s: &Statement) -> T;
    fn visit_expression(&mut self, s: &Expression) -> T;
    fn visit_literal(&mut self, e: &Literal) -> T;
    fn visit_ident(&mut self, e: &Ident) -> T;
    fn visit_number(&mut self, e: &Number) -> T;
}
