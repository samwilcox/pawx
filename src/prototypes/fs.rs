/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 *
 * File:      fs.rs
 * Purpose:   Filesystem prototype (Node.js-like "fs" library for PAWX)
 *
 * This module exposes a global `Fs` object to PAWX scripts with:
 *
 *   ✅ UTF-8 text I/O (default)
 *   ✅ Optional encoding support ("utf8", "ascii", "latin1")
 *   ✅ Raw binary file access
 *   ✅ JSON read/write helpers
 *   ✅ Append, mkdir, and rm helpers
 *   ✅ Promise-style async variants via `Value::Furure`
 *
 * --------------------------------------------------------------------------
 *  Synchronous API
 * --------------------------------------------------------------------------
 *   - Fs.readText(path, encoding?)            -> string
 *   - Fs.writeText(path, text, encoding?)     -> null
 *   - Fs.appendText(path, text, encoding?)    -> null
 *   - Fs.readBytes(path)                      -> array<number>
 *   - Fs.writeBytes(path, bytes)              -> null
 *   - Fs.exists(path)                         -> bool
 *   - Fs.readdir(path)                        -> array<string>
 *   - Fs.mkdir(path, recursive?)              -> null
 *   - Fs.rm(path, recursive?)                 -> null
 *   - Fs.readJson(path, encoding?)            -> any PAWX Value
 *   - Fs.writeJson(path, value, pretty?, enc?) -> null
 *
 * --------------------------------------------------------------------------
 *  Asynchronous API (Promise-style, thread-backed)
 * --------------------------------------------------------------------------
 *   - Fs.readTextAsync(path, encoding?)       -> Furure(string)
 *   - Fs.writeTextAsync(path, text, enc?)     -> Furure(null)
 *   - Fs.appendTextAsync(path, text, enc?)    -> Furure(null)
 *   - Fs.readBytesAsync(path)                 -> Furure(array<number>)
 *   - Fs.writeBytesAsync(path, bytes)         -> Furure(null)
 *   - Fs.existsAsync(path)                    -> Furure(bool)
 *   - Fs.readdirAsync(path)                   -> Furure(array<string>)
 *   - Fs.mkdirAsync(path, recursive?)         -> Furure(null)
 *   - Fs.rmAsync(path, recursive?)            -> Furure(null)
 *   - Fs.readJsonAsync(path, encoding?)       -> Furure(any)
 *   - Fs.writeJsonAsync(path, value, pretty?, enc?) -> Furure(null)
 *
 * All file paths are interpreted relative to the PAWX process working
 * directory unless absolute paths are provided.
 * 
 * --------------------------------------------------------------------------
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * Github:   https://github.com/samwilcox/pawx
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
 *    https://license.pawx-lang.com
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under these licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * 
 * ==========================================================================
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use serde_json::{self, Value as JsonValue};

use crate::prototypes::array::create_array_proto;
use crate::value::Value;


/// ===============================================
/// Argument Helpers
/// ===============================================

/// Extracts a UTF-8 string argument from a PAWX `Value`.
///
/// This helper is used by all FS functions that expect string inputs
/// such as file paths, encodings, or text content.
///
/// # Panics
/// - If the value is not a `Value::String`.
fn expect_string(arg: &Value, method: &str, position: usize) -> String {
    match arg {
        Value::String(s) => s.clone(),
        other => panic!(
            "Fs.{}: argument #{} expected string, got {:?}",
            method, position, other
        ),
    }
}

/// Extracts a boolean argument from a PAWX `Value`.
///
/// Used by functions such as `Fs.mkdir`/`Fs.rm` when handling the
/// optional `recursive` flag.
///
/// # Panics
/// - If the value is not a `Value::Bool`.
fn expect_bool(arg: &Value, method: &str, position: usize) -> bool {
    match arg {
        Value::Bool(b) => *b,
        other => panic!(
            "Fs.{}: argument #{} expected boolean, got {:?}",
            method, position, other
        ),
    }
}

/// Extracts a raw byte vector from a PAWX `Value::Array<number>`.
///
/// Each element must be a numeric value (0–255), which is truncated
/// to `u8`.
///
/// # Panics
/// - If the value is not an array
/// - If any element is not a number
fn expect_bytes(arg: &Value, method: &str) -> Vec<u8> {
    match arg {
        Value::Array { values, .. } => values
            .borrow()
            .iter()
            .map(|v| match v {
                Value::Number(n) => *n as u8,
                other => panic!("Fs.{}: expected byte array, got {:?}", method, other),
            })
            .collect(),
        other => panic!("Fs.{}: expected byte array, got {:?}", method, other),
    }
}


/// ===============================================
/// Core Synchronous Filesystem Layer
/// ===============================================

/// Reads the full contents of a file as raw binary bytes.
///
/// # Panics
/// - If the file cannot be opened or read.
fn fs_read_bytes_sync(path: &str) -> Vec<u8> {
    match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("Fs.readBytes('{}'): {}", path, e),
    }
}

/// Writes raw binary bytes to a file, creating or truncating it.
///
/// # Panics
/// - If the file cannot be created or written.
fn fs_write_bytes_sync(path: &str, bytes: &[u8]) {
    if let Err(e) = fs::write(path, bytes) {
        panic!("Fs.writeBytes('{}'): {}", path, e);
    }
}

/// Reads a text file using a specified encoding.
///
/// Supported encodings:
/// - `"utf8"` / `"utf-8"`
/// - `"ascii"`
/// - `"latin1"`
///
/// # Panics
/// - If the file cannot be read
/// - If the text is not valid UTF-8 when `"utf8"` is selected
/// - If an unsupported encoding is requested.
fn fs_read_text_sync(path: &str, encoding: &str) -> Value {
    let bytes = fs_read_bytes_sync(path);

    let text = match encoding {
        "utf8" | "utf-8" => String::from_utf8(bytes)
            .unwrap_or_else(|_| panic!("Fs.readText('{}'): invalid UTF-8", path)),

        "ascii" => bytes.iter().map(|b| *b as char).collect(),

        "latin1" => bytes.iter().map(|b| *b as char).collect(),

        other => panic!("Fs.readText: unsupported encoding '{}'", other),
    };

    Value::String(text)
}

/// Writes text to a file using a specified encoding.
///
/// # Panics
/// - If the encoding is unsupported
/// - If the file cannot be written.
fn fs_write_text_sync(path: &str, text: &str, encoding: &str) {
    let bytes: Vec<u8> = match encoding {
        "utf8" | "utf-8" => text.as_bytes().to_vec(),
        "ascii" | "latin1" => text.chars().map(|c| c as u8).collect(),
        other => panic!("Fs.writeText: unsupported encoding '{}'", other),
    };

    fs_write_bytes_sync(path, &bytes);
}

/// Appends text to a file using a specified encoding.
///
/// If the file does not exist, it is created.
///
/// # Panics
/// - If the file cannot be opened or written.
fn fs_append_text_sync(path: &str, text: &str, encoding: &str) -> Value {
    let bytes: Vec<u8> = match encoding {
        "utf8" | "utf-8" => text.as_bytes().to_vec(),
        "ascii" | "latin1" => text.chars().map(|c| c as u8).collect(),
        other => panic!("Fs.appendText: unsupported encoding '{}'", other),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|e| panic!("Fs.appendText('{}'): {}", path, e));

    if let Err(e) = file.write_all(&bytes) {
        panic!("Fs.appendText('{}'): {}", path, e);
    }

    Value::Null
}

/// Determines whether a file or directory exists.
fn fs_exists_sync(path: &str) -> Value {
    Value::Bool(Path::new(path).exists())
}

/// Reads the contents of a directory into an array of filenames.
fn fs_readdir_sync(path: &str) -> Value {
    let entries = match fs::read_dir(path) {
        Ok(iter) => iter,
        Err(e) => panic!("Fs.readdir('{}'): {}", path, e),
    };

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        names.push(Value::String(entry.file_name().to_string_lossy().to_string()));
    }

    Value::Array {
        values: Rc::new(RefCell::new(names)),
        proto: create_array_proto(),
    }
}

/// Creates a directory at the given path.
///
/// If `recursive` is true, parent directories are created as needed.
fn fs_mkdir_sync(path: &str, recursive: bool) -> Value {
    if recursive {
        if let Err(e) = fs::create_dir_all(path) {
            panic!("Fs.mkdir('{}', recursive): {}", path, e);
        }
    } else {
        if let Err(e) = fs::create_dir(path) {
            panic!("Fs.mkdir('{}'): {}", path, e);
        }
    }
    Value::Null
}

/// Removes a file or directory.
///
/// If `recursive` is true:
/// - Directories are removed with all contents.
/// - Files are removed normally.
///
/// If `recursive` is false:
/// - Files are removed normally
/// - Directories must be empty.
fn fs_rm_sync(path: &str, recursive: bool) -> Value {
    let p = Path::new(path);

    if recursive {
        if p.is_dir() {
            if let Err(e) = fs::remove_dir_all(p) {
                panic!("Fs.rm('{}', recursive): {}", path, e);
            }
        } else if p.is_file() {
            if let Err(e) = fs::remove_file(p) {
                panic!("Fs.rm('{}', recursive): {}", path, e);
            }
        }
    } else {
        if p.is_dir() {
            if let Err(e) = fs::remove_dir(p) {
                panic!("Fs.rm('{}'): {}", path, e);
            }
        } else if p.is_file() {
            if let Err(e) = fs::remove_file(p) {
                panic!("Fs.rm('{}'): {}", path, e);
            }
        }
    }

    Value::Null
}

/// Converts a JSON value into a PAWX runtime `Value`.
fn json_to_pawx(j: &JsonValue) -> Value {
    match j {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(json_to_pawx).collect();
            Value::Array {
                values: Rc::new(RefCell::new(values)),
                proto: create_array_proto(),
            }
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_pawx(v));
            }
            Value::Object {
                fields: Rc::new(RefCell::new(map)),
            }
        }
    }
}

/// Converts a PAWX runtime `Value` into a JSON value.
///
/// Non-JSON-compatible values (functions, classes, futures, etc.) are
/// serialized as `null`.
fn pawx_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Number(n) => {
            serde_json::Number::from_f64(*n).map(JsonValue::Number).unwrap_or(JsonValue::Null)
        }
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Array { values, .. } => {
            let arr = values.borrow().iter().map(pawx_to_json).collect();
            JsonValue::Array(arr)
        }
        Value::Object { fields } => {
            let mut map = serde_json::Map::new();
            for (k, v2) in fields.borrow().iter() {
                map.insert(k.clone(), pawx_to_json(v2));
            }
            JsonValue::Object(map)
        }
        // Fallback for non-serializable values
        _ => JsonValue::Null,
    }
}

/// Reads a JSON file from disk and converts it into a PAWX `Value`.
fn fs_read_json_sync(path: &str, encoding: &str) -> Value {
    let text_val = fs_read_text_sync(path, encoding);
    if let Value::String(s) = text_val {
        let parsed: JsonValue =
            serde_json::from_str(&s).unwrap_or_else(|e| panic!("Fs.readJson('{}'): {}", path, e));
        json_to_pawx(&parsed)
    } else {
        unreachable!("fs_read_text_sync did not return a string");
    }
}

/// Serializes a PAWX `Value` to JSON and writes it to disk.
///
/// If `pretty` is true, the JSON is formatted with indentation.
fn fs_write_json_sync(
    path: &str,
    value: &Value,
    pretty: bool,
    encoding: &str,
) -> Value {
    let json_val = pawx_to_json(value);

    let text = if pretty {
        serde_json::to_string_pretty(&json_val)
            .unwrap_or_else(|e| panic!("Fs.writeJson('{}'): {}", path, e))
    } else {
        serde_json::to_string(&json_val)
            .unwrap_or_else(|e| panic!("Fs.writeJson('{}'): {}", path, e))
    };

    fs_write_text_sync(path, &text, encoding);
    Value::Null
}


/// ===============================================
/// Async Helper – Thread-backed Furure
/// ===============================================

/// Creates a Promise-style PAWX future without spawning OS threads.
///
/// This wraps a deferred computation (FnOnce) inside a `Value::Furure`.
/// The job is executed **exactly once** when the future is resolved.
/// 
/// This avoids:
/// - Threading
/// - `Send` / `Sync` issues
/// - `Rc` / `RefCell` breakage
/// 
/// While still preserving proper Promise-style behavior.
fn spawn_fs_future<F>(job: F) -> Value
where
    F: FnOnce() -> Value + 'static,
{
    // Wrap the job so it can be "taken" exactly once
    let job_cell = std::cell::RefCell::new(Some(job));

    let deferred = Value::NativeFunction(Arc::new(move |_args: Vec<Value>| -> Value {
        let job_opt = job_cell
            .take()
            .expect("Furure has already been resolved");

        job_opt()
    }));

    Value::Furure(Box::new(deferred))
}

/// ===============================================
/// Global Fs Prototype
/// ===============================================

/// Creates the global PAWX `Fs` object.
///
/// Registers all synchronous and async filesystem helpers into a
/// single `Value::Object` that can be bound into the root environment:
///
/// ```rust
/// global.borrow_mut()
///     .define_public("Fs".to_string(), create_fs_global());
/// ```
pub fn create_fs_global() -> Value {
    let mut map: HashMap<String, Value> = HashMap::new();

    // ============================================================
    // TEXT FILE API (SYNC)
    // ============================================================

    /// Fs.readText(path, encoding = "utf8") -> string
    map.insert(
        "readText".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readText(path, encoding?): missing `path` argument");
            }

            let path = expect_string(&args[0], "readText", 1);
            let encoding = if args.len() > 1 {
                expect_string(&args[1], "readText", 2)
            } else {
                "utf8".to_string()
            };

            fs_read_text_sync(&path, &encoding)
        })),
    );

    /// Fs.writeTextAsync(path, text, encoding?) -> Furure(null)
    map.insert(
        "writeTextAsync".to_string(),
        Value::NativeFunction(Arc::new(|args: Vec<Value>| -> Value {
            if args.len() < 2 {
                panic!("Fs.writeTextAsync(path, text, encoding?): expected at least 2 arguments");
            }

            let path = expect_string(&args[0], "writeTextAsync", 1);
            let text = expect_string(&args[1], "writeTextAsync", 2);
            let encoding = if args.len() > 2 {
                expect_string(&args[2], "writeTextAsync", 3)
            } else {
                "utf8".to_string()
            };

            // Do the real work RIGHT HERE
            fs_write_text_sync(&path, &text, &encoding);

            // Wrap resolved value (null) for the Furure pipeline
            Value::Furure(Box::new(Value::Null))
        })),
    );

    /// Fs.appendText(path, text, encoding = "utf8") -> null
    map.insert(
        "appendText".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.appendText(path, text, encoding?): expected at least 2 arguments");
            }

            let path = expect_string(&args[0], "appendText", 1);
            let text = expect_string(&args[1], "appendText", 2);
            let encoding = if args.len() > 2 {
                expect_string(&args[2], "appendText", 3)
            } else {
                "utf8".to_string()
            };

            fs_append_text_sync(&path, &text, &encoding)
        })),
    );

    // ============================================================
    // BINARY FILE API (SYNC)
    // ============================================================

    /// Fs.readBytes(path) -> array<number>
    map.insert(
        "readBytes".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readBytes(path): missing `path` argument");
            }

            let path = expect_string(&args[0], "readBytes", 1);
            let bytes = fs_read_bytes_sync(&path);

            let values = bytes.into_iter().map(|b| Value::Number(b as f64)).collect();

            Value::Array {
                values: Rc::new(RefCell::new(values)),
                proto: create_array_proto(),
            }
        })),
    );

    /// Fs.writeBytes(path, bytes) -> null
    map.insert(
        "writeBytes".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.writeBytes(path, bytes): expected 2 arguments");
            }

            let path = expect_string(&args[0], "writeBytes", 1);
            let bytes = expect_bytes(&args[1], "writeBytes");

            fs_write_bytes_sync(&path, &bytes);
            Value::Null
        })),
    );

    // ============================================================
    // OTHER SYNC HELPERS
    // ============================================================

    /// Fs.exists(path) -> bool
    map.insert(
        "exists".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.exists(path): missing `path` argument");
            }

            let path = expect_string(&args[0], "exists", 1);
            fs_exists_sync(&path)
        })),
    );

    /// Fs.readdir(path) -> array<string>
    map.insert(
        "readdir".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readdir(path): missing `path` argument");
            }

            let path = expect_string(&args[0], "readdir", 1);
            fs_readdir_sync(&path)
        })),
    );

    /// Fs.mkdir(path, recursive = false) -> null
    map.insert(
        "mkdir".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.mkdir(path, recursive?): missing `path` argument");
            }

            let path = expect_string(&args[0], "mkdir", 1);
            let recursive = if args.len() > 1 {
                expect_bool(&args[1], "mkdir", 2)
            } else {
                false
            };

            fs_mkdir_sync(&path, recursive)
        })),
    );

    /// Fs.rm(path, recursive = false) -> null
    map.insert(
        "rm".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.rm(path, recursive?): missing `path` argument");
            }

            let path = expect_string(&args[0], "rm", 1);
            let recursive = if args.len() > 1 {
                expect_bool(&args[1], "rm", 2)
            } else {
                false
            };

            fs_rm_sync(&path, recursive)
        })),
    );

    /// Fs.readJson(path, encoding = "utf8") -> any
    map.insert(
        "readJson".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readJson(path, encoding?): missing `path` argument");
            }

            let path = expect_string(&args[0], "readJson", 1);
            let encoding = if args.len() > 1 {
                expect_string(&args[1], "readJson", 2)
            } else {
                "utf8".to_string()
            };

            fs_read_json_sync(&path, &encoding)
        })),
    );

    /// Fs.writeJson(path, value, pretty = false, encoding = "utf8") -> null
    map.insert(
        "writeJson".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.writeJson(path, value, pretty?, encoding?): expected at least 2 arguments");
            }

            let path = expect_string(&args[0], "writeJson", 1);
            let value = &args[1];

            let pretty = if args.len() > 2 {
                expect_bool(&args[2], "writeJson", 3)
            } else {
                false
            };

            let encoding = if args.len() > 3 {
                expect_string(&args[3], "writeJson", 4)
            } else {
                "utf8".to_string()
            };

            fs_write_json_sync(&path, value, pretty, &encoding)
        })),
    );

    // ============================================================
    // ASYNC PROMISE-STYLE WRAPPERS (THREAD-BACKED)
    // ============================================================

    map.insert(
        "readTextAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readTextAsync(path, encoding?): missing `path` argument");
            }

            let path = expect_string(&args[0], "readTextAsync", 1);
            let encoding = if args.len() > 1 {
                expect_string(&args[1], "readTextAsync", 2)
            } else {
                "utf8".to_string()
            };

            // ✅ Do the real work immediately
            let result = fs_read_text_sync(&path, &encoding);

            // ✅ Store the *resolved* value in the Furure
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "writeTextAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.writeTextAsync(path, text, encoding?): expected at least 2 arguments");
            }

            let path = expect_string(&args[0], "writeTextAsync", 1);
            let text = expect_string(&args[1], "writeTextAsync", 2);
            let encoding = if args.len() > 2 {
                expect_string(&args[2], "writeTextAsync", 3)
            } else {
                "utf8".to_string()
            };

            // ✅ Actually write now
            fs_write_text_sync(&path, &text, &encoding);

            // ✅ The async result is just `null`
            Value::Furure(Box::new(Value::Null))
        })),
    );

    map.insert(
        "appendTextAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.appendTextAsync(path, text, encoding?): expected at least 2 arguments");
            }

            let path = expect_string(&args[0], "appendTextAsync", 1);
            let text = expect_string(&args[1], "appendTextAsync", 2);
            let encoding = if args.len() > 2 {
                expect_string(&args[2], "appendTextAsync", 3)
            } else {
                "utf8".to_string()
            };

            let result = fs_append_text_sync(&path, &text, &encoding);
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "readBytesAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readBytesAsync(path): missing `path` argument");
            }

            let path = expect_string(&args[0], "readBytesAsync", 1);
            let bytes = fs_read_bytes_sync(&path);

            let values = bytes.into_iter().map(|b| Value::Number(b as f64)).collect();

            let arr = Value::Array {
                values: Rc::new(RefCell::new(values)),
                proto: create_array_proto(),
            };

            Value::Furure(Box::new(arr))
        })),
    );

   map.insert(
        "writeBytesAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.writeBytesAsync(path, bytes): expected 2 arguments");
            }

            let path = expect_string(&args[0], "writeBytesAsync", 1);
            let bytes = expect_bytes(&args[1], "writeBytesAsync");

            fs_write_bytes_sync(&path, &bytes);
            Value::Furure(Box::new(Value::Null))
        })),
    );

    map.insert(
        "existsAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.existsAsync(path): missing `path` argument");
            }

            let path = expect_string(&args[0], "existsAsync", 1);
            let result = fs_exists_sync(&path);
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "readdirAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readdirAsync(path): missing `path` argument");
            }

            let path = expect_string(&args[0], "readdirAsync", 1);
            let result = fs_readdir_sync(&path);
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "mkdirAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.mkdirAsync(path, recursive?): missing `path` argument");
            }

            let path = expect_string(&args[0], "mkdirAsync", 1);
            let recursive = if args.len() > 1 {
                expect_bool(&args[1], "mkdirAsync", 2)
            } else {
                false
            };

            let result = fs_mkdir_sync(&path, recursive);
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "rmAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.rmAsync(path, recursive?): missing `path` argument");
            }

            let path = expect_string(&args[0], "rmAsync", 1);
            let recursive = if args.len() > 1 {
                expect_bool(&args[1], "rmAsync", 2)
            } else {
                false
            };

            let result = fs_rm_sync(&path, recursive);
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "readJsonAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.is_empty() {
                panic!("Fs.readJsonAsync(path, encoding?): missing `path` argument");
            }

            let path = expect_string(&args[0], "readJsonAsync", 1);
            let encoding = if args.len() > 1 {
                expect_string(&args[1], "readJsonAsync", 2)
            } else {
                "utf8".to_string()
            };

            let result = fs_read_json_sync(&path, &encoding);
            Value::Furure(Box::new(result))
        })),
    );

    map.insert(
        "writeJsonAsync".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if args.len() < 2 {
                panic!("Fs.writeJsonAsync(path, value, pretty?, encoding?): expected at least 2 arguments");
            }

            let path = expect_string(&args[0], "writeJsonAsync", 1);
            let value = args[1].clone();

            let pretty = if args.len() > 2 {
                expect_bool(&args[2], "writeJsonAsync", 3)
            } else {
                false
            };

            let encoding = if args.len() > 3 {
                expect_string(&args[3], "writeJsonAsync", 4)
            } else {
                "utf8".to_string()
            };

            let result = fs_write_json_sync(&path, &value, pretty, &encoding);
            Value::Furure(Box::new(result))
        })),
    );

    Value::Object {
        fields: Rc::new(RefCell::new(map)),
    }
}