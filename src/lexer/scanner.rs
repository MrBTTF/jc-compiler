use super::token::Token;

fn whitespace(s: &str) -> (Option<Token>, usize) {
    let c = s.chars().nth(0).unwrap();
    if matches!(c, ' ' | '\t') {
        (Some(Token::Whitespace), 1)
    } else {
        (None, 0)
    }
}

fn newline(s: &str) -> (Option<Token>, usize) {
    let c = s.chars().nth(0).unwrap();
    if c == '\n' {
        (Some(Token::Newline), 1)
    } else {
        (None, 0)
    }
}

fn operator(s: &str) -> (Option<Token>, usize) {
    let c = s.chars().nth(0).unwrap();
    match c {
        '=' => (Some(Token::Equal), 1),
        '.' => (Some(Token::Dot), 1),
        '(' => (Some(Token::LeftP), 1),
        ')' => (Some(Token::RightP), 1),
        _ => (None, 0),
    }
}

// fn string(s: &str) -> (Option<Token>, usize) {
//     if !s.starts_with('.') {
//         return (None, 0);
//     }
//     let mut i: usize = 0;
//     for c in s.chars() {
//         if !c.is_alphanumeric() && !matches!(c, '\\' | '_' | '!' | ' ') {
//             break;
//         }
//         i += 1;
//     }
//     if i == 0 {
//         return (None, 0);
//     }
//     let s = s[1..i].to_string().replace("\\n", "\n");
//     (Some(Token::Ident(s)), i)
// }

fn number(s: &str) -> (Option<Token>, usize) {
    let mut i: usize = 0;

    for c in s.chars() {
        if !c.is_numeric() {
            break;
        }
        i += 1;
    }
    if i == 0 {
        return (None, 0);
    }
    let token = {
        match s[..i].parse() {
            Ok(value) => Some(Token::Number(value)),
            Err(err) => match err.kind() {
                std::num::IntErrorKind::Empty => None,
                _ => panic!("{err}"),
            },
        }
    };
    (token, i)
}

fn identifier(s: &str) -> (Option<Token>, usize) {
    let mut i: usize = 0;
    for c in s.chars() {
        if !c.is_alphanumeric() && !matches!(c, '\\' | '_' | '!') {
            break;
        }
        i += 1;
    }
    if i == 0 {
        return (None, 0);
    }
    let s = s[..i].to_string().replace("\\n", "\n");
    (Some(Token::Ident(s)), i)
}

type Parser = fn(&str) -> (Option<Token>, usize);

fn scan_token(s: &str) -> (Option<Token>, usize) {
    let parsers: Vec<Parser> = vec![newline, operator, number, identifier, whitespace];
    for (i, parser) in parsers.iter().enumerate() {
        let (token, advanced) = parser(s);
        // dbg!(i, &token, advanced);
        if token.is_some() {
            return (token, advanced);
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
            println!("Token: {:?}", token);
            if token == Token::Newline {
                if !line.is_empty() {
                    tokens.push(line.clone());
                    line.clear();
                }
            } else if token != Token::Whitespace {
                line.push(token);
            }
        }
        start += advanced;
    }
    if !line.is_empty() {
        tokens.push(line.clone());
    }

    tokens
}
