pub mod ast_printer;

use anyhow::{anyhow, bail, Context, Result};

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{emitter::ast, lexer::token::Token};

/*
program := item*
item := func | declaration
func_declaration := "func" ident (args) type block
args := (arg,)*
arg := "const"? ident: type
type := &* type_name
type_name := ("String" | "int" | "float" | "bool" | unit)
block := { statement*  }
statement := var_declaration | assignment | expression
                        | func_declaration | loop | control_flow | block
loop := "for" ident..ident block

var_declaration := ("let" | "const") ident [: type] "=" rhs_expression
assignment := ident "=" rhs_expression

rhs_expression := block | expression
expression := unary | binary | literal | ident | call
unary := unary_operator expression
binary := expression binary_operator expression
unary_operator := - | !
operator :=  + | - | * | /

call := ident(expression(, expression)+)
literal := string | int | float | bool
bool := "true" | "false"
string := . ident

control_flow := return_cf
return_cf := "return" expression
*/

pub fn parse(tokens: Vec<Token>) -> Result<ast::Block> {
    let (statment_list, tokens) = block(&tokens, "global")?;
    assert!(tokens.is_empty(), "there are unparsed tokens: {tokens:#?}");
    Ok(ast::Block::new(
        "global".to_string(),
        statment_list.into_iter().collect(),
    ))
}

fn block<'a>(tokens: &'a [Token], scope: &str) -> Result<(Vec<ast::Statement>, &'a [Token])> {
    let tokens = match_next(tokens, Token::BlockStart)?;

    let mut tokens = skip(&tokens, Token::StatementEnd);

    let mut result = vec![];
    let mut stmt: Option<ast::Statement>;
    loop {
        // panic!("stop");

        (stmt, tokens) = statement(&tokens, scope)?;
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

fn statement<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::Statement>, &'a [Token])> {
    if let (Some(decl), tokens) =
        var_declaration(tokens, scope).context("Couldn't parse var_declaration statement")?
    {
        Ok((Some(ast::Statement::VarDeclaration(decl)), tokens))
    } else if let (Some(func_def), tokens) =
        func_definition(tokens, scope).context("Couldn't parse statement")?
    {
        Ok((Some(ast::Statement::FuncDeclaration(func_def)), tokens))
    } else if let (Some(assgn), tokens) =
        assignment(tokens, scope).context("Couldn't parse assignment statement")?
    {
        Ok((Some(ast::Statement::Assignment(assgn)), tokens))
    } else if let (Some(l), tokens) = _loop(tokens, scope)? {
        return Ok((Some(ast::Statement::Loop(l)), tokens));
    } else if let (Some(ctrl_flow), tokens) =
        control_flow(tokens, scope).context("Couldn't parse statement")?
    {
        Ok((Some(ast::Statement::ControlFlow(ctrl_flow)), tokens))
    } else if let (Some(expr), tokens) =
        expression(tokens, scope).context("Couldn't parse expression statement")?
    {
        Ok((Some(ast::Statement::Expression(expr)), tokens))
    } else {
        bail!("Unexpected statement: {:?}", &tokens[0])
    }
}

fn var_declaration<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::VarDeclaration>, &'a [Token])> {
    let Ok((keyword, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };
    let Ok(decl_type) = keyword.try_into() else {
        return Ok((None, tokens));
    };

    let (id, tokens) =
        match_ident(tokens).context(format!("Expected identifier, found: {:#?}", &tokens[0]))?;

    let tokens = match_next(tokens, Token::Equal)
        .context(format!("Expected =, found: {:#?}", &tokens[0]))?;

    let (Some(expr), tokens) = expression(&tokens, scope).context("Expected expression")? else {
        return Ok((None, tokens));
    };

    Ok((
        Some(ast::VarDeclaration::new(
            ident(id),
            ast::RhsExpression::Expression(expr),
            decl_type,
        )),
        tokens,
    ))
}

fn func_definition<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::FuncDeclaration>, &'a [Token])> {
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

        let type_modifiers = if has_ref {
            vec![ast::TypeModifer::Ref]
        } else {
            vec![]
        };
        let arg_type = ast::Type::new(arg_type.into(), type_modifiers);

        let arg_name = ident(arg_name);
        let arg = ast::Arg::new(arg_name, arg_type);
        args.push(arg);

        tokens = _tokens;
    }

    let func_name = ident(func_name);

    let current_scope = format!("{}::{}", scope, func_name.value);
    let (block, tokens) = block(&tokens, &current_scope)?;

    let return_type = ast::Type::new(ast::TypeName::Unit, vec![]);
    let func_definition = ast::FuncDeclaration::new(
        func_name.clone(),
        args,
        return_type,
        ast::Block::new(current_scope, block),
    );

    Ok((Some(func_definition), tokens))
}

fn assignment<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::Assignment>, &'a [Token])> {
    let Ok((id, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };

    let Ok(tokens) = match_next(tokens, Token::Equal) else {
        return Ok((None, tokens));
    };

    let (rhs_expr, tokens) = rhs_expression(&tokens, scope)?;
    if let Some(rhs_expr) = rhs_expr {
        Ok((Some(ast::Assignment::new(ident(id), rhs_expr)), tokens))
    } else {
        Ok((None, tokens))
    }
}

fn rhs_expression<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::RhsExpression>, &'a [Token])> {
    if let (Some(expr), tokens) = expression(tokens, scope)? {
        return Ok((Some(ast::RhsExpression::Expression(expr)), tokens));
    }
    if let (block, tokens) = block(tokens, scope)? {
        return todo!(); //Ok((Some(ast::RhsExpression::Block(Block::new(id, stmts))), tokens));
    }
    bail!("invalid rhs expression: {:?}", &tokens)
}

fn expression<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::Expression>, &'a [Token])> {
    if let (Some(call), tokens) = call(tokens, scope)? {
        return Ok((Some(ast::Expression::Call(call)), tokens));
    } else if let (Some(literal), tokens) = literal(tokens, scope) {
        return Ok((Some(ast::Expression::Literal(literal)), tokens));
    } else if let [Token::Ident(id), ..] = tokens {
        return Ok((Some(ast::Expression::Ident(ident(id))), &tokens[1..]));
    }
    bail!("invalid expression: {:?}", &tokens)
}

fn control_flow<'a>(
    tokens: &'a [Token],
    scope: &str,
) -> Result<(Option<ast::ControlFlow>, &'a [Token])> {
    let Ok((keyword, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };

    if keyword == "return" {
        Ok((Some(ast::ControlFlow::Return(None)), &tokens))
    } else {
        Ok((None, tokens))
    }
}

static LOOP_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn _loop<'a>(tokens: &'a [Token], scope: &str) -> Result<(Option<ast::Loop>, &'a [Token])> {
    let Ok(tokens) = starts_with_ident(tokens, "for") else {
        return Ok((None, tokens));
    };

    let (var, tokens) = match_ident(tokens)?;
    let tokens = starts_with_ident(tokens, "in")?;
    let (start, tokens) = match_number(tokens)?;
    let tokens = match_next(tokens, Token::Range)?;
    let (end, tokens) = match_number(tokens)?;

    let current_scope = format!(
        "{scope}::loop_{}",
        LOOP_COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    let (body, tokens) = block(tokens, scope)?;
    Ok((
        Some(ast::Loop {
            var: ast::Ident {
                value: var.to_string(),
            },
            start: *start as u64,
            end: *end as u64,
            body: ast::Block::new(current_scope, body),
        }),
        tokens,
    ))
}

fn literal<'a>(tokens: &'a [Token], scope: &str) -> (Option<ast::Literal>, &'a [Token]) {
    match tokens {
        [Token::String(s), ..] => (Some(ast::Literal::String(string(s))), &tokens[1..]),
        [Token::Number(num), ..] => (Some(ast::Literal::Integer(number(num))), &tokens[1..]),
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

fn number(number: &i64) -> ast::Integer {
    ast::Integer {
        value: number.to_owned(),
    }
}

fn call<'a>(tokens: &'a [Token], scope: &str) -> Result<(Option<ast::Call>, &'a [Token])> {
    let Ok((id, tokens)) = match_ident(tokens) else {
        return Ok((None, tokens));
    };

    let Ok(tokens) = match_next(tokens, Token::LeftP) else {
        return Ok((None, tokens));
    };

    if &tokens[0] == &Token::RightP {
        return Ok((Some(ast::Call::new(ident(id), vec![])), &tokens[1..]));
    }

    let (expr, tokens) = expression(tokens, scope).unwrap();

    let tokens = match_next(tokens, Token::RightP)?;

    Ok((
        expr.map(|expr| ast::Call::new(ident(id), vec![expr])),
        &tokens,
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

fn advance<'a>(tokens: &'a [Token]) -> &[Token] {
    &tokens[1..]
}

fn match_ident<'a>(tokens: &'a [Token]) -> Result<(&str, &'a [Token])> {
    let token = tokens.first().ok_or(anyhow!("Expected ident not found"))?;

    match token {
        Token::Ident(v) => {
            return Ok((v, advance(&tokens)));
        }
        _ => bail!("Expected ident but found: {:#?}", token),
    }
}

fn match_number<'a>(tokens: &'a [Token]) -> Result<(&i64, &'a [Token])> {
    let token = tokens.first().ok_or(anyhow!("Expected number not found"))?;

    match token {
        Token::Number(v) => {
            return Ok((v, advance(&tokens)));
        }
        _ => bail!("Expected number but found: {:#?}", token),
    }
}

fn starts_with_ident<'a>(tokens: &'a [Token], s: &str) -> Result<&'a [Token]> {
    let (keyword, tokens) = match_ident(tokens)?;
    if keyword != s {
        bail!("Expected ident {s} but found: {:#?}", &tokens[0]);
    }
    Ok(&tokens)
}
