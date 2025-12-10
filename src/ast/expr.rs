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

 #[derive(Debug, Clone)]
 pub enum Expr {
    Literal(Value),
    Identifier(String),
    Assign { name: String, value: Box<Expr> },
    Binary { left: Box<Expr>, operator: String, right: Box<Expr> },
    Unary { operator: String, right: Box<Expr> },
    Call { callee: Box<Expr>, arguments: Vec<Expr> },
    Get { object: Box<Expr>, name: String },
    Set { object: Box<Expr>, name: String, value: Box<Expr> },
    Index { object: Box<Expr>, index: Box<Expr> },
    IndexAssign { object: Box<Expr>, index: Box<Expr>, value: Box<Expr> },
    ArrayLiteral { values: Vec<Expr> },
    ObjectLiteral { fields: Vec<(String, Expr)> },
    Lambda { params: Vec<String>, body: Vec<Stmt> },
    Tap { path: Box<Expr> },
    New { class_name: String, arguments: Vec<Expr> },
    PostIncrement { name: String },
    PostDecrement { name: String },
    Tuple(Vec<Expr>),
    Grouping(Box<Expr>),
    Logical { left: Box<Expr>, operator: Token, right: Box<Expr>, },
}