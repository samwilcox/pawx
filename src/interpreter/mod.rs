/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * Interpreter Entry & Runtime Bootstrap
 * -------------------------------------
 * This module is the **primary runtime entrypoint** for the PAWX programming
 * language. It is responsible for:
 * 
 *  - Creating the global execution environment
 *  - Installing all built-in global functions and objects
 *  - Managing the runtime timer system (setTimeout / setInterval)
 *  - Driving the main statement execution loop
 *  - Handling top-level returns and uncaught throws
 * 
 * All actual evaluation logic is delegated to the following submodules:
 * 
 *  - timers.rs      → Timer scheduling and dispatch
 *  - statements.rs → Statement execution (exec_stmt)
 *  - expressions.rs→ Expression evaluation (eval_expr)
 *  - calls.rs       → Function and method invocation
 *  - display.rs     → Value formatting utilities
 *  - classes.rs     → Class & instance behavior
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

pub mod timers;
pub mod statements;
pub mod expressions;
pub mod calls;
pub mod display;
pub mod classes;
pub mod environment;
pub mod helpers;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::Stmt;
use crate::interpreter::environment::Environment;
use crate::value::Value;

use timers::{install_timers, TimerRuntime};
use statements::{exec_stmt, ExecSignal};
use display::value_to_string;

/// Executes a full PAWX program from a list of parsed statements.
pub fn run(statements: Vec<Stmt>) {
    let env = Rc::new(RefCell::new(Environment::new(None)));

    // -------------------------------------------------------------------------
    // Install Timers (MOVED TO timers.rs)
    // -------------------------------------------------------------------------
    let timer_runtime: TimerRuntime = install_timers(env.clone());

    // -------------------------------------------------------------------------
    // Built-in: meow(...)
    // -------------------------------------------------------------------------
    env.borrow_mut().define_public(
        "meow".to_string(),
        Value::NativeFunction(Arc::new(|args: Vec<Value>| -> Value {
            if args.is_empty() {
                println!();
                return Value::Null;
            }

            if let Value::String(format) = &args[0] {
                if format.contains('$') {
                    let mut output = String::new();
                    let mut arg_index = 1;
                    let mut chars = format.chars();

                    while let Some(c) = chars.next() {
                        if c == '$' && arg_index < args.len() {
                            output.push_str(&value_to_string(&args[arg_index]));
                            arg_index += 1;
                        } else {
                            output.push(c);
                        }
                    }

                    println!("{}", output);
                    return Value::Null;
                }
            }

            let mut parts = Vec::new();
            for val in args {
                parts.push(value_to_string(&val));
            }

            println!("{}", parts.join(" "));
            Value::Null
        })),
    );

    // -------------------------------------------------------------------------
    // Standard Global Objects
    // -------------------------------------------------------------------------
    env.borrow_mut().define_public("Error".to_string(), Value::NativeFunction(Arc::new(|args| {
        let message = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "Unknown error".to_string(),
        };
        Value::Error { message }
    })));

    env.borrow_mut().define_public("Array".to_string(), crate::prototypes::array::create_global_array_object());
    env.borrow_mut().define_public(
    "String".to_string(), Value::Object { fields: Rc::new(RefCell::new(crate::prototypes::string::create_global_string_object())) });
    env.borrow_mut().define_public("Math".to_string(), crate::prototypes::math::create_global_math_value());
    env.borrow_mut().define_public("Time".to_string(), crate::prototypes::time::create_global_time_value());
    env.borrow_mut().define_public("Date".to_string(), crate::prototypes::time::create_global_time_value());
    env.borrow_mut().define_public("Http".to_string(), crate::prototypes::http::create_global_http_object());
    env.borrow_mut().define_public("String".to_string(), Value::Object { fields: Rc::new(RefCell::new(crate::prototypes::string::create_global_string_object())) });
    env.borrow_mut().define_public("Regex".to_string(), Value::Object { fields: Rc::new(RefCell::new(crate::prototypes::regex::create_global_regex_object())) });
    env.borrow_mut().define_public("Fs".to_string(), crate::prototypes::fs::create_fs_global());

    // -------------------------------------------------------------------------
    // Main Execution Loop (WITH TIMER PUMP)
    // -------------------------------------------------------------------------
    for stmt in statements {
        match exec_stmt(stmt, env.clone()) {
            ExecSignal::None => {}
            ExecSignal::Return(_) => break,
            ExecSignal::Throw(err) => panic!("Uncaught Pawx error: {:?}", err),
        }

        // Timer pump delegated to timers.rs
        timers::pump_timers(&timer_runtime);
    }

    // Final drain
    timers::pump_timers(&timer_runtime);
}

/// Executes a module inside an existing environment.
pub fn run_in_env(statements: Vec<Stmt>, env: Rc<RefCell<Environment>>) {
    for stmt in statements {
        match exec_stmt(stmt, env.clone()) {
            ExecSignal::None => {}
            ExecSignal::Return(_) => break,
            ExecSignal::Throw(err) => panic!("Uncaught Pawx error in module: {:?}", err),
        }
    }
}