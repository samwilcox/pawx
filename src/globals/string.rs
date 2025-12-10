/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Global String Object Implementation
 * ==========================================================================
 *
 * This module defines the global `String` object for the PAWX runtime.
 *
 * Unlike the String *prototype* (which provides instance methods such as
 * `"text".split()`), this module defines **static String utilities** such as:
 *
 *   - String.split(string, delimiter)
 *
 * These helpers are installed once into the global namespace and are
 * accessible without needing a string instance.
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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::array_proto::create_array_proto;
use crate::runtime::{NativeFn, PawxValue};

/// Creates and returns the global `String` object for the PAWX runtime.
///
/// This object contains **static String utilities**, such as:
/// - `String.split(string, delimiter)`
///
/// # Returns
/// A `PawxValue::Object` representing the global `String` namespace.
///
/// # PAWX Example
/// ```pawx
/// snuggle parts = String.split("a,b,c", ",");
/// meow(parts); // ["a", "b", "c"]
/// ```
pub fn create_string_global() -> PawxValue {
    let mut methods = HashMap::new();

    // String.split(string, delimiter)
    methods.insert(
        "split".to_string(),
        PawxValue::NativeFunction(NativeFn::new(2, string_split)),
    );

    PawxValue::Object {
        fields: Rc::new(RefCell::new(methods)),
    }
}

/// Native implementation of `String.split()` for PAWX.
///
/// Splits a string into substrings using the specified delimiter.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source string
/// - `args[1]`: The delimiter string
///
/// # Returns
/// A **PAWX array of strings** resulting from the split operation.
///
/// # Behavior
/// - Fully JS-compatible behavior.
/// - Delimiter must be a string.
///
/// # PAWX Example
/// ```pawx
/// snuggle parts = String.split("one,two,three", ",");
/// meow(parts); // ["one", "two", "three"]
/// ```
fn string_split(args: Vec<PawxValue>) -> PawxValue {
    if args.len() != 2 {
        panic!("String.split(string, delimiter) requires 2 arguments");
    }

    let string = match &args[0] {
        PawxValue::String(s) => s.clone(),
        _ => panic!("String.split() expects a string as the first argument"),
    };

    let delimiter = match &args[1] {
        PawxValue::String(s) => s.clone(),
        _ => panic!("String.split() expects a string as the delimiter"),
    };

    let parts: Vec<PawxValue> = string
        .split(&delimiter)
        .map(|p| PawxValue::String(p.to_string()))
        .collect();

    PawxValue::Array {
        values: Rc::new(RefCell::new(parts)),
        proto: create_array_proto(),
    }
}