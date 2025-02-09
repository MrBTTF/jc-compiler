pub mod ast_printer;

use anyhow::{anyhow, bail, Context, Result};

use std::sync::atomic::{AtomicUsize, Ordering};

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

fn block(tokens: &[Token]) -> Result<(Vec<ast::Statement>, &[Token])> {
    let tokens = match_next(tokens, Token::BlockStart)?;

    let mut tokens = skip(&tokens, Token::StatementEnd);

    let mut result = vec![];
    let mut stmt: Option<ast::Statement>;
    loop {
        // panic!("stop");

        (stmt, tokens) = statement(&tokens)?;
        if tokens.is_empty() {
            bail!("Expected end of block");
        }

        tokens = match_next(tokens, Token::StatementEnd).context("statement end not reached")?;

        result.push(stmt.unwrap());

        if let Ok(_tokens) = match_next(tokens, Token::BlockEnd) {
            tokens = _tokens;
            break;
        };
    }
    Ok((result, tokens))
}

fn statement(tokens: &[Token]) -> Result<(Option<ast::Statement>, &[Token])> {
    if let (Some(decl), tokens) = declaration(tokens).context("Couldn't parse statement")? {
        Ok((Some(ast::Statement::Declaration(decl)), tokens))
    } else if let (Some(func_def), tokens) =
        func_definition(tokens).context("Couldn't parse statement")?
    {
        Ok((Some(ast::Statement::FuncDefinition(func_def)), tokens))
    } else if let (Some(assgn), tokens) = assignment(tokens).context("Couldn't parse statement")? {
        Ok((Some(ast::Statement::Assignment(assgn)), tokens))
    } else if let (Some(expr), tokens) = expression(tokens).context("Couldn't parse statement")? {
        Ok((Some(ast::Statement::Expression(expr)), tokens))
    } else if let (Some(ctrl_flow), tokens) =
        control_flow(tokens).context("Couldn't parse statement")?
    {
        Ok((Some(ast::Statement::ControlFlow(ctrl_flow)), tokens))
    } else {
        bail!("Unexpected statement: {:?}", &tokens[0])
    }
}

fn declaration(tokens: &[Token]) -> Result<(Option<ast::Declaration>, &[Token])> {
    let Ok((keyword, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };
    let Ok(assign_type) = keyword.try_into() else {
        return Ok((None, tokens));
    };

    let (id, tokens) =
        match_ident(tokens).context(format!("Expected identifier, found: {:#?}", &tokens[0]))?;

    let tokens = match_next(tokens, Token::Equal)
        .context(format!("Expected =, found: {:#?}", &tokens[0]))?;

    let (Some(expr), tokens) = expression(&tokens).context("Expected expression")? else {
        return Ok((None, tokens));
    };

    Ok((Some(ast::Declaration(ident(id), expr, assign_type)), tokens))
}

fn func_definition(tokens: &[Token]) -> Result<(Option<ast::FuncDefinition>, &[Token])> {
    let Ok((keyword, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };
    if keyword != "func" {
        return Ok((None, tokens));
    }

    let (func_name, tokens) =
        match_ident(tokens).context(format!("Expected function name found: {:#?}", &tokens[0]))?;

    let tokens = match_next(tokens, Token::LeftP)
        .context(format!("Expected (, found: {:#?}", &tokens[0]))?;

    let mut tokens = tokens;
    let mut args: Vec<ast::Arg> = vec![];
    loop {
        // dbg!(&tokens[0], &tokens[1], &tokens[2]);
        if &tokens[0] == &Token::RightP {
            tokens = advance(tokens);
            break;
        };
        let _tokens = tokens;

        let (arg_name, mut _tokens) = match_ident(_tokens)
            .context(format!("Expected argument name, found: {:#?}", &tokens[0]))?;

        let has_ref = if let Ok(__tokens) = match_next(_tokens, Token::Ref) {
            _tokens = __tokens;
            true
        } else {
            false
        };

        let (arg_type, _tokens) = match_ident(_tokens)
            .context(format!("Expected argument type, found: {:#?}", &tokens[0]))?;

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

    let (block, tokens) = block(&tokens)?;

    let func_name = ident(func_name);
    let func_definition = ast::FuncDefinition(
        func_name.clone(),
        args,
        None,
        StatementList::new(func_name.value, block),
    );

    Ok((Some(func_definition), tokens))
}

fn assignment(tokens: &[Token]) -> Result<(Option<ast::Assignment>, &[Token])> {
    if tokens.len() < 2 {
        return Ok((None, tokens));
    }
    let id = match &tokens[..2] {
        [Token::Ident(id), Token::Equal] => ident(id),
        _ => return Ok((None, tokens)),
    };
    let (expr, tokens) = expression(&tokens[2..])?;
    if let Some(expr) = expr {
        Ok((Some(ast::Assignment(id, expr)), tokens))
    } else {
        Ok((None, tokens))
    }
}

fn expression(tokens: &[Token]) -> Result<(Option<ast::Expression>, &[Token])> {
    if let (Some((id, exprs)), tokens) = call(tokens) {
        return Ok((Some(ast::Expression::Call(id, exprs)), tokens));
    } else if let (Some(l), tokens) = _loop(tokens)? {
        return Ok((Some(ast::Expression::Loop(l)), tokens));
    } else if let (Some(literal), tokens) = literal(tokens) {
        return Ok((Some(ast::Expression::Literal(literal)), tokens));
    } else if let [Token::Ident(id), ..] = tokens {
        return Ok((Some(ast::Expression::Ident(ident(id))), &tokens[1..]));
    }
    bail!("invalid expression: {:?}", &tokens)
}

fn control_flow(tokens: &[Token]) -> Result<(Option<ast::ControlFlow>, &[Token])> {
    let Ok((keyword, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };

    if keyword == "return" {
        Ok((Some(ast::ControlFlow::Return), &tokens))
    } else {
        Ok((None, tokens))
    }
}

static LOOP_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn _loop(tokens: &[Token]) -> Result<(Option<ast::Loop>, &[Token])> {
    if let Token::Ident(id) = &tokens[0] {
        if id != "for" {
            return Ok((None, tokens));
        }
    }

    let (var, start, end, statements) = match &tokens[1..] {
        [Token::Ident(var), Token::Ident(_), Token::Number(start), Token::Range, Token::Number(end), statements @ ..] => {
            (var.clone(), *start as u64, *end as u64, statements)
        }
        _ => return Ok((None, tokens)),
    };

    let (body, tokens) = block(statements)?;
    let id = format!("loop_{}", LOOP_COUNTER.fetch_add(1, Ordering::Relaxed));
    Ok((
        Some(ast::Loop {
            var: ast::Ident { value: var },
            start,
            end,
            body: ast::StatementList::new(id, body),
        }),
        tokens,
    ))
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

    let (expr, rest_tokens) = expression(arg_tokens).unwrap();
    assert_eq!(rest_tokens.first().unwrap(), &Token::RightP, "expected )");

    (expr.map(|expr| (id, vec![expr])), &rest_tokens[1..])
}

pub fn parse(tokens: Vec<Token>) -> Result<StatementList> {
    let (statment_list, tokens) = block(&tokens)?;
    assert!(tokens.is_empty(), "there are unparsed tokens: {tokens:#?}");
    Ok(StatementList::new(
        "global".to_string(),
        statment_list.into_iter().collect(),
    ))
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

fn match_next(tokens: &[Token], target: Token) -> Result<&[Token]> {
    if tokens
        .first()
        .ok_or(anyhow!("Expected token {:#?}", target))?
        == &target
    {
        return Ok(advance(&tokens));
    }
    bail!("Unexptected token: {:#?}", target);
}

fn advance(tokens: &[Token]) -> &[Token] {
    &tokens[1..]
}

fn match_ident(tokens: &[Token]) -> Result<(&str, &[Token])> {
    let token = tokens.first().ok_or(anyhow!("Expected ident not found"))?;

    match token {
        Token::Ident(v) => {
            return Ok((v, advance(&tokens)));
        }
        _ => bail!("Expected ident but found: {:#?}", token),
    }
}
