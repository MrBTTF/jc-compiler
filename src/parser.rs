pub mod ast_printer;

use crate::{
    emitter::ast::{self, AssignmentType, StatementList},
    lexer::token::Token,
};

/*
statement_list := statement*
statement := assignment | expression
assignment := ("let" | "const") ident "=" expression
expression := literal | call
call := ident(expression)
literal := ident | string | number
string := . ident
 */

fn statement(tokens: &[Token]) -> Option<ast::Statement> {
    assignment(tokens)
        .map(ast::Statement::Assignment)
        .or(expression(tokens).map(ast::Statement::Expression))
}

fn assignment(tokens: &[Token]) -> Option<ast::Assignment> {
    let (assign_type, id) = match &tokens[..3] {
        [Token::Ident(keyword), Token::Ident(id), Token::Equal]
            if matches!(keyword.as_str(), "let" | "const") =>
        {
            if let Ok(assign_type) = keyword.as_str().try_into() {
                (assign_type, ident(id))
            } else {
                return None;
            }
        }
        _ => return None,
    };
    let expr = expression(&tokens[3..]);
    expr.map(|expr| ast::Assignment(id, expr, assign_type))
}

fn expression(tokens: &[Token]) -> Option<ast::Expression> {
    if let Some(literal) = literal(tokens) {
        return Some(ast::Expression::Literal(literal));
    } else if let Some((id, expr)) = call(tokens) {
        return Some(ast::Expression::Call(id, Box::new(expr)));
    }
    None
    // panic!("invalid expression: {:?}", &tokens)
}

fn literal(tokens: &[Token]) -> Option<ast::Literal> {
    if tokens.first().is_some_and(|t| *t == Token::Dot) {
        let s = tokens.iter().skip(1).fold(String::new(), |mut acc, t| {
            if let Token::Ident(id) = t {
                acc += id;
            }
            acc + " "
        });
        let s = &s[..s.len() - 1];
        if s.is_empty() {
            panic!("invalid literal: {:?}", &tokens);
        }
        return Some(ast::Literal::String(string(s)));
    }
    match tokens {
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

fn number(number: &i64) -> ast::Number {
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
