/*
 * =============================================================================
 *  PAWX - Code with Claws!
 * =============================================================================
 *
 *  File:     display.rs
 *  Purpose:  Runtime Value Display & Serialization Utilities.
 *            Responsible for converting runtime `Value` objects into:
 *              - Human-readable strings (for meow, debugging, REPL)
 *              - JSON-safe serialized output (for modules, HTTP, IPC, etc.)
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

use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;


/// ============================================================================
/// value_to_string
/// ============================================================================
/// Converts a PAWX runtime `Value` into a **human-readable string**.
/// This is used by:
///   - `meow()`
///   - Debug output
///   - REPL display
///
/// This follows **JavaScript-style formatting** where applicable.
///
/// Examples:
///   - Number(3.14)      → "3.14"
///   - String("cat")    → "cat"
///   - Array([1,2,3])   → "[1, 2, 3]"
///   - Object           → "{ key: value }"
///   - Function         → "[function]"
///   - Class            → "[class Cat]"
///   - Instance         → "[instance Cat]"
/// ============================================================================
pub fn value_to_string(val: &Value) -> String {
    match val {
        // ------------------------
        // Primitive Types
        // ------------------------

        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),

        // ------------------------
        // Regex
        // ------------------------

        Value::Regex(r) => format!("/{}/", r.as_str()),

        // ------------------------
        // Arrays
        // ------------------------

        Value::Array { values, .. } => {
            let borrowed = values.borrow();
            let mut out = String::from("[");
            for (i, v) in borrowed.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&value_to_string(v));
            }
            out.push(']');
            out
        }

        // ------------------------
        // Objects
        // ------------------------

        Value::Object { fields } => {
            let map = fields.borrow();
            let mut out = String::from("{ ");
            let mut first = true;

            for (k, v) in map.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(k);
                out.push_str(": ");
                out.push_str(&value_to_string(v));
            }

            out.push_str(" }");
            out
        }

        // ------------------------
        // Tuples
        // ------------------------

        Value::Tuple(values) => {
            let mut out = String::from("(");
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&value_to_string(v));
            }
            out.push(')');
            out
        }

        // ------------------------
        // Runtime Types
        // ------------------------

        Value::NativeFunction(_) => "[function]".to_string(),

        Value::Class { name, .. } => format!("[class {}]", name),

        Value::Instance { class_name, .. } => {
            format!("[instance {}]", class_name)
        }

        Value::Furure(_) => "[future]".to_string(),

        Value::Error { message } => format!("Error({})", message),

        Value::Module { exports, .. } => {
            format!("[module {} exports]", exports.len())
        }
    }
}

/// Converts a PAWX runtime `Value` into a **valid JSON string**.
/// This is used for:
///   - HTTP output
///   - API responses
///   - Debug serialization
///
/// Unsupported runtime values (functions, classes, instances) are serialized
/// as placeholder strings.
///
/// Examples:
///   - Number(3)        → "3"
///   - String("cat")   → "\"cat\""
///   - Bool(true)      → "true"
///   - Null            → "null"
///   - Array           → "[1,2,3]"
///   - Object          → "{\"x\":1,\"y\":2}"
/// ============================================================================
pub fn value_to_json(val: &Value) -> String {
    match val {
        // ------------------------
        // JSON Primitives
        // ------------------------

        Value::Number(n) => n.to_string(),

        Value::String(s) => {
            // Proper JSON escaping
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        }

        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),

        // ------------------------
        // JSON Regex (serialized as string)
        // ------------------------

        Value::Regex(r) => {
            format!("\"/{}/\"", r.as_str())
        }

        // ------------------------
        // JSON Arrays
        // ------------------------

        Value::Array { values, .. } => {
            let arr = values.borrow();
            let inner: Vec<String> = arr.iter().map(value_to_json).collect();
            format!("[{}]", inner.join(","))
        }

        // ------------------------
        // JSON Objects
        // ------------------------

        Value::Object { fields } => {
            let map = fields.borrow();
            let mut parts = Vec::new();

            for (k, v) in map.iter() {
                parts.push(format!(
                    "\"{}\":{}",
                    k.replace('\\', "\\\\").replace('"', "\\\""),
                    value_to_json(v)
                ));
            }

            format!("{{{}}}", parts.join(","))
        }

        // ------------------------
        // Unsupported JSON Values
        // ------------------------

        Value::NativeFunction(_) => "\"[function]\"".to_string(),

        Value::Class { name, .. } => {
            format!("\"[class {}]\"", name)
        }

        Value::Instance { class_name, .. } => {
            format!("\"[instance {}]\"", class_name)
        }

        Value::Furure(_) => "\"[future]\"".to_string(),

        Value::Module { .. } => "\"[module]\"".to_string(),

        Value::Error { message } => {
            format!("\"Error({})\"", message.replace('"', "\\\""))
        }

        Value::Tuple(values) => {
            let inner: Vec<String> = values.iter().map(value_to_json).collect();
            format!("[{}]", inner.join(","))
        }
    }
}