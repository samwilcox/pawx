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
 * File:     param.rs
 * Purpose:  Defines the AST structure for function & method parameters
 *
 * This file defines the `Param` struct used by:
 *  - Function declarations (`purr`)
 *  - Lambda expressions
 *  - Class methods
 *  - Getters & setters
 *
 * It supports:
 *  - Default values (JavaScript-style)
 *  - Optional type annotations
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

use crate::ast::Expr;

/// Represents **one declared parameter** in a function, lambda, or method.
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name (identifier)
    pub name: String,

    /// Optional default value:
    /// `purr test -> (x = 5) -> { ... }`
    pub default: Option<Expr>,

    /// Optional static type annotation:
    /// `purr test -> (x: Number) -> { ... }`
    pub type_annotation: Option<String>,
}