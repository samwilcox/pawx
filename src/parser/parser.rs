/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * Core Recursive-Descent Parser Entry Point
 * 
 * This file defines the primary `Parser` structure and the public `parse()`
 * driver function used to transform a token stream into a full Abstract
 * Syntax Tree (AST) statement list for the PAWX programming language.
 * 
 * The parsing implementation itself is split across multiple modules:
 * - `statements.rs`   → Statement-level grammar (`if`, `while`, `purr`, etc.)
 * - `expressions.rs`  → Expression grammar & operator precedence
 * - `helpers.rs`      → Token matching, consumption, and navigation utilities
 * 
 * This file serves as the **root coordinator** of the parsing process.
 * 
 * --------------------------------------------------------------------------
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

use crate::ast::Stmt;
use crate::lexer::token::{Token};

/// The core PAWX recursive-descent parser.
///
/// This structure maintains:
/// - The full token stream produced by the lexer
/// - The current cursor position into that stream
///
/// The actual grammar logic is implemented through extension modules
/// (`statements`, `expressions`, `helpers`) via additional `impl Parser` blocks.
pub struct Parser {
    /// Complete list of tokens to be parsed.
    pub tokens: Vec<Token>,

    /// Current cursor position within the token stream.
    pub current: usize,
}

/// Public entry point for the PAWX parsing phase.
///
/// This function:
/// 1. Creates a new `Parser` instance from the provided token list
/// 2. Executes the full recursive-descent parsing process
/// 3. Returns the resulting list of top-level AST statements
///
/// # Parameters
/// - `tokens`: The full token stream produced by the lexer
///
/// # Returns
/// A vector of fully parsed top-level `Stmt` nodes.
///
/// # PAWX Compilation Pipeline
/// ```text
/// Source → Lexer → Tokens → Parser → AST → Interpreter
/// ```
///
/// # Example
/// ```rust
/// let tokens = tokenize(source_code);
/// let ast = parse(tokens);
/// ```
pub fn parse(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut parser = Parser { tokens, current: 0 };
    parser.parse()
}

impl Parser {
    /// Parses the entire token stream into a list of top-level statements.
    ///
    /// This is the **main driver** of the recursive-descent parser.
    /// It continuously consumes statements until the End-Of-File (EOF)
    /// token is reached.
    ///
    /// # Returns
    /// A fully built AST represented as a vector of `Stmt` nodes.
    ///
    /// # Behavior
    /// - Guarantees full token consumption.
    /// - Statements are parsed in strict left-to-right order.
    /// - Structural errors will trigger immediate panics.
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            stmts.push(self.statement());
        }

        stmts
    }
}