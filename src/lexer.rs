/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * Github:   https://github.com/samwilcox/pawx
 * 
 * License:
 * This file is part of the PAWX programming language project.
 * 
 * PAWX is dual-licensed under the terms of:
 *   - The MIT license
 *   - The Apache License, Version 2.0
 * 
 * You may choose either license to govern your use of this software.
 * Full license text available at:
 *    https://license.pawx-lang.com
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under these licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * 
 * ==========================================================================
 */

#![allow(dead_code, unused_variables, unused_imports)]

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number,
    String,
    Identifier,
    Keyword,
    Symbol,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    lexer.scan_tokens();
    lexer.tokens
}

struct Lexer {
    chars: Vec<char>,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.scan_token();
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".to_string(),
            line: self.line,
        });
    }

    fn scan_token(&mut self) {
        let ch = self.advance();

        match ch {
            // Whitespace
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,

            // Single-line comment //
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    self.block_comment();
                } else {
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "/".to_string(),
                        line: self.line,
                    });
                }
            }

            // Multi-char operators: !, =, <, >
            '!' | '=' | '<' | '>' => {
                let mut lex = ch.to_string();

                if self.peek() == '=' {
                    lex.push(self.advance());

                    // ✅ Support === (STRICT EQUALITY)
                    if lex == "==" && self.peek() == '=' {
                        lex.push(self.advance());
                    }
                }

                self.tokens.push(Token {
                    kind: TokenKind::Symbol,
                    lexeme: lex,
                    line: self.line,
                });
            }

            // Strings
            // Strings (single or double quoted)
            '"' | '\'' => self.string_with_delimiter(ch),

            // Numbers
            '0'..='9' => self.number(),

            // Identifiers / keywords
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(ch),

            // Everything else = single-char symbol
            _ => {
                self.tokens.push(Token {
                    kind: TokenKind::Symbol,
                    lexeme: ch.to_string(),
                    line: self.line,
                });
            }
        }
    }

    fn string_with_delimiter(&mut self, delimiter: char) {
        let start = self.current;

        while self.peek() != delimiter && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            panic!("Unterminated string at line {}", self.line);
        }

        self.advance(); // closing quote

        let value: String = self.chars[start..self.current - 1].iter().collect();

        self.tokens.push(Token {
            kind: TokenKind::String,
            lexeme: value,
            line: self.line,
        });
    }

    fn identifier(&mut self, first: char) {
        let start = self.current - 1;

        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.chars[start..self.current].iter().collect();

        let kind = if is_keyword(&text) {
            TokenKind::Keyword
        } else {
            TokenKind::Identifier
        };

        self.tokens.push(Token {
            kind,
            lexeme: text,
            line: self.line,
        });
    }

    fn number(&mut self) {
        let start = self.current - 1;

        while self.peek().is_numeric() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_numeric() {
            self.advance(); // consume '.'
            while self.peek().is_numeric() {
                self.advance();
            }
        }

        let value: String = self.chars[start..self.current].iter().collect();

        self.tokens.push(Token {
            kind: TokenKind::Number,
            lexeme: value,
            line: self.line,
        });
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.chars[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn block_comment(&mut self) {
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                return;
            }

            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        panic!("Unterminated block comment at line {}", self.line);
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "den" |
        "lair" |
        "pride" |
        "purr" |
        "zoom" |      // async cat keyword
        "snuggle" |
        "return" |
        "if" |
        "else" |
        "while" |
        "true" |
        "false" |
        "null" |
        "nap" |
        "try" |
        "catch" |
        "finally" |
        "throw" |
        "new" |
        "clowder" |
        "instinct" |
        "inherits" |
        "practices" |
        "static" |
        "get" |
        "set" |
        "this" |
        "exports" |
        "tap"
    )
}