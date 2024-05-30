#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Equal,
    Newline,
    LeftP,
    RightP,
    BlockStart,
    BlockEnd,
    Whitespace,
    Range,
    StatementEnd,
    Ident(String),
    String(String),
    Number(i64),
}
