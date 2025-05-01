// src/parser.rs

use crate::ast::{BinaryOp, Expr, Param, Statement, StatementKind};
use crate::error::PawError;
use crate::token::Token;

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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Helper: look ahead n tokens without consuming
    fn peek_n(&self, n: usize) -> Option<Token> {
        self.tokens.get(self.position + n).cloned()
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.position).cloned();
        self.position += 1;
        t
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, PawError> {
        let mut out = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::Eof {
                break;
            }
            out.push(self.parse_statement()?);
        }
        Ok(out)
    }

    pub fn parse_statement(&mut self) -> Result<Statement, PawError> {
        if let Some(Token::Identifier(_)) = self.peek() {
            if let Some(Token::Assign) = self.peek_n(1) {
                // 真的就是 x = ...
                let name = self.expect_identifier()?; // consume IDENT
                self.expect_token(Token::Assign)?; // consume '='
                let value = self.parse_expr()?; // parse right-hand expr
                return Ok(Statement::new(StatementKind::Assign { name, value }));
            }
        }

        match self.peek() {
            Some(Token::Keyword(kw)) => match kw.as_str() {
                "let" => self.parse_let_statement(),
                "say" => self.parse_say_statement(),
                "ask" => self.parse_ask_prompt_statement(),
                "return" => self.parse_return_statement(),
                "break" => {
                    self.next();
                    Ok(Statement::new(StatementKind::Break))
                }
                "continue" => {
                    self.next();
                    Ok(Statement::new(StatementKind::Continue))
                }
                "if" => self.parse_if_statement(),
                "loop" => self.parse_loop_statement(),
                "fun" => self.parse_fun_statement(),
                _ => self.parse_expr_statement(),
            },
            Some(Token::LBrace) => {
                let blk = self.parse_block_statement()?;
                Ok(blk)
            }
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, PawError> {
        // we already know the next token is Keyword("let")
        self.expect_keyword("let")?;

        // 1) consume the variable name and its declared type
        let name = self.expect_identifier()?;
        self.expect_token(Token::Colon)?;
        let ty = self.expect_type()?;

        // 2) if the next token is `<-`, it's an `ask` initialization:
        if self.peek() == Some(&Token::LeftArrow) {
            self.next(); // consume `<-`
            self.expect_keyword("ask")?; // consume `ask`
            let prompt = self.expect_string_literal()?; // consume the string
            return Ok(Statement::new(StatementKind::Ask { name, ty, prompt }));
        }

        // 3) otherwise it must be a normal `=` assignment
        self.expect_token(Token::Assign)?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Let {
            name,
            ty,
            value: expr,
        }))
    }

    fn parse_say_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("say")?;
        let e = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Say(e)))
    }

    fn parse_ask_prompt_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("ask")?;
        let p = self.expect_string_literal()?;
        Ok(Statement::new(StatementKind::AskPrompt(p)))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("return")?;
        if matches!(self.peek(), Some(Token::Eof) | Some(Token::RBrace)) {
            Ok(Statement::new(StatementKind::Return(None)))
        } else {
            let e = self.parse_expr()?;
            Ok(Statement::new(StatementKind::Return(Some(e))))
        }
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, PawError> {
        let e = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Expr(e)))
    }

    pub fn parse_if_statement(&mut self) -> Result<Statement, PawError> {
        // 1) consume the `if` keyword
        self.expect_keyword("if")?;

        // 2) parse the full expression (this will consume `a`, `==`, and `0`)
        let condition = self.parse_expr()?;

        // 3) parse the `{ … }` block
        let body = self.parse_block()?;
        //    parse_block itself does expect_token(LBrace), loop stmts, expect_token(RBrace)

        // 4) optional else / else if
        let else_branch = if self.peek_keyword("else") {
            self.next(); // consume `else`
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
        // 1) consume the `loop` keyword
        self.expect_keyword("loop")?;

        // 2) special case: `loop forever { … }`
        if self.peek_keyword("forever") {
            self.next(); // consume `forever`
            let body = self.parse_block()?;
            return Ok(Statement::new(StatementKind::LoopForever(body)));
        }

        // 3) maybe it's a range loop? look ahead without consuming:
        if let (Some(Token::Identifier(var)), Some(Token::Keyword(ref kw))) =
            (self.peek(), self.peek_n(1))
        {
            if kw == "in" {
                // yes!  consume the `i` and the `in`
                let var = var.clone();
                self.next(); // consume Identifier(var)
                self.next(); // consume Keyword("in")

                // now parse start..end
                let start = self.parse_expr()?;
                self.expect_token(Token::Range)?;
                let end = self.parse_expr()?;
                let body = self.parse_block()?;

                return Ok(Statement::new(StatementKind::LoopRange {
                    var,
                    start,
                    end,
                    body,
                }));
            }
        }

        // 4) fallback: a simple while‐style loop: `loop <expr> { … }`
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Statement::new(StatementKind::LoopWhile { condition, body }))
    }

    fn parse_fun_statement(&mut self) -> Result<Statement, PawError> {
        self.expect_keyword("fun")?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.peek(), Some(Token::RParen)) {
            let pn = self.expect_identifier()?;
            self.expect_token(Token::Colon)?;
            let pt = self.expect_type()?;
            params.push(Param { name: pn, ty: pt });
            if matches!(self.peek(), Some(Token::Comma)) {
                self.next();
            } else {
                break;
            }
        }
        self.expect_token(Token::RParen)?;
        let ret = if matches!(self.peek(), Some(Token::Colon)) {
            self.next();
            Some(self.expect_type()?)
        } else {
            None
        };
        let b = self.parse_block()?;
        Ok(Statement::new(StatementKind::FunDecl {
            name,
            params,
            return_type: ret,
            body: b,
        }))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, PawError> {
        let stmts = self.parse_block()?;
        Ok(Statement::new(StatementKind::Block(stmts)))
    }

    /// parses `{ … }`, including the braces
    fn parse_block(&mut self) -> Result<Vec<Statement>, PawError> {
        self.expect_token(Token::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            stmts.push(self.parse_statement()?);
        }
        self.expect_token(Token::RBrace)?;
        Ok(stmts)
    }

    pub fn parse_expr(&mut self) -> Result<Expr, PawError> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr, PawError> {
        let mut left = self.parse_unary_expr()?;

        while let Some(tok) = self.peek() {
            // assign precedences as you see fit
            let (prec, op) = match tok {
                // arithmetic
                Token::Plus => (6, BinaryOp::Add),
                Token::Minus => (6, BinaryOp::Sub),
                Token::Star => (7, BinaryOp::Mul),
                Token::Slash => (7, BinaryOp::Div),
                Token::Percent => (7, BinaryOp::Mod),

                // comparisons
                Token::EqEq => (5, BinaryOp::EqEq),
                Token::NotEq => (5, BinaryOp::NotEq),
                Token::Lt => (5, BinaryOp::Lt),
                Token::Le => (5, BinaryOp::Le),
                Token::Gt => (5, BinaryOp::Gt),
                Token::Ge => (5, BinaryOp::Ge),

                // boolean
                Token::AndAnd => (4, BinaryOp::And),
                Token::OrOr => (3, BinaryOp::Or),

                _ => break,
            };

            // standard precedence climbing
            if prec < min_prec {
                break;
            }
            self.next(); // consume the operator token
            let right = self.parse_binary_expr(prec + 1)?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, PawError> {
        if let Some(Token::Minus) = self.peek() {
            self.next();
            let e = self.parse_unary_expr()?;
            return Ok(Expr::UnaryOp {
                op: "-".into(),
                expr: Box::new(e),
            });
        }
        if let Some(Token::Not) = self.peek() {
            self.next();
            let e = self.parse_unary_expr()?;
            return Ok(Expr::UnaryOp {
                op: "!".into(),
                expr: Box::new(e),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, PawError> {
        let mut expr = match self.next() {
            Some(Token::IntLiteral(n)) => Expr::LiteralInt(n),
            Some(Token::LongLiteral(n)) => Expr::LiteralLong(n),
            Some(Token::FloatLiteral(f)) => Expr::LiteralFloat(f),
            Some(Token::BoolLiteral(b))   => Expr::LiteralBool(b),
            Some(Token::StringLiteral(s)) => Expr::LiteralString(s),
            Some(Token::CharLiteral(c)) => Expr::LiteralChar(c),
            Some(Token::Identifier(n)) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    // call
                    self.next();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Some(Token::RParen)) {
                        args.push(self.parse_expr()?);
                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.next();
                        } else {
                            break;
                        }
                    }
                    self.expect_token(Token::RParen)?;
                    Expr::Call { name: n, args }
                } else {
                    Expr::Var(n)
                }
            }
            Some(Token::LParen) => {
                let e = self.parse_expr()?;
                self.expect_token(Token::RParen)?;
                e
            }
            Some(Token::LBracket) => {
                let mut elems = Vec::new();
                while !matches!(self.peek(), Some(Token::RBracket)) {
                    elems.push(self.parse_expr()?);
                    if matches!(self.peek(), Some(Token::Comma)) {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.expect_token(Token::RBracket)?;
                Expr::ArrayLiteral(elems)
            }
            other => {
                return Err(PawError::Syntax {
                    message: format!("Unexpected {:?} in primary", other),
                });
            }
        };

        loop {
            expr = match self.peek() {
                Some(Token::LBracket) => {
                    self.next();
                    let idx = self.parse_expr()?;
                    self.expect_token(Token::RBracket)?;
                    Expr::Index {
                        array: Box::new(expr),
                        index: Box::new(idx),
                    }
                }
                Some(Token::Dot) => {
                    self.next();
                    let prop = self.expect_identifier()?;
                    Expr::Property {
                        object: Box::new(expr),
                        name: prop,
                    }
                }
                _ => break Ok(expr),
            }
        }
    }

    // --- helpers ---

    fn peek_keyword(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Token::Keyword(k)) if k == kw)
    }

    fn expect_token(&mut self, t: Token) -> Result<(), PawError> {
        match self.next() {
            Some(tok) if tok == t => Ok(()),
            Some(tok) => Err(PawError::Syntax {
                message: format!("Expected {:?}, got {:?}", t, tok),
            }),
            None => Err(PawError::Syntax {
                message: format!("Expected {:?}, got EOF", t),
            }),
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), PawError> {
        match self.next() {
            Some(Token::Keyword(k)) if k == kw => Ok(()),
            other => Err(PawError::Syntax {
                message: format!("Expected keyword '{}', got {:?}", kw, other),
            }),
        }
    }

    fn expect_identifier(&mut self) -> Result<String, PawError> {
        match self.next() {
            Some(Token::Identifier(n)) => Ok(n),
            other => Err(PawError::Syntax {
                message: format!("Expected identifier, got {:?}", other),
            }),
        }
    }

    fn expect_type(&mut self) -> Result<String, PawError> {
        let base = match self.next() {
            Some(Token::Type(n)) => n,
            other => {
                return Err(PawError::Syntax {
                    message: format!("Expected type, got {:?}", other),
                })
            }
        };
        if matches!(self.peek(), Some(Token::LBracket)) {
            self.next();
            self.expect_token(Token::RBracket)?;
            Ok(format!("{}[]", base))
        } else {
            Ok(base)
        }
    }

    fn expect_string_literal(&mut self) -> Result<String, PawError> {
        match self.next() {
            Some(Token::StringLiteral(s)) => Ok(s),
            other => Err(PawError::Syntax {
                message: format!("Expected string literal, got {:?}", other),
            }),
        }
    }
}
