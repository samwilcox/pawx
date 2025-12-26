/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * File:      token.rs
 * Purpose:   Defines the fundamental lexical token types used by the PAWX
 *            compiler during the lexing and parsing stages.
 * 
 * Author:    Sam Wilcox
 * Email:     sam@pawx-lang.com
 * Website:   https://www.pawx-lang.com
 * GitHub:    https://github.com/samwilcox/pawx
 * 
 * License:
 * This file is part of the PAWX programming language project.
 * 
 * PAWX is dual-licensed under the terms of:
 *   - The MIT License
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

use crate::span::Span;
use std::fmt;

/// Represents the **category of a lexical token** in the PAWX language.
///
/// `TokenKind` identifies how a sequence of characters from the source
/// code should be interpreted by the parser.
///
/// # Compiler Pipeline Role
/// ```text
/// Source Code → Lexer → TokenKind → Parser → AST
/// ```
///
/// Each token kind directly influences:
/// - Expression parsing
/// - Operator precedence
/// - Statement classification
/// - Error reporting
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// A numeric literal.
    ///
    /// Includes:
    /// - Integer values: `42`
    /// - Floating-point values: `3.14`
    Number,

    /// A quoted string literal.
    ///
    /// Examples:
    /// - `"hello"`
    /// - `'world'`
    String,

    /// A user-defined name.
    ///
    /// Used for:
    /// - Variable names
    /// - Function names
    /// - Class names
    /// - Object property identifiers
    Identifier,

    /// A reserved PAWX language keyword.
    ///
    /// Examples:
    /// - `snuggle`
    /// - `purr`
    /// - `return`
    /// - `if`, `else`, `while`
    ///
    /// Keyword detection is handled by `keywords.rs`.
    Keyword,

    /// A symbolic operator or punctuation character.
    ///
    /// Includes:
    /// - Arithmetic operators: `+`, `-`, `*`, `/`
    /// - Comparison operators: `==`, `!=`, `===`
    /// - Structural symbols: `{`, `}`, `(`, `)`, `[`, `]`
    /// - Language operators: `->`, `++`, `--`
    Symbol,

    /// End-of-file marker.
    ///
    /// This token is always appended as the **final token**
    /// during lexing and is used by the parser to determine
    /// when input has been fully consumed.
    Eof,
}

/// Represents a **single lexical token** produced by the PAWX lexer.
///
/// A `Token` is a fully classified unit of source code consisting of:
/// - A token category (`TokenKind`)
/// - The original source text (`lexeme`)
/// - The line number for error reporting
///
/// # Example Tokens
/// ```text
/// snuggle  →  { kind: Keyword,    lexeme: "snuggle", line: 1 }
/// age      →  { kind: Identifier, lexeme: "age",     line: 1 }
/// 42       →  { kind: Number,     lexeme: "42",      line: 1 }
/// ```
///
/// # Compiler Usage
/// Tokens are consumed by the PAWX parser to construct:
/// - Expressions
/// - Statements
/// - Control flow
/// - Function and class declarations
#[derive(Debug, Clone)]
pub struct Token {
    /// The classified category of the token.
    pub kind: TokenKind,

    /// The exact source text that produced this token.
    ///
    /// This value is preserved verbatim for:
    /// - Error messages
    /// - Debug output
    /// - Literal evaluation
    pub lexeme: String,

    /// The 1-based line number where this token appeared.
    ///
    /// Used for:
    /// - Syntax error reporting
    /// - Runtime diagnostics
    /// - Debug traces
    pub span: Span,
}

impl fmt::Display for Token {
    /// Formats a token for **user-facing output**.
    ///
    /// This implementation intentionally prints **only the token’s lexeme**
    /// (the exact source text), rather than its full internal structure.
    ///
    /// ## Why `Display` vs `Debug`
    /// - `Display` (`{}`) is used for **error messages and diagnostics**
    /// - `Debug` (`{:?}`) is reserved for **developer introspection**
    ///
    /// In compiler error output, users care about *what they wrote*:
    /// ```text
    /// Invalid binary operation: ===
    /// ```
    /// not:
    /// ```text
    /// Token { kind: Symbol, lexeme: "===", span: ... }
    /// ```
    ///
    /// ## Design Rationale
    /// - Keeps error messages clean and readable
    /// - Allows tokens to carry rich metadata (kind, span, line info)
    ///   without leaking implementation details into user output
    /// - Mirrors how professional compilers (rustc, clang) format tokens
    ///
    /// ## Usage
    /// ```rust
    /// panic!("Unexpected token: {}", token);
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}