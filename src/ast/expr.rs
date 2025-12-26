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

use crate::{ast::Stmt, lexer::token::Token, value::Value};
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal {
        value: Value,
        span: Span,
    },

    Identifier {
        name: String,
        span: Span,
    },

    Assign {
        name: String,
        value: Box<Expr>,
        span: Span,
    },

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
        span: Span,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
        span: Span,
    },

    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        span: Span,
    },

    Get {
        object: Box<Expr>,
        name: String,
        span: Span,
    },

    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
        span: Span,
    },

    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    IndexAssign {
        object: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    ArrayLiteral {
        values: Vec<Expr>,
        span: Span,
    },

    ObjectLiteral {
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    Lambda {
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },

    Tap {
        path: Box<Expr>,
        span: Span,
    },

    New {
        class_name: String,
        arguments: Vec<Expr>,
        span: Span,
    },

    PostIncrement {
        name: String,
        span: Span,
    },

    PostDecrement {
        name: String,
        span: Span,
    },

    Tuple {
        values: Vec<Expr>,
        span: Span,
    },

    Grouping {
        expr: Box<Expr>,
        span: Span,
    },

    Logical {
        left: Box<Expr>,
        operator: Token, // keep Token so we can reuse its span
        right: Box<Expr>,
        span: Span,
    },
}