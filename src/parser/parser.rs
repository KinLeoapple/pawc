// File: src/parser.rs

use crate::ast::{BinaryOp, Expr, Param, Statement, StatementKind};
use crate::error::error::PawError;
use crate::lexer::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Peek at the current token without consuming
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Look ahead n tokens without consuming
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }

    /// Peek at the kind of the current token
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    /// Peek at the kind of the token n ahead
    fn peek_n_kind(&self, n: usize) -> Option<&TokenKind> {
        self.tokens.get(self.position + n).map(|t| &t.kind)
    }

    /// Consume and return the next token
    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.position).cloned();
        self.position += 1;
        tok
    }

    /// Parse a full program (list of statements)
    pub fn parse_program(&mut self) -> Result<Vec<Statement>, PawError> {
        let mut out = Vec::new();
        while let Some(tok) = self.peek() {
            if tok.kind == TokenKind::Eof {
                break;
            }
            out.push(self.parse_statement()?);
        }
        Ok(out)
    }

    /// Parse one statement
    pub fn parse_statement(&mut self) -> Result<Statement, PawError> {
        // Assignment: Identifier = ...
        if let Some(tok) = self.peek() {
            if let TokenKind::Identifier(_) = &tok.kind {
                if let Some(next) = self.peek_n_kind(1) {
                    if *next == TokenKind::Assign {
                        let name = self.expect_identifier()?;
                        self.expect_token(TokenKind::Assign)?;
                        let value = self.parse_expr()?;
                        return Ok(Statement::new(StatementKind::Assign { name, value }));
                    }
                }
            }
        }

        // import
        if self.peek_keyword("import") {
            return self.parse_import_statement();
        }

        // other statements
        if let Some(tok) = self.peek() {
            match &tok.kind {
                TokenKind::Keyword(kw) => match kw.as_str() {
                    "let" => return self.parse_let_statement(),
                    "say" => return self.parse_say_statement(),
                    "ask" => return self.parse_ask_prompt_statement(),
                    "return" => return self.parse_return_statement(),
                    "break" => {
                        self.next();
                        return Ok(Statement::new(StatementKind::Break));
                    }
                    "continue" => {
                        self.next();
                        return Ok(Statement::new(StatementKind::Continue));
                    }
                    "if" => return self.parse_if_statement(),
                    "loop" => return self.parse_loop_statement(),
                    "fun" => return self.parse_fun_statement(),
                    "bark" => return self.parse_throw(),
                    "sniff" => return self.parse_try_catch_finally(),
                    "snatch" => {
                        return Err(PawError::Syntax {
                            code: "E1001",
                            message: "`snatch` cannot appear alone".into(),
                            line: tok.line,
                            column: tok.column,
                            snippet: None,
                            hint: Some("`snatch` must follow a `sniff` block".into()),
                        })
                    }
                    "lastly" => {
                        return Err(PawError::Syntax {
                            code: "E1001",
                            message: "`lastly` cannot appear alone".into(),
                            line: tok.line,
                            column: tok.column,
                            snippet: None,
                            hint: Some("`lastly` must follow a `snatch` block".into()),
                        })
                    }
                    _ => {}
                },
                TokenKind::LBrace => {
                    let blk = self.parse_block_statement()?;
                    return Ok(blk);
                }
                _ => {}
            }
        }

        // fallback: expression statement
        self.parse_expr_statement()
    }

    fn parse_let_statement(&mut self) -> Result<Statement, PawError> {
        self.next().unwrap(); // consume 'let'
        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        // ask init
        if let Some(tok) = self.peek() {
            if tok.kind == TokenKind::LeftArrow {
                self.next();
                self.expect_keyword("ask")?;
                let prompt = self.expect_string_literal()?;
                return Ok(Statement::new(StatementKind::Ask { name, ty, prompt }));
            }
        }

        // normal let
        self.expect_token(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Let { name, ty, value }))
    }

    fn parse_say_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("say")?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Say(expr)))
    }

    fn parse_ask_prompt_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("ask")?;
        let p = self.expect_string_literal()?;
        Ok(Statement::new(StatementKind::AskPrompt(p)))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, PawError> {
        self.next().unwrap();
        if let Some(kind) = self.peek_kind() {
            if *kind == TokenKind::Eof || *kind == TokenKind::RBrace {
                return Ok(Statement::new(StatementKind::Return(None)));
            }
        }
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Return(Some(expr))))
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, PawError> {
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Expr(expr)))
    }

    pub fn parse_if_statement(&mut self) -> Result<Statement, PawError> {
        self.next().unwrap();
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        let else_branch = if self.peek_keyword("else") {
            self.next();
            if self.peek_keyword("if") {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(Box::new(Statement::new(StatementKind::Block(
                    self.parse_block()?,
                ))))
            }
        } else {
            None
        };
        Ok(Statement::new(StatementKind::If {
            condition,
            body,
            else_branch,
        }))
    }

    fn parse_loop_statement(&mut self) -> Result<Statement, PawError> {
        self.next().unwrap();
        if self.peek_keyword("forever") {
            self.next();
            let body = self.parse_block()?;
            return Ok(Statement::new(StatementKind::LoopForever(body)));
        }
        // range loop
        if let (Some(tok1), Some(tok2)) = (self.peek(), self.peek_n(1)) {
            if let TokenKind::Identifier(var) = &tok1.kind {
                if tok2.kind == TokenKind::Keyword("in".into()) {
                    let var_name = var.clone();
                    self.next();
                    self.next();
                    let start = self.parse_expr()?;
                    self.expect_token(TokenKind::Range)?;
                    let end = self.parse_expr()?;
                    let body = self.parse_block()?;
                    return Ok(Statement::new(StatementKind::LoopRange {
                        var: var_name,
                        start,
                        end,
                        body,
                    }));
                }
            }
        }
        // while loop
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Statement::new(StatementKind::LoopWhile { condition, body }))
    }

    fn parse_fun_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("fun")?;
        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
            let pn = self.expect_identifier()?;
            self.expect_token(TokenKind::Colon)?;
            let pt = self.parse_type()?;
            params.push(Param { name: pn, ty: pt });
            if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                self.next();
            }
        }
        self.expect_token(TokenKind::RParen)?;
        let ret = if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.next();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Statement::new(StatementKind::FunDecl {
            name,
            params,
            return_type: ret,
            body,
        }))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, PawError> {
        let stmts = self.parse_block()?;
        Ok(Statement::new(StatementKind::Block(stmts)))
    }

    /// Parse `{ ... }`
    fn parse_block(&mut self) -> Result<Vec<Statement>, PawError> {
        self.expect_token(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
            stmts.push(self.parse_statement()?);
        }
        self.expect_token(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_import_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("import")?;
        let mut module = Vec::new();
        loop {
            if let Some(tok) = self.next() {
                if let TokenKind::Identifier(seg) = tok.kind {
                    module.push(seg);
                } else {
                    return Err(PawError::Syntax {
                        code: "E1001",
                        message: format!("Expected module path segment, got {:?}", tok.kind),
                        line: tok.line,
                        column: tok.column,
                        snippet: None,
                        hint: Some("Module path must be identifiers separated by dots".into()),
                    });
                }
            }
            if matches!(self.peek_kind(), Some(TokenKind::Dot)) {
                self.next();
                continue;
            }
            break;
        }
        let alias = if self.peek_keyword("as") {
            self.next();
            self.expect_identifier()?
        } else {
            module.last().cloned().unwrap_or_default()
        };
        Ok(Statement::new(StatementKind::Import { module, alias }))
    }

    fn parse_type(&mut self) -> Result<String, PawError> {
        let base = if let Some(tok) = self.next() {
            match tok.kind {
                TokenKind::Type(name) | TokenKind::Identifier(name) => name,
                _ => {
                    return Err(PawError::Syntax {
                        code: "E1001",
                        message: format!("Expected type, got {:?}", tok.kind),
                        line: tok.line,
                        column: tok.column,
                        snippet: None,
                        hint: Some("Type names must be identifiers or built-in types".into()),
                    })
                }
            }
        } else {
            return Err(PawError::Syntax {
                code: "E1001",
                message: "Expected type, got EOF".into(),
                line: 0,
                column: 0,
                snippet: None,
                hint: Some("Perhaps you forgot a type annotation".into()),
            });
        };
        let mut ty = base;
        if matches!(self.peek_kind(), Some(TokenKind::Lt)) {
            self.next();
            let inner = self.parse_type()?;
            self.expect_token(TokenKind::Gt)?;
            ty = format!("{}<{}>", ty, inner);
        }
        if matches!(self.peek_kind(), Some(TokenKind::Question)) {
            self.next();
            ty.push('?');
        }
        Ok(ty)
    }

    fn parse_throw(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("bark")?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Throw(expr)))
    }

    fn parse_try_catch_finally(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("sniff")?;
        let body = self.parse_block()?;
        self.expect_keyword("snatch")?;
        self.expect_token(TokenKind::LParen)?;
        let err_name = self.expect_identifier()?;
        self.expect_token(TokenKind::RParen)?;
        let handler = self.parse_block()?;
        let finally = if self.peek_keyword("lastly") {
            self.next();
            self.parse_block()?
        } else {
            Vec::new()
        };
        Ok(Statement::new(StatementKind::TryCatchFinally {
            body,
            err_name,
            handler,
            finally,
        }))
    }

    pub fn parse_expr(&mut self) -> Result<Expr, PawError> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr, PawError> {
        let mut left = self.parse_unary_expr()?;
        while let Some(tok) = self.peek() {
            // cast 'as'
            if let TokenKind::Keyword(k) = &tok.kind {
                if k == "as" && min_prec == 0 {
                    self.next();
                    let ty = self.parse_type()?;
                    left = Expr::Cast {
                        expr: Box::new(left),
                        ty,
                    };
                    continue;
                }
            }
            // other binary ops
            let (prec, right_assoc, op_kind) = match &tok.kind {
                TokenKind::Plus => (6, false, BinaryOp::Add),
                TokenKind::Minus => (6, false, BinaryOp::Sub),
                TokenKind::Star => (7, false, BinaryOp::Mul),
                TokenKind::Slash => (7, false, BinaryOp::Div),
                TokenKind::Percent => (7, false, BinaryOp::Mod),
                TokenKind::EqEq => (5, false, BinaryOp::EqEq),
                TokenKind::NotEq => (5, false, BinaryOp::NotEq),
                TokenKind::Lt => (5, false, BinaryOp::Lt),
                TokenKind::Le => (5, false, BinaryOp::Le),
                TokenKind::Gt => (5, false, BinaryOp::Gt),
                TokenKind::Ge => (5, false, BinaryOp::Ge),
                TokenKind::AndAnd => (4, false, BinaryOp::And),
                TokenKind::OrOr => (3, false, BinaryOp::Or),
                _ => break,
            };
            if prec < min_prec {
                break;
            }
            self.next();
            let next_min = if right_assoc { prec } else { prec + 1 };
            let right = self.parse_binary_expr(next_min)?;
            left = Expr::BinaryOp {
                op: op_kind,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, PawError> {
        if let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Minus => {
                    self.next();
                    let e = self.parse_unary_expr()?;
                    return Ok(Expr::UnaryOp {
                        op: "-".into(),
                        expr: Box::new(e),
                    });
                }
                TokenKind::Not => {
                    self.next();
                    let e = self.parse_unary_expr()?;
                    return Ok(Expr::UnaryOp {
                        op: "!".into(),
                        expr: Box::new(e),
                    });
                }
                _ => {}
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, PawError> {
        if self.peek_keyword("nopaw") {
            self.next().unwrap();
            return Ok(Expr::LiteralNopaw);
        }
        let tok = self.next().ok_or_else(|| PawError::Syntax {
            code: "E1001",
            message: "Unexpected EOF in primary".into(),
            line: 0,
            column: 0,
            snippet: None,
            hint: Some("Expression expected".into()),
        })?;
        let mut expr = match tok.kind {
            TokenKind::IntLiteral(n) => Expr::LiteralInt(n),
            TokenKind::LongLiteral(n) => Expr::LiteralLong(n),
            TokenKind::FloatLiteral(f) => Expr::LiteralFloat(f),
            TokenKind::BoolLiteral(b) => Expr::LiteralBool(b),
            TokenKind::StringLiteral(s) => Expr::LiteralString(s),
            TokenKind::CharLiteral(c) => Expr::LiteralChar(c),
            TokenKind::Keyword(k) if k == "nopaw" => Expr::LiteralNopaw,
            TokenKind::Identifier(n) => Expr::Var(n),
            TokenKind::LParen => {
                let e = self.parse_expr()?;
                self.expect_token(TokenKind::RParen)?;
                e
            }
            TokenKind::LBracket => {
                let mut elems = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RBracket)) {
                    elems.push(self.parse_expr()?);
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.next();
                    }
                }
                self.expect_token(TokenKind::RBracket)?;
                Expr::ArrayLiteral(elems)
            }
            other => {
                return Err(PawError::Syntax {
                    code: "E1001",
                    message: format!("Unexpected {:?} in primary", other),
                    line: tok.line,
                    column: tok.column,
                    snippet: None,
                    hint: Some("Check expression syntax".into()),
                })
            }
        };
        if let Expr::LiteralNopaw = expr {
            return Ok(expr);
        }
        // suffix loop
        loop {
            match self.peek_kind() {
                Some(TokenKind::LParen) => {
                    self.next();
                    let mut args = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                        args.push(self.parse_expr()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.next();
                        }
                    }
                    self.expect_token(TokenKind::RParen)?;
                    expr = Expr::Call {
                        name: match expr {
                            Expr::Var(n) => n,
                            _ => {
                                return Err(PawError::Syntax {
                                    code: "E1001",
                                    message: "Invalid call target".into(),
                                    line: 0,
                                    column: 0,
                                    snippet: None,
                                    hint: None,
                                })
                            }
                        },
                        args,
                    };
                }
                Some(TokenKind::LBracket) => {
                    self.next();
                    let idx = self.parse_expr()?;
                    self.expect_token(TokenKind::RBracket)?;
                    expr = Expr::Index {
                        array: Box::new(expr),
                        index: Box::new(idx),
                    };
                }
                Some(TokenKind::Dot) => {
                    self.next();
                    let prop = self.expect_identifier()?;
                    expr = Expr::Property {
                        object: Box::new(expr),
                        name: prop,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn peek_keyword(&self, kw: &str) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Keyword(k)) if k == kw)
    }

    fn expect_token(&mut self, expected: TokenKind) -> Result<(), PawError> {
        match self.next() {
            Some(tok) if tok.kind == expected => Ok(()),
            Some(tok) => Err(PawError::Syntax {
                code: "E1001",
                message: format!("Expected {:?}, got {:?}", expected, tok.kind),
                line: tok.line,
                column: tok.column,
                snippet: None,
                hint: Some("Check token".into()),
            }),
            None => Err(PawError::Syntax {
                code: "E1001",
                message: format!("Expected {:?}, got EOF", expected),
                line: 0,
                column: 0,
                snippet: None,
                hint: Some("Unexpected end of input".into()),
            }),
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), PawError> {
        if let Some(tok) = self.next() {
            // borrow the inner string instead of moving it
            if let TokenKind::Keyword(ref k2) = tok.kind {
                if k2 == kw {
                    return Ok(());
                }
            }
            // if we reach here, it wasn't the keyword we wanted:
            let got = tok.kind.clone(); // clone for error‑reporting
            return Err(PawError::Syntax {
                code: "E1001",
                message: format!("Expected keyword '{}', got {:?}", kw, got),
                line: tok.line,
                column: tok.column,
                snippet: None,
                hint: Some("Check your keyword spelling".into()),
            });
        }
        Err(PawError::Syntax {
            code: "E1001",
            message: format!("Expected keyword '{}', got EOF", kw),
            line: 0,
            column: 0,
            snippet: None,
            hint: None,
        })
    }

    fn expect_identifier(&mut self) -> Result<String, PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::Identifier(n) = tok.kind {
                return Ok(n);
            }
            return Err(PawError::Syntax {
                code: "E1001",
                message: format!("Expected identifier, got {:?}", tok.kind),
                line: tok.line,
                column: tok.column,
                snippet: None,
                hint: None,
            });
        }
        Err(PawError::Syntax {
            code: "E1001",
            message: "Expected identifier, got EOF".into(),
            line: 0,
            column: 0,
            snippet: None,
            hint: None,
        })
    }

    fn expect_string_literal(&mut self) -> Result<String, PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::StringLiteral(s) = tok.kind {
                return Ok(s);
            }
            return Err(PawError::Syntax {
                code: "E1001",
                message: format!("Expected string literal, got {:?}", tok.kind),
                line: tok.line,
                column: tok.column,
                snippet: None,
                hint: None,
            });
        }
        Err(PawError::Syntax {
            code: "E1001",
            message: "Expected string literal, got EOF".into(),
            line: 0,
            column: 0,
            snippet: None,
            hint: None,
        })
    }
}
