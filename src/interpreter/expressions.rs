/*
 * ============================================================================
 * PAWX - Code with Claws!
 * ============================================================================
 * 
 * Expression Evaluation Engine
 * -----------------------------
 * This module is responsible for **evaluating all PAWX expressions** at
 * runtime. It converts AST `Expr` nodes into concrete runtime `Value`s.
 * 
 * This includes:
 *   - Literals and identifiers
 *   - Binary and unary operations
 *   - Function and method calls
 *   - Object and array literals
 *   - Indexing and assignment
 *   - Class construction (`new`)
 *   - Lambdas and closures
 *   - Postfix operators (++, --)
 *   - tap() imports
 *   - Property access and mutation
 * 
 * This module is **pure evaluation only** and never executes statements.
 * All statement control flow is handled by `statements.rs`.
 * 
 * ---------------------------------------------------------------------------
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * Github:   https://github.com/samwilcox/pawx
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
 * ============================================================================
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::Expr;
use crate::interpreter::environment::Environment;
use crate::value::Value;

// Call dispatch (from calls.rs)
use crate::interpreter::calls::{call_user_function, call_value};

// Array prototype
use crate::prototypes::array::create_array_proto;

// Interpreter execution bridge (for lambdas)
use crate::interpreter::{exec_stmt, ExecSignal};
use crate::interpreter::helpers::is_truthy;

fn resolve_furure(value: &Value) -> Value {
    let mut current = value.clone();

    loop {
        match current {
            Value::Furure(inner) => {
                current = *inner;
            }

            Value::NativeFunction(f) => {
                return f(vec![]);
            }

            other => return other,
        }
    }
}

/// Evaluates a single PAWX expression and returns its runtime value.
///
/// This is the **core expression dispatcher** used throughout the interpreter.
/// Every expression in the language eventually passes through this function.
///
/// # Arguments
/// - `expr` → The AST expression to evaluate
/// - `env`  → The current runtime environment
///
/// # Returns
/// - The evaluated runtime `Value`
pub fn eval_expr(expr: Expr, env: Rc<RefCell<Environment>>) -> Value {
    match expr {
        // ---------------------------------------------------------------------
        // Literal Values
        // ---------------------------------------------------------------------
        Expr::Literal(v) => v,

        // ---------------------------------------------------------------------
        // Identifier Lookup
        // ---------------------------------------------------------------------
        Expr::Identifier(name) => {
            if name == "this" {
                env.borrow()
                    .get("this", false)
                    .unwrap_or_else(|| panic!("'this' used outside of class"))
            } else {
                env.borrow()
                    .get(&name, false)
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name))
            }
        }

        // ---------------------------------------------------------------------
        // Tuple Literal
        // ---------------------------------------------------------------------
        Expr::Tuple(values) => {
            let evaluated = values
                .into_iter()
                .map(|v| eval_expr(v, env.clone()))
                .collect();
            Value::Tuple(evaluated)
        }

        // ---------------------------------------------------------------------
        // Assignment (simple identifier = expr)
        // ---------------------------------------------------------------------
        Expr::Assign { name, value } => {
            let assigned = eval_expr(*value, env.clone());

            // If we assign a module, automatically unwrap its default export
            if let Value::Module { default: Some(default_val), .. } = &assigned {
                let real = *default_val.clone();
                env.borrow_mut().define_public(name, real.clone());
                return real;
            }

            if !env.borrow_mut().assign(&name, assigned.clone()) {
                panic!("Undefined variable '{}'", name);
            }

            assigned
        }

        // ---------------------------------------------------------------------
        // Unary Operators
        // ---------------------------------------------------------------------
        Expr::Unary { operator, right } => {
            let r = eval_expr(*right, env);
            match (operator.as_str(), r) {
                ("-", Value::Number(n)) => Value::Number(-n),
                ("!", Value::Bool(b)) => Value::Bool(!b),
                _ => panic!("Invalid unary operation: {}", operator),
            }
        }

        // ---------------------------------------------------------------------
        // Binary Operators
        // ---------------------------------------------------------------------
        Expr::Binary { left, operator, right } => {
            let l = eval_expr(*left, env.clone());
            let r = eval_expr(*right, env);

            match (l, r, operator.as_str()) {
                // -------------------------------
                // Arithmetic
                // -------------------------------
                (Value::Number(a), Value::Number(b), "+") => Value::Number(a + b),
                (Value::Number(a), Value::Number(b), "-") => Value::Number(a - b),
                (Value::Number(a), Value::Number(b), "*") => Value::Number(a * b),
                (Value::Number(a), Value::Number(b), "/") => Value::Number(a / b),
                (Value::Number(a), Value::Number(b), "%") => Value::Number(a % b),

                (Value::String(a), Value::String(b), "+") => Value::String(format!("{}{}", a, b)),
                (Value::String(a), Value::Number(b), "+") => Value::String(format!("{}{}", a, b)),
                (Value::Number(a), Value::String(b), "+") => Value::String(format!("{}{}", a, b)),

                // -------------------------------
                // Loose Equality (==)
                // -------------------------------
                (Value::Number(a), Value::Number(b), "==") => Value::Bool(a == b),
                (Value::String(a), Value::String(b), "==") => Value::Bool(a == b),
                (Value::Bool(a), Value::Bool(b), "==")     => Value::Bool(a == b),
                (Value::Null, Value::Null, "==")           => Value::Bool(true),

                // universal fallback ==
                (a, b, "==") => {
                    Value::Bool(std::mem::discriminant(&a) == std::mem::discriminant(&b))
                }

                // -------------------------------
                // Strict Equality (===)
                // -------------------------------
                (Value::Number(a), Value::Number(b), "===") => Value::Bool(a == b),
                (Value::String(a), Value::String(b), "===") => Value::Bool(a == b),
                (Value::Bool(a), Value::Bool(b), "===")     => Value::Bool(a == b),
                (Value::Null, Value::Null, "===")           => Value::Bool(true),

                (a, b, "===") => Value::Bool(values_equal_strict(&a, &b)),

                // -------------------------------
                // Loose Inequality (!=)
                // -------------------------------
                (Value::Number(a), Value::Number(b), "!=") => Value::Bool(a != b),
                (Value::String(a), Value::String(b), "!=") => Value::Bool(a != b),
                (Value::Bool(a), Value::Bool(b), "!=")     => Value::Bool(a != b),
                (Value::Null, Value::Null, "!=")           => Value::Bool(false),

                (a, b, "!=") => {
                    Value::Bool(std::mem::discriminant(&a) != std::mem::discriminant(&b))
                }

                // -------------------------------
                // Strict Inequality (!==)
                // -------------------------------
                (Value::Number(a), Value::Number(b), "!==") => Value::Bool(a != b),
                (Value::String(a), Value::String(b), "!==") => Value::Bool(a != b),
                (Value::Bool(a), Value::Bool(b), "!==")     => Value::Bool(a != b),
                (Value::Null, Value::Null, "!==")           => Value::Bool(false),

                (a, b, "!==") => Value::Bool(!values_equal_strict(&a, &b)),

                // -------------------------------
                // Comparisons
                // -------------------------------
                (Value::Number(a), Value::Number(b), ">")  => Value::Bool(a > b),
                (Value::Number(a), Value::Number(b), "<")  => Value::Bool(a < b),
                (Value::Number(a), Value::Number(b), ">=") => Value::Bool(a >= b),
                (Value::Number(a), Value::Number(b), "<=") => Value::Bool(a <= b),

                // -------------------------------
                // Fallback
                // -------------------------------
                _ => panic!("Invalid binary operation: {}", operator),
            }
        }

        // ---------------------------------------------------------------------
        // Function Calls
        // ---------------------------------------------------------------------
        Expr::Call { callee, arguments } => {
            match *callee {
                // Direct named call: foo(...)
                Expr::Identifier(name) => {
                    // User-defined function
                    if let Some(func) = env.borrow().get_function(&name) {
                        let arg_vals = arguments
                            .into_iter()
                            .map(|a| eval_expr(a, env.clone()))
                            .collect();
                        return call_user_function(func, arg_vals, env.clone());
                    }

                    // Anything else callable by name (class, built-in, etc.)
                    let callee_val = env.borrow().get(&name, false)
                        .unwrap_or_else(|| panic!("Undefined function or callable '{}'", name));

                    call_value(callee_val, arguments, env.clone())
                }

                // Method calls & higher-order funcs: something()(...) or obj.method(...)
                other => {
                    let callee_val = eval_expr(other, env.clone());
                    call_value(callee_val, arguments, env)
                }
            }
        }

        // ---------------------------------------------------------------------
        // Grouping
        // ---------------------------------------------------------------------
        Expr::Grouping(expr) => eval_expr(*expr, env),

        // ---------------------------------------------------------------------
        // Array Literal
        // ---------------------------------------------------------------------
        Expr::ArrayLiteral { values } => {
            let evaluated = values
                .into_iter()
                .map(|v| eval_expr(v, env.clone()))
                .collect();

            Value::Array {
                values: Rc::new(RefCell::new(evaluated)),
                proto: create_array_proto(),
            }
        }

        // ---------------------------------------------------------------------
        // Index Read: arr[i]
        // ---------------------------------------------------------------------
        Expr::Index { object, index } => {
            let obj = eval_expr(*object, env.clone());
            let idx = eval_expr(*index, env);

            let i = match idx {
                Value::Number(n) => n as usize,
                _ => panic!("Array index must be a number"),
            };

            match obj {
                Value::Array { values, .. } => {
                    values.borrow().get(i).cloned().unwrap_or(Value::Null)
                }
                _ => panic!("Indexing only supported on arrays"),
            }
        }

        // ---------------------------------------------------------------------
        // Index Assignment: arr[i] = value
        // ---------------------------------------------------------------------
        Expr::IndexAssign { object, index, value } => {
            let obj = eval_expr(*object, env.clone());
            let idx = eval_expr(*index, env.clone());
            let val = eval_expr(*value, env);

            let i = match idx {
                Value::Number(n) => n as usize,
                _ => panic!("Array index must be a number"),
            };

            match obj {
                Value::Array { values, .. } => {
                    let mut arr = values.borrow_mut();
                    if i >= arr.len() {
                        panic!("Array index out of bounds");
                    }
                    arr[i] = val.clone();
                    val
                }
                _ => panic!("Index assignment only supported on arrays"),
            }
        }

        // ---------------------------------------------------------------------
        // Object Literal: { a: 1, b: 2 }
        // ---------------------------------------------------------------------
        Expr::ObjectLiteral { fields } => {
            let mut map = HashMap::new();
            for (name, expr) in fields {
                map.insert(name, eval_expr(expr, env.clone()));
            }

            Value::Object {
                fields: Rc::new(RefCell::new(map)),
            }
        }

        // ---------------------------------------------------------------------
        // Property Get: obj.prop
        // ---------------------------------------------------------------------
        Expr::Get { object, name } => {
            let target = eval_expr(*object, env.clone());
            let prop_name = name; // move into local for convenience

            match target {
                // ---------------------------------
                // Plain object: obj.prop
                // ---------------------------------
                Value::Object { fields } => {
                    fields
                        .borrow()
                        .get(&prop_name)
                        .cloned()
                        .unwrap_or(Value::Null)
                }

                // ---------------------------------
                // Array: arr.length or arr.method
                // ---------------------------------
                Value::Array { values, proto } => {
                    if prop_name == "length" {
                        Value::Number(values.borrow().len() as f64)
                    } else {
                        proto
                            .get(&prop_name)
                            .cloned()
                            .unwrap_or(Value::Null)
                    }
                }

                // ---------------------------------
                // Furure: .then / .catch / .finally
                // ---------------------------------
                Value::Furure(inner) => {
                    let resolved = (*inner).clone(); // the stored result

                    match prop_name.as_str() {
                        // then(callback) – always runs, passes the resolved value
                        "then" => {
                            Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                                if args.is_empty() {
                                    panic!("then(callback): missing callback");
                                }

                                let callback = args[0].clone();
                                let value_for_chain = resolved.clone();

                                if let Value::NativeFunction(cb) = callback {
                                    cb(vec![resolved.clone()]);
                                } else {
                                    panic!("then(...) expects a function");
                                }

                                // return a new Furure carrying the same resolved value for chaining
                                Value::Furure(Box::new(value_for_chain))
                            }))
                        }

                        // catch(callback) – only runs if resolved is an Error
                        "catch" => {
                            Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                                if args.is_empty() {
                                    panic!("catch(callback): missing callback");
                                }

                                let callback = args[0].clone();
                                let value_for_chain = resolved.clone();

                                if let Value::Error { .. } = resolved {
                                    if let Value::NativeFunction(cb) = callback {
                                        cb(vec![resolved.clone()]);
                                    } else {
                                        panic!("catch(...) expects a function");
                                    }
                                }

                                // chain always continues with the same value
                                Value::Furure(Box::new(value_for_chain))
                            }))
                        }

                        // finally(callback) – always runs, ignores result, preserves chain
                        "finally" => {
                            Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                                if args.is_empty() {
                                    panic!("finally(callback): missing callback");
                                }

                                let callback = args[0].clone();
                                let value_for_chain = resolved.clone();

                                if let Value::NativeFunction(cb) = callback {
                                    cb(vec![]);
                                } else {
                                    panic!("finally(...) expects a function");
                                }

                                Value::Furure(Box::new(value_for_chain))
                            }))
                        }

                        other => panic!("Property '{}' not supported on Furure", other),
                    }
                }

                // ---------------------------------
                // Fallback
                // ---------------------------------
                other => panic!("Property '{}' not supported on {:?}", prop_name, other),
            }
        }

        // ---------------------------------------------------------------------
        // Property Set: obj.prop = value
        // ---------------------------------------------------------------------
        Expr::Set { object, name, value } => {
            let target = eval_expr(*object, env.clone());
            let val = eval_expr(*value, env);

            match target {
                Value::Object { fields } => {
                    fields.borrow_mut().insert(name, val.clone());
                    val
                }

                other => {
                    panic!("Cannot assign property on non-object value: {:?}", other);
                }
            }
        }

        // ---------------------------------------------------------------------
        // Lambda
        // ---------------------------------------------------------------------
        Expr::Lambda { params, body } => {
            let captured_env = env.clone();

            Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                let local_env = Rc::new(RefCell::new(Environment::new(Some(captured_env.clone()))));

                // Bind parameters
                for (i, name) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Null);
                    local_env.borrow_mut().define_public(name.clone(), val);
                }

                // Execute body
                for stmt in &body {
                    match exec_stmt(stmt.clone(), local_env.clone()) {
                        ExecSignal::None => {}
                        ExecSignal::Return(v) => return v,
                        ExecSignal::Throw(e) => return e,
                    }
                }

                Value::Null
            }))
        }

        // ---------------------------------------------------------------------
        // Postfix Operators: i++, i--
        // ---------------------------------------------------------------------
        Expr::PostIncrement { name } => {
            let current = env.borrow().get(&name, false)
                .unwrap_or_else(|| panic!("Undefined variable '{}'", name));

            if let Value::Number(n) = current {
                let new_val = Value::Number(n + 1.0);
                env.borrow_mut().assign(&name, new_val);
                Value::Number(n)
            } else {
                panic!("++ only allowed on numbers");
            }
        }

        Expr::PostDecrement { name } => {
            let current = env.borrow().get(&name, false)
                .unwrap_or_else(|| panic!("Undefined variable '{}'", name));

            if let Value::Number(n) = current {
                let new_val = Value::Number(n - 1.0);
                env.borrow_mut().assign(&name, new_val);
                Value::Number(n)
            } else {
                panic!("-- only allowed on numbers");
            }
        }

        // ---------------------------------------------------------------------
        // `new` Class Construction
        // ---------------------------------------------------------------------
        Expr::New { class_name, arguments } => {
            // For now, treat `new Foo(a, b)` as sugar for `Foo(a, b)` and let
            // `call_value` decide how to construct instances from class values.
            let call_expr = Expr::Call {
                callee: Box::new(Expr::Identifier(class_name)),
                arguments,
            };

            eval_expr(call_expr, env)
        }

        // ---------------------------------------------------------------------
        // tap() Module Import
        // ---------------------------------------------------------------------
        Expr::Tap { path } => {
            let pval = eval_expr(*path, env);

            let path_str = match pval {
                Value::String(s) => s,
                other => panic!("tap() path must be a string, got {:?}", other),
            };

            // For now, we don't have a full module loader wired in the Rust
            // version. If you want, we can add a loader that:
            //  • resolves the path
            //  • reads + lexes + parses + executes the module
            //  • returns Value::Module { exports, default }
            panic!(
                "tap() is not yet implemented in the Rust interpreter for path '{}'",
                path_str
            );
        }

        Expr::Logical { left, operator, right } => {
            let left_val = eval_expr(*left, env.clone());

            match operator.lexeme.as_str() {
                "||" => {
                    if is_truthy(&left_val) {
                        left_val
                    } else {
                        eval_expr(*right, env)
                    }
                }
                "&&" => {
                    if !is_truthy(&left_val) {
                        left_val
                    } else {
                        eval_expr(*right, env)
                    }
                }
                _ => panic!("Invalid logical operator: {}", operator.lexeme),
            }
        }

        // ---------------------------------------------------------------------
        // Fallback – should be unreachable once all variants handled
        // ---------------------------------------------------------------------
        other => panic!("Unhandled expression variant: {:?}", other),
    }
}

fn values_equal_strict(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,

        // Arrays, objects, functions, classes, instances:
        // strict equality only if they are the SAME reference
        (Value::Array { values: a, .. }, Value::Array { values: b, .. }) => {
            Rc::ptr_eq(a, b)
        }

        (Value::Object { fields: a }, Value::Object { fields: b }) => {
            Rc::ptr_eq(a, b)
        }

        (Value::NativeFunction(a), Value::NativeFunction(b)) => {
            Arc::ptr_eq(a, b)
        }

        // Everything else is strictly unequal
        _ => false,
    }
}