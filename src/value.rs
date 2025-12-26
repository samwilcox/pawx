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
    Object {
        fields: Rc<RefCell<HashMap<String, Value>>>,
    },

    // Class definition:
    Class {
        name: String,
        methods: HashMap<String, FunctionDef>,
        getters: HashMap<String, FunctionDef>,
        setters: HashMap<String, FunctionDef>,
        fields: HashMap<String, Value>,
    },

    // Instance of a class:
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
    Module {
        exports: HashMap<String, Value>,
        default: Option<Box<Value>>,
    },

    // Tuple literal
    Tuple(Vec<Value>),

    // Regex literal / constructed regex
    Regex(Regex),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(*n),
            Value::String(s) => Value::String(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Null => Value::Null,

            Value::NativeFunction(f) => Value::NativeFunction(f.clone()),

            Value::Array { values, proto } => Value::Array {
                values: values.clone(),
                proto: proto.clone(),
            },

            Value::Object { fields } => Value::Object {
                fields: fields.clone(),
            },

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

            Value::Furure(inner) => Value::Furure(inner.clone()),

            Value::Error { message } => Value::Error {
                message: message.clone(),
            },

            Value::Module { exports, default } => Value::Module {
                exports: exports.clone(),
                default: default.clone(),
            },

            Value::Tuple(values) => Value::Tuple(values.clone()),

            Value::Regex(r) => Value::Regex(r.clone()),
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

            Value::Regex(r) => write!(f, "[Regex /{}/]", r.as_str()),

            Value::Object { .. } => write!(f, "[Object]"),

            Value::Array { values, .. } => write!(f, "[Array len={}]", values.borrow().len()),

            Value::Class { name, .. } => write!(f, "[Class {}]", name),

            Value::Instance { class_name, .. } => write!(f, "[Instance {}]", class_name),

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
    /// Returns a stable type name string (useful for errors).
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_)         => "Number",
            Value::String(_)         => "String",
            Value::Bool(_)           => "Bool",
            Value::Null              => "Null",
            Value::Array { .. }      => "Array",
            Value::Object { .. }     => "Object",
            Value::Tuple(_)          => "Tuple",
            Value::Class { .. }      => "Class",
            Value::Instance { .. }   => "Instance",
            Value::NativeFunction(_) => "Function",
            Value::Furure(_)         => "Furure",
            Value::Error { .. }      => "Error",
            Value::Module { .. }     => "Module",
            Value::Regex(_)          => "Regex",
        }
    }

    /// PAWX truthiness (JS-ish, but intentionally simple/stable).
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::String(s) => !s.is_empty(),
            // everything else is truthy
            _ => true,
        }
    }

    /// Human-ish string form for debug/errors (NOT meant to be exact serialization).
    pub fn stringify(&self) -> String {
        match self {
            Value::Number(n) => {
                // keep it simple; you can add nicer formatting later
                n.to_string()
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),

            Value::Regex(r) => format!("/{}/", r.as_str()),

            Value::Tuple(v) => {
                let inner = v.iter().map(|x| x.stringify()).collect::<Vec<_>>().join(", ");
                format!("({})", inner)
            }

            Value::Array { values, .. } => {
                let inner = values
                    .borrow()
                    .iter()
                    .map(|x| x.stringify())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", inner)
            }

            Value::Object { .. } => "[object Object]".to_string(),
            Value::NativeFunction(_) => "[function]".to_string(),
            Value::Class { name, .. } => format!("[class {}]", name),
            Value::Instance { class_name, .. } => format!("[instance {}]", class_name),
            Value::Module { .. } => "[module]".to_string(),
            Value::Furure(_) => "[furure]".to_string(),
            Value::Error { message } => format!("Error({})", message),
        }
    }

    /// Loose equality (`==`) — conservative:
    /// - primitives compare by value
    /// - tuples deep-compare
    /// - everything else: false unless same discriminant and both Null (handled above)
    pub fn equals_loose(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,

            (Value::Tuple(x), Value::Tuple(y)) => {
                if x.len() != y.len() {
                    return false;
                }
                for (a, b) in x.iter().zip(y.iter()) {
                    if !Value::equals_loose(a, b) {
                        return false;
                    }
                }
                true
            }

            _ => false,
        }
    }

    /// Strict equality (`===`) — JS-style identity for reference types:
    /// - primitives compare by value
    /// - tuples deep-compare
    /// - arrays/objects/functions compare by pointer identity
    pub fn equals_strict(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,

            (Value::Tuple(x), Value::Tuple(y)) => {
                if x.len() != y.len() {
                    return false;
                }
                for (a, b) in x.iter().zip(y.iter()) {
                    if !Value::equals_strict(a, b) {
                        return false;
                    }
                }
                true
            }

            (Value::Array { values: a, .. }, Value::Array { values: b, .. }) => Rc::ptr_eq(a, b),

            (Value::Object { fields: a }, Value::Object { fields: b }) => Rc::ptr_eq(a, b),

            (Value::NativeFunction(a), Value::NativeFunction(b)) => Arc::ptr_eq(a, b),

            // You can decide how strict should behave for Regex:
            // Here: equal if pattern string matches.
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),

            // Classes/Instances/Modules/Furure:
            // treat as identity types unless you want deeper behavior later.
            _ => false,
        }
    }

    /// Attempts to extract a String reference from a Value.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Convenience version that panics with a useful error.
    pub fn expect_string(&self) -> &str {
        self.as_string()
            .unwrap_or_else(|| panic!("Expected String, got {:?}", self))
    }

    /// Attempts to extract a number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn expect_number(&self) -> f64 {
        self.as_number()
            .unwrap_or_else(|| panic!("Expected Number, got {:?}", self))
    }

    pub fn to_pawx_string(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),

            Value::Regex(r) => format!("/{}/", r.as_str()),

            Value::Tuple(values) => {
                let inner = values
                    .iter()
                    .map(|v| v.to_pawx_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }

            Value::Array { values, .. } => {
                let inner = values
                    .borrow()
                    .iter()
                    .map(|v| v.to_pawx_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", inner)
            }

            Value::Object { .. } => "[object]".to_string(),

            Value::NativeFunction(_) => "[function]".to_string(),

            Value::Class { name, .. } => format!("[class {}]", name),

            Value::Instance { class_name, .. } => format!("[instance {}]", class_name),

            Value::Module { .. } => "[module]".to_string(),

            Value::Furure(_) => "[furure]".to_string(),

            Value::Error { message } => message.clone(),
        }
    }

    fn native_to_string(this: Value) -> Value {
        Value::String(this.stringify())
    }
}