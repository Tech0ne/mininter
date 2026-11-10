#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    StringLit(String),
    Identifier(String),

    // Keywords
    If,
    Then,
    Else,
    End,
    While,
    For,
    In,
    Function,
    Return,
    True,
    False,
    Null,
    And,
    Or,
    Not,
    New,
    Self_,
    Globals,
    Intrinsics,
    Break,
    Continue,
    Isa,
    Mod,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret, // + - * / % ^
    Equal,
    NotEqual, // == !=
    Less,
    Greater,
    LessEq,
    GreaterEq, // < > <= >=
    At,        // @

    // Assignment
    Assign, // =
    PlusAssign,
    MinusAssign, // += -=
    StarAssign,
    SlashAssign, // *= /=
    PercentAssign,
    CaretAssign, // %= ^=

    // Delimiters
    LParen,
    RParen, // ( )
    LBracket,
    RBracket, // [ ]
    LBrace,
    RBrace, // { }
    Comma,
    Colon,
    Dot,
    Semicolon,

    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize) -> Self {
        Token { kind, line }
    }
}

pub fn lookup_keyword(word: &str) -> Option<TokenKind> {
    match word {
        "if" => Some(TokenKind::If),
        "then" => Some(TokenKind::Then),
        "else" => Some(TokenKind::Else),
        "end" => Some(TokenKind::End),
        "while" => Some(TokenKind::While),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "function" => Some(TokenKind::Function),
        "return" => Some(TokenKind::Return),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "null" => Some(TokenKind::Null),
        "and" => Some(TokenKind::And),
        "or" => Some(TokenKind::Or),
        "not" => Some(TokenKind::Not),
        "new" => Some(TokenKind::New),
        "self" => Some(TokenKind::Self_),
        "globals" => Some(TokenKind::Globals),
        "intrinsics" => Some(TokenKind::Intrinsics),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        "isa" => Some(TokenKind::Isa),
        "mod" => Some(TokenKind::Mod),
        _ => None,
    }
}
