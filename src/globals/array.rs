/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Global Array Object Implementation
 * ==========================================================================
 *
 * This module defines the global `Array` object for the PAWX runtime.
 *
 * Unlike the Array *prototype* (which provides instance methods like
 * `push`, `map`, and `filter`), this module defines **static Array utilities**
 * such as:
 *
 *   - Array.isArray(value)
 *
 * These helpers are installed once into the global namespace and are
 * accessible without needing an array instance.
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

use crate::runtime::{NativeFn, PawxValue};

/// Creates and returns the global `Array` object for the PAWX runtime.
///
/// This object contains **static Array utilities**, such as:
/// - `Array.isArray(value)`
///
/// # Returns
/// A `PawxValue::Object` representing the global `Array` namespace.
///
/// # PAWX Example
/// ```pawx
/// meow(Array.isArray([1, 2, 3])); // true
/// meow(Array.isArray(42));       // false
/// ```
pub fn create_array_global() -> PawxValue {
    let mut methods = HashMap::new();

    // Array.isArray(value)
    methods.insert(
        "isArray".to_string(),
        PawxValue::NativeFunction(NativeFn::new(1, array_is_array)),
    );

    PawxValue::Object {
        fields: Rc::new(RefCell::new(methods)),
    }
}

/// Native implementation of `Array.isArray()` for PAWX.
///
/// Determines whether the provided value is an array.
///
/// # Parameters (via `args`)
/// - `args[0]`: The value to test
///
/// # Returns
/// - `true` if the value is a PAWX array
/// - `false` otherwise
///
/// # Behavior
/// - Works on any runtime value.
/// - Matches JavaScript `Array.isArray` semantics.
///
/// # PAWX Example
/// ```pawx
/// meow(Array.isArray([1,2,3])); // true
/// meow(Array.isArray("test")); // false
/// ```
fn array_is_array(args: Vec<PawxValue>) -> PawxValue {
    match args.get(0) {
        Some(PawxValue::Array { .. }) => PawxValue::Bool(true),
        _ => PawxValue::Bool(false),
    }
}