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
 *   - THe Apache License, Version 2.0
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

use crate::value::Value;

 /// Determines whether a runtime `Value` is considered **truthy** in PAWX.
///
/// This function defines the **boolean coercion rules** used by:
/// - Logical operators (`&&`, `||`)
/// - Conditional statements (`if`, `while`)
/// - Short-circuit evaluation
///
/// # Truthiness Rules
/// The following values are considered **false**:
/// - `Value::Bool(false)`
/// - `Value::Null`
/// - `Value::Number(0)`
/// - `Value::String("")` (empty string)
///
/// All other values are considered **true**, including:
/// - Non-zero numbers
/// - Non-empty strings
/// - Objects
/// - Arrays
/// - Functions
/// - Classes
/// - Instances
///
/// # Design Philosophy
/// These rules closely mirror the behavior of languages like:
/// - JavaScript
/// - Python
/// - Lua
///
/// This allows PAWX to support:
/// - Natural boolean expressions
/// - Idiomatic short-circuit logic
/// - Guard-style conditionals in control flow
///
/// # Parameters
/// - `value` → The runtime `Value` to test for truthiness
///
/// # Returns
/// - `true` if the value is truthy
/// - `false` if the value is falsy
pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        _ => true, // objects, arrays, functions, etc. are truthy
    }
}