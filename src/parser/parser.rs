// File: src/parser.rs

use crate::ast::expr::ExprKind;
use crate::ast::{BinaryOp, Expr, Param, Statement, StatementKind};
use crate::error::error::PawError;
use crate::lexer::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    lines: Vec<String>,
    file: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: &str, filename: &str) -> Self {
        let lines = source.lines().map(|l| l.to_string()).collect();
        Self {
            tokens,
            position: 0,
            lines,
            file: filename.to_string(),
        }
    }

    /// 获取指定行的源码片段（1-based）
    fn snippet(&self, line: usize) -> String {
        self.lines
            .get(line.saturating_sub(1))
            .cloned()
            .unwrap_or_default()
    }

    /// 获取当前 token 的行列 (line, col)
    fn wrap_position(&self) -> (usize, usize) {
        if let Some(t) = self.peek() {
            (t.line(), t.column())
        } else {
            (0, 0)
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
        let (start_line, start_col) = self.wrap_position();

        // Assignment: Identifier = ...
        if let Some(tok) = self.peek() {
            if let TokenKind::Identifier(_) = &tok.kind {
                if let Some(next) = self.peek_n_kind(1) {
                    if *next == TokenKind::Assign {
                        let name = self.expect_identifier()?;
                        self.expect_token(TokenKind::Assign)?;
                        let value = self.parse_expr()?;
                        return Ok(Statement::new(
                            StatementKind::Assign { name, value },
                            start_line,
                            start_col,
                        ));
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
                        return Ok(Statement::new(StatementKind::Break, start_line, start_col));
                    }
                    "continue" => {
                        self.next();
                        return Ok(Statement::new(
                            StatementKind::Continue,
                            start_line,
                            start_col,
                        ));
                    }
                    "if" => return self.parse_if_statement(),
                    "loop" => return self.parse_loop_statement(),
                    "fun" => return self.parse_fun_statement(),
                    "bark" => return self.parse_throw(),
                    "sniff" => return self.parse_try_catch_finally(),
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
        let (start_line, start_col) = self.wrap_position();
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
                return Ok(Statement::new(
                    StatementKind::Ask { name, ty, prompt },
                    start_line,
                    start_col,
                ));
            }
        }

        // normal let
        self.expect_token(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Let { name, ty, value },
            start_line,
            start_col,
        ))
    }

    fn parse_say_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        self.expect_keyword("say")?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Say(expr),
            start_line,
            start_col,
        ))
    }

    fn parse_ask_prompt_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        self.expect_keyword("ask")?;
        let p = self.expect_string_literal()?;
        Ok(Statement::new(
            StatementKind::AskPrompt(p),
            start_line,
            start_col,
        ))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        self.next().unwrap();
        if let Some(kind) = self.peek_kind() {
            if *kind == TokenKind::Eof || *kind == TokenKind::RBrace {
                return Ok(Statement::new(
                    StatementKind::Return(None),
                    start_line,
                    start_col,
                ));
            }
        }
        let expr = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Return(Some(expr)),
            start_line,
            start_col,
        ))
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        let expr = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Expr(expr),
            start_line,
            start_col,
        ))
    }

    pub fn parse_if_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        self.next().unwrap();
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        let else_branch = if self.peek_keyword("else") {
            self.next();
            if self.peek_keyword("if") {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(Box::new(Statement::new(
                    StatementKind::Block(self.parse_block()?),
                    start_line,
                    start_col,
                )))
            }
        } else {
            None
        };
        Ok(Statement::new(
            StatementKind::If {
                condition,
                body,
                else_branch,
            },
            start_line,
            start_col,
        ))
    }

    fn parse_loop_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        self.next().unwrap();
        if self.peek_keyword("forever") {
            self.next();
            let body = self.parse_block()?;
            return Ok(Statement::new(
                StatementKind::LoopForever(body),
                start_line,
                start_col,
            ));
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
                    return Ok(Statement::new(
                        StatementKind::LoopRange {
                            var: var_name,
                            start,
                            end,
                            body,
                        },
                        start_line,
                        start_col,
                    ));
                }
            }
        }
        // while loop
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Statement::new(
            StatementKind::LoopWhile { condition, body },
            start_line,
            start_col,
        ))
    }

    fn parse_fun_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
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
        Ok(Statement::new(
            StatementKind::FunDecl {
                name,
                params,
                return_type: ret,
                body,
            },
            start_line,
            start_col,
        ))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
        let stmts = self.parse_block()?;
        Ok(Statement::new(
            StatementKind::Block(stmts),
            start_line,
            start_col,
        ))
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
        let (start_line, start_col) = self.wrap_position();
        self.expect_keyword("import")?;
        let mut module = Vec::new();
        loop {
            if let Some(tok) = self.next() {
                if let TokenKind::Identifier(seg) = tok.kind {
                    module.push(seg);
                } else {
                    return Err(PawError::Syntax {
                        file: self.file.clone(),
                        code: "E1001",
                        message: format!("Expected module path segment, got {:?}", tok.kind),
                        line: tok.line(),
                        column: tok.column(),
                        snippet: Some(self.snippet(tok.line())),
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
        Ok(Statement::new(
            StatementKind::Import { module, alias },
            start_line,
            start_col,
        ))
    }

    fn parse_type(&mut self) -> Result<String, PawError> {
        let (err_line, err_col) = self.wrap_position();
        let base = match self.next() {
            Some(tok) => match tok.kind.clone() {
                TokenKind::Type(name) | TokenKind::Identifier(name) => name,
                other => {
                    return Err(PawError::Syntax {
                        file: self.file.clone(),
                        code: "E1001",
                        message: format!("Expected type, got {:?}", other),
                        line: tok.line(),
                        column: tok.column(),
                        snippet: Some(self.snippet(tok.line())),
                        hint: Some("Type names must be identifiers or built‑in types".into()),
                    })
                }
            },
            None => {
                return Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: "Expected type, got EOF".into(),
                    line: err_line,
                    column: err_col,
                    snippet: Some(self.snippet(err_line)),
                    hint: Some("Perhaps you forgot a type annotation".into()),
                })
            }
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
        let (start_line, start_col) = self.wrap_position();
        self.expect_keyword("bark")?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Throw(expr),
            start_line,
            start_col,
        ))
    }

    fn parse_try_catch_finally(&mut self) -> Result<Statement, PawError> {
        let (start_line, start_col) = self.wrap_position();
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
        Ok(Statement::new(
            StatementKind::TryCatchFinally {
                body,
                err_name,
                handler,
                finally,
            },
            start_line,
            start_col,
        ))
    }

    pub fn parse_expr(&mut self) -> Result<Expr, PawError> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr, PawError> {
        let (expr_line, expr_col) = self.wrap_position();
        let mut left = self.parse_unary_expr()?;
        while let Some(tok) = self.peek() {
            let is_cast = if let TokenKind::Keyword(k) = &tok.kind {
                k == "as" && min_prec == 0
            } else {
                false
            };
            if is_cast {
                self.next();
                let ty = self.parse_type()?;
                left = Expr {
                    kind: ExprKind::Cast {
                        expr: Box::new(left),
                        ty,
                    },
                    line: expr_line,
                    col: expr_col,
                };
                continue;
            }
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
            left = Expr {
                kind: ExprKind::BinaryOp {
                    op: op_kind,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                line: expr_line,
                col: expr_col,
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, PawError> {
        if let Some(tok) = self.peek() {
            let (expr_line, expr_col) = self.wrap_position();
            match tok.kind {
                TokenKind::Minus => {
                    self.next();
                    let e = self.parse_unary_expr()?;
                    return Ok(Expr {
                        kind: ExprKind::UnaryOp {
                            op: "-".into(),
                            expr: Box::new(e),
                        },
                        line: expr_line,
                        col: expr_col,
                    });
                }
                TokenKind::Not => {
                    self.next();
                    let e = self.parse_unary_expr()?;
                    return Ok(Expr {
                        kind: ExprKind::UnaryOp {
                            op: "!".into(),
                            expr: Box::new(e),
                        },
                        line: expr_line,
                        col: expr_col,
                    });
                }
                _ => {}
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, PawError> {
        let (expr_line, expr_col) = self.wrap_position();
        if self.peek_keyword("nopaw") {
            self.next().unwrap();
            return Ok(Expr {
                kind: ExprKind::LiteralNopaw,
                line: expr_line,
                col: expr_col,
            });
        }
        let (err_line, err_col) = self.wrap_position();
        let tok = self.next().ok_or_else(|| PawError::Syntax {
            file: self.file.clone(),
            code: "E1001",
            message: "Unexpected EOF in primary".into(),
            line: err_line,
            column: err_col,
            snippet: Some(self.snippet(err_line)),
            hint: Some("Expression expected".into()),
        })?;
        let mut expr = match tok.kind.clone() {
            TokenKind::IntLiteral(n) => Expr {
                kind: ExprKind::LiteralInt(n),
                line: expr_line,
                col: expr_col,
            },
            TokenKind::LongLiteral(n) => Expr {
                kind: ExprKind::LiteralLong(n),
                line: expr_line,
                col: expr_col,
            },
            TokenKind::FloatLiteral(f) => Expr {
                kind: ExprKind::LiteralFloat(f),
                line: expr_line,
                col: expr_col,
            },
            TokenKind::BoolLiteral(b) => Expr {
                kind: ExprKind::LiteralBool(b),
                line: expr_line,
                col: expr_col,
            },
            TokenKind::StringLiteral(s) => Expr {
                kind: ExprKind::LiteralString(s),
                line: expr_line,
                col: expr_col,
            },
            TokenKind::CharLiteral(c) => Expr {
                kind: ExprKind::LiteralChar(c),
                line: expr_line,
                col: expr_col,
            },
            TokenKind::Identifier(n) => Expr {
                kind: ExprKind::Var(n),
                line: expr_line,
                col: expr_col,
            },
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
                Expr {
                    kind: ExprKind::ArrayLiteral(elems),
                    line: expr_line,
                    col: expr_col,
                }
            }
            other => {
                return Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Unexpected {:?} in primary", other),
                    line: tok.line(),
                    column: tok.column(),
                    snippet: Some(self.snippet(tok.line())),
                    hint: Some("Check expression syntax".into()),
                })
            }
        };
        loop {
            match self.peek_kind() {
                Some(TokenKind::LParen) => {
                    let (call_line, call_col) = self.wrap_position();
                    self.next();
                    let mut args = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                        args.push(self.parse_expr()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.next();
                        }
                    }
                    self.expect_token(TokenKind::RParen)?;
                    let name = if let Expr {
                        kind: ExprKind::Var(n),
                        ..
                    } = expr.clone()
                    {
                        n
                    } else {
                        return Err(PawError::Syntax {
                            file: self.file.clone(),
                            code: "E1001",
                            message: "Invalid call target".into(),
                            line: call_line,
                            column: call_col,
                            snippet: Some(self.snippet(call_line)),
                            hint: None,
                        });
                    };
                    expr = Expr {
                        kind: ExprKind::Call { name, args },
                        line: expr_line,
                        col: expr_col,
                    };
                }
                Some(TokenKind::LBracket) => {
                    let (idx_line, idx_col) = self.wrap_position();
                    self.next();
                    let idx = self.parse_expr()?;
                    self.expect_token(TokenKind::RBracket)?;
                    expr = Expr {
                        kind: ExprKind::Index {
                            array: Box::new(expr),
                            index: Box::new(idx),
                        },
                        line: expr_line,
                        col: expr_col,
                    };
                }
                Some(TokenKind::Dot) => {
                    let (prop_line, prop_col) = self.wrap_position();
                    self.next();
                    let prop = self.expect_identifier()?;
                    expr = Expr {
                        kind: ExprKind::Property {
                            object: Box::new(expr),
                            name: prop,
                        },
                        line: expr_line,
                        col: expr_col,
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
                file: self.file.clone(),
                code: "E1001",
                message: format!("Expected {:?}, got {:?}", expected, tok.kind),
                line: tok.line(),
                column: tok.column(),
                snippet: Some(self.snippet(tok.line())),
                hint: Some("Check token".into()),
            }),
            None => {
                let (line, column) = self
                    .tokens
                    .get(self.position.saturating_sub(1))
                    .map(|t| (t.line(), t.column()))
                    .unwrap_or((0, 0));
                Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Expected {:?}, got EOF", expected),
                    line,
                    column,
                    snippet: Some(self.snippet(line)),
                    hint: Some("Unexpected end of input".into()),
                })
            }
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::Keyword(ref k2) = tok.kind {
                if k2 == kw {
                    return Ok(());
                }
            }
            return Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: format!("Expected keyword '{}``, got {:?}", kw, tok.kind),
                line: tok.line(),
                column: tok.column(),
                snippet: Some(self.snippet(tok.line())),
                hint: Some("Check your keyword spelling".into()),
            });
        }
        let (line, column) = self
            .tokens
            .get(self.position.saturating_sub(1))
            .map(|t| (t.line(), t.column()))
            .unwrap_or((0, 0));
        Err(PawError::Syntax {
            file: self.file.clone(),
            code: "E1001",
            message: format!("Expected keyword '{}`, got EOF", kw),
            line,
            column,
            snippet: Some(self.snippet(line)),
            hint: None,
        })
    }

    fn expect_identifier(&mut self) -> Result<String, PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::Identifier(n) = tok.kind {
                return Ok(n);
            }
            return Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: format!("Expected identifier, got {:?}", tok.kind),
                line: tok.line(),
                column: tok.column(),
                snippet: Some(self.snippet(tok.line())),
                hint: None,
            });
        }
        let (line, column) = self
            .tokens
            .get(self.position.saturating_sub(1))
            .map(|t| (t.line(), t.column()))
            .unwrap_or((0, 0));
        Err(PawError::Syntax {
            file: self.file.clone(),
            code: "E1001",
            message: "Expected identifier, got EOF".into(),
            line,
            column,
            snippet: Some(self.snippet(line)),
            hint: None,
        })
    }

    fn expect_string_literal(&mut self) -> Result<String, PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::StringLiteral(s) = tok.kind {
                return Ok(s);
            }
            return Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: format!("Expected string literal, got {:?}", tok.kind),
                line: tok.line(),
                column: tok.column(),
                snippet: Some(self.snippet(tok.line())),
                hint: None,
            });
        }
        let (line, column) = self
            .tokens
            .get(self.position.saturating_sub(1))
            .map(|t| (t.line(), t.column()))
            .unwrap_or((0, 0));
        Err(PawError::Syntax {
            file: self.file.clone(),
            code: "E1001",
            message: "Expected string literal, got EOF".into(),
            line,
            column,
            snippet: Some(self.snippet(line)),
            hint: None,
        })
    }
}
