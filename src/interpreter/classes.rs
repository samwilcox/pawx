/*
 * =============================================================================
 *  PAWX - Code with Claws!
 * =============================================================================
 *
 *  File:     classes.rs
 *  Purpose:  Runtime Class System for the PAWX Programming Language.
 *            Implements clowder (class) construction, instance creation,
 *            method dispatch, getters/setters, and `this` binding.
 *
 *  Author:   Sam Wilcox
 *  Email:    sam@pawx-lang.com
 *  Website:  https://www.pawx-lang.com
 *  GitHub:   https://github.com/samwilcox/pawx
 *
 * -----------------------------------------------------------------------------
 *  License:
 * -----------------------------------------------------------------------------
 *  This file is part of the PAWX programming language project.
 *
 *  PAWX is dual-licensed under the terms of:
 *    - The MIT License
 *    - The Apache License, Version 2.0
 *
 *  You may choose either license to govern your use of this software.
 *
 *  Full license text available at:
 *      https://license.pawx-lang.com
 *
 * -----------------------------------------------------------------------------
 *  Warranty Disclaimer:
 * -----------------------------------------------------------------------------
 *  Unless required by applicable law or agreed to in writing, this software is
 *  distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 *  either express or implied.
 *
 * =============================================================================
 */

use crate::ast::{ClassMember, Expr, Param};
use crate::error::PawxError;
use crate::interpreter::environment::{Environment, FunctionDef};
use crate::span::Span;
use crate::value::Value;
use crate::interpreter::expressions::{eval_expr};
use crate::interpreter::statements::{exec_stmt, ExecSignal};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// ==========================================================================
/// CLASS CONSTRUCTION
/// ==========================================================================

/// Builds a complete runtime `Value::Class` from a parsed `clowder` AST node.
///
/// This function:
/// - Extracts all fields
/// - Registers all methods
/// - Registers getters and setters
/// - Produces the final executable `Value::Class` object
///
/// # Parameters
/// - `name` - Class name
/// - `members` - All class members parsed from the AST
/// - `env` - Current runtime environment
///
/// # Returns
/// A fully constructed `Value::Class`
pub fn build_class_value(
    name: String,
    members: Vec<ClassMember>,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, PawxError> {
    let mut methods  = HashMap::new();
    let mut getters  = HashMap::new();
    let mut setters  = HashMap::new();
    let mut fields   = HashMap::new();

    for member in members {
        match member {
            ClassMember::Field { name, value, .. } => {
                let val = if let Some(expr) = value {
                    eval_expr(expr, env.clone())
                } else {
                    Ok(Value::Null)
                };

                fields.insert(name, val?);
            }

            ClassMember::Method { name, params, body, .. } => {
                let func = FunctionDef {
                    params,
                    body,
                    return_type: None,
                    is_async: false,
                    name_span: Span::new(0, 0),
                };
                methods.insert(name, func);
            }

            ClassMember::Getter { name, body, .. } => {
                let func = FunctionDef {
                    params: vec![],
                    body,
                    return_type: None,
                    is_async: false,
                    name_span: Span::new(0, 0),
                };
                getters.insert(name, func);
            }

            ClassMember::Setter { name, param_name, body, .. } => {
                let func = FunctionDef {
                    params: vec![Param {
                        name: param_name,
                        default: None,
                        type_annotation: None,
                    }],
                    body,
                    return_type: None,
                    is_async: false,
                    name_span: Span::new(0, 0),
                };
                setters.insert(name, func);
            }

            _ => {}
        }
    }

    Ok(Value::Class {
        name,
        methods,
        getters,
        setters,
        fields,
    })
}

/// ==========================================================================
/// INSTANCE CONSTRUCTION
/// ==========================================================================

/// Constructs a new runtime instance of a class using `new Class(...)`.
///
/// This function:
/// - Evaluates constructor arguments
/// - Creates an instance with shared mutable fields
/// - Binds and executes the constructor (`new`) if present
///
/// # Parameters
/// - `class_name` - Name of the class
/// - `arguments` - Constructor arguments
/// - `env` - Current runtime environment
///
/// # Returns
/// A fully initialized `Value::Instance`
pub fn construct_instance(
    class_name: String,
    arguments: Vec<Expr>,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, PawxError> {
    let class_val = env
        .borrow()
        .get(&class_name, false)
        .ok_or_else(|| PawxError::runtime_error(
            format!("Undefined class '{}'", class_name),
            Span::new(0, 0),
        ))?;

    let (methods, getters, setters, fields) = match &class_val {
        Value::Class { methods, getters, setters, fields, .. } => (
            methods.clone(),
            getters.clone(),
            setters.clone(),
            fields.clone(),
        ),
        _ => {
            return Err(PawxError::runtime_error(
                format!("'{}' is not a class", class_name),
                Span::new(0, 0),
            ));
        }
    };

    let instance = Value::Instance {
        class_name: class_name.clone(),
        fields: Rc::new(RefCell::new(fields)),
        methods: methods.clone(),
        getters: getters.clone(),
        setters: setters.clone(),
    };

    // build Vec<Value> (NOT Vec<Result<...>>)
    let mut arg_values = Vec::new();
    for arg in arguments {
        arg_values.push(eval_expr(arg, env.clone())?);
    }

    if let Some(constructor) = methods.get("new") {
        call_method_value(constructor.clone(), instance.clone(), arg_values, env.clone())?;
    }

    Ok(instance)
}

/// ==========================================================================
/// INSTANCE PROPERTY ACCESS
/// ==========================================================================

/// Resolves property access on a class instance (`obj.property`).
///
/// Supports:
/// - Getters
/// - Direct fields
/// - Methods (returned as bound native functions)
///
/// # Parameters
/// - `instance` - Target object
/// - `name` - Property name
/// - `env` - Runtime environment
///
/// # Returns
/// The resolved property value
pub fn get_instance_property(
    instance: Value,
    name: String,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, PawxError> {
    match instance {
        Value::Instance {
            fields,
            methods,
            getters,
            setters,
            ..
        } => {
            // Getter
            if let Some(getter) = getters.get(&name) {
                return call_method(
                    getter.clone(),
                    Value::Instance {
                        class_name: "".into(),
                        fields,
                        methods,
                        getters,
                        setters,
                    },
                    vec![],
                    env,
                );
            }

            // Direct field
            if let Some(val) = fields.borrow().get(&name) {
                return Ok(val.clone());
            }

            // Method → return a bound native function
            if let Some(method) = methods.get(&name) {
                let method = method.clone();
                let instance = Value::Instance {
                    class_name: "".into(),
                    fields,
                    methods,
                    getters,
                    setters,
                };

                return Ok(Value::NativeFunction(std::sync::Arc::new(
                    move |_args| {
                        match call_method(
                            method.clone(),
                            instance.clone(),
                            vec![],
                            env.clone(),
                        ) {
                            Ok(v) => v,
                            Err(e) => Value::Error { message: e.message },
                        }
                    },
                )));
            }

            Err(PawxError::runtime_error(
                format!("Undefined property '{}' on instance", name),
                Span::new(0, 0),
            ))
        }

        _ => Err(PawxError::runtime_error(
            "Property access only valid on class instances".to_string(),
            Span::new(0, 0),
        )),
    }
}

/// ==========================================================================
/// INSTANCE PROPERTY ASSIGNMENT
/// ==========================================================================

/// Assigns a value to a property on a class instance (`obj.property = value`).
///
/// If a setter exists, it is executed instead of direct assignment.
///
/// # Returns
/// The assigned value
pub fn set_instance_property(
    instance: Value,
    name: String,
    value: Value,
    env: Rc<RefCell<Environment>>,
) -> Value {
    match instance {
        Value::Instance {
            class_name,
            fields,
            methods,
            getters,
            setters,
        } => {
            if let Some(setter_def) = setters.get(&name) {
                call_method_value(
                    setter_def.clone(),
                    Value::Instance {
                        class_name,
                        fields,
                        methods,
                        getters,
                        setters,
                    },
                    vec![value.clone()],
                    env,
                );
                value
            } else {
                fields.borrow_mut().insert(name, value.clone());
                value
            }
        }

        _ => panic!("Only instances support field assignment"),
    }
}

/// ==========================================================================
/// METHOD & CONSTRUCTOR EXECUTION
/// ==========================================================================

/// Executes a class constructor or setter method with pre-evaluated arguments.
pub fn call_method_value(
    func: FunctionDef,
    instance: Value,
    args: Vec<Value>,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, PawxError> {
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env))));

    func_env
        .borrow_mut()
        .define_public("this".to_string(), instance);

    for (i, param) in func.params.iter().enumerate() {
        let val = args.get(i).cloned().unwrap_or(Value::Null);
        func_env
            .borrow_mut()
            .define_public(param.name.clone(), val);
    }

    for stmt in func.body {
        match exec_stmt(stmt, func_env.clone())? {
            ExecSignal::None => {}
            ExecSignal::Return(value) => return Ok(value),
            ExecSignal::Throw(value) => {
                return Err(PawxError::runtime_error(
                    format!("Uncaught exception: {}", value.stringify()),
                    Span::new(0, 0), // until you plumb real stmt spans
                ));
            }
        }
    }

    Ok(Value::Null)
}

/// Executes a class method and returns the return value.
pub fn call_method(
    func: FunctionDef,
    instance: Value,
    args: Vec<Expr>,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, PawxError> {
    let func_env = Rc::new(RefCell::new(Environment::new(Some(env))));

    // Bind `this`
    func_env
        .borrow_mut()
        .define_public("this".to_string(), instance);

    // Bind parameters
    for (i, param) in func.params.iter().enumerate() {
        let val = if i < args.len() {
            eval_expr(args[i].clone(), func_env.clone())
        } else {
            Ok(Value::Null)
        }?;

        func_env
            .borrow_mut()
            .define_public(param.name.clone(), val);
    }

    // Execute body
    for stmt in func.body {
        match exec_stmt(stmt, func_env.clone())? {
            ExecSignal::None => {}

            ExecSignal::Return(v) => {
                return Ok(v);
            }

            ExecSignal::Throw(v) => {
                return Err(PawxError::runtime_error(
                    format!("Uncaught exception: {}", v.stringify()),
                    Span::new(0, 0),
                ));
            }
        }
    }

    Ok(Value::Null)
}