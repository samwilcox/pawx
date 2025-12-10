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

use crate::parser::parser::Parser;
use crate::lexer::token::{Token, TokenKind};

impl Parser {
    /// Matches a PAWX keyword and consumes it if present.
    pub fn match_keyword(&mut self, kw: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.tokens[self.current].kind == TokenKind::Keyword
            && self.tokens[self.current].lexeme == kw
        {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Checks for a keyword without consuming it.
    pub fn check_keyword(&self, kw: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current].kind == TokenKind::Keyword
            && self.tokens[self.current].lexeme == kw
    }

    /// Checks the next token's lexeme without advancing.
    pub fn peek_is(&self, ch: &str) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[self.current + 1].lexeme == ch
    }

    /// Matches a symbol and consumes it.
    pub(crate) fn match_symbol(&mut self, ch: char) -> bool {
        if self.check_symbol(ch) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Checks if the current token matches a symbol.
    pub fn check_symbol(&self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current].kind == TokenKind::Symbol
            && self.tokens[self.current].lexeme == ch.to_string()
    }

    /// Consumes a required symbol or panics.
    pub fn consume_symbol(&mut self, ch: char) {
        if self.check_symbol(ch) {
            self.advance();
        } else {
            panic!("Expected '{}'", ch);
        }
    }

    /// Consumes and returns an identifier or panics.
    pub fn consume_identifier(&mut self) -> String {
        let token = self.advance();
        if token.kind != TokenKind::Identifier {
            panic!("Expected identifier");
        }
        token.lexeme
    }

    /// Advances one token forward.
    pub fn advance(&mut self) -> Token {
        let t = self.tokens[self.current].clone();
        self.current += 1;
        t
    }

    /// Returns the previously consumed token.
    pub fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Returns true if the parser is at EOF.
    pub fn is_at_end(&self) -> bool {
        self.tokens[self.current].kind == TokenKind::Eof
    }

    /// Attempts to match a symbolic operator and consume it if present.
    ///
    /// This function is used for parsing binary and unary operators such as:
    /// - `+`, `-`, `*`, `/`, `%`
    /// - `==`, `!=`, `===`, `!==`
    /// - `<`, `<=`, `>`, `>=`
    ///
    /// If the current token:
    ///   - Is of kind `TokenKind::Symbol`
    ///   - And its lexeme exactly matches `op`
    ///
    /// Then the token is consumed and `true` is returned.  
    /// Otherwise, the token stream is left untouched and `false` is returned.
    ///
    /// # Parameters
    /// - `op`: The operator string to match against (e.g. `"+"`, `"=="`)
    ///
    /// # Returns
    /// - `true` if the operator matched and was consumed
    /// - `false` otherwise
    ///
    /// # Safety
    /// This method never panics and is safe to call at end-of-input.
    pub fn match_operator(&mut self, op: &str) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.tokens[self.current].kind == TokenKind::Symbol
            && self.tokens[self.current].lexeme == op
        {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Attempts to match and consume the PAWX arrow operator (`->`).
    ///
    /// This function is used exclusively for parsing:
    /// - Function declarations
    /// - Lambda expressions
    /// - Getters & setters
    /// - Class and interface method signatures
    ///
    /// If the current token is the arrow symbol (`"->"`), it:
    /// - Consumes **exactly one token**
    /// - Returns `true`
    ///
    /// Otherwise:
    /// - The parser state remains unchanged
    /// - `false` is returned
    ///
    /// # Returns
    /// - `true` if the arrow was matched and consumed
    /// - `false` otherwise
    ///
    /// # Critical Invariant
    /// This function MUST perform **exactly one token advance** on success.
    /// Multiple advances would corrupt parser state and break grammar alignment.
    pub fn match_arrow(&mut self) -> bool {
        if self.tokens[self.current].kind == TokenKind::Symbol
            && self.tokens[self.current].lexeme == "->"
        {
            self.advance(); // EXACTLY ONE ADVANCE
            true
        } else {
            false
        }
    }

    /// Consumes the PAWX arrow operator (`->`) or raises a hard syntax error.
    ///
    /// This is a **strict consumption** helper used when the grammar
    /// *requires* the presence of an arrow token.
    ///
    /// Internally, this delegates to `match_arrow()` and panics if no match
    /// is found.
    ///
    /// # Panics
    /// - If the current token is not `"->"`
    ///
    /// # Usage Examples
    /// Used in:
    /// - Function declarations
    /// - Lambda parsing
    /// - Getters and setters
    /// - Method signatures
    pub fn consume_arrow(&mut self) {
        if !self.match_arrow() {
            panic!("Expected '->'");
        }
    }

    /// Attempts to match an exact symbol lexeme and consume it if successful.
    ///
    /// Unlike `match_symbol(char)`, this function:
    /// - Matches against a **full string lexeme**
    /// - Works with multi-character symbols like:
    ///   - `"++"`
    ///   - `"--"`
    ///   - `"->"`
    ///
    /// This is primarily used for:
    /// - Post-increment (`i++`)
    /// - Post-decrement (`i--`)
    /// - Special operator parsing
    ///
    /// # Parameters
    /// - `s`: The exact symbol string to match
    ///
    /// # Returns
    /// - `true` if the symbol matched and was consumed
    /// - `false` otherwise
    ///
    /// # Safety
    /// This function performs bounds checks and will not panic at EOF.
    pub fn match_symbol_lexeme(&mut self, s: &str) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.tokens[self.current].kind == TokenKind::Symbol
            && self.tokens[self.current].lexeme == s
        {
            self.current += 1;
            return true;
        }

        false
    }
}