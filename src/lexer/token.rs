#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Equal,
    Newline,
    LeftP,
    RightP,
    Whitespace,
    Ident(String),
    String(String),
    Number(i64),
}
