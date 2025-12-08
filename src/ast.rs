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

use crate::value::Value;

#[derive(Debug, Clone)]
pub enum AccessLevel {
    Public,   // pride
    Private,  // den
    Protected // lair
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
    pub type_annotation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Identifier(String),

    Assign {
        name: String,
        value: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },

    Unary {
        operator: String,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },

    Get {
        object: Box<Expr>,
        name: String,
    },

    Nap(Box<Expr>),
    Grouping(Box<Expr>),

    Lambda {
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    New {
        class_name: String,
        arguments: Vec<Expr>,
    },

    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },

    Tap {
        path: Box<Expr>,
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),

    // pride x = ...  (PUBLIC)
    PublicVar {
        name: String,
        value: Expr,
    },

    // den x = ...    (PRIVATE)
    PrivateVar {
        name: String,
        value: Expr,
    },

    // lair x = ...   (PROTECTED)
    ProtectedVar {
        name: String,
        value: Expr,
    },

    // pride Cat { ... }
    Pride {
        name: String,
        body: Vec<Stmt>,
    },

    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        return_type: Option<String>,
        is_async: bool, // zoom
    },

    Return(Option<Expr>),
    Nap(Expr),

    Throw(Expr),

    Try {
        try_block: Vec<Stmt>,
        catch_param: Option<String>,
        catch_block: Option<Vec<Stmt>>,
        finally_block: Option<Vec<Stmt>>,
    },

    Clowder {
        name: String,
        base: Option<String>,          // inherits
        interfaces: Vec<String>,      // practices
        members: Vec<ClassMember>,
        is_exported: bool,
        is_default: bool,
    },

    Instinct {
        name: String,
        members: Vec<InstinctMember>,
        is_exported: bool,
        is_default: bool,
    },

    Export {
        name: Option<String>,
        value: Expr,
    }
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Field {
        name: String,
        access: AccessLevel,
        is_static: bool,
        type_annotation: Option<String>,
        value: Option<Expr>,
    },
    Method {
        name: String,
        access: AccessLevel,
        is_static: bool,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },
    Getter {
        name: String,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },
    Setter {
        name: String,
        param_name: String,
        param_type: Option<String>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct InstinctMember {
    pub name: String,
    pub kind: InstinctMemberKind,
}

#[derive(Debug, Clone)]
pub enum InstinctMemberKind {
    Method {
        params: Vec<Param>,
        return_type: Option<String>,
    },
    Getter {
        return_type: Option<String>,
    },
    Setter {
        param_name: String,
        param_type: Option<String>,
    },
}