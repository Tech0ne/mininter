use std::fmt::Display;

use crate::core::number::Float;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'src> {
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]

    Comma, // ,
    Dot,   // .

    Plus,       // +
    PlusEqual,  // +=
    Minus,      // -
    MinusEqual, // -=
    Star,       // *
    StarEqual,  // *=
    Slash,      // /
    SlashEqual, // /=

    Bang,         // !
    BangEqual,    // !=
    Equal,        // =
    EqualEqual,   // ==
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=

    Ident(&'src str),  // var name
    String(&'src str), // "Hello, world !"
    Number(Float),     // 1, 12, 3.14, 0.33333333333
    Nan,               // Not A Number
    Null,              // null

    True,  // true
    False, // false

    And, // and
    Or,  // or

    Break,    // break
    Continue, // continue

    For,      // for
    Function, // function
    If,       // if
    While,    // while

    End, // end (if not matched: error)

    At,      // @
    In,      // in
    Isa,     // isa
    Globals, // globals
    Locals,  // locals
    NewLine, // \n
    New,     // new
    Outer,   // outer
    Return,  // return
    This,    // self
    Super,   // super

    EOF,
}

impl<'src> Display for TokenKind<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),

            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),

            Self::Plus => write!(f, "+"),
            Self::PlusEqual => write!(f, "+="),
            Self::Minus => write!(f, "-"),
            Self::MinusEqual => write!(f, "-="),
            Self::Star => write!(f, "*"),
            Self::StarEqual => write!(f, "*="),
            Self::Slash => write!(f, "/"),
            Self::SlashEqual => write!(f, "/="),

            Self::Bang => write!(f, "!"),
            Self::BangEqual => write!(f, "!="),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => write!(f, "=="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),

            Self::Ident(s) => write!(f, "{}", s),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Nan => write!(f, "[Not A Number]"),
            Self::Null => write!(f, "null"),

            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),

            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),

            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),

            Self::For => write!(f, "for"),
            Self::Function => write!(f, "function"),
            Self::If => write!(f, "if"),
            Self::While => write!(f, "while"),

            Self::End => write!(f, "end"),

            Self::At => write!(f, "@"),
            Self::In => write!(f, "in"),
            Self::Isa => write!(f, "isa"),
            Self::Globals => write!(f, "globals"),
            Self::Locals => write!(f, "locals"),
            Self::NewLine => writeln!(f, ""),
            Self::New => write!(f, "new"),
            Self::Outer => write!(f, "outer"),
            Self::Return => write!(f, "return"),
            Self::This => write!(f, "self"),
            Self::Super => write!(f, "super"),

            Self::EOF => write!(f, "[EOF]"),
        }
    }
}

pub struct Token<'src> {
    kind: TokenKind<'src>,
    line: usize,
    prev_line: Option<usize>,
}

impl<'src> Token<'src> {
    pub fn new(kind: TokenKind<'src>, line: usize) -> Self {
        Self {
            kind,
            line,
            prev_line: None,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn set_prev_line(&mut self, prev_line: usize) {
        self.prev_line = Some(prev_line);
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::EOF
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.prev_line {
            Some(l) if l == self.line => {
                write!(f, "     | {}", self.kind)
            }
            _ => {
                write!(f, " {:5} {}", self.line, self.kind)
            }
        }
    }
}
