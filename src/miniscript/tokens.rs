use std::fmt::Display;

#[derive(Debug)]
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
    Number(f64),      // 1, 12, 3.14, 0.33333333333
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

    End,         // end (if not matched: error)
    EndFor,      // end for
    EndFunction, // end function
    EndIf,       // end if
    EndWhile,    // end while

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
        write!(f, "{:?}", self)
    }
}

pub type Token<'src> = crate::core::token::Token<TokenKind<'src>>;
