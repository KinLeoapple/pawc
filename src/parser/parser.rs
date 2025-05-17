// src/parser.rs

use crate::ast::expr::{BinaryOp, Expr, ExprKind};
use crate::ast::method::{Method, MethodSig};
use crate::ast::param::Param;
use crate::ast::statement::{Statement, StatementKind};
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
        Self {
            tokens,
            position: 0,
            lines: source.lines().map(|l| l.to_string()).collect(),
            file: filename.into(),
        }
    }

    // --- Utility methods ---
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }
    fn peek_n_kind(&self, n: usize) -> Option<&TokenKind> {
        self.peek_n(n).map(|t| &t.kind)
    }
    /// 如果下一 token 是一个 Keyword，并且它的内容等于给定的字符串，就返回 true
    fn peek_keyword(&self, kw: &str) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Keyword(k)) if k == kw)
    }
    /// 看当前 token 是否和给定 kind 匹配（不移动 position）
    fn peek_token(&self, kind: TokenKind) -> bool {
        matches!(self.peek_kind(), Some(k) if *k == kind)
    }
    fn next(&mut self) -> Option<Token> {
        let t = self.peek().cloned();
        self.position += 1;
        t
    }

    fn wrap_position(&self) -> (usize, usize) {
        if let Some(tok) = self.peek() {
            (tok.line, tok.column)
        } else {
            (0, 0)
        }
    }
    fn snippet(&self, line: usize) -> Option<String> {
        self.lines.get(line.saturating_sub(1)).cloned().into()
    }

    fn expect_token(&mut self, expected: TokenKind) -> Result<(), PawError> {
        if let Some(tok) = self.next() {
            if tok.kind == expected {
                Ok(())
            } else {
                Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Expected {:?}, got {:?}", expected, tok.kind),
                    line: tok.line,
                    column: tok.column,
                    snippet: self.snippet(tok.line),
                    hint: Some("Check token".into()),
                })
            }
        } else {
            Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: "Unexpected EOF".into(),
                line: 0,
                column: 0,
                snippet: None,
                hint: None,
            })
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::Keyword(ref k) = tok.kind {
                if k == kw {
                    return Ok(());
                }
            }
            Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: format!("Expected keyword '{}', got {:?}", kw, tok.kind),
                line: tok.line,
                column: tok.column,
                snippet: self.snippet(tok.line),
                hint: Some("Check keyword".into()),
            })
        } else {
            Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: "Unexpected EOF".into(),
                line: 0,
                column: 0,
                snippet: None,
                hint: None,
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, PawError> {
        if let Some(tok) = self.next() {
            if let TokenKind::Identifier(name) = tok.kind {
                Ok(name)
            } else {
                Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Expected identifier, got {:?}", tok.kind),
                    line: tok.line,
                    column: tok.column,
                    snippet: self.snippet(tok.line),
                    hint: None,
                })
            }
        } else {
            Err(PawError::Syntax {
                file: self.file.clone(),
                code: "E1001",
                message: "Unexpected EOF".into(),
                line: 0,
                column: 0,
                snippet: None,
                hint: None,
            })
        }
    }

    // --- Top-level parse ---
    pub fn parse_program(&mut self) -> Result<Vec<Statement>, PawError> {
        let mut stmts = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Eof)) {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    pub fn parse_statement(&mut self) -> Result<Statement, PawError> {
        while matches!(
            self.peek_kind(),
            Some(TokenKind::Comment(_)) | Some(TokenKind::Error(_))
        ) {
            self.next();
        }
        let (line, col) = self.wrap_position();

        if self.peek_keyword("tail") {
            return self.parse_interface_decl();
        }
        if self.peek_keyword("record") {
            return self.parse_record_decl();
        }
        if self.peek_keyword("async") {
            return self.parse_fun_statement(true);
        }
        if self.peek_keyword("fun") {
            return self.parse_fun_statement(false);
        }
        if self.peek_keyword("let") {
            return self.parse_let_statement();
        }
        if let Some(TokenKind::Identifier(_)) = self.peek_kind() {
            if self.peek_n_kind(1) == Some(&TokenKind::Assign) {
                return self.parse_assign_statement();
            }
        }
        if self.peek_keyword("say") {
            return self.parse_say_statement();
        }
        if self.peek_keyword("ask") {
            return self.parse_ask_prompt_statement();
        }
        if self.peek_keyword("import") {
            return self.parse_import_statement();
        }
        if self.peek_keyword("return") {
            return self.parse_return_statement();
        }
        if self.peek_keyword("bark") {
            return self.parse_throw();
        }
        if self.peek_keyword("if") {
            return self.parse_if_statement();
        }
        if self.peek_keyword("loop") {
            return self.parse_loop_statement();
        }
        if self.peek_keyword("sniff") {
            return self.parse_try_catch_finally();
        }
        if self.peek_keyword("break") {
            self.next();
            return Ok(Statement::new(StatementKind::Break, line, col));
        }
        if self.peek_keyword("continue") {
            self.next();
            return Ok(Statement::new(StatementKind::Continue, line, col));
        }

        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Expr(expr), line, col))
    }

    // 以下方法补全于 `impl Parser` 中

    /// 解析 `fun` 或 `async fun` 声明
    fn parse_fun_statement(&mut self, is_async: bool) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        if is_async {
            self.expect_keyword("async")?;
        }
        self.expect_keyword("fun")?;

        // —— 可选接收者 (TypeName) ——
        let receiver = if self.peek_token(TokenKind::LParen) {
            self.next(); // consume '('
            let recv = self.expect_identifier()?;
            self.expect_token(TokenKind::RParen)?;
            Some(recv)
        } else {
            None
        };

        // 函数名
        let name = self.expect_identifier()?;

        // 参数列表
        self.expect_token(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect_token(TokenKind::RParen)?;

        // 可选返回类型
        let return_type = if self.peek_token(TokenKind::Colon) {
            self.next();
            Some(self.parse_type()?)
        } else {
            None
        };

        // 函数体
        let body = self.parse_block()?;

        Ok(Statement::new(
            StatementKind::FunDecl {
                receiver,
                name,
                params,
                is_async,
                return_type,
                body,
            },
            line,
            col,
        ))
    }

    /// 解析 `tail Name { … }`
    fn parse_interface_decl(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        // 消耗 `tail`
        self.expect_keyword("tail")?;
        // 接口名
        let name = self.expect_identifier()?;
        // 消耗 `{`
        self.expect_token(TokenKind::LBrace)?;
        // 按“方法签名, 方法签名, …”格式读取，并允许尾随逗号
        let mut methods = Vec::new();
        loop {
            // 1. 读取一个方法签名
            methods.push(self.parse_method_sig()?);

            // 2. 签名后必须有逗号，否则说明列表结束
            if self.peek_token(TokenKind::Comma) {
                self.next(); // 消耗逗号

                // 2a. 如果逗号后立刻是 '}'，也当结束
                if self.peek_token(TokenKind::RBrace) {
                    break;
                }
                // 否则继续读下一个方法
                continue;
            }
            // 没有逗号，结束循环
            break;
        }
        // 消耗 `}`
        self.expect_token(TokenKind::RBrace)?;
        Ok(Statement::new(
            StatementKind::InterfaceDecl { name, methods },
            line,
            col,
        ))
    }

    /// 解析接口里的一行方法签名，形如
    ///   async? foo(a: A, b: B): R;
    fn parse_method_sig(&mut self) -> Result<MethodSig, PawError> {
        let (line, col) = self.wrap_position();
        // 可选 async
        let is_async = if self.peek_keyword("async") {
            self.next();
            true
        } else {
            false
        };
        // 方法名
        let name = self.expect_identifier()?;
        // 参数列表
        self.expect_token(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect_token(TokenKind::RParen)?;
        // 可选返回类型
        let return_type = if self.peek_token(TokenKind::Colon) {
            self.next();
            Some(self.parse_type()?)
        } else {
            None
        };
        // 方法签名以分号结束
        self.expect_token(TokenKind::Comma)?;
        Ok(MethodSig {
            name,
            params,
            is_async,
            return_type,
        })
    }

    /// 解析 `record Name { field: Type, ... }` 声明
    fn parse_record_decl(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("record")?;
        let name = self.expect_identifier()?;
        // —— 可选的接口实现列表 ——
        let mut impls = Vec::new();
        if self.peek_token(TokenKind::LParen) {
            self.next(); // consume '('
            while !self.peek_token(TokenKind::RParen) {
                // 每个接口名
                let iface = self.expect_identifier()?;
                impls.push(iface);
                // 逗号分隔
                if self.peek_token(TokenKind::Comma) {
                    self.next();
                } else {
                    break;
                }
            }
            self.expect_token(TokenKind::RParen)?;
        }
        self.expect_token(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.peek_token(TokenKind::RBrace) {
            let field_name = self.expect_identifier()?;
            self.expect_token(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            fields.push(Param::new(field_name, ty, line, col));
            if self.peek_token(TokenKind::Comma) {
                self.next();
            }
        }
        self.expect_token(TokenKind::RBrace)?;
        Ok(Statement::new(
            StatementKind::RecordDecl { name, fields, impls },
            line,
            col,
        ))
    }

    /// 解析 `let` 或 `let ... <- ask "..."` 语句
    fn parse_let_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("let")?;
        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        // 支持 ask 初始化
        if self.peek_token(TokenKind::LeftArrow) {
            self.next();
            self.expect_keyword("ask")?;
            let prompt = match self.next() {
                Some(Token {
                    kind: TokenKind::StringLiteral(s),
                    ..
                }) => s,
                tok => {
                    return Err(PawError::Syntax {
                        file: self.file.clone(),
                        code: "E1001",
                        message: format!("Expected string literal after ask, got {:?}", tok),
                        line,
                        column: col,
                        snippet: None,
                        hint: None,
                    })
                }
            };
            return Ok(Statement::new(
                StatementKind::Ask { name, ty, prompt },
                line,
                col,
            ));
        }
        // 普通 let
        self.expect_token(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Let { name, ty, value },
            line,
            col,
        ))
    }

    /// 解析赋值语句 `x = expr`
    fn parse_assign_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        Ok(Statement::new(
            StatementKind::Assign { name, value },
            line,
            col,
        ))
    }

    /// 解析 `say expr` 语句
    fn parse_say_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("say")?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Say(expr), line, col))
    }

    /// 解析 `ask "..."` 提示语句
    fn parse_ask_prompt_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("ask")?;
        let prompt = match self.next() {
            Some(Token {
                kind: TokenKind::StringLiteral(s),
                ..
            }) => s,
            tok => {
                return Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Expected string literal in ask, got {:?}", tok),
                    line,
                    column: col,
                    snippet: None,
                    hint: None,
                })
            }
        };
        Ok(Statement::new(StatementKind::AskPrompt(prompt), line, col))
    }

    /// 解析 `import a.b.c as d` 语句
    fn parse_import_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("import")?;
        let mut module = Vec::new();
        loop {
            module.push(self.expect_identifier()?);
            if !self.peek_token(TokenKind::Dot) {
                break;
            }
            self.next();
        }
        let alias = if self.peek_keyword("as") {
            self.next();
            self.expect_identifier()?
        } else {
            module.last().cloned().unwrap()
        };
        Ok(Statement::new(
            StatementKind::Import { module, alias },
            line,
            col,
        ))
    }

    /// 解析 `return [expr]` 语句
    fn parse_return_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("return")?;
        let expr = if !self.peek_token(TokenKind::RBrace) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Statement::new(StatementKind::Return(expr), line, col))
    }

    /// 解析 `bark expr` 异常抛出
    fn parse_throw(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("bark")?;
        let expr = self.parse_expr()?;
        Ok(Statement::new(StatementKind::Throw(expr), line, col))
    }

    /// 解析 `if cond { ... } [else ...]`
    fn parse_if_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("if")?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        let else_branch = if self.peek_keyword("else") {
            self.next();
            if self.peek_keyword("if") {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(Box::new(Statement::new(
                    StatementKind::Block(self.parse_block()?),
                    line,
                    col,
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
            line,
            col,
        ))
    }

    /// 解析各种 loop：forever / range / while
    fn parse_loop_statement(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
        self.expect_keyword("loop")?;
        // forever
        if self.peek_keyword("forever") {
            self.next();
            let body = self.parse_block()?;
            return Ok(Statement::new(StatementKind::LoopForever(body), line, col));
        }
        // —— range-loop 或 array-loop 都是 “ident in …” 开头 ——
        if let Some(TokenKind::Identifier(var)) = self.peek_kind().cloned() {
            if matches!(self.peek_n_kind(1),Some(TokenKind::Keyword(k)) if k == "in") {
                let var = var.clone();
                self.next(); // 消耗变量名
                self.next(); // 消耗 `in`

                // 先 parse_expr 拿到第一个 Expr，既可能是 range 的 start，也可能是 array 本身
                let first = self.parse_expr()?;

                // 如果紧接着是 `..`，就是 range-loop
                if self.peek_token(TokenKind::Range) {
                    self.next(); // consume `..`
                    let end = self.parse_expr()?;
                    let body = self.parse_block()?;
                    return Ok(Statement::new(
                        StatementKind::LoopRange {
                            var,
                            start: first,
                            end,
                            body,
                        },
                        line,
                        col,
                    ));
                }

                // 否则就是 array-loop
                let array = first;
                let body = self.parse_block()?;
                return Ok(Statement::new(
                    StatementKind::LoopArray { var, array, body },
                    line,
                    col,
                ));
            }
        }
        // while
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Statement::new(
            StatementKind::LoopWhile { condition, body },
            line,
            col,
        ))
    }

    /// 解析 `sniff { ... } snatch(err) { ... } [lastly { ... }]`
    fn parse_try_catch_finally(&mut self) -> Result<Statement, PawError> {
        let (line, col) = self.wrap_position();
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
            line,
            col,
        ))
    }

    /// 一元操作和 await
    fn parse_unary_expr(&mut self) -> Result<Expr, PawError> {
        let (line, col) = self.wrap_position();

        // await e
        if self.peek_keyword("await") {
            self.next();
            let inner = self.parse_unary_expr()?;
            return Ok(Expr {
                kind: ExprKind::Await {
                    expr: Box::new(inner),
                },
                line,
                col,
            });
        }

        // -e
        if self.peek_token(TokenKind::Minus) {
            self.next();
            let e = self.parse_unary_expr()?;
            return Ok(Expr {
                kind: ExprKind::UnaryOp {
                    op: "-".into(),
                    expr: Box::new(e),
                },
                line,
                col,
            });
        }

        // !e
        if self.peek_token(TokenKind::Not) {
            self.next();
            let e = self.parse_unary_expr()?;
            return Ok(Expr {
                kind: ExprKind::UnaryOp {
                    op: "!".into(),
                    expr: Box::new(e),
                },
                line,
                col,
            });
        }

        // 否则就是 primary
        self.parse_primary()
    }

    /// 字面量、变量、调用、索引、RecordInit、属性访问…
    fn parse_primary(&mut self) -> Result<Expr, PawError> {
        let (line, col) = self.wrap_position();

        // nopaw 字面量
        if self.peek_keyword("nopaw") {
            self.next();
            return Ok(Expr {
                kind: ExprKind::LiteralNopaw,
                line,
                col,
            });
        }

        // 读一个 token，开始构造基础 expr
        let tok = self.next().ok_or_else(|| PawError::Syntax {
            file: self.file.clone(),
            code: "E1001",
            message: "Unexpected EOF in primary".into(),
            line,
            column: col,
            snippet: self.snippet(line),
            hint: Some("Expression expected".into()),
        })?;

        let mut expr = match tok.kind {
            TokenKind::IntLiteral(n) => Expr::new(ExprKind::LiteralInt(n), line, col),
            TokenKind::LongLiteral(n) => Expr::new(ExprKind::LiteralLong(n), line, col),
            TokenKind::FloatLiteral(f) => Expr::new(ExprKind::LiteralFloat(f), line, col),
            TokenKind::DoubleLiteral(f) => Expr::new(ExprKind::LiteralDouble(f), line, col),
            TokenKind::StringLiteral(s) => Expr::new(ExprKind::LiteralString(s), line, col),
            TokenKind::CharLiteral(c) => Expr::new(ExprKind::LiteralChar(c), line, col),
            TokenKind::BoolLiteral(b) => Expr::new(ExprKind::LiteralBool(b), line, col),

            TokenKind::Identifier(name) => {
                // 只有在紧跟 `{` 且 `{` 之后马上是字段名（Identifier）的情况下，
                // 我们才把它当成 record initializer；否则让后续的 parse_block 去消费这个 `{`
                if self.peek_token(TokenKind::LBrace)
                    && matches!(self.peek_n_kind(1), Some(TokenKind::Identifier(_)))
                {
                    // RecordInit
                    self.next(); // consume '{'
                    let mut fields = Vec::new();
                    while !self.peek_token(TokenKind::RBrace) {
                        let fname = self.expect_identifier()?;
                        self.expect_token(TokenKind::Colon)?;
                        let fexpr = self.parse_expr()?;
                        fields.push((fname, fexpr));
                        if self.peek_token(TokenKind::Comma) {
                            self.next();
                        }
                    }
                    self.expect_token(TokenKind::RBrace)?;
                    Expr {
                        kind: ExprKind::RecordInit { name, fields },
                        line,
                        col,
                    }
                } else {
                    Expr {
                        kind: ExprKind::Var(name),
                        line,
                        col,
                    }
                }
            }

            TokenKind::LParen => {
                let e = self.parse_expr()?;
                self.expect_token(TokenKind::RParen)?;
                return Ok(e); // 早返回，避免吞掉后缀
            }

            TokenKind::LBracket => {
                let mut elems = Vec::new();
                while !self.peek_token(TokenKind::RBracket) {
                    elems.push(self.parse_expr()?);
                    if self.peek_token(TokenKind::Comma) {
                        self.next();
                    }
                }
                self.expect_token(TokenKind::RBracket)?;
                Expr::new(ExprKind::ArrayLiteral(elems), line, col)
            }

            other => {
                return Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Unexpected token in primary: {:?}", other),
                    line,
                    column: col,
                    snippet: self.snippet(line),
                    hint: Some("Check expression syntax".into()),
                });
            }
        };

        // —— 后缀循环：函数调用 / 索引 / 方法调用 ——
        loop {
            match self.peek_kind() {
                // 普通函数调用 foo(...)
                Some(TokenKind::LParen) => {
                    let (cl, cc) = self.wrap_position();
                    self.next();
                    let mut args = Vec::new();
                    while !self.peek_token(TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek_token(TokenKind::Comma) {
                            self.next();
                        }
                    }
                    self.expect_token(TokenKind::RParen)?;
                    // 匹配 Var 或者 FieldAccess 都可以调用
                    expr = match expr.kind {
                        ExprKind::Var(n) => Expr {
                            kind: ExprKind::Call { name: n, args },
                            line,
                            col,
                        },
                        ExprKind::FieldAccess { expr: obj, field } => Expr {
                            kind: ExprKind::MethodCall {
                                receiver: obj,
                                method: self.parse_method(&*field),
                                args,
                            },
                            line,
                            col,
                        },
                        _ => {
                            return Err(PawError::Syntax {
                                file: self.file.clone(),
                                code: "E1001",
                                message: "Invalid call target".into(),
                                line: cl,
                                column: cc,
                                snippet: self.snippet(cl),
                                hint: None,
                            });
                        }
                    };
                }
                Some(TokenKind::LBracket) => {
                    self.next();
                    let idx = self.parse_expr()?;
                    self.expect_token(TokenKind::RBracket)?;
                    expr = Expr {
                        kind: ExprKind::Index {
                            array: Box::new(expr),
                            index: Box::new(idx),
                        },
                        line,
                        col,
                    };
                }
                Some(TokenKind::Dot) => {
                    // 如果后面不是调用，就当 FieldAccess（为了支持 record.field）
                    self.next();
                    let field = self.expect_identifier()?;
                    expr = Expr {
                        kind: ExprKind::FieldAccess {
                            expr: Box::new(expr),
                            field,
                        },
                        line,
                        col,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// parse `{ … }`，返回一组 Statement
    fn parse_block(&mut self) -> Result<Vec<Statement>, PawError> {
        // consume `{`
        self.expect_token(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.peek_token(TokenKind::RBrace) {
            stmts.push(self.parse_statement()?);
        }
        // consume `}`
        self.expect_token(TokenKind::RBrace)?;
        Ok(stmts)
    }

    /// parse 类型标注，比如 `Array<Int?>`
    fn parse_type(&mut self) -> Result<String, PawError> {
        let mut ty = match self.next() {
            Some(Token {
                kind: TokenKind::Type(s),
                ..
            })
            | Some(Token {
                kind: TokenKind::Identifier(s),
                ..
            }) => s,
            other => {
                return Err(PawError::Syntax {
                    file: self.file.clone(),
                    code: "E1001",
                    message: format!("Expected type, got {:?}", other),
                    line: 0,
                    column: 0,
                    snippet: None,
                    hint: None,
                })
            }
        };
        if self.peek_token(TokenKind::Lt) {
            self.next();
            let inner = self.parse_type()?;
            self.expect_token(TokenKind::Gt)?;
            ty = format!("{}<{}>", ty, inner);
        }
        if self.peek_token(TokenKind::Question) {
            self.next();
            ty.push('?');
        }
        Ok(ty)
    }

    /// parse 任意表达式的入口
    pub fn parse_expr(&mut self) -> Result<Expr, PawError> {
        // 从最低优先级开始
        self.parse_binary_expr(0)
    }

    /// 最低优先级入口：parse_expr 调用它
    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr, PawError> {
        let (line, col) = self.wrap_position();
        // 先读左边
        let mut left = self.parse_unary_expr()?;
        // 再循环读后续的二元操作
        loop {
            if let Some(TokenKind::Keyword(k)) = self.peek_kind() {
                if k == "as" && min_prec == 0 {
                    self.next(); // consume `as`
                    let ty = self.parse_type()?;
                    left = Expr {
                        kind: ExprKind::Cast {
                            expr: Box::new(left),
                            ty,
                        },
                        line,
                        col,
                    };
                    continue;
                }
            }

            let (prec, right_assoc, op) = match self.peek_kind() {
                Some(TokenKind::Plus) => (6, false, BinaryOp::Add),
                Some(TokenKind::Minus) => (6, false, BinaryOp::Sub),
                Some(TokenKind::Star) => (7, false, BinaryOp::Mul),
                Some(TokenKind::Slash) => (7, false, BinaryOp::Div),
                Some(TokenKind::Percent) => (7, false, BinaryOp::Mod),
                Some(TokenKind::EqEq) => (5, false, BinaryOp::EqEq),
                Some(TokenKind::NotEq) => (5, false, BinaryOp::NotEq),
                Some(TokenKind::Lt) => (5, false, BinaryOp::Lt),
                Some(TokenKind::Le) => (5, false, BinaryOp::Le),
                Some(TokenKind::Gt) => (5, false, BinaryOp::Gt),
                Some(TokenKind::Ge) => (5, false, BinaryOp::Ge),
                Some(TokenKind::AndAnd) => (4, false, BinaryOp::And),
                Some(TokenKind::OrOr) => (3, false, BinaryOp::Or),
                _ => break,
            };
            if prec < min_prec {
                break;
            }
            self.next(); // 吃掉运算符
            let next_min = if right_assoc { prec } else { prec + 1 };
            let right = self.parse_binary_expr(next_min)?;
            left = Expr {
                kind: ExprKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                line,
                col,
            };
        }
        Ok(left)
    }

    /// 参数列表：fun foo(a: Int, b: String?) { … }
    fn parse_params(&mut self) -> Result<Vec<Param>, PawError> {
        let mut params = Vec::new();
        while !self.peek_token(TokenKind::RParen) {
            // 名字
            let (p_line, p_col) = self.wrap_position();
            let name = self.expect_identifier()?;
            // 冒号
            self.expect_token(TokenKind::Colon)?;
            // 类型
            let ty = self.parse_type()?;
            params.push(Param::new(name, ty, p_line, p_col));
            // 如果逗号，继续
            if self.peek_token(TokenKind::Comma) {
                self.next();
            }
        }
        Ok(params)
    }

    fn parse_method(&self, name: &str) -> Method {
        match name {
            "trim" => Method::Trim,
            "to_uppercase" => Method::ToUppercase,
            "to_lowercase" => Method::ToLowercase,
            "length" => {
                // 根据前面解析的 receiver 类型决定是 String 还是 Array
                // 这里暂时都存 Length，让解释器再分支
                Method::Length
            }
            "starts_with" => Method::StartsWith,
            "ends_with" => Method::EndsWith,
            "contains" => Method::Contains,
            "push" => Method::Push,
            "pop" => Method::Pop,
            _ => Method::Other,
        }
    }
}
