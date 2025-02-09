pub mod ast_printer;

use std::{
    iter,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    emitter::ast::{self, StatementList},
    lexer::token::Token,
};

/*
func_or_global := func ident (args) block | global
args := (ident: ident,)*
block := { statement_list  }
statement_list := statement*
statement := declaration | assignment | expression | func_definition | control_flow
declaration := ("let" | "const") ident "=" expression
assignment := ident "=" expression
expression := literal | ident | call | loop
control_flow := return
func_definition := func ident (args) ident block
args := (ident: ident,)*
loop := "for" ident..ident block
call := ident(expression)
literal := string | number
string := . ident
*/

fn block(tokens: &[Token]) -> (Vec<ast::Statement>, &[Token]) {
    let Some(tokens) = match_next(tokens, Token::BlockStart) else {
        return (vec![], tokens);
    };

    let tokens = skip(&tokens, Token::StatementEnd);

    let mut result = vec![];
    let (mut stmt, mut tokens) = statement(tokens);
    while stmt.is_some() {
        // dbg!(&stmt);
        // dbg!(&tokens[0..4]);

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
    let tokens = advance(&tokens);
    (result, tokens)
}

fn statement(tokens: &[Token]) -> (Option<ast::Statement>, &[Token]) {
    if let (Some(decl), tokens) = declaration(tokens) {
        (Some(ast::Statement::Declaration(decl)), &tokens)
    } else if let (Some(func_def), tokens) = func_definition(tokens) {
        (Some(ast::Statement::FuncDefinition(func_def)), tokens)
    } else if let (Some(assgn), tokens) = assignment(tokens) {
        (Some(ast::Statement::Assignment(assgn)), &tokens)
    } else if let (Some(expr), tokens) = expression(tokens) {
        (Some(ast::Statement::Expression(expr)), tokens)
    } else if let (Some(ctrl_flow), tokens) = control_flow(tokens) {
        (Some(ast::Statement::ControlFlow(ctrl_flow)), tokens)
    } else {
        (None, tokens)
    }
}

fn declaration(tokens: &[Token]) -> (Option<ast::Declaration>, &[Token]) {
    let Some((keyword, tokens)) = match_ident(tokens) else {
        return (None, tokens);
    };
    let Ok(assign_type) = keyword.try_into() else {
        return (None, tokens);
    };

    let Some((id, tokens)) = match_ident(tokens) else {
        return (None, tokens);
    };

    let Some(tokens) = match_next(tokens, Token::Equal) else {
        return (None, tokens);
    };

    let (Some(expr), tokens) = expression(&tokens) else {
        return (None, tokens);
    };

    (Some(ast::Declaration(ident(id), expr, assign_type)), tokens)
}

fn func_definition(tokens: &[Token]) -> (Option<ast::FuncDefinition>, &[Token]) {
    let Some((keyword, tokens)) = match_ident(tokens) else {
        return (None, tokens);
    };
    if keyword != "func" {
        return (None, tokens);
    }

    let Some((func_name, tokens)) = match_ident(tokens) else {
        return (None, tokens);
    };
    let Some(tokens) = match_next(tokens, Token::LeftP) else {
        return (None, tokens);
    };

    let mut tokens = tokens;
    let mut args: Vec<ast::Arg> = vec![];
    loop {
        dbg!(&tokens[0], &tokens[1], &tokens[2]);
        if &tokens[0] == &Token::RightP {
            tokens = advance(tokens);
            break;
        };
        let _tokens = tokens;

        let Some((arg_name, mut _tokens)) = match_ident(_tokens) else {
            return (None, tokens);
        };

        let has_ref = if let Some(__tokens) = match_next(_tokens, Token::Ref) {
            _tokens = __tokens;
            true
        } else {
            false
        };

        dbg!(&_tokens[0], &_tokens[1], &_tokens[2]);
        let Some((arg_type, _tokens)) = match_ident(_tokens) else {
            return (None, tokens);
        };

        let arg_type = if has_ref {
            ast::Type::Ref(Box::new(arg_type.into()))
        } else {
            arg_type.into()
        };

        let arg_name = ident(arg_name);
        let arg = ast::Arg::new(arg_name, arg_type);
        args.push(arg);

        tokens = _tokens;
    }

    let (block, tokens) = block(&tokens);

    let func_name = ident(func_name);
    let func_definition = ast::FuncDefinition(
        func_name.clone(),
        args,
        None,
        StatementList::new(func_name.value, block),
    );

    (Some(func_definition), tokens)
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
    if let (Some((id, exprs)), tokens) = call(tokens) {
        return (Some(ast::Expression::Call(id, exprs)), tokens);
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

fn control_flow(tokens: &[Token]) -> (Option<ast::ControlFlow>, &[Token]) {
    let id = match &tokens[0] {
        Token::Ident(id) => id,
        _ => {
            return (None, tokens);
        }
    };
    if id == "return" {
        (Some(ast::ControlFlow::Return), &tokens[1..])
    } else {
        (None, tokens)
    }
}

static LOOP_COUNTER: AtomicUsize = AtomicUsize::new(1);

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
    let id = format!("loop_{}", LOOP_COUNTER.fetch_add(1, Ordering::Relaxed));
    (
        Some(ast::Loop {
            var: ast::Ident { value: var },
            start,
            end,
            body: ast::StatementList::new(id, body),
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

fn call(tokens: &[Token]) -> (Option<(ast::Ident, Vec<ast::Expression>)>, &[Token]) {
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

    if &arg_tokens[0] == &Token::RightP {
        return (Some((id, vec![])), &tokens[3..]);
    }

    let (expr, rest_tokens) = expression(arg_tokens);
    assert_eq!(rest_tokens.first().unwrap(), &Token::RightP, "expected )");

    (expr.map(|expr| (id, vec![expr])), &rest_tokens[1..])
}

pub fn parse(tokens: Vec<Token>) -> StatementList {
    let (statment_list, tokens) = block(&tokens);
    assert!(tokens.is_empty(), "there are unparsed tokens: {tokens:#?}");
    StatementList::new("global".to_string(), statment_list.into_iter().collect())
}

fn skip(tokens: &[Token], to_skip: Token) -> &[Token] {
    if let Some(t) = tokens.first() {
        if *t == to_skip {
            advance(&tokens)
        } else {
            tokens
        }
    } else {
        tokens
    }
}

fn match_next(tokens: &[Token], target: Token) -> Option<&[Token]> {
    if tokens.first().unwrap() == &target {
        return Some(advance(&tokens));
    }
    None
}


fn advance(tokens: &[Token]) -> &[Token] {
    &tokens[1..]
}

fn match_ident(tokens: &[Token]) -> Option<(&str, &[Token])> {
    match tokens.first().unwrap() {
        Token::Ident(v) => {
            return Some((v, advance(&tokens)));
        }
        _ => None,
    }
}
