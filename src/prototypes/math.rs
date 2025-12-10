/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Math Prototype Implementation
 * ==========================================================================
 * 
 * This module defines the native Rust-backed implementation of the
 * JavaScript-style `Math` standard library used by the PAWX runtime.
 * 
 * It provides core mathematical constants and functions, including:
 *   - Constants: PI, E9
 *   - Rounding: floor, ceil, round
 *   - Powers & Roots: pow, sqrt
 *   - Magnitude: abs
 *   - Aggregates: min, max
 *   - Randomness: random
 * 
 * These functions are installed once onto the global `Math` object
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
use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

/// Creates and returns the global `Math` object for the PAWX runtime.
///
/// This function installs all math constants and functions as
/// native callable values inside a HashMap, matching JavaScript-style
/// `Math.*` behavior.
///
/// # Returns
/// A fully populated `HashMap<String, Value>` representing the global Math object.
pub fn create_global_math_object() -> HashMap<String, Value> {
    let mut math = HashMap::new();

    // ---------------------------------------------------------------------
    // Constants
    // ---------------------------------------------------------------------

    math.insert("PI".to_string(), Value::NativeFunction(Arc::new(math_PI)));
    math.insert("E9".to_string(), Value::NativeFunction(Arc::new(math_E9)));

    // ---------------------------------------------------------------------
    // Rounding
    // ---------------------------------------------------------------------

    math.insert("floor".to_string(), Value::NativeFunction(Arc::new(math_floor)));
    math.insert("ceil".to_string(), Value::NativeFunction(Arc::new(math_ceil)));
    math.insert("round".to_string(), Value::NativeFunction(Arc::new(math_round)));

    // ---------------------------------------------------------------------
    // Powers & Roots
    // ---------------------------------------------------------------------

    math.insert("pow".to_string(), Value::NativeFunction(Arc::new(math_pow)));
    math.insert("sqrt".to_string(), Value::NativeFunction(Arc::new(math_sqrt)));

    // ---------------------------------------------------------------------
    // Magnitude
    // ---------------------------------------------------------------------

    math.insert("abs".to_string(), Value::NativeFunction(Arc::new(math_abs)));

    // ---------------------------------------------------------------------
    // Aggregates
    // ---------------------------------------------------------------------

    math.insert("min".to_string(), Value::NativeFunction(Arc::new(math_min)));
    math.insert("max".to_string(), Value::NativeFunction(Arc::new(math_max)));

    // ---------------------------------------------------------------------
    // Randomness
    // ---------------------------------------------------------------------

    math.insert("random".to_string(), Value::NativeFunction(Arc::new(math_random)));

    math
}

pub fn create_global_math_value() -> Value {
    let math_map = create_global_math_object();

    Value::Object {
        fields: Rc::new(RefCell::new(math_map)),
    }
}

/// Native implementation of the mathematical constant `Math.PI` for PAWX.
///
/// Represents the ratio of a circle’s circumference to its diameter.
///
/// # Returns
/// A `Number` value equal to π (approximately `3.141592653589793`).
///
/// # PAWX Example
/// ```pawx
/// meow(Math.PI); // 3.141592653589793
/// ```
pub fn math_PI(args: Vec<Value>) -> Value {
    Value::Number(std::f64::consts::PI)
}

/// Native implementation of the mathematical constant `Math.E9` for PAWX.
///
/// Represents Euler’s number raised to the 9th power (`e⁹`).
/// This constant is useful for scientific and exponential calculations.
///
/// # Returns
/// A `Number` representing `e⁹`.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.E9);
/// ```
pub fn math_E9(args: Vec<Value>) -> Value {
    Value::Number(std::f64::consts::E)
}

/// Native implementation of `Math.floor()` for PAWX.
///
/// Rounds a number **downward** to the nearest integer.
///
/// # Parameters (via `args`)
/// - `args[0]`: The input number
///
/// # Returns
/// The largest integer less than or equal to the input value.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.floor(4.9)); // 4
/// ```
pub fn math_floor(args: Vec<Value>) -> Value {
    let x = match args.get(0) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.floor(x) expects a number"),
    };

    Value::Number(x.floor())
}

/// Native implementation of `Math.ceil()` for PAWX.
///
/// Rounds a number **upward** to the nearest integer.
///
/// # Parameters (via `args`)
/// - `args[0]`: The input number
///
/// # Returns
/// The smallest integer greater than or equal to the input value.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.ceil(4.1)); // 5
/// ```
pub fn math_ceil(args: Vec<Value>) -> Value {
    let x = match args.get(0) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.ceil(x) expects a number"),
    };

    Value::Number(x.ceil())
}

/// Native implementation of `Math.round()` for PAWX.
///
/// Rounds a number to the **nearest integer** using standard rounding rules.
///
/// # Parameters (via `args`)
/// - `args[0]`: The input number
///
/// # Returns
/// The nearest integer to the given value.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.round(4.5)); // 5
/// meow(Math.round(4.4)); // 4
/// ```
pub fn math_round(args: Vec<Value>) -> Value {
    let x = match args.get(0) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.round(x) expects a number"),
    };

    Value::Number(x.round())
}

/// Native implementation of `Math.pow()` for PAWX.
///
/// Raises a base number to the power of an exponent.
///
/// # Parameters (via `args`)
/// - `args[0]`: Base value
/// - `args[1]`: Exponent value
///
/// # Returns
/// The result of `base ^ exponent`.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.pow(2, 3)); // 8
/// ```
pub fn math_pow(args: Vec<Value>) -> Value {
    let base = match args.get(0) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.pow(x, y) expects numbers"),
    };

    let exp = match args.get(1) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.pow(x, y) expects numbers"),
    };

    Value::Number(base.powf(exp))
}

/// Native implementation of `Math.sqrt()` for PAWX.
///
/// Computes the **square root** of a number.
///
/// # Parameters (via `args`)
/// - `args[0]`: The input number
///
/// # Returns
/// The square root of the input value.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.sqrt(16)); // 4
/// ```
pub fn math_sqrt(args: Vec<Value>) -> Value {
    let x = match args.get(0) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.sqrt(x) expects a number"),
    };

    Value::Number(x.sqrt())
}

/// Native implementation of `Math.abs()` for PAWX.
///
/// Returns the **absolute value** of a number.
///
/// # Parameters (via `args`)
/// - `args[0]`: The input number
///
/// # Returns
/// A number representing the absolute magnitude.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.abs(-10)); // 10
/// ```
pub fn math_abs(args: Vec<Value>) -> Value {
    let x = match args.get(0) {
        Some(Value::Number(n)) => *n,
        _ => panic!("Math.abs(x) expects a number"),
    };

    Value::Number(x.abs())
}

/// Native implementation of `Math.min()` for PAWX.
///
/// Returns the **smallest** of one or more numbers.
///
/// # Parameters (via `args`)
/// - `args[*]`: One or more numeric values
///
/// # Returns
/// The smallest numerical value from the arguments.
///
/// # Behavior
/// - Supports variable-length argument lists.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.min(4, 1, 9)); // 1
/// ```
pub fn math_min(args: Vec<Value>) -> Value {
    if args.is_empty() {
        panic!("Math.min() requires at least one number");
    }

    let mut min = match args[0] {
        Value::Number(n) => n,
        _ => panic!("Math.min() only accepts numbers"),
    };

    for arg in &args[1..] {
        match arg {
            Value::Number(n) => {
                if *n < min {
                    min = *n;
                }
            }
            _ => panic!("Math.min() only accepts numbers"),
        }
    }

    Value::Number(min)
}

/// Native implementation of `Math.max()` for PAWX.
///
/// Returns the **largest** of one or more numbers.
///
/// # Parameters (via `args`)
/// - `args[*]`: One or more numeric values
///
/// # Returns
/// The largest numerical value from the arguments.
///
/// # Behavior
/// - Supports variable-length argument lists.
///
/// # PAWX Example
/// ```pawx
/// meow(Math.max(4, 1, 9)); // 9
/// ```
pub fn math_max(args: Vec<Value>) -> Value {
    if args.is_empty() {
        panic!("Math.max() requires at least one number");
    }

    let mut max = match args[0] {
        Value::Number(n) => n,
        _ => panic!("Math.max() only accepts numbers"),
    };

    for arg in &args[1..] {
        match arg {
            Value::Number(n) => {
                if *n > max {
                    max = *n;
                }
            }
            _ => panic!("Math.max() only accepts numbers"),
        }
    }

    Value::Number(max)
}

/// Native implementation of `Math.random()` for PAWX.
///
/// Returns a pseudo-random floating-point number in the range:
/// `0.0 <= n < 1.0`
///
/// # Returns
/// A random `Number` between `0` (inclusive) and `1` (exclusive).
///
/// # Behavior
/// - Suitable for non-cryptographic randomness.
/// - Intended for general-purpose use.
///
/// # PAWX Example
/// ```pawx
/// snuggle r = Math.random();
/// meow(r); // 0.0 -> 0.999...
/// ```
pub fn math_random(args: Vec<Value>) -> Value {
    let r = rand::random::<f64>();
    Value::Number(r)
}