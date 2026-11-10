pub mod ast;

pub use ast::*;

use crate::error::ParseError;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        self.skip_terminators();
        while !self.is_at_end() {
            stmts.push(self.statement()?);
            self.skip_terminators();
        }
        Ok(stmts)
    }

    // ── Statements ───────────────────────────────────────────────

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::If => self.if_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::Return => self.return_stmt(),
            TokenKind::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            TokenKind::Continue => {
                self.advance();
                Ok(Stmt::Continue)
            }
            TokenKind::Mod => self.mod_stmt(),
            _ => self.expr_or_assign(),
        }
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::If)?;
        let cond = self.expression()?;
        self.expect(TokenKind::Then)?;
        self.skip_terminators();

        let body = self.block_until(&[TokenKind::Else, TokenKind::End])?;
        let mut branches = vec![(cond, body)];
        let mut else_body = None;

        loop {
            if self.check(TokenKind::End) {
                self.advance(); // 'end'
                self.expect_ident("if")?;
                break;
            }
            if self.check(TokenKind::Else) {
                self.advance(); // 'else'
                // 'else if' branch
                if self.check(TokenKind::If) {
                    self.advance(); // 'if'
                    let cond = self.expression()?;
                    self.expect(TokenKind::Then)?;
                    self.skip_terminators();
                    let body = self.block_until(&[TokenKind::Else, TokenKind::End])?;
                    branches.push((cond, body));
                } else {
                    // final else
                    self.skip_terminators();
                    let body = self.block_until(&[TokenKind::End])?;
                    else_body = Some(body);
                    self.advance(); // 'end'
                    self.expect_ident("if")?;
                    break;
                }
            }
        }
        Ok(Stmt::If {
            branches,
            else_body,
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::While)?;
        let condition = self.expression()?;
        self.skip_terminators();
        let body = self.block_until(&[TokenKind::End])?;
        self.advance(); // 'end'
        self.expect_ident("while")?;
        Ok(Stmt::While { condition, body })
    }

    fn for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::For)?;
        let var = self.expect_identifier()?;
        self.expect(TokenKind::In)?;
        let iterable = self.expression()?;
        self.skip_terminators();
        let body = self.block_until(&[TokenKind::End])?;
        self.advance(); // 'end'
        self.expect_ident("for")?;
        Ok(Stmt::For {
            var,
            iterable,
            body,
        })
    }

    fn return_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'return'
        if self.is_terminator() || self.is_at_end() {
            Ok(Stmt::Return(None))
        } else {
            Ok(Stmt::Return(Some(self.expression()?)))
        }
    }

    fn mod_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'mod'
        let mod_name = self.expect_identifier()?;
        Ok(Stmt::Module(mod_name))
    }

    /// Parse expression-statement or assignment (including compound: +=, -=, etc).
    fn expr_or_assign(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        // Plain assignment: target = value
        if self.check(TokenKind::Assign) {
            self.advance();
            let value = self.expression()?;
            let target = Self::expr_to_target(expr)?;
            return Ok(Stmt::Assignment { target, value });
        }

        // Compound assignment: += -= *= /= %= ^=
        if let Some(op) = self.match_compound_assign() {
            let rhs = self.expression()?;
            let target = Self::expr_to_target(expr.clone())?;
            let value = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(rhs),
            };
            return Ok(Stmt::Assignment { target, value });
        }

        // Paren-free call: `print "hi"`, `foo 1, 2`, `obj.method arg`
        // Only if the expr looks callable (identifier or member) and is NOT already a call,
        // and the next token can start an argument expression on the same line.
        if !matches!(expr, Expr::Call { .. })
            && self.is_callable_expr(&expr)
            && self.can_start_paren_free_arg()
        {
            let args = self.paren_free_args()?;
            return Ok(Stmt::Expression(Expr::Call {
                callee: Box::new(expr),
                args,
            }));
        }

        Ok(Stmt::Expression(expr))
    }

    fn is_callable_expr(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Identifier(_) | Expr::Member { .. })
    }

    /// Can the next token begin a paren-free argument?
    fn can_start_paren_free_arg(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Number(_)
                | TokenKind::StringLit(_)
                | TokenKind::Identifier(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::Not
                | TokenKind::Function
                | TokenKind::New
                | TokenKind::At
                | TokenKind::Self_
                | TokenKind::Globals
        )
    }

    fn paren_free_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = vec![self.expression()?];
        while self.check(TokenKind::Comma) {
            self.advance();
            args.push(self.expression()?);
        }
        Ok(args)
    }

    fn match_compound_assign(&mut self) -> Option<BinOp> {
        let op = match self.peek_kind() {
            TokenKind::PlusAssign => BinOp::Add,
            TokenKind::MinusAssign => BinOp::Sub,
            TokenKind::StarAssign => BinOp::Mul,
            TokenKind::SlashAssign => BinOp::Div,
            TokenKind::PercentAssign => BinOp::Mod,
            TokenKind::CaretAssign => BinOp::Power,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn expr_to_target(expr: Expr) -> Result<AssignTarget, ParseError> {
        match expr {
            Expr::Identifier(name) => Ok(AssignTarget::Variable(name)),
            Expr::Index { object, index } => Ok(AssignTarget::Index { object, index }),
            Expr::Member { object, field } => Ok(AssignTarget::Member { object, field }),
            Expr::GlobalAccess(name) => Ok(AssignTarget::Global(name)),
            _ => Err(ParseError {
                line: 0,
                message: "invalid assignment target".into(),
            }),
        }
    }

    // ── Expressions (precedence climbing) ────────────────────────

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.and_expr()?;
        while self.check(TokenKind::Or) {
            self.advance();
            let right = self.and_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.not_expr()?;
        while self.check(TokenKind::And) {
            self.advance();
            let right = self.not_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn not_expr(&mut self) -> Result<Expr, ParseError> {
        if self.check(TokenKind::Not) {
            self.advance();
            let expr = self.not_expr()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        self.isa_expr()
    }

    fn isa_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.comparison()?;
        while self.check(TokenKind::Isa) {
            self.advance();
            let right = self.comparison()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Isa,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.addition()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Equal => BinOp::Eq,
                TokenKind::NotEqual => BinOp::NotEq,
                TokenKind::Less => BinOp::Lt,
                TokenKind::Greater => BinOp::Gt,
                TokenKind::LessEq => BinOp::LtEq,
                TokenKind::GreaterEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.addition()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.multiplication()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.multiplication()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(TokenKind::Minus) {
            self.advance();
            let expr = self.unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
            });
        }
        if self.check(TokenKind::New) {
            self.advance();
            let expr = self.unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::New,
                expr: Box::new(expr),
            });
        }
        if self.check(TokenKind::At) {
            self.advance();
            let expr = self.power()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::AddressOf,
                expr: Box::new(expr),
            });
        }
        self.power()
    }

    fn power(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.postfix()?;
        // Right-associative: a ^ b ^ c = a ^ (b ^ c)
        if self.check(TokenKind::Caret) {
            self.advance();
            let right = self.unary()?; // right-recursive for right-assoc
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Power,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Postfix: function calls, indexing/slicing, member access.
    fn postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.check(TokenKind::LParen) {
                self.advance();
                let args = self.arg_list()?;
                self.expect(TokenKind::RParen)?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.check(TokenKind::LBracket) {
                self.advance();
                expr = self.index_or_slice(expr)?;
            } else if self.check(TokenKind::Dot) {
                self.advance();
                let field = self.expect_identifier()?;
                expr = Expr::Member {
                    object: Box::new(expr),
                    field,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn index_or_slice(&mut self, object: Expr) -> Result<Expr, ParseError> {
        // Could be a[i], a[i:j], a[:j], a[i:]
        if self.check(TokenKind::Colon) {
            // a[:j]
            self.advance();
            let to = if self.check(TokenKind::RBracket) {
                None
            } else {
                Some(Box::new(self.expression()?))
            };
            self.expect(TokenKind::RBracket)?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                from: None,
                to,
            });
        }
        let from_expr = self.expression()?;
        if self.check(TokenKind::Colon) {
            // a[i:j] or a[i:]
            self.advance();
            let to = if self.check(TokenKind::RBracket) {
                None
            } else {
                Some(Box::new(self.expression()?))
            };
            self.expect(TokenKind::RBracket)?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                from: Some(Box::new(from_expr)),
                to,
            });
        }
        self.expect(TokenKind::RBracket)?;
        Ok(Expr::Index {
            object: Box::new(object),
            index: Box::new(from_expr),
        })
    }

    fn arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        self.skip_terminators();
        if !self.check(TokenKind::RParen) {
            args.push(self.expression()?);
            self.skip_terminators();
            while self.check(TokenKind::Comma) {
                self.advance();
                self.skip_terminators();
                args.push(self.expression()?);
                self.skip_terminators();
            }
        }
        Ok(args)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.advance_token();
        match tok.kind {
            TokenKind::Number(n) => Ok(Expr::Number(n)),
            TokenKind::StringLit(s) => Ok(Expr::StringLit(s)),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Null => Ok(Expr::Null),

            TokenKind::Self_ => Ok(Expr::Self_),

            TokenKind::Globals => {
                // globals.field
                self.expect(TokenKind::Dot)?;
                let field = self.expect_identifier()?;
                Ok(Expr::GlobalAccess(field))
            }

            TokenKind::Identifier(name) => Ok(Expr::Identifier(name)),

            TokenKind::LParen => {
                let expr = self.expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            TokenKind::LBracket => {
                let mut items = Vec::new();
                self.skip_terminators();
                if !self.check(TokenKind::RBracket) {
                    items.push(self.expression()?);
                    self.skip_terminators();
                    while self.check(TokenKind::Comma) {
                        self.advance();
                        self.skip_terminators();
                        if self.check(TokenKind::RBracket) {
                            break;
                        }
                        items.push(self.expression()?);
                        self.skip_terminators();
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::ListLiteral(items))
            }

            TokenKind::LBrace => {
                let mut pairs = Vec::new();
                self.skip_terminators();
                if !self.check(TokenKind::RBrace) {
                    let key = self.expression()?;
                    self.expect(TokenKind::Colon)?;
                    let val = self.expression()?;
                    pairs.push((key, val));
                    self.skip_terminators();
                    while self.check(TokenKind::Comma) {
                        self.advance();
                        self.skip_terminators();
                        if self.check(TokenKind::RBrace) {
                            break;
                        }
                        let key = self.expression()?;
                        self.expect(TokenKind::Colon)?;
                        let val = self.expression()?;
                        pairs.push((key, val));
                        self.skip_terminators();
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::MapLiteral(pairs))
            }

            TokenKind::Function => self.func_expr(),

            _ => Err(ParseError {
                line: tok.line,
                message: format!("unexpected token {:?}", tok.kind),
            }),
        }
    }

    fn func_expr(&mut self) -> Result<Expr, ParseError> {
        let params = if self.check(TokenKind::LParen) {
            self.advance();
            let p = self.param_list()?;
            self.expect(TokenKind::RParen)?;
            p
        } else {
            Vec::new()
        };
        self.skip_terminators();
        let body = self.block_until(&[TokenKind::End])?;
        self.advance(); // 'end'
        self.expect_ident("function")?;
        Ok(Expr::FuncExpr { params, body })
    }

    fn param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            params.push(self.param()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                params.push(self.param()?);
            }
        }
        Ok(params)
    }

    fn param(&mut self) -> Result<Param, ParseError> {
        let name = self.expect_identifier()?;
        let default = if self.check(TokenKind::Assign) {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };
        Ok(Param { name, default })
    }

    // ── Block parsing ────────────────────────────────────────────

    fn block_until(&mut self, terminators: &[TokenKind]) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        self.skip_terminators();
        while !self.is_at_end() && !terminators.iter().any(|t| self.check_match(t)) {
            stmts.push(self.statement()?);
            self.skip_terminators();
        }
        Ok(stmts)
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn peek_line(&self) -> usize {
        self.tokens.get(self.pos).map(|t| t.line).unwrap_or(0)
    }

    fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(&kind)
    }

    /// Check if current token matches (for terminators that carry data).
    fn check_match(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn advance_token(&mut self) -> Token {
        let tok = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(Token::new(TokenKind::Eof, 0));
        self.advance();
        tok
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.check(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                line: self.peek_line(),
                message: format!("expected {:?}, got {:?}", kind, self.peek_kind()),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let TokenKind::Identifier(name) = self.peek_kind() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError {
                line: self.peek_line(),
                message: format!("expected identifier, got {:?}", self.peek_kind()),
            })
        }
    }

    /// Expect an identifier with a specific name (for `end if`, `end while`, etc).
    fn expect_ident(&mut self, expected: &str) -> Result<(), ParseError> {
        if let TokenKind::Identifier(ref name) = self.peek_kind() {
            if name == expected {
                self.advance();
                return Ok(());
            }
        }
        // Also allow as keyword (e.g. "if" is a keyword token, not Identifier)
        // For `end if`, `end while`, `end for`, `end function` the second word is
        // always a keyword, so match those too:
        let matched = match self.peek_kind() {
            TokenKind::If if expected == "if" => true,
            TokenKind::While if expected == "while" => true,
            TokenKind::For if expected == "for" => true,
            TokenKind::Function if expected == "function" => true,
            _ => false,
        };
        if matched {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                line: self.peek_line(),
                message: format!("expected '{expected}', got {:?}", self.peek_kind()),
            })
        }
    }

    fn is_terminator(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Newline | TokenKind::Semicolon | TokenKind::Eof
        )
    }

    fn skip_terminators(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Semicolon) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}
