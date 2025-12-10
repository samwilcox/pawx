/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Object Prototype Implementation
 * ==========================================================================
 * 
 * This module defines the native Rust-backed implementation of the
 * JavaScript-style `Object` standard library used by the PAWX runtime.
 * 
 * It provides core object inspection utilities, including:
 *   - Object.keys(obj)
 *   - Object.values(obj)
 *   - Object.entries(obj)
 * 
 * These helpers allow developers to introspect plain PAWX objects in a
 * predictable, JavaScript-compatible way.
 * 
 * All functions in this module are installed once onto the global `Object`
 * namespace and are shared across all PAWX programs.
 * 
 * --------------------------------------------------------------------------
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * GitHub:   https://github.com/samwilcox/pawx
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
 *     https://license.pawx-lang.com
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under these licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * 
 * ==========================================================================
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;
use crate::prototypes::array::create_array_proto;

/// Creates and returns the global `Object` namespace for the PAWX runtime.
///
/// This function installs JavaScript-style static object helper functions:
/// - `Object.keys(obj)`
/// - `Object.values(obj)`
/// - `Object.entries(obj)`
///
/// These functions are globally available and do **not** rely on instance
/// prototypes, matching JavaScript semantics.
///
/// # Returns
/// A fully populated `HashMap<String, Value>` representing the global `Object` object.
pub fn create_global_object_object() -> HashMap<String, Value> {
    let mut object = HashMap::new();

    // ---------------------------------------------------------------------
    // Object Inspection Utilities
    // ---------------------------------------------------------------------

    object.insert(
        "keys".to_string(),
        Value::NativeFunction(Arc::new(object_keys)),
    );

    object.insert(
        "values".to_string(),
        Value::NativeFunction(Arc::new(object_values)),
    );

    object.insert(
        "entries".to_string(),
        Value::NativeFunction(Arc::new(object_entries)),
    );

    object
}

pub fn create_global_object_value() -> Value {
    let object_map = create_global_object_object();

    Value::Object {
        fields: Rc::new(RefCell::new(object_map)),
    }
}


/// Native implementation of `Object.keys()` for PAWX.
///
/// Returns an array containing the **enumerable property names**
/// of the given object.
///
/// # Parameters (via `args`)
/// - `args[0]`: The target object
///
/// # Returns
/// A **PAWX array of strings** representing the object’s keys.
///
/// # Behavior
/// - Only operates on plain objects.
/// - Property order follows insertion order.
///
/// # PAWX Example
/// ```pawx
/// snuggle obj = { a: 1, b: 2 };
/// meow(Object.keys(obj)); // ["a", "b"]
/// ```
pub fn object_keys(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("Object.keys(obj) requires 1 argument");
    }

    match &args[0] {
        Value::Object { fields } => {
            let keys = fields
                .borrow()
                .keys()
                .map(|k| Value::String(k.clone()))
                .collect::<Vec<_>>();

            Value::Array {
                values: std::rc::Rc::new(std::cell::RefCell::new(keys)),
                proto: create_array_proto(),
            }
        }
        _ => panic!("Object.keys() requires an object"),
    }
}

/// Native implementation of `Object.values()` for PAWX.
///
/// Returns an array containing the **enumerable property values**
/// of the given object.
///
/// # Parameters (via `args`)
/// - `args[0]`: The target object
///
/// # Returns
/// A **PAWX array** of the object’s values.
///
/// # Behavior
/// - Values are returned in key insertion order.
///
/// # PAWX Example
/// ```pawx
/// snuggle obj = { a: 1, b: 2 };
/// meow(Object.values(obj)); // [1, 2]
/// ```
pub fn object_values(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("Object.values(obj) requires 1 argument");
    }

    match &args[0] {
        Value::Object { fields } => {
            let values = fields.borrow().values().cloned().collect::<Vec<_>>();

            Value::Array {
                values: std::rc::Rc::new(std::cell::RefCell::new(values)),
                proto: create_array_proto(),
            }
        }
        _ => panic!("Object.values() requires an object"),
    }
}

/// Native implementation of `Object.entries()` for PAWX.
///
/// Returns an array of **`[key, value]` pairs** for each enumerable
/// property in the object.
///
/// # Parameters (via `args`)
/// - `args[0]`: The target object
///
/// # Returns
/// A **PAWX array of arrays**, each containing:
/// `[String, Value]`
///
/// # Behavior
/// - Preserves insertion order.
/// - Fully JS-compatible semantics.
///
/// # PAWX Example
/// ```pawx
/// snuggle obj = { a: 1, b: 2 };
/// meow(Object.entries(obj));
/// // [["a", 1], ["b", 2]]
/// ```
pub fn object_entries(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("Object.entries(obj) requires 1 argument");
    }

    match &args[0] {
        Value::Object { fields } => {
            let entries = fields
                .borrow()
                .iter()
                .map(|(k, v)| {
                    Value::Tuple(vec![
                        Value::String(k.clone()),
                        v.clone(),
                    ])
                })
                .collect::<Vec<_>>();

            Value::Array {
                values: std::rc::Rc::new(std::cell::RefCell::new(entries)),
                proto: create_array_proto(),
            }
        }
        _ => panic!("Object.entries() requires an object"),
    }
}