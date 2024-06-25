pub mod ast_printer;

use crate::{
    emitter::ast::{self, StatementList},
    lexer::token::Token,
};

/*
block := { statement_list  }
statement_list := statement*
statement := declaration | assignment | expression
declaration := ("let" | "const") ident "=" expression
assignment := ident "=" expression
expression := literal | ident | call | loop
loop := "for" ident..ident block
call := ident(expression)
literal := string | number
string := . ident
*/

fn block(tokens: &[Token]) -> (Vec<ast::Statement>, &[Token]) {
    if tokens.first().unwrap() != &Token::BlockStart {
        return (vec![], tokens);
    }
    let tokens = skip(&tokens[1..], Token::StatementEnd);

    let mut result = vec![];
    let (mut stmt, mut tokens) = statement(tokens);
    while stmt.is_some() {
        // dbg!(&stmt);

        // panic!("stop");
        assert_eq!(
            tokens.first().unwrap(),
            &Token::StatementEnd,
            "statement end not reached"
        );
        if let Some(s) = stmt {
            result.push(s)
        }
        (stmt, tokens) = statement(&tokens[1..]);
    }
    let tokens = &tokens[1..];
    (result, tokens)
}

fn statement(tokens: &[Token]) -> (Option<ast::Statement>, &[Token]) {
    if let (Some(decl), tokens) = declaration(tokens) {
        (Some(ast::Statement::Declaration(decl)), &tokens)
    } else if let (Some(assgn), tokens) = assignment(tokens) {
        (Some(ast::Statement::Assignment(assgn)), &tokens)
    } else if let (Some(expr), tokens) = expression(tokens) {
        (Some(ast::Statement::Expression(expr)), tokens)
    } else {
        (None, tokens)
    }
}

fn declaration(tokens: &[Token]) -> (Option<ast::Declaration>, &[Token]) {
    if tokens.len() < 3 {
        return (None, tokens);
    }
    let (assign_type, id) = match &tokens[..3] {
        [Token::Ident(keyword), Token::Ident(id), Token::Equal]
            if matches!(keyword.as_str(), "let" | "const") =>
        {
            if let Ok(assign_type) = keyword.as_str().try_into() {
                (assign_type, ident(id))
            } else {
                return (None, tokens);
            }
        }
        _ => return (None, tokens),
    };
    let (expr, tokens) = expression(&tokens[3..]);
    if let Some(expr) = expr {
        (Some(ast::Declaration(id, expr, assign_type)), tokens)
    } else {
        (None, tokens)
    }
}

fn assignment(tokens: &[Token]) -> (Option<ast::Assignment>, &[Token]) {
    if tokens.len() < 2 {
        return (None, tokens);
    }
    let id = match &tokens[..2] {
        [Token::Ident(id), Token::Equal] => ident(id),
        _ => return (None, tokens),
    };
    let (expr, tokens) = expression(&tokens[2..]);
    if let Some(expr) = expr {
        (Some(ast::Assignment(id, expr)), tokens)
    } else {
        (None, tokens)
    }
}

fn expression(tokens: &[Token]) -> (Option<ast::Expression>, &[Token]) {
    if let (Some((id, expr)), tokens) = call(tokens) {
        return (Some(ast::Expression::Call(id, Box::new(expr))), tokens);
    } else if let (Some(l), tokens) = _loop(tokens) {
        return (Some(ast::Expression::Loop(l)), tokens);
    } else if let (Some(literal), tokens) = literal(tokens) {
        return (Some(ast::Expression::Literal(literal)), tokens);
    } else if let [Token::Ident(id), ..] = tokens {
        return (Some(ast::Expression::Ident(ident(id))), &tokens[1..]);
    }
    (None, tokens)
    // panic!("invalid expression: {:?}", &tokens)
}

fn _loop(tokens: &[Token]) -> (Option<ast::Loop>, &[Token]) {
    if let Token::Ident(id) = &tokens[0] {
        if id != "for" {
            return (None, tokens);
        }
    }

    let (var, start, end, statements) = match &tokens[1..] {
        [Token::Ident(var), Token::Ident(_), Token::Number(start), Token::Range, Token::Number(end), statements @ ..] => {
            (var.clone(), *start as u64, *end as u64, statements)
        }
        _ => return (None, tokens),
    };

    let (body, tokens) = block(statements);

    (
        Some(ast::Loop {
            var: ast::Ident { value: var },
            start,
            end,
            body,
        }),
        tokens,
    )
}

fn literal(tokens: &[Token]) -> (Option<ast::Literal>, &[Token]) {
    match tokens {
        [Token::String(s), Token::StatementEnd, ..] => {
            (Some(ast::Literal::String(string(s))), &tokens[1..])
        }
        [Token::Number(num), Token::StatementEnd, ..] => {
            (Some(ast::Literal::Number(number(num))), &tokens[1..])
        }
        _ => (None, tokens),
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

fn call(tokens: &[Token]) -> (Option<(ast::Ident, ast::Expression)>, &[Token]) {
    let id: ast::Ident = {
        let Token::Ident(id) = &tokens[0] else {
            return (None, tokens);
        };
        ident(id)
    };

    let arg_tokens = match &tokens[..2] {
        [Token::Ident(_), Token::LeftP] => &tokens[2..],
        _ => return (None, tokens),
    };

    let (expr, rest_tokens) = expression(arg_tokens);
    assert_eq!(rest_tokens.first().unwrap(), &Token::RightP, "expected )");

    (expr.map(|expr| (id, expr)), &rest_tokens[1..])
}

pub fn parse(tokens: Vec<Token>) -> StatementList {
    let (statment_list, tokens) = block(&tokens);
    assert!(tokens.is_empty(), "there are unparsed tokens: {tokens:#?}");
    StatementList(statment_list.into_iter().collect())
}

fn skip(tokens: &[Token], to_skip: Token) -> &[Token] {
    if let Some(t) = tokens.first() {
        if *t == to_skip {
            &tokens[1..]
        } else {
            tokens
        }
    } else {
        tokens
    }
}
