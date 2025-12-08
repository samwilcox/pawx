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

use crate::ast::{ClassMember, Expr, Stmt};
use crate::environment::{Environment, FunctionDef};
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub fn run(statements: Vec<Stmt>) {
    let env = Rc::new(RefCell::new(Environment::new(None)));

    // Built-in meow()
    env.borrow_mut().define_public(
        "meow".to_string(),
        Value::NativeFunction(Arc::new(|args: Vec<Value>| -> Value {
            if let Some(val) = args.get(0) {
                match val {
                    Value::String(s) => println!("{}", s),
                    Value::Number(n) => println!("{}", n),
                    Value::Bool(b) => println!("{}", b),
                    Value::Null => println!("null"),
                    Value::Object { .. } => println!("[object]"),

                    // ✅ NEW: classes
                    Value::Class { name, .. } => println!("[class {}]", name),

                    // ✅ NEW: instances
                    Value::Instance { class_name, .. } => println!("[instance {}]", class_name),

                    Value::Furure(inner) => println!("[Furure {:?}]", inner),
                    Value::Error { message } => println!("Error({})", message),
                    Value::NativeFunction(_) => println!("[function]"),
                    Value::Module { name, .. } => println!("[module {}]", name),
                }
            }
            Value::Null
        })),
    );

    env.borrow_mut().define_public(
        "Error".to_string(),
        Value::NativeFunction(Arc::new(|args: Vec<Value>| -> Value {
            let message = if let Some(Value::String(s)) = args.get(0) {
                s.clone()
            } else {
                "Unknown error".to_string()
            };

            Value::Error { message }
        })),
    );

    for stmt in statements {
        match exec_stmt(stmt, env.clone()) {
            ExecSignal::None => {}

            ExecSignal::Return(_) => break,

            ExecSignal::Throw(err) => {
                panic!("Uncaught Pawx error: {:?}", err);
            }
        }
    }
}

#[derive(Debug)]
enum ExecSignal {
    None,
    Return(Value),
    Throw(Value),
}

fn exec_stmt(stmt: Stmt, env: Rc<RefCell<Environment>>) -> ExecSignal {
    match stmt {
        Stmt::Expression(expr) => {
            let _ = eval_expr(expr, env);
            ExecSignal::None
        }

        Stmt::PublicVar { name, value } => {
            let val = eval_expr(value, env.clone());
            env.borrow_mut().define_public(name, val);
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

        Stmt::Pride { name, body } => {
            let pride_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

            for s in body {
                if let ExecSignal::Return(v) = exec_stmt(s, pride_env.clone()) {
                    // Return inside pride currently just bubbles up
                    return ExecSignal::Return(v);
                }
            }

            let mut fields = std::collections::HashMap::new();
            for (k, v) in &pride_env.borrow().values {
                fields.insert(k.clone(), v.value.clone());
            }

            env.borrow_mut()
                .define_public(name, Value::Object { fields });

            ExecSignal::None
        }

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
                    if let ExecSignal::Return(v) = exec_stmt(s, env.clone()) {
                        return ExecSignal::Return(v);
                    }
                }
            } else if let Some(else_body) = else_branch {
                for s in else_body {
                    if let ExecSignal::Return(v) = exec_stmt(s, env.clone()) {
                        return ExecSignal::Return(v);
                    }
                }
            }

            ExecSignal::None
        }

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
                    if let ExecSignal::Return(v) = exec_stmt(s.clone(), env.clone()) {
                        return ExecSignal::Return(v);
                    }
                }
            }

            ExecSignal::None
        }

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

        Stmt::Return(expr_opt) => {
            let val = match expr_opt {
                Some(expr) => eval_expr(expr, env),
                None => Value::Null,
            };
            ExecSignal::Return(val)
        }

        Stmt::Nap(expr) => {
            let val = eval_expr(expr, env);

            match val {
                Value::Furure(inner) => {
                    // Await unwrap
                    return ExecSignal::Return(*inner);
                }
                _ => panic!("nap can only be used on a Furure"),
            }
        }

        Stmt::Throw(expr) => {
            let val = eval_expr(expr, env);
            ExecSignal::Throw(val)
        }

        Stmt::Try {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        } => {
            let mut result = ExecSignal::None;

            // RUN TRY BLOCK
            for stmt in try_block {
                match exec_stmt(stmt, env.clone()) {
                    ExecSignal::None => {}

                    ExecSignal::Return(v) => {
                        result = ExecSignal::Return(v);
                        break;
                    }

                    ExecSignal::Throw(err) => {
                        // HANDLE CATCH
                        if let (Some(name), Some(catch_body)) =
                            (catch_param.clone(), catch_block.clone())
                        {
                            let catch_env =
                                Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
                            catch_env
                                .borrow_mut()
                                .define_public(name, err);

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
                            result = ExecSignal::Throw(err);
                        }

                        break;
                    }
                }
            }

            // ALWAYS RUN FINALLY
            if let Some(finally_body) = finally_block {
                for fstmt in finally_body {
                    match exec_stmt(fstmt, env.clone()) {
                        ExecSignal::None => {}

                        other => {
                            return other;
                        }
                    }
                }
            }

            result
        }

        Stmt::Clowder {
            name,
            base: _,
            interfaces: _,
            members,
            is_exported: _,
            is_default: _,
        } => {
            use std::collections::HashMap;

            let mut methods = HashMap::new();
            let mut getters = HashMap::new();
            let mut setters = HashMap::new();   // ✅ ADD THIS
            let mut fields  = HashMap::new();

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
                        let func = FunctionDef {
                            params,
                            body,
                            return_type: None,
                            is_async: false,
                        };
                        methods.insert(name, func);
                    }

                    ClassMember::Getter { name, body, .. } => {
                        let func = FunctionDef {
                            params: vec![],
                            body,
                            return_type: None,
                            is_async: false,
                        };
                        getters.insert(name, func);
                    }

                    ClassMember::Setter { name, param_name, param_type: _, body, .. } => {
                        // create a FunctionDef that takes 1 param with that name
                        let func = FunctionDef {
                            params: vec![crate::ast::Param {
                                name: param_name,
                                default: None,
                                type_annotation: None,
                            }],
                            body,
                            return_type: None,
                            is_async: false,
                        };
                        setters.insert(name, func);               // ✅ store setter
                    }

                    _ => {}
                }
            }

            let class_val = Value::Class {
                name: name.clone(),
                methods,
                getters,
                setters,   // ✅ ADD THIS
                fields,
            };

            env.borrow_mut().define_public(name, class_val);
            ExecSignal::None
        }

        Stmt::Instinct {
            name,
            members: _,
            is_exported: _,
            is_default: _,
        } => {
            // ✅ Interfaces are compile-time only for now (no runtime behavior yet)
            // We just register a placeholder so the name exists.
            env.borrow_mut().define_public(name, Value::Null);
            ExecSignal::None
        }

        Stmt::Export { name, value } => {
            let val = eval_expr(value, env.clone());

            match name {
                // ✅ Named export: export name = value;
                Some(name) => {
                    env.borrow_mut().define_public(name, val);
                }

                // ✅ Default export: export default value;
                None => {
                    env.borrow_mut().define_public("default".to_string(), val);
                }
            }

            ExecSignal::None
        }
    }
}

fn eval_expr(expr: Expr, env: Rc<RefCell<Environment>>) -> Value {
    match expr {
        Expr::Literal(v) => v,

        Expr::Identifier(name) => {
            if name == "this" {
                env.borrow().get("this", false)
                    .unwrap_or_else(|| panic!("'this' used outside of class"))
            } else {
                env.borrow().get(&name, false)
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name))
            }
        }

        Expr::Assign { name, value } => {
            let assigned = eval_expr(*value, env.clone());

            // ✅ Handle: snuggle X = tap(...)
            if let Value::Module { default: Some(default_val), .. } = &assigned {
                let real = *default_val.clone();
                env.borrow_mut().define_public(name, real.clone());
                return real;
            }

            // ✅ Normal assignment
            if !env.borrow_mut().assign(&name, assigned.clone()) {
                panic!("Undefined variable '{}'", name);
            }

            assigned
        }

        Expr::Unary { operator, right } => {
            let r = eval_expr(*right, env);

            match (operator.as_str(), r) {
                ("-", Value::Number(n)) => Value::Number(-n),
                ("!", Value::Bool(b)) => Value::Bool(!b),
                _ => panic!("Invalid unary op"),
            }
        }

        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let l = eval_expr(*left, env.clone());
            let r = eval_expr(*right, env);

            match (l, r, operator.as_str()) {
                // ✅ Number math
                (Value::Number(a), Value::Number(b), "+") => Value::Number(a + b),
                (Value::Number(a), Value::Number(b), "-") => Value::Number(a - b),
                (Value::Number(a), Value::Number(b), "*") => Value::Number(a * b),
                (Value::Number(a), Value::Number(b), "/") => Value::Number(a / b),

                // ✅ Number comparisons
                (Value::Number(a), Value::Number(b), ">")  => Value::Bool(a > b),
                (Value::Number(a), Value::Number(b), "<")  => Value::Bool(a < b),
                (Value::Number(a), Value::Number(b), ">=") => Value::Bool(a >= b),
                (Value::Number(a), Value::Number(b), "<=") => Value::Bool(a <= b),

                // ✅ Number equality
                (Value::Number(a), Value::Number(b), "==") => Value::Bool(a == b),
                (Value::Number(a), Value::Number(b), "!=") => Value::Bool(a != b),

                // ✅ ✅ ✅ STRING EQUALITY (THIS FIXES YOUR CRASH)
                (Value::String(a), Value::String(b), "==") => Value::Bool(a == b),
                (Value::String(a), Value::String(b), "!=") => Value::Bool(a != b),

                // ✅ ✅ ✅ BOOL EQUALITY
                (Value::Bool(a), Value::Bool(b), "==") => Value::Bool(a == b),
                (Value::Bool(a), Value::Bool(b), "!=") => Value::Bool(a != b),

                // ✅ STRICT EQUALITY (===)
                (Value::Number(a), Value::Number(b), "===") => Value::Bool(a == b),
                (Value::String(a), Value::String(b), "===") => Value::Bool(a == b),
                (Value::Bool(a), Value::Bool(b), "===")     => Value::Bool(a == b),

                // ✅ STRICT inequality !== fallback
                (_, _, "!==") => Value::Bool(true),

                // ✅ Different types → false
                (_, _, "===") => Value::Bool(false),

                _ => panic!("Invalid binary op"),
            }
        }

        Expr::Call { callee, arguments } => {
            match *callee {
                Expr::Identifier(name) => {
                    // Try user-defined function first
                    if let Some(func) = env.borrow().get_function(&name) {
                        let mut arg_vals = Vec::new();
                        for arg in arguments {
                            arg_vals.push(eval_expr(arg, env.clone()));
                        }
                        // ✅ clone env so we don't move the original
                        return call_user_function(func, arg_vals, env.clone());
                    } else {
                        // Fallback: treat as value (e.g., native function)
                        let callee_val = env
                            .borrow()
                            .get(&name, false)
                            .unwrap_or_else(|| {
                                panic!("Undefined function or value '{}'", name)
                            });

                        // ✅ also clone env here
                        return call_value(callee_val, arguments, env.clone());
                    }
                }
                other => {
                    let callee_val = eval_expr(other, env.clone());
                    // this one is fine, but cloning for consistency is OK too
                    call_value(callee_val, arguments, env)
                }
            }
        }

        Expr::Get { object, name } => {
            let obj = eval_expr(*object, env.clone());

            match obj {
                // ✅ CLASS INSTANCE PROPERTIES
                Value::Instance { fields, getters, setters, methods, .. } => {

                    // ✅ Getter: c.name
                    if let Some(getter) = getters.get(&name) {
                        return call_method(
                            getter.clone(),
                            Value::Instance {
                                class_name: "".to_string(), // not used here
                                fields,
                                methods,
                                getters,
                                setters,   // ✅ REQUIRED FIX
                            },
                            vec![],
                            env,
                        );
                    }

                    // ✅ Direct field access
                    if let Some(val) = fields.borrow().get(&name) {
                        return val.clone();
                    }

                    // ✅ Method access: c.method
                    if let Some(method) = methods.get(&name) {
                        let method = method.clone();
                        let instance = Value::Instance {
                            class_name: "".to_string(),
                            fields,
                            methods,
                            getters,
                            setters,
                        };
                        let env = env.clone();

                        return Value::NativeFunction(Arc::new(move |_args: Vec<Value>| {
                            call_method(method.clone(), instance.clone(), vec![], env.clone())
                        }));
                    }

                    panic!("Undefined property '{}' on instance", name);
                }

                // ✅ NORMAL OBJECT PROPERTIES
                Value::Object { fields } => {
                    return fields
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| panic!("Undefined property '{}'", name));
                }

                // ✅ FURURE.then / .catch ✅ FULLY `'static` SAFE
                Value::Furure(inner) => {
                    let inner = inner.clone(); // ✅ detach from stack completely

                    if name == "then" {
                        return Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                            if args.len() != 1 {
                                panic!("then() expects one function");
                            }

                            let callback = args[0].clone();
                            let resolved = (*inner).clone();

                            match callback {
                                Value::NativeFunction(f) => {
                                    let result = f(vec![resolved]);
                                    Value::Furure(Box::new(result))
                                }
                                _ => panic!("then() requires a function"),
                            }
                        }));
                    }

                    if name == "catch" {
                        return Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                            if args.len() != 1 {
                                panic!("catch() expects one function");
                            }

                            let callback = args[0].clone();
                            let resolved = (*inner).clone();

                            if let Value::Error { .. } = resolved {
                                match callback {
                                    Value::NativeFunction(f) => {
                                        let result = f(vec![resolved]);
                                        return Value::Furure(Box::new(result));
                                    }
                                    _ => panic!("catch() requires a function"),
                                }
                            }

                            Value::Furure(inner.clone())
                        }));
                    }

                    panic!("Furure has no property '{}'", name);
                }

                _ => panic!("Only objects, instances, and Furure have properties"),
            }
        }

        Expr::Nap(expr) => {
            let val = eval_expr(*expr, env);

            match val {
                Value::Furure(inner) => *inner,  // ✅ Unbox to Value
                _ => panic!("nap can only be used on a Furure"),
            }
        }

        Expr::Grouping(expr) => {
            eval_expr(*expr, env)
        }

        Expr::Lambda { params, body } => {
            use std::sync::Arc;

            let captured_env = env.clone();

            Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
                let local_env = std::rc::Rc::new(std::cell::RefCell::new(
                    crate::environment::Environment::new(Some(captured_env.clone())),
                ));

                for (i, name) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Null);
                    local_env.borrow_mut().define_public(name.clone(), val);
                }

                Value::Null
            }))
        }

        Expr::New { class_name, arguments } => {
            // ✅ 1. Lookup class from Environment
            let class_val = env
                .borrow()
                .get(&class_name, false)
                .unwrap_or_else(|| panic!("Undefined class '{}'", class_name));

            // ✅ 2. Extract class data safely
            let (methods, getters, setters, fields) = match &class_val {
                Value::Class {
                    methods,
                    getters,
                    setters,   // ✅ ADD THIS
                    fields,
                    ..
                } => (
                    methods.clone(),
                    getters.clone(),
                    setters.clone(),   // ✅ ADD THIS
                    fields.clone(),
                ),
                _ => panic!("'{}' is not a class", class_name),
            };

            // ✅ 3. Create instance WITH SHARED MUTABLE FIELDS
            let mut instance = Value::Instance {
                class_name: class_name.clone(),
                fields: Rc::new(RefCell::new(fields)),
                methods: methods.clone(),
                getters: getters.clone(),
                setters: setters.clone(),   // ✅ REQUIRED FIX
            };

            // ✅ 4. Evaluate constructor args
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(eval_expr(arg, env.clone()));
            }

            // ✅ 5. Call constructor if present
            if let Some(constructor) = methods.get("new") {
                call_method_value(
                    constructor.clone(),
                    instance.clone(),
                    arg_values,
                    env.clone(),
                );
            }

            // ✅ 6. Return the mutated instance
            instance
        }

        Expr::Set { object, name, value } => {
            let obj = eval_expr(*object, env.clone());
            let val = eval_expr(*value, env.clone());

            match obj {
                Value::Instance {
                    class_name,
                    fields,
                    methods,
                    getters,
                    setters,
                } => {
                    // ✅ If there's a setter, call it: set name -> (param) { ... }
                    if let Some(setter_def) = setters.get(&name) {
                        let setter_def = setter_def.clone();

                        let instance = Value::Instance {
                            class_name,
                            fields,
                            methods,
                            getters,
                            setters,
                        };

                        // call setter with `this = instance` and [val]
                        call_method_value(
                            setter_def,
                            instance,
                            vec![val.clone()],
                            env.clone(),
                        );

                        val
                    } else {
                        // ✅ Fallback: direct field write (no setter defined)
                        fields.borrow_mut().insert(name, val.clone());
                        val
                    }
                }

                _ => panic!("Only instances support field assignment"),
            }
        }

        Expr::Tap { path } => {
            use std::fs;

            let path_val = eval_expr(*path, env.clone());

            let path_str = match path_val {
                Value::String(s) => s,
                _ => panic!("tap() expects a string path"),
            };

            let real_path = if path_str.ends_with(".px") {
                path_str
            } else {
                format!("{}.px", path_str)
            };

            let source = fs::read_to_string(&real_path)
                .unwrap_or_else(|_| panic!("Cannot tap file '{}'", real_path));

            let tokens = crate::lexer::tokenize(&source);
            let statements = crate::parser::parse(tokens);

            // ✅ Create module scope
            let module_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

            crate::interpreter::run_in_env(statements, module_env.clone());

            // ✅ Collect exports
            let mut exports = HashMap::new();
            let mut default_export = None;

            for (name, val) in &module_env.borrow().values {
                if name == "default" {
                    default_export = Some(Box::new(val.value.clone()));
                } else {
                    exports.insert(name.clone(), val.value.clone());
                }
            }

            Value::Module {
                exports,
                default: default_export,
            }
        }
    }
}

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

fn call_method_value(
    func: FunctionDef,
    instance: Value,
    args: Vec<Value>,
    env: Rc<RefCell<Environment>>,  // ✅ use Environment here
) {
    // Create a new environment for the constructor, chained to the outer env
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    // Bind `this`
    func_env
        .borrow_mut()
        .define_public("this".to_string(), instance);

    // Bind parameters from the already-evaluated `args`
    for (i, param) in func.params.iter().enumerate() {
        let val = args.get(i).cloned().unwrap_or(Value::Null);
        func_env
            .borrow_mut()
            .define_public(param.name.clone(), val);
    }

    // Execute constructor body
    for stmt in func.body {
        match exec_stmt(stmt, func_env.clone()) {
            ExecSignal::None => {}
            ExecSignal::Return(_) => break, // ignore return value for constructor
            ExecSignal::Throw(e) => {
                panic!("Constructor threw error: {:?}", e);
            }
        }
    }
}

fn call_method(
    func: FunctionDef,
    instance: Value,
    args: Vec<Expr>,
    env: Rc<RefCell<Environment>>,
) -> Value {
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env))));

    func_env.borrow_mut().define_public("this".to_string(), instance);

    for (i, param) in func.params.iter().enumerate() {
        let val = if i < args.len() {
            eval_expr(args[i].clone(), func_env.clone())
        } else {
            Value::Null
        };

        func_env.borrow_mut()
            .define_public(param.name.clone(), val);
    }

    for stmt in func.body {
        if let ExecSignal::Return(v) = exec_stmt(stmt, func_env.clone()) {
            return v;
        }
    }

    Value::Null
}

fn call_value(
    callee_val: Value,
    arguments: Vec<Expr>,
    env: Rc<RefCell<Environment>>,
) -> Value {
    let mut args = Vec::new();
    for arg in arguments {
        args.push(eval_expr(arg, env.clone()));
    }

    match callee_val {
        Value::NativeFunction(f) => f(args),
        _ => panic!("Can only call functions"),
    }
}

fn call_user_function(
    func: FunctionDef,
    arg_vals: Vec<Value>,
    env: Rc<RefCell<Environment>>,
) -> Value {
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    // Bind parameters
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

    // Execute body
    for stmt in func.body {
        match exec_stmt(stmt, func_env.clone()) {
            ExecSignal::None => {}

            ExecSignal::Return(v) => {
                return v;
            }

            // ✅ CONVERT Throw → Value
            ExecSignal::Throw(e) => {
                return e;
            }
        }
    }

    Value::Null
}