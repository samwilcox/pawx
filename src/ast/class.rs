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

 /*
 * ==========================================================================
 * PAWX - Code with Claws! 🐾
 * ==========================================================================
 *
 * File:     class.rs
 * Purpose:  Defines all AST structures related to PAWX `clowder` (class)
 *
 * This file defines:
 *  - ClassMember
 *  - AccessLevel
 *  - Field / Method / Getter / Setter structures
 *
 * These are produced by:
 *  - parser/statements.rs (clowder_declaration)
 *
 * And consumed by:
 *  - runtime/class_runtime.rs (or equivalent)
 *
 * --------------------------------------------------------------------------
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * GitHub:   https://github.com/samwilcox/pawx
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
 *     https://license.pawx-lang.com
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under these licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *
 * ==========================================================================
 */

use crate::ast::{Expr, Param, Stmt};

/// Controls visibility of class members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessLevel {
    Public,
    Private,
    Protected,
}

/// Represents **one declared member inside a PAWX `clowder`**.
#[derive(Debug, Clone)]
pub enum ClassMember {
    /// Class field
    Field {
        name: String,
        access: AccessLevel,
        is_static: bool,
        type_annotation: Option<String>,
        value: Option<Expr>,
    },

    /// Class method
    Method {
        name: String,
        access: AccessLevel,
        is_static: bool,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },

    /// Getter method
    Getter {
        name: String,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },

    /// Setter method
    Setter {
        name: String,
        param_name: String,
        param_type: Option<String>,
        body: Vec<Stmt>,
    },
}