/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * File:     parser/mod.rs
 * Purpose:  Root module for the PAWX recursive-descent parser.
 * 
 * This module wires together all parser sub-modules, including:
 *   - Core parser control logic
 *   - Statement parsing
 *   - Expression parsing
 *   - Shared helper utilities
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

/// Core parser orchestration:
/// - Owns the `Parser` struct
/// - Exposes the main `parse(tokens)` entry point
pub mod parser;

/// Statement-level parsing:
/// - if / while / return / try / throw
/// - clowder / instinct
/// - variable declarations
pub mod statements;

/// Expression-level parsing:
/// - assignment → equality → comparison → term → factor → unary → call → primary
/// - lambdas, arrays, objects, indexing, calls, etc.
pub mod expressions;

/// Shared parser helpers:
/// - token matching
/// - lookahead checks
/// - symbol consumption
/// - arrow handling
pub mod helpers;

/// Re-export the public parse entry point so callers can use:
/// `crate::parser::parse(...)`
pub use parser::parse;