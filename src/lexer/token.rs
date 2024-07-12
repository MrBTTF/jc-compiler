#[derive(Debug, Clone, Eq, PartialEq)]
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
    Column,
    Ident(String),
    String(String),
    Number(i64),
}
