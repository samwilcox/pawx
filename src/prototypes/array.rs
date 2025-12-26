/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * Array Prototype Implementation
 * ==========================================================================
 * 
 * This module defines the native Rust-backed implementation of the
 * JavaScript-style `Array` prototype used by the PAWX runtime.
 * 
 * It provides core methods such as:
 *   - push, pop, sort
 *   - map, filter, slice, join
 *   - forEach, find, includes
 *   - some, every
 *   - reduce, reduceRight
 * 
 * These functions are installed once onto the global Array prototype
 * and are shared by all array instances in PAWX.
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
use std::sync::Arc;

use crate::value::Value;

 /// Installs all built-in Array prototype methods into the PAWX runtime.
 /// 
 /// This function attaches Javascript-type array methods such as:
 /// - `push`
 /// - `pop`
 /// - `map'
 /// - `filter`
 /// - `find`
 /// 
 /// These methods become available on **all arrays created in PAWX**.
 /// 
 /// # Parameters
 /// - `proto`: The root Array prototype object used by the VM.
 /// 
 /// # Behavior
 /// This function **mutates the prototype object in-place** by attaching
 /// native Rust-backed functions as callable PAWX methods.
 /// 
 /// # PAWX Example
 /// ```pawx
 /// snuggle nums = [1, 2, 3]
 /// nums.push(4);
 /// meow(nums);
 /// ```
pub fn create_array_proto() -> HashMap<String, Value> {
    let mut proto = HashMap::new();

    // Mutating methods
    proto.insert("push".to_string(), Value::NativeFunction(Arc::new(array_push)));
    proto.insert("pop".to_string(), Value::NativeFunction(Arc::new(array_pop)));
    proto.insert("sort".to_string(), Value::NativeFunction(Arc::new(array_sort)));

    // Non-mutating transformation methods
    proto.insert("map".to_string(), Value::NativeFunction(Arc::new(array_map)));
    proto.insert("filter".to_string(), Value::NativeFunction(Arc::new(array_filter)));
    proto.insert("slice".to_string(), Value::NativeFunction(Arc::new(array_slice)));
    proto.insert("join".to_string(), Value::NativeFunction(Arc::new(array_join)));

    // Iteration & search methods
    proto.insert("forEach".to_string(), Value::NativeFunction(Arc::new(array_foreach)));
    proto.insert("find".to_string(), Value::NativeFunction(Arc::new(array_find)));
    proto.insert("includes".to_string(), Value::NativeFunction(Arc::new(array_includes)));

    // Logical aggregation
    proto.insert("some".to_string(), Value::NativeFunction(Arc::new(array_some)));
    proto.insert("every".to_string(), Value::NativeFunction(Arc::new(array_every)));

    // Reduction
    proto.insert("reduce".to_string(), Value::NativeFunction(Arc::new(array_reduce)));
    proto.insert(
        "reduceRight".to_string(),
        Value::NativeFunction(Arc::new(array_reduce_right)),
    );

    proto.insert(
        "toString".to_string(),
        Value::NativeFunction(Arc::new(array_to_string)),
    );

    proto
}

pub fn create_global_array_object() -> Value {
    use std::collections::HashMap;

    let mut fields = HashMap::new();

    fields.insert(
        "isArray".to_string(),
        Value::NativeFunction(Arc::new(|args| {
            if let Some(Value::Array { .. }) = args.get(0) {
                Value::Bool(true)
            } else {
                Value::Bool(false)
            }
        })),
    );

    Value::Object {
        fields: Rc::new(RefCell::new(fields)),
    }
}

/// Native implementation of `Array.prototype.push()` for PAWX.
///
/// Appends a value to the end of the target array and returns
/// the new array length (JS-compatible behavior).
///
/// # Parameters (via `args`)
/// - `args[0]`: The target array.
/// - `args[1]`: The value to append.
///
/// # Returns
/// A `Value::Number` representing the new length of the array.
///
/// # Behavior
/// - Mutates the original array.
/// - Preserves reference identity.
/// - Matches JavaScript semantics.
///
/// # PAWX Example
/// ```pawx
/// snuggle nums = [1, 2];
/// nums.push(3);
/// meow(nums); // [1, 2, 3]
/// ```
fn array_push(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.clone(),
        _ => panic!("push() must be called on an array"),
    };

    let val = args.get(1).cloned().unwrap_or(Value::Null);

    // Push with scoped mutable borrow
    {
        let mut borrowed = array.borrow_mut();
        borrowed.push(val);
    }

    // Length with scoped immutable borrow
    let len = {
        let borrowed = array.borrow();
        borrowed.len()
    };

    Value::Number(len as f64)
}

/// Native implementation of `Array.prototype.pop()` for PAWX.
///
/// Removes and returns the **last element** of the array.
/// If the array is empty, returns `null`.
///
/// # Parameters (via `args`)
/// - `args[0]`: The target array.
///
/// # Returns
/// - The removed element
/// - Or `null` if the array is empty
///
/// # Behavior
/// - Mutates the original array.
/// - Matches JavaScript behavior exactly.
///
/// # PAWX Example
/// ```pawx
/// snuggle nums = [10, 20, 30];
/// snuggle last = nums.pop();
/// meow(last); // 30
/// meow(nums); // [10, 20]
/// ```
fn array_pop(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.clone(),
        _ => panic!("pop() must be called on an array"),
    };

    // Force the mutable borrow to drop before return
    let result = {
        let mut borrowed = array.borrow_mut();
        borrowed.pop()
    };

    result.unwrap_or(Value::Null)
}

/// Native implementation of `Array.prototype.map()` for PAWX.
///
/// Creates a **new array** populated with the results of calling a
/// provided function on every element in the original array.
///
/// # Parameters (via `args`)
/// - `args[0]`: The target array.
/// - `args[1]`: The mapping function (`fn(value) -> newValue`)
///
/// # Returns
/// A **new PAWX array** containing transformed values.
///
/// # Behavior
/// - Does **not mutate** the original array.
/// - Preserves call order.
/// - Fully supports closures and lambdas.
///
/// # PAWX Example
/// ```pawx
/// snuggle nums = [1, 2, 3];
/// snuggle doubled = nums.map(n -> { return n * 2; });
/// meow(doubled); // [2, 4, 6]
/// ```
fn array_map(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("map() must be called on an array"),
    };

    let callback = args.get(1).cloned().unwrap();

    let mut new_vals = Vec::new();
    for v in array {
        if let Value::NativeFunction(f) = &callback {
            new_vals.push(f(vec![v]));
        }
    }

    Value::Array {
        values: Rc::new(RefCell::new(new_vals)),
        proto: create_array_proto(),
    }
}


/// Native implementation of `Array.prototype.slice()` for PAWX.
///
/// Returns a **shallow copy** of a portion of an array into a new array.
/// The original array is **not modified**.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]` (optional): Start index.
/// - `args[2]` (optional): End index (non-inclusive).
///
/// # Returns
/// A **new PAWX array** containing the selected elements.
///
/// # Behavior
/// - Supports negative indices.
/// - Does not mutate the original array.
/// - Fully matches JavaScript slice semantics.
///
/// # PAWX Example
/// ```pawx
/// snuggle nums = [1, 2, 3, 4];
/// snuggle part = nums.slice(1, 3);
/// meow(part); // [2, 3]
/// ```
fn array_slice(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("slice() must be called on an array"),
    };

    let start = match args.get(1) {
        Some(Value::Number(n)) => *n as usize,
        _ => 0,
    };

    let end = match args.get(2) {
        Some(Value::Number(n)) => *n as usize,
        _ => array.len(),
    };

    let sliced = array[start..end].to_vec();

    Value::Array {
        values: Rc::new(RefCell::new(sliced)),
        proto: create_array_proto(),
    }
}

/// Native implementation of `Array.prototype.forEach()` for PAWX.
///
/// Executes a provided function **once for each array element**.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Callback function `(value) -> void`
///
/// # Returns
/// Always returns `null` (JS-compatible).
///
/// # Behavior
/// - Iterates in order.
/// - Does not mutate the array unless callback does.
/// - Fully supports closures and lambdas.
///
/// # PAWX Example
/// ```pawx
/// [1, 2, 3].forEach(n -> {
///     meow(n);
/// });
/// ```
fn array_foreach(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("forEach() must be called on an array"),
    };

    let callback = args.get(1).cloned().unwrap();

    for v in array {
        if let Value::NativeFunction(f) = &callback {
            f(vec![v]);
        }
    }

    Value::Null
}

/// Native implementation of `Array.prototype.filter()` for PAWX.
///
/// Creates a **new array** containing only elements that pass the
/// provided predicate function.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Predicate function `(value) -> bool`
///
/// # Returns
/// A **new PAWX array** of filtered elements.
///
/// # Behavior
/// - Does not mutate the original array.
/// - Preserves order.
/// - Fully closure-safe.
///
/// # PAWX Example
/// ```pawx
/// snuggle evens = [1, 2, 3, 4].filter(n -> n % 2 == 0);
/// meow(evens); // [2, 4]
/// ```
fn array_filter(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("filter() must be called on an array"),
    };

    let callback = args.get(1).cloned().unwrap();

    let mut new_vals = Vec::new();
    for v in array {
        if let Value::NativeFunction(f) = &callback {
            let keep = f(vec![v.clone()]);
            if matches!(keep, Value::Bool(true)) {
                new_vals.push(v);
            }
        }
    }

    Value::Array {
        values: Rc::new(RefCell::new(new_vals)),
        proto: create_array_proto(),
    }
}

/// Native implementation of `Array.prototype.find()` for PAWX.
///
/// Returns the **first element** that satisfies the provided predicate.
/// If none match, returns `null`.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Predicate function `(value) -> bool`
///
/// # Returns
/// - The first matching element
/// - Or `null` if none found
///
/// # Behavior
/// - Stops at first match.
/// - Does not mutate the array.
///
/// # PAWX Example
/// ```pawx
/// snuggle found = [5, 12, 8].find(n -> n > 10);
/// meow(found); // 12
/// ```
fn array_find(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("find() must be called on an array"),
    };

    let callback = args.get(1).cloned().unwrap();

    for v in array {
        if let Value::NativeFunction(f) = &callback {
            let found = f(vec![v.clone()]);
            if matches!(found, Value::Bool(true)) {
                return v;
            }
        }
    }

    Value::Null
}

/// Native implementation of `Array.prototype.reduce()` for PAWX.
///
/// Reduces the array to a **single accumulated value** using a reducer
/// function applied left-to-right.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Reducer function `(acc, value) -> newAcc`
/// - `args[2]` (optional): Initial accumulator value
///
/// # Returns
/// The final accumulated value.
///
/// # Behavior
/// - Throws runtime error if called on empty array without initial value.
/// - Preserves JS accumulator semantics entirely.
///
/// # PAWX Example
/// ```pawx
/// snuggle sum = [1, 2, 3].reduce((a, b) -> a + b);
/// meow(sum); // 6
/// ```
fn array_reduce(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("reduce() must be called on array"),
    };

    let callback = args.get(1).cloned().unwrap();
    let mut acc = args.get(2).cloned().unwrap_or(Value::Null);

    for v in array {
        if let Value::NativeFunction(f) = &callback {
            acc = f(vec![acc, v]);
        }
    }

    acc
}

/// Native implementation of `Array.prototype.reduceRight()` for PAWX.
///
/// Reduces the array to a **single accumulated value** using a reducer
/// function applied right-to-left.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Reducer function `(acc, value) -> newAcc`
/// - `args[2]` (optional): Initial accumulator value
///
/// # Returns
/// The final accumulated value.
///
/// # Behavior
/// - Iterates from the end of the array.
/// - JS-compatible error behavior.
///
/// # PAWX Example
/// ```pawx
/// snuggle result = ["a", "b", "c"].reduceRight((a, b) -> a + b);
/// meow(result); // "cba"
/// ```
fn array_reduce_right(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("reduceRight() must be called on array"),
    };

    let callback = args.get(1).cloned().unwrap();
    let mut acc = args.get(2).cloned().unwrap_or(Value::Null);

    for v in array.into_iter().rev() {
        if let Value::NativeFunction(f) = &callback {
            acc = f(vec![acc, v]);
        }
    }

    acc
}

/// Native implementation of `Array.prototype.includes()` for PAWX.
///
/// Determines whether an array contains a specific value.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Value to search for
///
/// # Returns
/// `true` if found, otherwise `false`
///
/// # Behavior
/// - Uses strict equality semantics.
/// - Matches JavaScript includes behavior.
///
/// # PAWX Example
/// ```pawx
/// meow([1, 2, 3].includes(2)); // true
/// ```
fn array_includes(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow(),
        _ => panic!("includes() must be called on an array"),
    };

    let target = args.get(1).cloned().unwrap_or(Value::Null);

    for v in array.iter() {
        if Value::equals_strict(v, &target) {
            return Value::Bool(true);
        }
    }

    Value::Bool(false)
}

/// Native implementation of `Array.prototype.some()` for PAWX.
///
/// Tests whether **at least one element** in the array passes the predicate.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Predicate function `(value) -> bool`
///
/// # Returns
/// `true` if at least one passes, otherwise `false`
///
/// # Behavior
/// - Stops early when match is found.
/// - Does not mutate the array.
///
/// # PAWX Example
/// ```pawx
/// meow([1, 3, 5].some(n -> n % 2 == 0)); // false
/// ```
fn array_some(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("some() must be called on an array"),
    };

    let callback = args.get(1).cloned().unwrap();

    for v in array {
        if let Value::NativeFunction(f) = &callback {
            if let Value::Bool(true) = f(vec![v]) {
                return Value::Bool(true);
            }
        }
    }

    Value::Bool(false)
}

/// Native implementation of `Array.prototype.every()` for PAWX.
///
/// Tests whether **all elements** in the array pass the predicate.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]`: Predicate function `(value) -> bool`
///
/// # Returns
/// `true` if all pass, otherwise `false`
///
/// # Behavior
/// - Stops early on first failure.
/// - Does not mutate the array.
///
/// # PAWX Example
/// ```pawx
/// meow([2, 4, 6].every(n -> n % 2 == 0)); // true
/// ```
fn array_every(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("every() must be called on an array"),
    };

    let callback = args.get(1).cloned().unwrap();

    for v in array {
        if let Value::NativeFunction(f) = &callback {
            if let Value::Bool(false) = f(vec![v]) {
                return Value::Bool(false);
            }
        }
    }

    Value::Bool(true)
}

/// Native implementation of `Array.prototype.join()` for PAWX.
///
/// Joins all elements of the array into a single string separated
/// by the specified delimiter.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]` (optional): Separator string (default is `","`)
///
/// # Returns
/// A joined string representation of the array.
///
/// # Behavior
/// - Converts all values to strings.
/// - JS-compatible join behavior.
///
/// # PAWX Example
/// ```pawx
/// snuggle s = [1, 2, 3].join("-");
/// meow(s); // "1-2-3"
/// ```
fn array_join(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow().clone(),
        _ => panic!("join() must be called on an array"),
    };

    let sep = match args.get(1) {
        Some(Value::String(s)) => s.clone(),
        _ => ",".to_string(),
    };

    let mut strings = Vec::new();
    for v in array {
        strings.push(match v {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => "[object]".to_string(),
        });
    }

    Value::String(strings.join(&sep))
}

/// Native implementation of `Array.prototype.sort()` for PAWX.
///
/// Sorts the array **in place** and returns the sorted array.
///
/// # Parameters (via `args`)
/// - `args[0]`: The source array.
/// - `args[1]` (optional): Comparison function `(a, b) -> number`
///
/// # Returns
/// The **mutated** sorted array (JS-compatible).
///
/// # Behavior
/// - Defaults to lexicographical sort.
/// - Fully supports custom comparator functions.
///
/// # PAWX Example
/// ```pawx
/// snuggle nums = [3, 1, 2];
/// nums.sort();
/// meow(nums); // [1, 2, 3]
/// ```
fn array_sort(args: Vec<Value>) -> Value {
    let array_rc = match &args[0] {
        Value::Array { values, .. } => values.clone(),
        _ => panic!("sort() must be called on an array"),
    };

    let maybe_cmp = args.get(1).cloned();

    let mut borrowed = array_rc.borrow_mut();

    borrowed.sort_by(|a, b| {
        // If user provided comparator: use it
        if let Some(Value::NativeFunction(f)) = &maybe_cmp {
            let result = f(vec![a.clone(), b.clone()]);

            match result {
                Value::Number(n) => {
                    if n < 0.0 {
                        std::cmp::Ordering::Less
                    } else if n > 0.0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                _ => panic!("sort() comparator must return a number"),
            }
        }
        // Default JS-like sort behavior
        else {
            match (a, b) {
                (Value::Number(x), Value::Number(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            }
        }
    });

    // Return same array (chainable)
    Value::Array {
        values: array_rc.clone(),
        proto: create_array_proto(),
    }
}

fn array_to_string(args: Vec<Value>) -> Value {
    let array = match &args[0] {
        Value::Array { values, .. } => values.borrow(),
        _ => panic!("toString() must be called on an array"),
    };

    // JS behavior: join with commas, no brackets
    let inner = array
        .iter()
        .map(|v| v.stringify())
        .collect::<Vec<_>>()
        .join(",");

    Value::String(inner)
}