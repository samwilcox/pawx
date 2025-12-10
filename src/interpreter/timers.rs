/*
 * ============================================================================
 * PAWX - Code with Claws!
 * ============================================================================
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
 * ============================================================================
 */

/*!
 * PAWX Timer Runtime
 * ------------------
 * 
 * This module implements JavaScript-style asynchronous timers for PAWX:
 * 
 *  • setTimeout(fn, ms)
 *  • setInterval(fn, ms)
 *  • clearTimeout(id)
 *  • clearInterval(id)
 * 
 * The runtime is thread-backed but **event execution is always dispatched
 * back onto the main interpreter thread** via a message pump.
 * 
 * This design keeps:
 *  - Deterministic execution
 *  - No race conditions in the interpreter
 *  - Safe cancellation
 */

use crate::interpreter::environment::Environment;
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::Duration;

/* ============================================================================
 * Internal Timer Message Types
 * ============================================================================
 */

/// Messages sent from background timer threads back to the main interpreter.
#[derive(Debug, Clone, Copy)]
pub enum TimerMessage {
    /// Fired once after a timeout delay.
    Timeout(u64),

    /// Fired repeatedly at a fixed interval.
    IntervalTick(u64),
}

/// Internal record for each active timer.
#[derive(Clone)]
pub struct TimerEntry {
    /// Callback function invoked when the timer fires.
    pub callback: Value,

    /// True if this is an interval timer (repeats).
    pub is_interval: bool,

    /// Cancellation flag used for intervals.
    pub cancel_flag: Option<Arc<AtomicBool>>,
}

/// Shared runtime timer state.
pub struct TimerRuntime {
    pub tx: Sender<TimerMessage>,
    pub rx: Receiver<TimerMessage>,
    pub timers: Rc<RefCell<HashMap<u64, TimerEntry>>>,
    pub next_id: Rc<RefCell<u64>>,
}

impl TimerRuntime {
    /// Creates and initializes a new PAWX timer runtime.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            timers: Rc::new(RefCell::new(HashMap::new())),
            next_id: Rc::new(RefCell::new(1)),
            tx,
            rx,
        }
    }
}

/* ============================================================================
 * Timer Runtime Initialization
 * ============================================================================
 */

/// Creates a fresh timer runtime instance.
///
/// This should be created once per interpreter execution.
pub fn create_timer_runtime() -> TimerRuntime {
    let (tx, rx) = mpsc::channel();

    TimerRuntime {
        tx,
        rx,
        timers: Rc::new(RefCell::new(HashMap::new())),
        next_id: Rc::new(RefCell::new(1)),
    }
}

/* ============================================================================
 * Installing Built-in Timer Functions
 * ============================================================================
 */

/// Installs PAWX timer functions into the global environment.
///
/// This registers:
///  • setTimeout
///  • setInterval
///  • clearTimeout
///  • clearInterval
pub fn install_timers(env: Rc<RefCell<Environment>>) -> TimerRuntime {
    let runtime = TimerRuntime::new();

    let timers = runtime.timers.clone();
    let tx = runtime.tx.clone();
    let next_id = runtime.next_id.clone();

    env.borrow_mut().define_public(
        "setTimeout".to_string(),
        Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
            if args.len() != 2 {
                panic!("setTimeout(fn, ms) requires 2 arguments");
            }

            let callback = args[0].clone();
            let ms = match args[1] {
                Value::Number(n) => n as u64,
                _ => panic!("setTimeout delay must be a number"),
            };

            if !matches!(callback, Value::NativeFunction(_)) {
                panic!("setTimeout requires a function as first argument");
            }

            let id = {
                let mut counter = next_id.borrow_mut();
                let id = *counter;
                *counter += 1;
                id
            };

            timers.borrow_mut().insert(
                id,
                TimerEntry {
                    callback,
                    is_interval: false,
                    cancel_flag: None,
                },
            );

            let tx_cloned = tx.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                let _ = tx_cloned.send(TimerMessage::Timeout(id));
            });

            Value::Number(id as f64)
        })),
    );

    runtime
}

/* --------------------------------------------------------------------------
 * setTimeout(fn, ms)
 * ----------------------------------------------------------------------- */

fn install_set_timeout(env: &mut Environment, runtime: &TimerRuntime) {
    let timers = runtime.timers.clone();
    let next_id = runtime.next_id.clone();
    let tx = runtime.tx.clone();

    env.define_public(
        "setTimeout".to_string(),
        Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
            if args.len() != 2 {
                panic!("setTimeout(fn, ms) requires 2 arguments");
            }

            let callback = args[0].clone();
            let delay_ms = match args[1] {
                Value::Number(n) => n as u64,
                _ => panic!("setTimeout delay must be a number"),
            };

            if !matches!(callback, Value::NativeFunction(_)) {
                panic!("setTimeout requires a function as first argument");
            }

            // Allocate unique timer ID
            let id = {
                let mut counter = next_id.borrow_mut();
                let id = *counter;
                *counter += 1;
                id
            };

            timers.borrow_mut().insert(
                id,
                TimerEntry {
                    callback,
                    is_interval: false,
                    cancel_flag: None,
                },
            );

            let tx_cloned = tx.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(delay_ms));
                let _ = tx_cloned.send(TimerMessage::Timeout(id));
            });

            Value::Number(id as f64)
        })),
    );
}

/* --------------------------------------------------------------------------
 * setInterval(fn, ms)
 * ----------------------------------------------------------------------- */

fn install_set_interval(env: &mut Environment, runtime: &TimerRuntime) {
    let timers = runtime.timers.clone();
    let next_id = runtime.next_id.clone();
    let tx = runtime.tx.clone();

    env.define_public(
        "setInterval".to_string(),
        Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
            if args.len() != 2 {
                panic!("setInterval(fn, ms) requires 2 arguments");
            }

            let callback = args[0].clone();
            let delay_ms = match args[1] {
                Value::Number(n) => n as u64,
                _ => panic!("setInterval delay must be a number"),
            };

            if !matches!(callback, Value::NativeFunction(_)) {
                panic!("setInterval requires a function as first argument");
            }

            let id = {
                let mut counter = next_id.borrow_mut();
                let id = *counter;
                *counter += 1;
                id
            };

            let stop_flag = Arc::new(AtomicBool::new(false));

            timers.borrow_mut().insert(
                id,
                TimerEntry {
                    callback,
                    is_interval: true,
                    cancel_flag: Some(stop_flag.clone()),
                },
            );

            let tx_cloned = tx.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_millis(delay_ms));

                    if stop_flag.load(Ordering::SeqCst) {
                        break;
                    }

                    let _ = tx_cloned.send(TimerMessage::IntervalTick(id));
                }
            });

            Value::Number(id as f64)
        })),
    );
}

/* --------------------------------------------------------------------------
 * clearTimeout(id)
 * ----------------------------------------------------------------------- */

fn install_clear_timeout(env: &mut Environment, runtime: &TimerRuntime) {
    let timers = runtime.timers.clone();

    env.define_public(
        "clearTimeout".to_string(),
        Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
            if args.len() != 1 {
                panic!("clearTimeout(id) requires 1 argument");
            }

            let id = match args[0] {
                Value::Number(n) => n as u64,
                _ => panic!("clearTimeout(id) requires a numeric id"),
            };

            timers.borrow_mut().remove(&id);
            Value::Null
        })),
    );
}

/* --------------------------------------------------------------------------
 * clearInterval(id)
 * ----------------------------------------------------------------------- */

fn install_clear_interval(env: &mut Environment, runtime: &TimerRuntime) {
    let timers = runtime.timers.clone();

    env.define_public(
        "clearInterval".to_string(),
        Value::NativeFunction(Arc::new(move |args: Vec<Value>| -> Value {
            if args.len() != 1 {
                panic!("clearInterval(id) requires 1 argument");
            }

            let id = match args[0] {
                Value::Number(n) => n as u64,
                _ => panic!("clearInterval(id) requires a numeric id"),
            };

            if let Some(entry) = timers.borrow_mut().remove(&id) {
                if let Some(flag) = entry.cancel_flag {
                    flag.store(true, Ordering::SeqCst);
                }
            }

            Value::Null
        })),
    );
}

/* ============================================================================
 * Timer Event Dispatcher (Pump)
 * ============================================================================
 */

/// Dispatches any pending timer events onto the main interpreter thread.
///
/// This **must be called regularly** from the interpreter execution loop.
pub fn pump_timers(runtime: &TimerRuntime) {
    loop {
        let msg = match runtime.rx.try_recv() {
            Ok(m) => m,
            Err(_) => break,
        };

        match msg {
            TimerMessage::Timeout(id) => {
                let entry = runtime.timers.borrow_mut().remove(&id);
                if let Some(entry) = entry {
                    if let Value::NativeFunction(f) = entry.callback {
                        f(vec![]);
                    }
                }
            }

            TimerMessage::IntervalTick(id) => {
                let callback = {
                    let map = runtime.timers.borrow();
                    map.get(&id).map(|e| e.callback.clone())
                };

                if let Some(Value::NativeFunction(f)) = callback {
                    f(vec![]);
                }
            }
        }
    }
}