// File: src/lexer.rs

use crate::lexer::token::{Token, TokenKind};

/// A simple character-level lexer that produces spanned Tokens
pub struct Lexer {
    src: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            src: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Turn the entire input into a Vec<Token> (including final Eof)
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = matches!(tok.kind, TokenKind::Eof);
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Consume the next character, updating line/column
    fn next_char(&mut self) -> Option<char> {
        if let Some(&c) = self.src.get(self.pos) {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    /// Peek at the next character without consuming
    fn peek_char(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    /// Skip whitespace (spaces, tabs, newlines)
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Produce the next Token, with span information
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let start_line = self.line;
        let start_column = self.column;

        let c = match self.next_char() {
            Some(ch) => ch,
            None => return Token::new(TokenKind::Eof, start_line, start_column),
        };

        // Range operator `..`
        if c == '.' && self.peek_char() == Some('.') {
            self.next_char();
            return Token::new(TokenKind::Range, start_line, start_column);
        }

        // Single-character tokens
        match c {
            '[' => Token::new(TokenKind::LBracket, start_line, start_column),
            ']' => Token::new(TokenKind::RBracket, start_line, start_column),
            '+' => Token::new(TokenKind::Plus, start_line, start_column),
            '-' => Token::new(TokenKind::Minus, start_line, start_column),
            '*' => Token::new(TokenKind::Star, start_line, start_column),
            '/' => Token::new(TokenKind::Slash, start_line, start_column),
            '%' => Token::new(TokenKind::Percent, start_line, start_column),
            '(' => Token::new(TokenKind::LParen, start_line, start_column),
            ')' => Token::new(TokenKind::RParen, start_line, start_column),
            '{' => Token::new(TokenKind::LBrace, start_line, start_column),
            '}' => Token::new(TokenKind::RBrace, start_line, start_column),
            ',' => Token::new(TokenKind::Comma, start_line, start_column),
            ':' => Token::new(TokenKind::Colon, start_line, start_column),
            '.' => Token::new(TokenKind::Dot, start_line, start_column),
            '?' => Token::new(TokenKind::Question, start_line, start_column),
            '=' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::new(TokenKind::EqEq, start_line, start_column)
                } else {
                    Token::new(TokenKind::Assign, start_line, start_column)
                }
            }
            '<' => {
                if self.peek_char() == Some('-') {
                    self.next_char();
                    Token::new(TokenKind::LeftArrow, start_line, start_column)
                } else if self.peek_char() == Some('=') {
                    self.next_char();
                    return Token::new(TokenKind::Le, start_line, start_column);
                } else {
                    return Token::new(TokenKind::Lt, start_line, start_column);
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::new(TokenKind::Ge, start_line, start_column)
                } else {
                    Token::new(TokenKind::Gt, start_line, start_column)
                }
            }
            '&' => {
                if self.peek_char() == Some('&') {
                    self.next_char();
                    Token::new(TokenKind::AndAnd, start_line, start_column)
                } else {
                    Token::new(
                        TokenKind::Error("Unexpected character: &".into()),
                        start_line,
                        start_column,
                    )
                }
            }
            '|' => {
                if self.peek_char() == Some('|') {
                    self.next_char();
                    Token::new(TokenKind::OrOr, start_line, start_column)
                } else {
                    Token::new(
                        TokenKind::Error("Unexpected character: |".into()),
                        start_line,
                        start_column,
                    )
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::new(TokenKind::NotEq, start_line, start_column)
                } else {
                    Token::new(TokenKind::Not, start_line, start_column)
                }
            }
            '#' => {
                // comment until end of line
                while let Some(nc) = self.peek_char() {
                    if nc == '\n' {
                        break;
                    }
                    self.next_char();
                }
                self.next_token()
            }
            '"' => {
                let mut s = String::new();
                while let Some(nc) = self.next_char() {
                    if nc == '"' {
                        break;
                    }
                    if nc == '\\' {
                        if let Some(esc) = self.next_char() {
                            match esc {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                'r' => s.push('\r'),
                                '\\' => s.push('\\'),
                                '"' => s.push('"'),
                                other => {
                                    s.push('\\');
                                    s.push(other);
                                }
                            }
                            continue;
                        } else {
                            s.push('\\');
                            break;
                        }
                    }
                    s.push(nc);
                }
                Token::new(TokenKind::StringLiteral(s), start_line, start_column)
            }
            '\'' => {
                let ch = self.next_char().unwrap_or('\0');
                self.next_char(); // skip closing '
                Token::new(TokenKind::CharLiteral(ch), start_line, start_column)
            }
            c if c.is_ascii_digit() => self.lex_number(c, start_line, start_column),
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = c.to_string();
                while let Some(nc) = self.peek_char() {
                    if nc.is_alphanumeric() || nc == '_' {
                        ident.push(self.next_char().unwrap());
                    } else {
                        break;
                    }
                }
                let kind = match ident.as_str() {
                    "true" => TokenKind::BoolLiteral(true),
                    "false" => TokenKind::BoolLiteral(false),
                    // keywords
                    kw @ "import"
                    | kw @ "fun"
                    | kw @ "let"
                    | kw @ "say"
                    | kw @ "ask"
                    | kw @ "as"
                    | kw @ "if"
                    | kw @ "else"
                    | kw @ "loop"
                    | kw @ "forever"
                    | kw @ "return"
                    | kw @ "break"
                    | kw @ "continue"
                    | kw @ "in"
                    | kw @ "bark"
                    | kw @ "sniff"
                    | kw @ "snatch"
                    | kw @ "lastly"
                    | kw @ "nopaw" => TokenKind::Keyword(kw.into()),
                    // types
                    ty @ "Int"
                    | ty @ "Long"
                    | ty @ "Float"
                    | ty @ "Double"
                    | ty @ "String"
                    | ty @ "Char"
                    | ty @ "Bool"
                    | ty @ "Any"
                    | ty @ "Void"
                    | ty @ "Array" => TokenKind::Type(ty.into()),
                    _ => TokenKind::Identifier(ident.clone()),
                };
                Token::new(kind, start_line, start_column)
            }
            _ => Token::new(
                TokenKind::Error(format!("Unexpected character: {}", c)),
                start_line,
                start_column,
            ),
        }
    }

    /// Lex a number literal (Int, Float, or Long)
    fn lex_number(&mut self, first: char, line: usize, col: usize) -> Token {
        let mut num = first.to_string();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                num.push(self.next_char().unwrap());
            } else {
                break;
            }
        }
        // range lookahead
        if self.peek_char() == Some('.') {
            if let Some('.') = self.src.get(self.pos + 1).copied() {
                return match num.parse::<i32>() {
                    Ok(n) => Token::new(TokenKind::IntLiteral(n), line, col),
                    Err(_) => Token::new(TokenKind::Error("Invalid int literal".into()), line, col),
                };
            }
        }
        // float literal
        if self.peek_char() == Some('.') {
            num.push(self.next_char().unwrap());
            while let Some(c2) = self.peek_char() {
                if c2.is_ascii_digit() {
                    num.push(self.next_char().unwrap());
                } else {
                    break;
                }
            }
            return match num.parse::<f64>() {
                Ok(f) => Token::new(TokenKind::FloatLiteral(f), line, col),
                Err(_) => Token::new(TokenKind::Error("Invalid float".into()), line, col),
            };
        }
        // long suffix
        if let Some(c) = self.peek_char() {
            if c == 'L' || c == 'l' {
                self.next_char();
                return match num.parse::<i64>() {
                    Ok(n) => Token::new(TokenKind::LongLiteral(n), line, col),
                    Err(_) => {
                        Token::new(TokenKind::Error("Invalid long literal".into()), line, col)
                    }
                };
            }
        }
        // int fallback
        match num.parse::<i32>() {
            Ok(n) => Token::new(TokenKind::IntLiteral(n), line, col),
            Err(_) => Token::new(TokenKind::Error("Invalid int literal".into()), line, col),
        }
    }
}
