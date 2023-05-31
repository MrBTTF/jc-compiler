#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Dot,
    Equal,
    Newline,
    LeftP,
    RightP,
    Whitespace,
    Ident(String),
    Number(i128),
}
