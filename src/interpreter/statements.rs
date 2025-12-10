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

/*!
 * PAWX Statement Executor
 * -----------------------
 * 
 * This module is responsible for executing **all top-level and block-level
 * statements** in the PAWX interpreter.
 * 
 * It does NOT evaluate expressions (handled by `expressions.rs`).
 * It does NOT execute function calls (handled by `calls.rs`).
 * It does NOT process timers or runtime built-ins.
 * 
 * This file strictly handles:
 * 
 *  • Variable declarations
 *  • Control flow (if, while)
 *  • Function declarations
 *  • Class declarations (clowder)
 *  • Interfaces (instinct)
 *  • Try / catch / finally
 *  • Return, throw, export
 *  • Expression statements
 */

use crate::ast::{ClassMember, Stmt};
use crate::interpreter::environment::{Environment, FunctionDef};
use crate::value::Value;
use crate::interpreter::expressions::eval_expr;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/* ============================================================================
 * Execution Control Signals
 * ============================================================================
 */

/// Internal control flow signal used by the interpreter.
#[derive(Debug)]
pub enum ExecSignal {
    /// Normal fall-through execution.
    None,

    /// Early return from a function.
    Return(Value),

    /// Thrown runtime exception.
    Throw(Value),
}

/* ============================================================================
 * Statement Execution Entry Point
 * ============================================================================
 */

/// Executes a single PAWX statement inside the given environment.
///
/// This is the **core dispatch function for all statement execution**.
pub fn exec_stmt(stmt: Stmt, env: Rc<RefCell<Environment>>) -> ExecSignal {
    match stmt {
        /* ------------------------------------------------------------------
         * Expression Statement
         * ---------------------------------------------------------------- */
        Stmt::Expression(expr) => {
            let _ = eval_expr(expr, env);
            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Variable Declarations
         * ---------------------------------------------------------------- */
        Stmt::PublicVar { name, value } => {
            let val = eval_expr(value, env.clone());

            // If this is a module, unwrap its default export by default
            let final_value = match val {
                Value::Module { default: Some(default), .. } => (*default).clone(),
                other => other,
            };

            env.borrow_mut().define_public(name, final_value);
            ExecSignal::None
        }

        Stmt::PrivateVar { name, value } => {
            let val = eval_expr(value, env.clone());
            env.borrow_mut().define_private(name, val);
            ExecSignal::None
        }

        Stmt::ProtectedVar { name, value } => {
            let val = eval_expr(value, env.clone());
            env.borrow_mut().define_protected(name, val);
            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Function Declaration
         * ---------------------------------------------------------------- */
        Stmt::Function {
            name,
            params,
            body,
            return_type,
            is_async,
        } => {
            let func_def = FunctionDef {
                params,
                body,
                return_type,
                is_async,
            };

            env.borrow_mut().define_function(name, func_def);
            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Return Statement
         * ---------------------------------------------------------------- */
        Stmt::Return(expr_opt) => {
            let val = match expr_opt {
                Some(expr) => eval_expr(expr, env),
                None => Value::Null,
            };

            ExecSignal::Return(val)
        }

        /* ------------------------------------------------------------------
         * If / Else Control Flow
         * ---------------------------------------------------------------- */
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_val = eval_expr(condition, env.clone());

            let truthy = match cond_val {
                Value::Bool(b) => b,
                Value::Number(n) => n != 0.0,
                Value::Null => false,
                _ => true,
            };

            if truthy {
                for s in then_branch {
                    match exec_stmt(s, env.clone()) {
                        ExecSignal::None => {}
                        other => return other,
                    }
                }
            } else if let Some(else_body) = else_branch {
                for s in else_body {
                    match exec_stmt(s, env.clone()) {
                        ExecSignal::None => {}
                        other => return other,
                    }
                }
            }

            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * While Loop
         * ---------------------------------------------------------------- */
        Stmt::While { condition, body } => {
            loop {
                let cond_val = eval_expr(condition.clone(), env.clone());

                let truthy = match cond_val {
                    Value::Bool(b) => b,
                    Value::Number(n) => n != 0.0,
                    Value::Null => false,
                    _ => true,
                };

                if !truthy {
                    break;
                }

                for s in &body {
                    match exec_stmt(s.clone(), env.clone()) {
                        ExecSignal::None => {}
                        other => return other,
                    }
                }
            }

            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Try / Catch / Finally
         * ---------------------------------------------------------------- */
        Stmt::Try {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        } => {
            let mut result = ExecSignal::None;

            // TRY
            for stmt in try_block {
                match exec_stmt(stmt, env.clone()) {
                    ExecSignal::None => {}

                    ExecSignal::Return(v) => {
                        result = ExecSignal::Return(v);
                        break;
                    }

                    ExecSignal::Throw(err) => {
                        // CATCH
                        if let (Some(name), Some(catch_body)) =
                            (catch_param.clone(), catch_block.clone())
                        {
                            let catch_env =
                                Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

                            catch_env.borrow_mut().define_public(name, err);

                            for cstmt in catch_body {
                                match exec_stmt(cstmt, catch_env.clone()) {
                                    ExecSignal::None => {}
                                    other => {
                                        result = other;
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Unhandled inside try; propagate
                            result = ExecSignal::Throw(err);
                        }

                        break;
                    }
                }
            }

            // FINALLY
            if let Some(finally_body) = finally_block {
                for fstmt in finally_body {
                    match exec_stmt(fstmt, env.clone()) {
                        ExecSignal::None => {}
                        // finally wins over try/catch result
                        other => return other,
                    }
                }
            }

            result
        }

        /* ------------------------------------------------------------------
         * Class (Clowder)
         * ---------------------------------------------------------------- */
        Stmt::Clowder {
            name,
            base: _,
            interfaces: _,
            members,
            is_exported,
            is_default,
        } => {
            let mut methods: HashMap<String, FunctionDef> = HashMap::new();
            let mut getters: HashMap<String, FunctionDef> = HashMap::new();
            let mut setters: HashMap<String, FunctionDef> = HashMap::new();
            let mut fields: HashMap<String, Value> = HashMap::new();

            for member in members {
                match member {
                    ClassMember::Field { name, value, .. } => {
                        let val = if let Some(expr) = value {
                            eval_expr(expr, env.clone())
                        } else {
                            Value::Null
                        };

                        fields.insert(name, val);
                    }

                    ClassMember::Method { name, params, body, .. } => {
                        methods.insert(
                            name,
                            FunctionDef {
                                params,
                                body,
                                return_type: None,
                                is_async: false,
                            },
                        );
                    }

                    ClassMember::Getter { name, body, .. } => {
                        getters.insert(
                            name,
                            FunctionDef {
                                params: vec![],
                                body,
                                return_type: None,
                                is_async: false,
                            },
                        );
                    }

                    ClassMember::Setter { name, param_name, body, .. } => {
                        setters.insert(
                            name,
                            FunctionDef {
                                params: vec![crate::ast::Param {
                                    name: param_name,
                                    default: None,
                                    type_annotation: None,
                                }],
                                body,
                                return_type: None,
                                is_async: false,
                            },
                        );
                    }
                }
            }

            let class_val = Value::Class {
                name: name.clone(),
                methods,
                getters,
                setters,
                fields,
            };

            if is_exported && is_default {
                env.borrow_mut()
                    .define_public("default".to_string(), class_val);
            } else {
                env.borrow_mut().define_public(name, class_val);
            }

            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Interface (Instinct)
         * ---------------------------------------------------------------- */
        Stmt::Instinct { name, .. } => {
            // For now, instinct types are only compile-time;
            // at runtime we just expose a sentinel value.
            env.borrow_mut().define_public(name, Value::Null);
            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Export Statement
         * ---------------------------------------------------------------- */
        Stmt::Export { name, value } => {
            let val = eval_expr(value, env.clone());

            // 🔧 This avoids the fragile `match name { ... }` and
            // works cleanly with `Option<String>`.
            if let Some(export_name) = name {
                env.borrow_mut().define_public(export_name, val);
            } else {
                // default export: `exports default = expr;`
                env.borrow_mut()
                    .define_public("default".to_string(), val);
            }

            ExecSignal::None
        }

        /* ------------------------------------------------------------------
         * Throw Statement
         * ---------------------------------------------------------------- */
        Stmt::Throw(expr) => {
            let val = eval_expr(expr, env);
            ExecSignal::Throw(val)
        }

        /* ------------------------------------------------------------------
         * Nap (Await-like)
         * ---------------------------------------------------------------- */
        Stmt::Nap(expr) => {
            let val = eval_expr(expr, env);

            match val {
                // 🔧 Typo fix: assume your enum variant is `Future`
                Value::Furure(inner) => ExecSignal::Return(*inner),
                _ => panic!("nap can only be used on a Future"),
            }
        }

        /* ------------------------------------------------------------------
         * Pride Block
         * ---------------------------------------------------------------- */
        Stmt::Pride { name, body } => {
            // Create a new lexical scope for the pride block
            let pride_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

            // Execute all statements inside the pride block
            for stmt in body {
                match exec_stmt(stmt, pride_env.clone()) {
                    ExecSignal::None => {}
                    ExecSignal::Return(v) => return ExecSignal::Return(v),
                    ExecSignal::Throw(e) => return ExecSignal::Throw(e),
                }
            }

            // Expose the pride as an object in the outer scope
            let mut fields = HashMap::new();
            for (k, v) in &pride_env.borrow().values {
                fields.insert(k.clone(), v.value.clone());
            }

            env.borrow_mut().define_public(
                name,
                Value::Object {
                    fields: Rc::new(RefCell::new(fields)),
                },
            );

            ExecSignal::None
        }
    }
}

/* ============================================================================
 * Statement Runner
 * ============================================================================
 */

/// Executes a full statement block inside an environment.
///
/// This is used for:
///  • Modules
///  • Imported files (tap)
///  • Sub-execution contexts
pub fn run_in_env(statements: Vec<Stmt>, env: Rc<RefCell<Environment>>) {
    for stmt in statements {
        match exec_stmt(stmt, env.clone()) {
            ExecSignal::None => {}

            ExecSignal::Return(_) => break,

            ExecSignal::Throw(err) => {
                panic!("Uncaught Pawx error in module: {:?}", err);
            }
        }
    }
}