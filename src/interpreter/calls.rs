/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 *
 * Call Dispatch & Invocation Engine
 * ---------------------------------
 * This module defines the **core runtime call semantics** for the PAWX
 * interpreter. It is responsible for:
 *
 *  - Calling class constructors (`new`)
 *  - Invoking instance methods with proper `this` binding
 *  - Executing native runtime functions
 *  - Executing user-defined PAWX functions
 *  - Handling default parameters
 *  - Converting `throw` into returnable error values
 *
 * This module is one of the most **security-sensitive runtime layers**
 * because it controls execution boundaries, scope chaining, and error flow.
 *
 * --------------------------------------------------------------------------
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
 * ==========================================================================
 */

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::Expr;
use crate::interpreter::environment::{Environment, FunctionDef};
use crate::value::Value;

use crate::interpreter::statements::exec_stmt;
use crate::interpreter::expressions::eval_expr;
use crate::interpreter::ExecSignal;

/// Executes a **constructor or class method body** using already-evaluated
/// argument values.
///
/// This is used primarily for:
/// - Class constructors (`new`)
/// - Setter methods
///
/// This function:
/// 1. Creates a new execution scope chained to the caller
/// 2. Binds `this` to the instance
/// 3. Binds all evaluated parameters
/// 4. Executes the function body
///
/// Any `return` value inside a constructor is **ignored** (JS-style).
/// Any `throw` immediately aborts execution.
///
/// # Panics
/// - If the constructor throws an error
fn call_method_value(
    func: FunctionDef,
    instance: Value,
    args: Vec<Value>,
    env: Rc<RefCell<Environment>>,
) {
    // Create a new environment for the method call
    // This environment is lexically chained to the parent scope
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    // Bind `this` so the method can modify instance state
    func_env
        .borrow_mut()
        .define_public("this".to_string(), instance);

    // Bind evaluated parameters into the local scope
    for (i, param) in func.params.iter().enumerate() {
        let val = args.get(i).cloned().unwrap_or(Value::Null);
        func_env
            .borrow_mut()
            .define_public(param.name.clone(), val);
    }

    // Execute constructor / method body
    for stmt in func.body {
        match exec_stmt(stmt, func_env.clone()) {
            ExecSignal::None => {}

            // Constructors discard return values
            ExecSignal::Return(_) => break,

            // Hard-stop on thrown errors
            ExecSignal::Throw(e) => {
                panic!("Constructor threw error: {:?}", e);
            }
        }
    }
}

/// Invokes a **standard instance method** using unevaluated expression
/// arguments.
///
/// This function:
/// 1. Creates a local method scope
/// 2. Binds `this` to the instance
/// 3. Evaluates and binds arguments
/// 4. Executes the method body
/// 5. Returns the first encountered `return` value
///
/// If no return statement is encountered, `null` is returned.
///
/// # Returns
/// - The method's return value
/// - `null` if no return is executed
fn call_method(
    func: FunctionDef,
    instance: Value,
    args: Vec<Expr>,
    env: Rc<RefCell<Environment>>,
) -> Value {
    // Create method execution scope
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env))));

    // Bind `this`
    func_env.borrow_mut().define_public("this".to_string(), instance);

    // Bind parameters (evaluated lazily from expressions)
    for (i, param) in func.params.iter().enumerate() {
        let val = if i < args.len() {
            eval_expr(args[i].clone(), func_env.clone())
        } else {
            Value::Null
        };

        func_env
            .borrow_mut()
            .define_public(param.name.clone(), val);
    }

    // Execute body and return on first `return`
    for stmt in func.body {
        if let ExecSignal::Return(v) = exec_stmt(stmt, func_env.clone()) {
            return v;
        }
    }

    Value::Null
}

/// Executes a **callable runtime value**, such as native functions.
///
/// This function:
/// - Evaluates all argument expressions
/// - Dispatches directly into a native Rust function
///
/// # Panics
/// - If the callee is not callable
pub fn call_value(
    callee_val: Value,
    arguments: Vec<Expr>,
    env: Rc<RefCell<Environment>>,
) -> Value {
    // Evaluate all argument expressions eagerly
    let mut args = Vec::new();
    for arg in arguments {
        args.push(eval_expr(arg, env.clone()));
    }

    match callee_val {
        // Native functions work as usual
        Value::NativeFunction(f) => f(args),

        // Allow non-function values to pass through safely (for chaining)
        other => {
            // This fixes:
            //   res.status(200).json({...});
            // where `.json()` returns an object and should NOT be called again.
            other
        }
    }
}

/// Executes a **user-defined PAWX function**.
///
/// This is the primary execution path for:
/// - `purr` functions
/// - lambdas
/// - exported functions
///
/// Supports:
/// - Default parameters
/// - Nested returns
/// - Error propagation via `throw`
///
/// # Returns
/// - The function's return value
/// - `null` if no return was encountered
pub fn call_user_function(
    func: FunctionDef,
    arg_vals: Vec<Value>,
    env: Rc<RefCell<Environment>>,
) -> Value {
    // Create function-local scope chained to the outer environment
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    // Bind parameters with default support
    for (i, param) in func.params.iter().enumerate() {
        let value = if i < arg_vals.len() {
            arg_vals[i].clone()
        } else if let Some(default_expr) = &param.default {
            eval_expr(default_expr.clone(), func_env.clone())
        } else {
            Value::Null
        };

        func_env
            .borrow_mut()
            .define_public(param.name.clone(), value);
    }

    // Execute function body
    for stmt in func.body {
        match exec_stmt(stmt, func_env.clone()) {
            ExecSignal::None => {}

            // Hard return
            ExecSignal::Return(v) => return v,

            // Convert thrown value into returned error object
            ExecSignal::Throw(e) => return e,
        }
    }

    Value::Null
}