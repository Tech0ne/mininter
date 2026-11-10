pub mod token;

pub use token::{Token, TokenKind};

use crate::error::LexError;

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.at_end() {
            let ch = self.current();
            match ch {
                ' ' | '\t' | '\r' => { self.advance(); }
                '\n' => {
                    tokens.push(Token::new(TokenKind::Newline, self.line));
                    self.line += 1;
                    self.advance();
                }
                ';' => {
                    // Semicolons act as line separators (same as newline)
                    tokens.push(Token::new(TokenKind::Semicolon, self.line));
                    self.advance();
                }
                '/' if self.peek_next() == Some('/') => {
                    // Comment: skip to end of line
                    while !self.at_end() && self.current() != '\n' {
                        self.advance();
                    }
                }
                '"' => tokens.push(self.string()?),
                '(' => { tokens.push(Token::new(TokenKind::LParen, self.line)); self.advance(); }
                ')' => { tokens.push(Token::new(TokenKind::RParen, self.line)); self.advance(); }
                '[' => { tokens.push(Token::new(TokenKind::LBracket, self.line)); self.advance(); }
                ']' => { tokens.push(Token::new(TokenKind::RBracket, self.line)); self.advance(); }
                '{' => { tokens.push(Token::new(TokenKind::LBrace, self.line)); self.advance(); }
                '}' => { tokens.push(Token::new(TokenKind::RBrace, self.line)); self.advance(); }
                ',' => { tokens.push(Token::new(TokenKind::Comma, self.line)); self.advance(); }
                ':' => { tokens.push(Token::new(TokenKind::Colon, self.line)); self.advance(); }
                '.' => { tokens.push(Token::new(TokenKind::Dot, self.line)); self.advance(); }
                '@' => { tokens.push(Token::new(TokenKind::At, self.line)); self.advance(); }
                '+' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::PlusAssign, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Plus, self.line));
                    }
                }
                '-' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::MinusAssign, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Minus, self.line));
                    }
                }
                '*' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::StarAssign, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Star, self.line));
                    }
                }
                '/' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::SlashAssign, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Slash, self.line));
                    }
                }
                '%' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::PercentAssign, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Percent, self.line));
                    }
                }
                '^' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::CaretAssign, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Caret, self.line));
                    }
                }
                '=' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::Equal, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Assign, self.line));
                    }
                }
                '!' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::NotEqual, self.line));
                    } else {
                        return Err(LexError { line: self.line, message: "unexpected '!'".into() });
                    }
                }
                '<' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::LessEq, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Less, self.line));
                    }
                }
                '>' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::new(TokenKind::GreaterEq, self.line));
                    } else {
                        tokens.push(Token::new(TokenKind::Greater, self.line));
                    }
                }
                _ if ch.is_ascii_digit() => tokens.push(self.number()),
                _ if ch.is_alphanumeric() || ch == '_' => tokens.push(self.identifier()),
                _ => {
                    return Err(LexError {
                        line: self.line,
                        message: format!("unexpected character '{ch}'"),
                    });
                }
            }
        }
        tokens.push(Token::new(TokenKind::Eof, self.line));
        Ok(tokens)
    }

    // MiniScript uses doubled quotes for escaping: "say ""hi"""
    fn string(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        self.advance(); // skip opening "
        let mut s = String::new();
        loop {
            if self.at_end() {
                return Err(LexError { line, message: "unterminated string".into() });
            }
            let ch = self.current();
            if ch == '"' {
                self.advance();
                // Doubled quote "" is an escaped quote
                if !self.at_end() && self.current() == '"' {
                    s.push('"');
                    self.advance();
                } else {
                    break;
                }
            } else {
                if ch == '\n' { self.line += 1; }
                s.push(ch);
                self.advance();
            }
        }
        Ok(Token::new(TokenKind::StringLit(s), line))
    }

    fn number(&mut self) -> Token {
        let line = self.line;
        let start = self.pos;
        while !self.at_end() && self.current().is_ascii_digit() {
            self.advance();
        }
        if !self.at_end() && self.current() == '.' && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            self.advance(); // skip '.'
            while !self.at_end() && self.current().is_ascii_digit() {
                self.advance();
            }
        }
        let text: String = self.source[start..self.pos].iter().collect();
        let value: f64 = text.parse().unwrap_or(0.0);
        Token::new(TokenKind::Number(value), line)
    }

    fn identifier(&mut self) -> Token {
        let line = self.line;
        let start = self.pos;
        while !self.at_end() && (self.current().is_alphanumeric() || self.current() == '_') {
            self.advance();
        }
        let text: String = self.source[start..self.pos].iter().collect();
        let kind = token::lookup_keyword(&text).unwrap_or(TokenKind::Identifier(text));
        Token::new(kind, line)
    }

    fn current(&self) -> char { self.source[self.pos] }
    fn at_end(&self) -> bool { self.pos >= self.source.len() }
    fn advance(&mut self) { self.pos += 1; }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if !self.at_end() && self.current() == expected {
            self.advance();
            true
        } else {
            false
        }
    }
}
