/*
 * ============================================================================
 * PAWX - Code with Claws!
 * ============================================================================
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
 * Full license text available at:
 *    https://license.pawx-lang.com
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under these licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *
 * ============================================================================
 */

use crate::ast::{Expr, Param};
use crate::ast::class::{ClassMember, AccessLevel};
use crate::ast::instinct::{InstinctMember};

/// All executable PAWX statements.
#[derive(Debug, Clone)]
pub enum Stmt {
    /* ----------------------------- */
    /* EXPRESSIONS                   */
    /* ----------------------------- */

    Expression(Expr),

    /* ----------------------------- */
    /* VARIABLES                     */
    /* ----------------------------- */

    PublicVar {
        name: String,
        value: Expr,
    },

    PrivateVar {
        name: String,
        value: Expr,
    },

    ProtectedVar {
        name: String,
        value: Expr,
    },

    /* ----------------------------- */
    /* FUNCTIONS                     */
    /* ----------------------------- */

    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        return_type: Option<String>,
        is_async: bool,
    },

    Return(Option<Expr>),

    /* ----------------------------- */
    /* CONTROL FLOW                  */
    /* ----------------------------- */

    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    Try {
        try_block: Vec<Stmt>,
        catch_param: Option<String>,
        catch_block: Option<Vec<Stmt>>,
        finally_block: Option<Vec<Stmt>>,
    },

    Throw(Expr),

    Nap(Expr),

    /* ----------------------------- */
    /* CLASSES (CLOWDER)             */
    /* ----------------------------- */

    Clowder {
        name: String,
        base: Option<String>,
        interfaces: Vec<String>,
        members: Vec<ClassMember>,
        is_exported: bool,
        is_default: bool,
    },

    /* ----------------------------- */
    /* INTERFACES (INSTINCT)         */
    /* ----------------------------- */

    Instinct {
        name: String,
        members: Vec<InstinctMember>,
        is_exported: bool,
        is_default: bool,
    },

    /* ----------------------------- */
    /* MODULE SYSTEM                 */
    /* ----------------------------- */

    Export {
        name: Option<String>, // None = default export
        value: Expr,
    },

    /* ----------------------------- */
    /* PRIDE (NAMESPACE BLOCK)       */
    /* ----------------------------- */

    Pride {
        name: String,
        body: Vec<Stmt>,
    },
}