/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Regex Prototype Implementation
 * ==========================================================================
 *
 * This module defines the native Rust-backed implementation of the
 * JavaScript-style `Regex` standard library used by the PAWX runtime.
 *
 * It provides first-class regular expression support for:
 *   - Creating regex patterns at runtime
 *   - Performing boolean match tests against strings
 *
 * Installed API:
 *   - Regex.create(pattern)
 *   - Regex.test(regex, string)
 *
 * These functions are installed once onto the global `Regex` namespace
 * and are shared across all PAWX programs.
 *
 * --------------------------------------------------------------------------
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * Github:   https://github.com/samwilcox/pawx
 *
 * --------------------------------------------------------------------------
 * License:
 * --------------------------------------------------------------------------
 * This file is part of the PAWX programming language project.
 *
 * PAWX is dual-licensed under the terms of:
 *   - The MIT License
 *   - The Apache License, Version 2.0
 *
 * You may choose either license to govern your use of this software.
 *
 * Full license text available at:
 *     https://license.pawx-lang.com
 *
 * --------------------------------------------------------------------------
 * Warranty Disclaimer:
 * --------------------------------------------------------------------------
 * Unless required by applicable law or agreed to in writing, this software is
 * distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied.
 *
 * ==========================================================================
 */

use std::collections::HashMap;
use std::sync::Arc;

use crate::value::Value;

/* ==========================================================================
 * GLOBAL REGEX NAMESPACE
 * ==========================================================================
 */

/// Creates and returns the global `Regex` namespace for the PAWX runtime.
///
/// # Installed Functions
/// - `Regex.create(pattern)` → Compiles a string into a regex object
/// - `Regex.test(regex, str)` → Tests a regex against a string
///
/// This function is called once during runtime initialization and the
/// resulting object is injected into the global environment.
///
/// # Returns
/// A `HashMap<String, Value>` representing the `Regex` global namespace.
pub fn create_global_regex_object() -> HashMap<String, Value> {
    let mut regex_obj = HashMap::new();

    regex_obj.insert(
        "create".into(),
        Value::NativeFunction(Arc::new(regex_create)),
    );

    regex_obj.insert(
        "test".into(),
        Value::NativeFunction(Arc::new(regex_test)),
    );

    regex_obj
}

/* ==========================================================================
 * REGEX.create(pattern)
 * ==========================================================================
 */

/// Compiles a string pattern into a runtime `Regex` object.
///
/// # PAWX Usage
/// ```pawx
/// let r = Regex.create("[a-z]+");
/// ```
///
/// # Arguments
/// - `pattern` (String) → The regular expression pattern
///
/// # Returns
/// - `Value::Regex` containing a compiled Rust `regex::Regex`
///
/// # Panics
/// Panics if:
/// - The argument is not a string
/// - The regex pattern is invalid and fails to compile
fn regex_create(args: Vec<Value>) -> Value {
    match args.get(0) {
        Some(Value::String(pattern)) => {
            let re = regex::Regex::new(pattern)
                .expect("Invalid regex pattern");

            Value::Regex(re)
        }

        _ => panic!("Regex.create(pattern) expects a string"),
    }
}

/* ==========================================================================
 * REGEX.test(regex, string)
 * ==========================================================================
 */

/// Tests a regex pattern against a target string.
///
/// # PAWX Usage
/// ```pawx
/// let r = Regex.create("^cat");
/// Regex.test(r, "catnap");   // true
/// Regex.test(r, "dog");      // false
/// ```
///
/// # Arguments
/// - `regex` (Regex) → A previously created regex object
/// - `string` (String) → The target string to test
///
/// # Returns
/// - `Value::Bool(true)` if the regex matches
/// - `Value::Bool(false)` if it does not
///
/// # Panics
/// Panics if:
/// - The first argument is not a `Regex`
/// - The second argument is not a `String`
fn regex_test(args: Vec<Value>) -> Value {
    let regex = match args.get(0) {
        Some(Value::Regex(r)) => r,
        _ => panic!("Regex.test(regex, str) expects a regex as the first argument"),
    };

    let text = match args.get(1) {
        Some(Value::String(s)) => s,
        _ => panic!("Regex.test(regex, str) expects a string as the second argument"),
    };

    Value::Bool(regex.is_match(text))
}