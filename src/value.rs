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
use std::sync::{Arc, Mutex};

use crate::environment::{Environment, FunctionDef};

pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    NativeFunction(Arc<dyn Fn(Vec<Value>) -> Value>),

    // For pride objects
    Object {
        fields: HashMap<String, Value>,
    },

    // ✅ PATCHED: Class now supports setters
    Class {
        name: String,
        methods: HashMap<String, FunctionDef>,
        getters: HashMap<String, FunctionDef>,
        setters: HashMap<String, FunctionDef>,   // ✅ ADD
        fields: HashMap<String, Value>,
    },

    // ✅ PATCHED: Instance now supports setters
    Instance {
        class_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: HashMap<String, FunctionDef>,
        getters: HashMap<String, FunctionDef>,
        setters: HashMap<String, FunctionDef>,   // ✅ ADD
    },
    
    Furure(Box<Value>),

    Error {
        message: String,
    },

    Module {
        name: String,
        env: Rc<RefCell<Environment>>,
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(*n),
            Value::String(s) => Value::String(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Null => Value::Null,

            Value::NativeFunction(f) => Value::NativeFunction(f.clone()),

            Value::Object { fields } => Value::Object {
                fields: fields.clone(),
            },

            // ✅ PATCHED: clone for Class WITH setters
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
                setters: setters.clone(),   // ✅ ADD
                fields: fields.clone(),
            },

            // ✅ PATCHED: clone for Instance WITH setters
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
                setters: setters.clone(),   // ✅ ADD
            },

            Value::Furure(inner) => Value::Furure(inner.clone()),

            Value::Error { message } => Value::Error {
                message: message.clone(),
            },
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

            Value::Object { .. } => write!(f, "[Object]"),

            Value::Class { name, .. } => write!(f, "[Class {}]", name),

            Value::Instance { class_name, .. } => {
                write!(f, "[Instance {}]", class_name)
            }

            Value::Furure(inner) => write!(f, "[Furure {:?}]", inner),

            Value::Error { message } => write!(f, "Error({})", message),
        }
    }
}