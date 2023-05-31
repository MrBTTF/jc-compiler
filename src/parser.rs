use crate::lexer::token::Token;

/*
statement := assignment | expression
assignment := "let" ident "=" expression
expression := literal | call(expression)
literal := ident | string | number
string := . ident
 */

fn statement(tokens: &[Token]) {
    if let t = assignment(tokens) {
    } else if let t = expression(tokens) {
    }
}

fn assignment(tokens: &[Token]) {
    match &tokens[..3] {
        [Token::Ident(keyword), Token::Ident(ident), Token::Equal] if keyword == "let" => {
            "two elements"
        }
        _ => "otherwise",
    };
    if let t = expression(tokens) {}
}

fn expression(tokens: &[Token]) {
    if let t = literal(tokens) {
    } else if let t = call(tokens) {
    }
}

fn literal(tokens: &[Token]) {
    match tokens {
        tokens if string(tokens).is_ok() => todo!(),
        [Token::Ident(ident)] => todo!(),
        [Token::Number(ident)] => todo!(),
    };
}

fn string(tokens: &[Token]) {
    match tokens {
        [Token::Dot, Token::Ident(ident)] => todo!(),
    };
}

fn call(tokens: &[Token]) {
    if let t = literal(tokens) {
    } else if let t = call(tokens) {
    }
}

pub fn parse(tokens: Vec<Vec<Token>>) {
    for line in tokens.iter() {}
}
