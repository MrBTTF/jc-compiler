use super::token::Token;

fn whitespace(s: &str, i: usize) -> (Option<Token>, usize) {
    let c = s.chars().nth(i).unwrap();
    if c == ' ' {
        (Some(Token::Whitespace), 1)
    } else {
        (None, 0)
    }
}

fn newline(s: &str, i: usize) -> (Option<Token>, usize) {
    let c = s.chars().nth(i).unwrap();
    if c == '\n' {
        (Some(Token::Newline), 1)
    } else {
        (None, 0)
    }
}

fn operator(s: &str, i: usize) -> (Option<Token>, usize) {
    let c = s.chars().nth(i).unwrap();
    match c {
        '=' => (Some(Token::Equal), 1),
        '.' => (Some(Token::Dot), 1),
        '(' => (Some(Token::LeftP), 1),
        ')' => (Some(Token::RightP), 1),
        _ => (None, 0),
    }
}

fn number(s: &str, i: usize) -> (Option<Token>, usize) {
    let c = s.chars().nth(i).unwrap();
    if !c.is_numeric() {
        let token = {
            match s[..i].parse() {
                Ok(value) => Some(Token::Number(value)),
                Err(err) => match err.kind() {
                    std::num::IntErrorKind::Empty => None,
                    _ => panic!("{err}"),
                },
            }
        };
        return (token, i);
    }
    (None, i + 1)
}

fn identifier(s: &str, i: usize) -> (Option<Token>, usize) {
    let c = s.chars().nth(i).unwrap();
    if c.is_alphanumeric() {
        (None, i + 1)
    } else {
        (Some(Token::Ident(s[..i].to_string())), i)
    }
}

type Parser = fn(&str, usize) -> (Option<Token>, usize);

fn scan_token(s: &str) -> (Option<Token>, usize) {
    let parsers: Vec<Parser> = vec![whitespace, newline, operator, number, identifier];
    for parser in parsers.iter() {
        for i in 0..s.len() {
            let (token, advanced) = parser(s, i);
            // dbg!(i, &token, advanced);
            if token.is_some() {
                return (token, advanced);
            } else if advanced == 0 {
                break;
            }
        }
    }

    panic!("Invalid lexeme")
}

pub fn scan(source_code: String) -> Vec<Vec<Token>> {
    let mut tokens: Vec<Vec<Token>> = vec![];
    let mut start = 0;
    let mut line: Vec<Token> = vec![];
    while start < source_code.len() {
        let (token, advanced) = scan_token(&source_code[start..]);
        if let Some(token) = token {
            // println!("Token: {:?}", token);
            if token == Token::Newline {
                tokens.push(line.clone());
                line.clear();
            } else if token != Token::Whitespace {
                line.push(token);
            }
        }
        start += advanced;
    }
    tokens.push(line.clone());

    tokens
}
