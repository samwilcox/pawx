/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * String Prototype Implementation
 * ==========================================================================
 *
 * This module defines the native Rust-backed implementation of the
 * JavaScript-style `String` standard library used by the PAWX runtime.
 *
 * It provides common string utilities such as:
 *   - String.len(str)
 *   - String.upper(str)
 *   - String.lower(str)
 *   - String.trim(str)
 *   - String.split(str, sep)
 *
 * These functions are installed once onto the global `String` namespace
 * and are shared across all PAWX programs.
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

use crate::value::Value;
use crate::prototypes::array::create_array_proto;


/// Creates and returns the global `String` namespace for the PAWX runtime.
///
/// # Installed Functions
/// - `String.len(str)`
/// - `String.upper(str)`
/// - `String.lower(str)`
/// - `String.trim(str)`
/// - `String.split(str, sep)`
pub fn create_global_string_object() -> HashMap<String, Value> {
    let mut string = HashMap::new();

    string.insert("len".to_string(), Value::NativeFunction(Arc::new(string_len)));
    string.insert("upper".to_string(), Value::NativeFunction(Arc::new(string_upper)));
    string.insert("lower".to_string(), Value::NativeFunction(Arc::new(string_lower)));
    string.insert("trim".to_string(), Value::NativeFunction(Arc::new(string_trim)));
    string.insert("split".to_string(), Value::NativeFunction(Arc::new(string_split)));
    string.insert("contains".to_string(), Value::NativeFunction(Arc::new(string_contains)));
    string.insert("startsWith".to_string(), Value::NativeFunction(Arc::new(string_starts_with)));
    string.insert("endsWith".to_string(), Value::NativeFunction(Arc::new(string_ends_with)));
    string.insert("replace".to_string(), Value::NativeFunction(Arc::new(string_replace)));
    string.insert("repeat".to_string(), Value::NativeFunction(Arc::new(string_repeat)));
    string.insert("match".to_string(), Value::NativeFunction(Arc::new(string_match)));
    string.insert("replaceRegex".to_string(), Value::NativeFunction(Arc::new(string_replace_regex)));

    string
}

/// Returns the number of characters in a string.
///
/// # Arguments
/// - `str` → The input string.
///
/// # Returns
/// - A `Number` representing the length of the string.
///
/// # Example (PAWX)
/// ```pawx
/// let n = String.len("hello"); // 5
/// ```
pub fn string_len(args: Vec<Value>) -> Value {
    match args.get(0) {
        Some(Value::String(s)) => Value::Number(s.len() as f64),
        _ => panic!("String.len(str) expects a string"),
    }
}

/// Converts all characters in a string to uppercase.
///
/// # Arguments
/// - `str` → The input string.
///
/// # Returns
/// - A new `String` where all alphabetic characters are uppercase.
///
/// # Example (PAWX)
/// ```pawx
/// let s = String.upper("pawx"); // "PAWX"
/// ```
pub fn string_upper(args: Vec<Value>) -> Value {
    match args.get(0) {
        Some(Value::String(s)) => Value::String(s.to_uppercase()),
        _ => panic!("String.upper(str) expects a string"),
    }
}

/// Converts all characters in a string to lowercase.
///
/// # Arguments
/// - `str` → The input string.
///
/// # Returns
/// - A new `String` where all alphabetic characters are lowercase.
///
/// # Example (PAWX)
/// ```pawx
/// let s = String.lower("PAWX"); // "pawx"
/// ```
pub fn string_lower(args: Vec<Value>) -> Value {
    match args.get(0) {
        Some(Value::String(s)) => Value::String(s.to_lowercase()),
        _ => panic!("String.lower(str) expects a string"),
    }
}

/// Removes leading and trailing whitespace from a string.
///
/// # Arguments
/// - `str` → The input string.
///
/// # Returns
/// - A new `String` with surrounding whitespace removed.
///
/// # Example (PAWX)
/// ```pawx
/// let s = String.trim("  hello  "); // "hello"
/// ```
pub fn string_trim(args: Vec<Value>) -> Value {
    match args.get(0) {
        Some(Value::String(s)) => Value::String(s.trim().to_string()),
        _ => panic!("String.trim(str) expects a string"),
    }
}

/// Splits a string into an array of substrings using a separator.
///
/// # Arguments
/// - `str` → The input string.
/// - `sep` → The delimiter used to split the string.
///
/// # Returns
/// - An `Array` of `String` values.
///
/// # Example (PAWX)
/// ```pawx
/// let parts = String.split("a,b,c", ","); // ["a", "b", "c"]
/// ```
pub fn string_split(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s.clone(),
        _ => panic!("String.split(str, sep) expects a string"),
    };

    let sep = match args.get(1) {
        Some(Value::String(s)) => s.clone(),
        _ => panic!("String.split(str, sep) expects a string separator"),
    };

    let parts = s
        .split(&sep)
        .map(|p| Value::String(p.to_string()))
        .collect::<Vec<_>>();

    Value::Array {
        values: std::rc::Rc::new(std::cell::RefCell::new(parts)),
        proto: create_array_proto(),
    }
}

/// Checks whether a string contains a given substring.
///
/// # Arguments
/// - `str` → The input string.
/// - `search` → The substring to search for.
///
/// # Returns
/// - A `Bool` indicating whether the substring exists.
///
/// # Example (PAWX)
/// ```pawx
/// if (String.contains("pawx-lang", "lang")) {
///     meow("Found!");
/// }
/// ```
pub fn string_contains(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s,
        _ => panic!("String.contains(str, search) expects a string"),
    };

    let search = match args.get(1) {
        Some(Value::String(s)) => s,
        _ => panic!("String.contains(str, search) expects a string search value"),
    };

    Value::Bool(s.contains(search))
}

/// Checks whether a string starts with a given prefix.
///
/// # Arguments
/// - `str` → The input string.
/// - `prefix` → The string to match at the beginning.
///
/// # Returns
/// - A `Bool` indicating whether the string starts with the prefix.
///
/// # Example (PAWX)
/// ```pawx
/// String.startsWith("pawx-lang", "pawx"); // true
/// ```
pub fn string_starts_with(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s,
        _ => panic!("String.startsWith(str, prefix) expects a string"),
    };

    let prefix = match args.get(1) {
        Some(Value::String(s)) => s,
        _ => panic!("String.startsWith(str, prefix) expects a string"),
    };

    Value::Bool(s.starts_with(prefix))
}

/// Checks whether a string ends with a given suffix.
///
/// # Arguments
/// - `str` → The input string.
/// - `suffix` → The string to match at the end.
///
/// # Returns
/// - A `Bool` indicating whether the string ends with the suffix.
///
/// # Example (PAWX)
/// ```pawx
/// String.endsWith("pawx-lang", "lang"); // true
/// ```
pub fn string_ends_with(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s,
        _ => panic!("String.endsWith(str, suffix) expects a string"),
    };

    let suffix = match args.get(1) {
        Some(Value::String(s)) => s,
        _ => panic!("String.endsWith(str, suffix) expects a string"),
    };

    Value::Bool(s.ends_with(suffix))
}

/// Replaces all occurrences of a substring within a string.
///
/// # Arguments
/// - `str` → The input string.
/// - `find` → The substring to replace.
/// - `replace` → The replacement string.
///
/// # Returns
/// - A new `String` with all matches replaced.
///
/// # Example (PAWX)
/// ```pawx
/// String.replace("cat-cat-cat", "cat", "paw"); // "paw-paw-paw"
/// ```
pub fn string_replace(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s.clone(),
        _ => panic!("String.replace(str, find, replace) expects a string"),
    };

    let find = match args.get(1) {
        Some(Value::String(s)) => s,
        _ => panic!("String.replace(str, find, replace) expects a string"),
    };

    let replace = match args.get(2) {
        Some(Value::String(s)) => s,
        _ => panic!("String.replace(str, find, replace) expects a string"),
    };

    Value::String(s.replace(find, replace))
}

/// Repeats a string a specified number of times.
///
/// # Arguments
/// - `str` → The input string.
/// - `count` → The number of repetitions.
///
/// # Returns
/// - A new `String` consisting of repeated concatenations.
///
/// # Example (PAWX)
/// ```pawx
/// String.repeat("ha", 3); // "hahaha"
/// ```
pub fn string_repeat(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s.clone(),
        _ => panic!("String.repeat(str, n) expects a string"),
    };

    let n = match args.get(1) {
        Some(Value::Number(n)) => *n as usize,
        _ => panic!("String.repeat(str, n) expects a number"),
    };

    Value::String(s.repeat(n))
}

pub fn string_match(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s,
        _ => panic!("String.match(str, regex) expects a string"),
    };

    let regex = match args.get(1) {
        Some(Value::Regex(r)) => r,
        _ => panic!("String.match(str, regex) expects a regex"),
    };

    let matches = regex
        .find_iter(s)
        .map(|m| Value::String(m.as_str().to_string()))
        .collect::<Vec<_>>();

    Value::Array {
        values: std::rc::Rc::new(std::cell::RefCell::new(matches)),
        proto: create_array_proto(),
    }
}

pub fn string_replace_regex(args: Vec<Value>) -> Value {
    let s = match args.get(0) {
        Some(Value::String(s)) => s.clone(),
        _ => panic!("String.replaceRegex(str, regex, replace) expects a string"),
    };

    let regex = match args.get(1) {
        Some(Value::Regex(r)) => r,
        _ => panic!("String.replaceRegex(str, regex, replace) expects a regex"),
    };

    let replace = match args.get(2) {
        Some(Value::String(s)) => s,
        _ => panic!("String.replaceRegex(str, regex, replace) expects a string"),
    };

    Value::String(regex.replace_all(&s, replace).to_string())
}