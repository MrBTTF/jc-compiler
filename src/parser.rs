use crate::{
    emitter::ast::{self, StatementList},
    lexer::token::Token,
};

/*
statement_list := statement*
statement := assignment | expression
assignment := "let" ident "=" expression
expression := literal | call
call := ident(expression)
literal := ident | string | number
string := . ident
 */

fn statement(tokens: &[Token]) -> Option<ast::Statement> {
    if let Some((ident, expr)) = assignment(tokens) {
        return Some(ast::Statement::Assignment(ident, expr));
    } else if let Some(expr) = expression(tokens) {
        return Some(ast::Statement::Expression(expr));
    }
    assignment(tokens)
        .map(|(ident, expr)| ast::Statement::Assignment(ident, expr))
        .or(expression(tokens).map(ast::Statement::Expression))
}

fn assignment(tokens: &[Token]) -> Option<(ast::Ident, ast::Expression)> {
    let id = match &tokens[..3] {
        [Token::Ident(keyword), Token::Ident(id), Token::Equal] if keyword == "let" => ident(id),
        _ => return None,
    };
    let expr = expression(&tokens[3..]);
    expr.map(|expr| (id, expr))
}

fn expression(tokens: &[Token]) -> Option<ast::Expression> {
    if let Some(literal) = literal(tokens) {
        return Some(ast::Expression::Literal(literal));
    } else if let Some((id, expr)) = call(tokens) {
        return Some(ast::Expression::Call(id, Box::new(expr)));
    }
    None
}

fn literal(tokens: &[Token]) -> Option<ast::Literal> {
    match tokens {
        [Token::Dot, Token::Ident(str)] => Some(ast::Literal::String(string(str))),
        [Token::Ident(id)] => Some(ast::Literal::Ident(ident(id))),
        [Token::Number(num)] => Some(ast::Literal::Number(number(num))),
        _ => None,
    }
}

fn string(str: &str) -> String {
    str.to_owned()
}

fn ident(ident: &str) -> ast::Ident {
    ast::Ident {
        value: ident.to_owned(),
    }
}

fn number(number: &i128) -> ast::Number {
    ast::Number {
        value: number.to_owned(),
    }
}

fn call(tokens: &[Token]) -> Option<(ast::Ident, ast::Expression)> {
    let id = {
        let Token::Ident(id) = &tokens[0] else {
            return None;
        };
        ident(id)
    };

    let tokens = match &tokens[..2] {
        [Token::Ident(_), Token::LeftP] => &tokens[2..],
        _ => return None,
    };
    if tokens.last().unwrap() != &Token::RightP {
        return None;
    }
    expression(&tokens[..tokens.len() - 1]).map(|expr| (id, expr))
}

pub fn parse(tokens: Vec<Vec<Token>>) -> StatementList {
    let statment_list: Vec<Option<ast::Statement>> =
        tokens.iter().map(|line| statement(line)).collect();
    StatementList(statment_list.into_iter().flatten().collect())
}
