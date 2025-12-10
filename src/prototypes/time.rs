/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Time & Date Prototype Implementation
 * ==========================================================================
 * 
 * This module defines the native Rust-backed implementation of the
 * JavaScript-style `Time` and `Date` utilities used by the PAWX runtime.
 * 
 * It provides core time-based functionality such as:
 *   - Current timestamps (local & UTC)
 *   - Time formatting
 *   - Timezone offsets
 *   - Sleep / blocking delays
 * 
 * These functions are installed once onto the global `Time` namespace
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

use chrono::{Local, Utc};

use crate::value::Value;

/// Creates and returns the global `Time` namespace for the PAWX runtime.
///
/// This function installs JavaScript-style static time utilities:
/// - `Time.now()`
/// - `Time.utc()`
/// - `Time.local()`
/// - `Time.format(fmt)`
/// - `Time.tzOffset()`
/// - `Time.sleep(ms)`
///
/// These functions are globally available and do **not** rely on
/// per-instance prototypes.
///
/// # Returns
/// A fully populated `HashMap<String, Value>` representing the global `Time` object.
pub fn create_global_time_object() -> HashMap<String, Value> {
    let mut time = HashMap::new();

    // ---------------------------------------------------------------------
    // Core Time Accessors
    // ---------------------------------------------------------------------

    time.insert(
        "now".to_string(),
        Value::NativeFunction(Arc::new(time_now)),
    );

    time.insert(
        "utc".to_string(),
        Value::NativeFunction(Arc::new(time_utc)),
    );

    time.insert(
        "local".to_string(),
        Value::NativeFunction(Arc::new(time_local)),
    );

    // ---------------------------------------------------------------------
    // Formatting & Timezone
    // ---------------------------------------------------------------------

    time.insert(
        "format".to_string(),
        Value::NativeFunction(Arc::new(time_format)),
    );

    time.insert(
        "tzOffset".to_string(),
        Value::NativeFunction(Arc::new(time_tzOffset)),
    );

    // ---------------------------------------------------------------------
    // Blocking Utilities
    // ---------------------------------------------------------------------

    time.insert(
        "sleep".to_string(),
        Value::NativeFunction(Arc::new(time_sleep)),
    );

    time
}

/// Creates and returns the **runtime PAWX `Time` object**.
///
/// This wraps the internal `HashMap<String, Value>` inside a
/// `Value::Object` so it can be registered into the PAWX environment.
///
/// # Returns
/// A fully usable runtime `Value::Object` representing `Time`.
pub fn create_global_time_value() -> Value {
    let time_map = create_global_time_object();

    Value::Object {
        fields: Rc::new(RefCell::new(time_map)),
    }
}

/// Native implementation of `Time.now()` for PAWX.
///
/// Returns the **current Unix timestamp in milliseconds**.
///
/// # Returns
/// A `Number` representing milliseconds since the Unix epoch.
///
/// # PAWX Example
/// ```pawx
/// meow(Time.now());
/// ```
pub fn time_now(args: Vec<Value>) -> Value {
    let millis = Utc::now().timestamp_millis();
    Value::Number(millis as f64)
}

/// Native implementation of `Time.utc()` for PAWX.
///
/// Returns the current **UTC (Coordinated Universal Time)** timestamp.
///
/// # Returns
/// A PAWX time object or formatted UTC string (depending on runtime design).
///
/// # PAWX Example
/// ```pawx
/// meow(Time.utc());
/// ```
pub fn time_utc(args: Vec<Value>) -> Value {
    Value::String(Utc::now().to_rfc3339())
}

/// Native implementation of `Time.local()` for PAWX.
///
/// Returns the current **local system time**.
///
/// # Returns
/// A PAWX time object or formatted local time string.
///
/// # PAWX Example
/// ```pawx
/// meow(Time.local());
/// ```
pub fn time_local(args: Vec<Value>) -> Value {
    Value::String(Local::now().to_rfc3339())
}

/// Native implementation of `Time.format()` for PAWX.
///
/// Formats the current local time using a custom format string.
///
/// # Parameters (via `args`)
/// - `args[0]`: A format string (e.g. `"%Y-%m-%d %H:%M:%S"`)
///
/// # Returns
/// A formatted date/time string.
///
/// # Behavior
/// - Uses strftime-style formatting.
/// - Operates on local time.
///
/// # PAWX Example
/// ```pawx
/// meow(Time.format("%Y-%m-%d %H:%M:%S"));
/// ```
pub fn time_format(args: Vec<Value>) -> Value {
    let fmt = match args.get(0) {
        Some(Value::String(s)) => s.clone(),
        _ => panic!("Time.format() requires a format string"),
    };

    let formatted = Local::now().format(&fmt).to_string();
    Value::String(formatted)
}

/// Native implementation of `Time.tzOffset()` for PAWX.
///
/// Returns the **local timezone offset** in minutes from UTC.
///
/// # Returns
/// A `Number` representing the timezone offset.
///
/// # PAWX Example
/// ```pawx
/// meow(Time.tzOffset());
/// ```
pub fn time_tzOffset(args: Vec<Value>) -> Value {
    let offset = Local::now().offset().local_minus_utc() / 60;
    Value::Number(offset as f64)
}

/// Native implementation of `Time.sleep()` for PAWX.
///
/// Suspends execution for a specified duration in milliseconds.
///
/// # Parameters (via `args`)
/// - `args[0]`: Duration in milliseconds
///
/// # Returns
/// Always returns `null`.
///
/// # Behavior
/// - This function **blocks the current thread**.
/// - Integrated with PAWX’s async event loop when used via timers.
///
/// # PAWX Example
/// ```pawx
/// meow("Waiting...");
/// Time.sleep(1000);
/// meow("Done!");
/// ```
pub fn time_sleep(args: Vec<Value>) -> Value {
    let ms = match args.get(0) {
        Some(Value::Number(n)) => *n as u64,
        _ => panic!("Time.sleep(ms) requires a number"),
    };

    std::thread::sleep(std::time::Duration::from_millis(ms));

    Value::Null
}