mod token;
use token::Token;

use crate::{
    core::number::Float,
    error::{Error, Result},
    scanner::token::TokenKind,
};

use std::str::FromStr;

// #region Struct

pub struct Scanner<'src> {
    /// Source str
    source: &'src str,

    /// Byte offset of the start of the current lexeme
    start: usize,
    /// Byte offset of the current char
    current: usize,

    /// Current line nb (1 based)
    line: usize,
    /// Byte offset of the start of the current line
    line_start: usize,

    /// Cached current char
    current_char: Option<char>,
}

// #endregion

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            line_start: 0,
            current_char: source.chars().next(),
        }
    }

    // #region Getters extended

    #[inline]
    fn column(&self) -> usize {
        self.source[self.line_start..self.current].chars().count() + 1
    }

    #[inline]
    fn line(&self) -> usize {
        self.line
    }

    #[inline]
    fn offset(&self) -> usize {
        self.current
    }

    #[inline]
    fn lexeme(&self) -> &'src str {
        &self.source[self.start..self.current]
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current_char.is_none()
    }

    fn get_line(&self) -> &'src str {
        let end = self.source[self.current..]
            .find('\n')
            .map_or(self.source.len(), |i| self.current + i);

        &self.source[self.line_start..end]
    }

    fn get_line_from_index(&self, idx: usize) -> &'src str {
        let start = self.source[..idx].rfind('\n').map_or(0, |i| i + 1);

        let end = self.source[idx..]
            .find('\n')
            .map_or(self.source.len(), |i| idx + i);

        &self.source[start..end]
    }

    // #endregion

    // #region Movements

    #[inline]
    fn begin_token(&mut self) {
        self.start = self.current;
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.current_char
    }

    #[inline]
    fn peek_next(&self) -> Option<char> {
        let ch = self.peek()?;
        let next = self.current + ch.len_utf8();

        self.source[next..].chars().next()
    }

    #[inline]
    fn peek_is(&self, ch: char) -> bool {
        self.peek() == Some(ch)
    }

    #[inline]
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;

        self.current += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.line_start = self.current;
        }

        self.current_char = self.source[self.current..].chars().next();

        Some(ch)
    }

    #[inline]
    fn matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    #[inline]
    fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }

            self.advance();
        }
    }

    #[inline]
    fn skip_whitespaces(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }

                _ => break,
            }
        }

        self.begin_token();
    }

    // #endregion

    // #region Types

    fn ident(&mut self) -> TokenKind<'src> {
        while let Some(ch) = self.peek() {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }

        let ident = self.lexeme();

        match ident {
            "nan" => TokenKind::Nan,
            "null" => TokenKind::Null,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "for" => TokenKind::For,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "while" => TokenKind::While,
            "end" => TokenKind::End,
            "in" => TokenKind::In,
            "isa" => TokenKind::Isa,
            "globals" => TokenKind::Globals,
            "locals" => TokenKind::Locals,
            "new" => TokenKind::New,
            "outer" => TokenKind::Outer,
            "return" => TokenKind::Return,
            "self" => TokenKind::This,
            "super" => TokenKind::Super,
            _ => TokenKind::Ident(ident),
        }
    }

    fn number(&mut self) -> Result<TokenKind<'src>> {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        if let Some('.') = self.peek() {
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(TokenKind::Number(Float::from_str(self.lexeme()).map_err(
            |e| {
                Error::parse(
                    format!("failed to parse number: {}", e),
                    Error::line_infos(self.line, self.get_line(), (self.start, self.column())),
                )
            },
        )?))
    }

    fn string(&mut self) -> Result<TokenKind<'src>> {
        let line_nb = self.line;

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                if let Some('"') = self.peek() {
                    self.advance();
                } else {
                    break;
                }
            } else {
                self.advance();
            }
        }

        if let None = self.peek() {
            let line = self.get_line_from_index(self.start);

            let column = self.start - (self.source[..self.start].rfind('\n').map_or(0, |i| i + 1));

            return Err(Error::parse(
                "missing closing '\"'",
                Error::line_infos(line_nb, line, (column, line.len() - column)),
            ));
        }

        Ok(TokenKind::String(self.lexeme()))
    }

    // #endregion

    fn next_token(&mut self) -> Result<Token<'src>> {
        self.skip_whitespaces();

        if self.is_at_end() {
            return Ok(Token::new(TokenKind::EOF, self.line));
        }
        self.begin_token();

        let Some(ch) = self.advance() else {
            return Err(Error::parse(
                "failed to retreive current char, while not at end",
                Error::line_infos(self.line, self.get_line(), (self.column(), self.column())),
            ));
        };

        Ok(Token::new(
            match ch {
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                '[' => TokenKind::LeftBracket,
                ']' => TokenKind::RightBracket,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                '@' => TokenKind::At,
                '+' => {
                    if self.matches('=') {
                        TokenKind::PlusEqual
                    } else {
                        TokenKind::Plus
                    }
                }
                '-' => {
                    if self.matches('=') {
                        TokenKind::MinusEqual
                    } else {
                        TokenKind::Minus
                    }
                }
                '*' => {
                    if self.matches('=') {
                        TokenKind::StarEqual
                    } else {
                        TokenKind::Star
                    }
                }
                '/' => {
                    if self.matches('=') {
                        TokenKind::SlashEqual
                    } else if self.matches('/') {
                        self.advance_while(|c| c != '\n');
                        self.advance();
                        TokenKind::NewLine
                    } else {
                        TokenKind::Slash
                    }
                }
                '!' => {
                    if self.matches('=') {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    }
                }
                '=' => {
                    if self.matches('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    }
                }
                '>' => {
                    if self.matches('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    }
                }
                '<' => {
                    if self.matches('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    }
                }
                c if (c == '_' || c.is_ascii_alphanumeric()) => self.ident(),
                c if c.is_ascii_digit() => self.number()?,
                '"' => self.string()?,
                _ => {
                    return Err(Error::parse(
                        format!("unexpected character: \"{}\"", ch),
                        Error::line_infos(
                            self.line,
                            self.get_line(),
                            (self.column(), self.column()),
                        ),
                    ));
                }
            },
            self.line,
        ))
    }
}
impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(t) => {
                if t.is_eof() {
                    None
                } else {
                    Some(Ok(t))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}
