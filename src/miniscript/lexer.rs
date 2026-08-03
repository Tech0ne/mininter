use std::str::CharIndices;

use super::tokens::{Token, TokenKind};

pub struct Lexer<'src> {
    source: &'src str,
    chars: CharIndices<'src>,
    current: Option<(usize, char)>,
    line: usize,
    col: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut chars = source.char_indices();
        let current = chars.next();

        Self {
            source,
            chars,
            current,
            line: 1,
            col: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let (_, c) = self.current?;

        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        self.current = self.chars.next();

        Some(c)
    }

    fn peek(&self) -> Option<char> {
        self.current.map(|(_, c)| c)
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, c)| c)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_while<F>(&mut self, pred: F) -> Option<&'src str>
    where
        F: Fn(char) -> bool,
    {
        let start = self.current?.0;

        while let Some((_, c)) = self.current {
            if !pred(c) {
                break;
            }
            self.advance();
        }

        let end = self.current.map(|(i, _)| i).unwrap_or(self.source.len());

        Some(&self.source[start..end])
    }

    fn skip_whitespaces(&mut self) -> Option<Token<'src>> {
        let mut output = None;

        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            if let Some('\n') = self.advance() {
                output = Some(Token::new(TokenKind::NewLine, self.line, self.col));
            }
        }

        output
    }

    fn next_token(&mut self) -> Option<Token<'src>> {
        if let Some(t) = self.skip_whitespaces() {
            return Some(t);
        }

        let line = self.line;
        let col = self.col;

        let Some(c) = self.peek() else {
            return None;
        };

        let kind = if c.is_ascii_alphabetic() || c == '_' {
            self.identifier_or_keyword()
        } else if c.is_ascii_digit() {
            self.number()
        } else {
            self.operator_or_punctuation()
        };

        let Some(kind) = kind else {
            eprintln!(
                "Error: unexpected input at line {}, col {}: {}",
                line, col, c
            );
            return None;
        };

        Some(Token::new(kind, line, col))
    }

    fn identifier_or_keyword(&mut self) -> Option<TokenKind<'src>> {
        let ident = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '_')?;

        Some(match ident {
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
        })
    }

    fn number(&mut self) -> Option<TokenKind<'src>> {
        let start = self.current.map(|(i, _)| i)?;

        self.consume_while(|c| c.is_ascii_digit());

        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
            self.consume_while(|c| c.is_ascii_digit());
        }

        let end = self.current.map(|(i, _)| i).unwrap_or(self.source.len());

        let text = &self.source[start..end];

        Some(TokenKind::Number(text.parse().ok()?))
    }

    fn string(&mut self) -> Option<&'src str> {
        self.current?;

        let content_start = self.current.map(|(i, _)| i)?;

        loop {
            match self.current {
                None => {
                    return None;
                }

                Some((_, '"')) => {
                    self.advance();

                    if self.peek() == Some('"') {
                        self.advance();
                        continue;
                    }

                    let end = self
                        .current
                        .map(|(i, _)| i - 1)
                        .unwrap_or(self.source.len() - 1);

                    return Some(&self.source[content_start..end]);
                }

                _ => {
                    self.advance();
                }
            }
        }
    }

    fn operator_or_punctuation(&mut self) -> Option<TokenKind<'src>> {
        Some(match self.advance().unwrap() {
            '"' => match self.string() {
                None => return None,
                Some(s) => TokenKind::String(s),
            },

            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,

            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,

            '+' => {
                if self.match_char('=') {
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            }

            '-' => {
                if self.match_char('=') {
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                }
            }

            '*' => {
                if self.match_char('=') {
                    TokenKind::StarEqual
                } else {
                    TokenKind::Star
                }
            }

            '/' => {
                if self.match_char('=') {
                    TokenKind::SlashEqual
                } else {
                    TokenKind::Slash
                }
            }

            '!' => {
                if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }

            '=' => {
                if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }

            '>' => {
                if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }

            '<' => {
                if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }

            '@' => TokenKind::At,

            c => {
                eprintln!(
                    "Error: unexpected token at line {}, col {}: {}",
                    self.line, self.col, c
                );
                return None;
            }
        })
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.next_token() {
            if let TokenKind::EOF = token.kind {
                None
            } else {
                Some(token)
            }
        } else {
            None
        }
    }
}
