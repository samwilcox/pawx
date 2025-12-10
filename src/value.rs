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

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use regex::Regex;

use crate::interpreter::environment::FunctionDef;

/// PAWX runtime value representation.
///
/// This is the core type that flows through the interpreter.
/// Every expression ultimately evaluates to one of these.
pub enum Value {
    // Primitive scalars
    Number(f64),
    String(String),
    Bool(bool),
    Null,

    // Native host function:
    // takes a vector of PAWX Values → returns a PAWX Value
    NativeFunction(Arc<dyn Fn(Vec<Value>) -> Value>),

    // Dynamic array (JS-style)
    // - Shared across copies using Rc<RefCell<_>>
    // - Prototype table holds methods (push, map, etc.)
    Array {
        values: Rc<RefCell<Vec<Value>>>,
        proto: HashMap<String, Value>,
    },

    // Pride / object literal / plain object:
    // - Shared mutable field map
    // - Used for:
    //   * { ... } literals
    //   * Pride results
    Object {
        fields: Rc<RefCell<HashMap<String, Value>>>,
    },

    // Class definition:
    // - Contains methods, getters, setters, and default fields
    Class {
        name: String,
        methods: HashMap<String, FunctionDef>,
        getters: HashMap<String, FunctionDef>,
        setters: HashMap<String, FunctionDef>,
        fields: HashMap<String, Value>,
    },

    // Instance of a class:
    // - Shared fields (Rc<RefCell<_>>) so mutation is visible everywhere
    // - Methods/getters/setters copied from class at creation time
    Instance {
        class_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: HashMap<String, FunctionDef>,
        getters: HashMap<String, FunctionDef>,
        setters: HashMap<String, FunctionDef>,
    },

    // Simple "future" / promise-like wrapper
    Furure(Box<Value>),

    // Error wrapper used by the runtime and Error() constructor
    Error {
        message: String,
    },

    // Module value produced by tap()
    // - exports: named exports
    // - default: optional default export
    Module {
        exports: HashMap<String, Value>,
        default: Option<Box<Value>>,
    },

    // Tuple literal
    Tuple(Vec<Value>),

    Regex(regex::Regex),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(*n),
            Value::String(s) => Value::String(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Null => Value::Null,

            // Native function: clone the Arc handle
            Value::NativeFunction(f) => Value::NativeFunction(f.clone()),

            // Regex: regex::Regex is already cloneable
            Value::Regex(r) => Value::Regex(r.clone()),

            // Shared object fields
            Value::Object { fields } => Value::Object {
                fields: fields.clone(),
            },

            // Shared array values, cloned proto
            Value::Array { values, proto } => Value::Array {
                values: values.clone(),
                proto: proto.clone(),
            },

            // Class: just clone maps and name
            Value::Class {
                name,
                methods,
                getters,
                setters,
                fields,
            } => Value::Class {
                name: name.clone(),
                methods: methods.clone(),
                getters: getters.clone(),
                setters: setters.clone(),
                fields: fields.clone(),
            },

            // Instance: share fields, clone method maps
            Value::Instance {
                class_name,
                fields,
                methods,
                getters,
                setters,
            } => Value::Instance {
                class_name: class_name.clone(),
                fields: fields.clone(),
                methods: methods.clone(),
                getters: getters.clone(),
                setters: setters.clone(),
            },

            // Furure: clone the boxed inner value
            Value::Furure(inner) => Value::Furure(inner.clone()),

            // Error wrapper
            Value::Error { message } => Value::Error {
                message: message.clone(),
            },

            // Module: clone exports + default
            Value::Module { exports, default } => Value::Module {
                exports: exports.clone(),
                default: default.clone(),
            },

            // Tuple: deep clone values
            Value::Tuple(values) => Value::Tuple(values.clone()),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({})", s),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Null => write!(f, "Null"),

            Value::NativeFunction(_) => write!(f, "[NativeFunction]"),

            // ✅ Regex display support
            Value::Regex(r) => write!(f, "[Regex /{}/]", r.as_str()),

            Value::Object { .. } => write!(f, "[Object]"),

            Value::Array { values, .. } => {
                write!(f, "[Array len={}]", values.borrow().len())
            }

            Value::Class { name, .. } => write!(f, "[Class {}]", name),

            Value::Instance { class_name, .. } => {
                write!(f, "[Instance {}]", class_name)
            }

            Value::Module { exports, default } => {
                let default_str = if default.is_some() { " + default" } else { "" };
                write!(f, "[Module {} exports{}]", exports.len(), default_str)
            }

            Value::Furure(inner) => write!(f, "[Furure {:?}]", inner),

            Value::Error { message } => write!(f, "Error({})", message),

            Value::Tuple(values) => write!(f, "[Tuple {:?}]", values),
        }
    }
}

impl Value {
    /// Strict equality for PAWX runtime (similar to JS === semantics)
    pub fn pawx_equals(a: &Value, b: &Value) -> bool {
        match (a, b) {
            // Numbers
            (Value::Number(x), Value::Number(y)) => x == y,

            // Strings
            (Value::String(x), Value::String(y)) => x == y,

            // Booleans
            (Value::Bool(x), Value::Bool(y)) => x == y,

            // Null
            (Value::Null, Value::Null) => true,

            // Tuples (deep compare)
            (Value::Tuple(a), Value::Tuple(b)) => {
                if a.len() != b.len() {
                    return false;
                }

                for (x, y) in a.iter().zip(b.iter()) {
                    if !Value::pawx_equals(x, y) {
                        return false;
                    }
                }

                true
            }

            // Everything else is not strictly equal
            _ => false,
        }
    }
}

impl Value {
    /// Attempts to extract a String reference from a Value.
    /// Returns None if the value is not a String.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convenience version that panics with a useful error.
    pub fn expect_string(&self) -> &String {
        self.as_string()
            .unwrap_or_else(|| panic!("Expected String, got {:?}", self))
    }
}