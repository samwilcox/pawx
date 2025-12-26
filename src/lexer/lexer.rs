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

use crate::lexer::token::{Token, TokenKind};
use crate::lexer::keywords::is_keyword;
use crate::span::Span;

pub struct Lexer {
    chars: Vec<char>,
    current: usize,
    line: usize,
    pub tokens: Vec<Token>,
}

impl Lexer {
    /// Creates a new PAWX lexer instance from raw source code.
    ///
    /// This initializes the internal scanning state and prepares the lexer
    /// to convert source text into a stream of lexical tokens.
    ///
    /// # Parameters
    /// - `source`: A UTF-8 encoded PAWX source string.
    ///
    /// # Returns
    /// A fully initialized `Lexer` with:
    /// - Cursor at position `0`
    /// - Line counter set to `1`
    /// - Empty token output buffer
    ///
    /// # Compiler Stage
    /// This is the **entry point for lexical analysis** in the PAWX compiler pipeline.
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    /// Performs complete lexical analysis over the entire source input.
    ///
    /// This method repeatedly scans individual tokens until the end of
    /// the source is reached, then appends a final `EOF` token.
    ///
    /// # Behavior
    /// - Ignores whitespace and comments
    /// - Emits structured `Token` objects
    /// - Guarantees a terminating `TokenKind::Eof` marker
    ///
    /// # Output
    /// Results are written into `self.tokens`.
    ///
    /// # Safety
    /// This function **must be called exactly once** per lexer instance.
    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.scan_token();
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".to_string(),
            span: Span {
                line: self.line,
                column: 0,
            },
        });
    }

    /// Scans and emits a single token from the source stream.
    ///
    /// This method:
    /// - Advances the character cursor by one
    /// - Classifies the character
    /// - Routes to specialized parsers for:
    ///   - Strings
    ///   - Numbers
    ///   - Identifiers
    ///   - Keywords
    ///   - Operators
    ///   - Symbols
    ///
    /// # Behavior
    /// - Handles `//` and `/* */` comments
    /// - Supports multi-character operators (`==`, `===`, `!=`, `->`, `++`, `--`)
    /// - Updates line counter automatically
    ///
    /// # Panics
    /// May panic on unterminated strings or block comments.
    fn scan_token(&mut self) {
        let ch = self.advance();

        match ch {
            // Whitespace
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,

            // Single-line or block comment
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
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                }
            }

            // SAFE ARROW TOKENIZATION
            '-' => {
                if self.peek() == '>' {
                    self.advance();
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "->".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                } else if self.peek() == '-' {
                    self.advance();
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "--".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                } else {
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "-".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                }
            }

            '+' => {
                if self.peek() == '+' {
                    self.advance();
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "++".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                } else {
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "+".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                }
            }

            // Multi-char operators: !, =, <, >
            '!' | '=' | '<' | '>' => {
                let mut lex = ch.to_string();

                if self.peek() == '=' {
                    lex.push(self.advance());

                    // Support === and !==
                    if (lex == "==" || lex == "!=") && self.peek() == '=' {
                        lex.push(self.advance());
                    }
                }

                self.tokens.push(Token {
                    kind: TokenKind::Symbol,
                    lexeme: lex,
                    span: Span {
                        line: self.line,
                        column: 0
                    },
                });
            }

            // Strings
            '"' | '\'' => self.string_with_delimiter(ch),

            // Numbers
            '0'..='9' => self.number(),

            // Identifiers / keywords
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(ch),

            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "&&".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0,
                        },
                    });
                } else {
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "&".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                }
            }

            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "||".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                } else {
                    self.tokens.push(Token {
                        kind: TokenKind::Symbol,
                        lexeme: "|".to_string(),
                        span: Span {
                            line: self.line,
                            column: 0
                        },
                    });
                }
            }

            // Everything else = single-char symbol
            _ => {
                self.tokens.push(Token {
                    kind: TokenKind::Symbol,
                    lexeme: ch.to_string(),
                    span: Span {
                        line: self.line,
                        column: 0
                    },
                });
            }
        }
    }

    /// Parses a string literal using the provided quote delimiter.
    ///
    /// # Parameters
    /// - `delimiter`: Either `'` or `"` depending on opening quote.
    ///
    /// # Behavior
    /// - Consumes all characters until matching closing delimiter
    /// - Tracks line numbers for multi-line strings
    /// - Emits a `TokenKind::String` token
    ///
    /// # Panics
    /// If the string is not properly terminated before EOF.
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
            span: Span {
                line: self.line,
                column: 0
            },
        });
    }

    /// Parses an identifier or keyword token.
    ///
    /// # Behavior
    /// - Reads all alphanumeric and underscore characters
    /// - Classifies the resulting lexeme as:
    ///   - `TokenKind::Keyword` if reserved
    ///   - `TokenKind::Identifier` otherwise
    ///
    /// # Language Rules
    /// - Keywords are defined in `keywords.rs`
    /// - Case-sensitive
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
            span: Span {
                line: self.line,
                column: 0
            },
        });
    }

    /// Parses an integer or floating-point numeric literal.
    ///
    /// # Behavior
    /// - Consumes consecutive numeric characters
    /// - Supports decimal floating-point notation
    /// - Emits a `TokenKind::Number` token
    ///
    /// # Examples
    /// - `42`
    /// - `3.1415`
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
            span: Span {
                line: self.line,
                column: 0
            },
        });
    }

    /// Conditionally matches the next character without emitting a token.
    ///
    /// # Parameters
    /// - `expected`: The character to match
    ///
    /// # Returns
    /// - `true` if the next character matched and was consumed
    /// - `false` otherwise
    ///
    /// # Usage
    /// Used for multi-character operators such as:
    /// - `==`, `!=`, `++`, `--`, `->`
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

    /// Skips a block comment delimited by `/* ... */`.
    ///
    /// # Behavior
    /// - Consumes characters until closing delimiter is found
    /// - Tracks line numbers correctly
    ///
    /// # Panics
    /// If the block comment is not terminated before EOF.
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

    /// Advances the lexer cursor by one character.
    ///
    /// # Returns
    /// The character that was consumed.
    ///
    /// # Safety
    /// Caller must ensure EOF has not been reached.
    fn advance(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        ch
    }

    /// Returns the current character without consuming it.
    ///
    /// # Returns
    /// - The current character
    /// - `'\0'` if the end of file has been reached
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    /// Returns the next character after the current one without consuming it.
    ///
    /// # Returns
    /// - The next character
    /// - `'\0'` if the lookahead is invalid
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    /// Determines whether the lexer has reached the end of input.
    ///
    /// # Returns
    /// - `true` if all characters have been consumed
    /// - `false` otherwise
    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}