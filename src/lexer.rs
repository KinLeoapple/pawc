// src/lexer.rs

use crate::token::Token;

pub struct Lexer {
    src: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            src: input.chars().collect(),
            pos: 0,
        }
    }

    /// 将整个输入拆成 Token 序列（不包含最终的 Eof）
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == Token::Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pos < self.src.len() {
            let c = self.src[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let c = match self.next_char() {
            Some(c) => c,
            None => return Token::Eof,
        };

        // 先处理 Range 操作符 `..`
        if c == '.' && self.peek_char() == Some('.') {
            self.next_char(); // consume second '.'
            return Token::Range;
        }

        if c == '[' {
            return Token::LBracket;
        }
        if c == ']' {
            return Token::RBracket;
        }

        match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '=' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::EqEq
                } else {
                    Token::Assign
                }
            }
            '<' => {
                if self.peek_char() == Some('-') {
                    self.next_char();
                    Token::LeftArrow
                } else if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if self.peek_char() == Some('&') {
                    self.next_char();   // 消费第二个 '&'
                    Token::AndAnd       // 返回 &&
                } else {
                    Token::Error("Unexpected character: &".to_string())
                }
            }
            '|' => {
                if self.peek_char() == Some('|') {
                    self.next_char();   // 消费第二个 '|'
                    Token::OrOr         // 返回 ||
                } else {
                    Token::Error("Unexpected character: |".to_string())
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.next_char();
                    Token::NotEq
                } else {
                    Token::Not
                }
            }
            '#' => {
                // 注释到行尾
                while let Some(nc) = self.peek_char() {
                    if nc == '\n' {
                        break;
                    }
                    self.next_char();
                }
                return self.next_token();
            }
            '"' => {
                // 字符串字面量
                let mut s = String::new();
                while let Some(nc) = self.next_char() {
                    if nc == '"' {
                        break;
                    }
                    s.push(nc);
                }
                Token::StringLiteral(s)
            }
            '\'' => {
                // 字符字面量
                let ch = self.next_char().unwrap_or('\0');
                self.next_char(); // skip closing '
                Token::CharLiteral(ch)
            }
            c if c.is_ascii_digit() => {
                // 数字字面量（支持 Int/Float/Long，同时处理 Range 情况）
                self.lex_number(c)
            }
            c if c.is_alphabetic() || c == '_' => {
                // 标识符、关键字 或 类型
                let mut ident = c.to_string();
                while let Some(nc) = self.peek_char() {
                    if nc.is_alphanumeric() || nc == '_' {
                        ident.push(self.next_char().unwrap());
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "true"  => Token::BoolLiteral(true),
                    "false" => Token::BoolLiteral(false),
                    
                    // 关键字
                    "fun" | "let" | "say" | "ask"
                    | "if"  | "else"| "loop"| "forever"
                    | "return" | "break" | "continue"
                    | "in" => Token::Keyword(ident),
                    // 类型
                    "Int"    | "Long"   | "Float"  | "Double"
                    | "String" | "Char"   | "Bool"   | "Any"
                    | "Void" | "Array" => Token::Type(ident),
                    _ => Token::Identifier(ident),
                }
            }
            _ => Token::Error(format!("Unexpected character: {}", c)),
        }
    }

    fn lex_number(&mut self, first_digit: char) -> Token {
        let mut number = first_digit.to_string();

        // 连续数字
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                number.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        // 紧接着是 '..'（Range 操作）时，直接返回 Int
        if self.peek_char() == Some('.') {
            if let Some('.') = self.src.get(self.pos + 1).copied() {
                return match number.parse::<i32>() {
                    Ok(n) => Token::IntLiteral(n),
                    Err(_) => Token::Error("Invalid int literal".into()),
                };
            }
        }

        // 浮点字面量
        if self.peek_char() == Some('.') {
            number.push(self.next_char().unwrap());
            while let Some(c2) = self.peek_char() {
                if c2.is_ascii_digit() {
                    number.push(self.next_char().unwrap());
                } else {
                    break;
                }
            }
            return match number.parse::<f64>() {
                Ok(f) => Token::FloatLiteral(f),
                Err(_) => Token::Error("Invalid float".into()),
            };
        }

        // Long 字面量后缀
        if let Some(c) = self.peek_char() {
            if c == 'L' || c == 'l' {
                self.next_char(); // consume 'L'
                return match number.parse::<i64>() {
                    Ok(n) => Token::LongLiteral(n),
                    Err(_) => Token::Error("Invalid long literal".into()),
                };
            }
        }

        // 普通 Int 字面量
        match number.parse::<i32>() {
            Ok(n) => Token::IntLiteral(n),
            Err(_) => Token::Error("Invalid int literal".into()),
        }
    }
}
