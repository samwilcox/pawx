/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * File:      keywords.rs
 * Purpose:   Defines all reserved keywords for the PAWX programming language.
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

/// Determines whether a given identifier is a **reserved keyword** in PAWX.
///
/// This function is used exclusively by the lexer during tokenization to
/// distinguish **user-defined identifiers** from **language-defined keywords**
/// such as control flow, declarations, class syntax, and built-in behaviors.
///
/// # Parameters
/// - `word`: The identifier string extracted from source code.
///
/// # Returns
/// - `true` if the word is a reserved PAWX keyword.
/// - `false` if the word should be treated as a normal identifier.
///
/// # Behavior
/// - This function performs a **constant-time lookup** using Rust’s `matches!`
///   macro.
/// - All keywords defined here are treated as `TokenKind::Keyword` during
///   lexing.
/// - Any future language keywords should be added here.
///
/// # PAWX Examples
/// ```text
/// snuggle   -> keyword
/// purr      -> keyword
/// Cat       -> identifier
/// myVar     -> identifier
/// ```
///
/// # Internal Use Only
/// This function is intended for use by the lexer and should not be called
/// directly by user code.
pub fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "den" |
        "lair" |
        "pride" |
        "purr" |
        "zoom" |      // async
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
        "tap" |
        "default"
    )
}