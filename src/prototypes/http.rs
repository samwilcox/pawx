/*
 * ============================================================================
 * PAWX - Code with Claws! 🐾
 * ============================================================================
 *
 * HTTP Server + Request Parsing for PAWX
 *
 * Supports:
 *   - Query parsing      → req.query
 *   - JSON body parsing  → req.body
 *   - Form parsing       → req.body
 *   - Raw text fallback → req.body
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

use std::net::TcpListener;
use std::io::{Read, Write};
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::display::value_to_json;
use crate::value::Value;
use crate::interpreter::calls::call_value;
use crate::prototypes::array::create_array_proto;
use crate::ast::Expr;

use serde_json;

/* ============================================================================
 * PUBLIC API
 * ============================================================================
 */

pub fn create_global_http_object() -> Value {
    let mut map = HashMap::new();

    // Http.createServer(handler)
    map.insert(
        "createServer".into(),
        Value::NativeFunction(Arc::new(|args| {
            let handler = args.get(0).cloned().unwrap_or(Value::Null);

            let mut server = HashMap::new();

            // server.listen(port)
            server.insert(
                "listen".into(),
                Value::NativeFunction(Arc::new(move |listen_args| {
                    let port = match listen_args.get(0) {
                        Some(Value::Number(n)) => *n as u16,
                        _ => panic!("listen(port) requires a number"),
                    };

                    server_bind(port, handler.clone())
                })),
            );

            Value::Object {
                fields: Rc::new(RefCell::new(server)),
            }
        })),
    );

    Value::Object {
        fields: Rc::new(RefCell::new(map)),
    }
}

/* ============================================================================
 * SERVER CORE
 * ============================================================================
 */

fn server_bind(port: u16, handler: Value) -> Value {
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();

    println!("🐾 PAWX HTTP listening on http://localhost:{port}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Capture client IP safely
        let peer_ip = stream.peer_addr().ok().map(|a| a.ip());

        // Read request safely (prevents hanging)
        let mut buffer = [0u8; 8192];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            _ => continue,
        };

        let raw_request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

        // ✅ NOW RETURNS (req, res, response_body)
        let (req_val, res_val, response_body) = build_req_res(&raw_request, peer_ip);

        let handler_env = Rc::new(RefCell::new(
            crate::interpreter::environment::Environment::new(None),
        ));

        // Call handler(req, res) — we IGNORE whatever it returns.
        let _ = call_value(
            handler.clone(),
            vec![Expr::Literal(req_val), Expr::Literal(res_val)],
            handler_env,
        );

        // Prefer what res.json() stored; fall back to simple JSON
        let body_value = response_body.borrow().clone();

        let body = match body_value {
            // res.json wrote a proper JSON string
            Value::String(s) => s,

            // res.json was never called: send a minimal JSON object
            Value::Null => "{}".to_string(),

            // Some other value: stringify it once as JSON
            other => serde_json::to_string(&value_to_json_http(&other)).unwrap(),
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
            body.len(),
            body
        );

        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    }

    Value::Null
}

/* ============================================================================
 * REQUEST BUILDER
 * ============================================================================
 */

fn build_req_res(
    raw: &str,
    peer_ip: Option<std::net::IpAddr>,
) -> (Value, Value, Rc<RefCell<Value>>) {
    let mut lines = raw.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    let method = parts.get(0).unwrap_or(&"GET").to_string();
    let full_path = parts.get(1).unwrap_or(&"/");
    let (path, query_str) = split_path_query(full_path);

    let mut headers: HashMap<String, Value> = HashMap::new();
    let mut body = String::new();
    let mut reading_body = false;

    for line in lines {
        if line.is_empty() {
            reading_body = true;
            continue;
        }

        if reading_body {
            body.push_str(line);
        } else {
            let mut parts = line.splitn(2, ':');
            let k = parts.next().unwrap_or("").trim();
            let v = parts.next().unwrap_or("").trim();
            headers.insert(k.to_string(), Value::String(v.to_string()));
        }
    }

    let content_type = headers
        .get("Content-Type")
        .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
        .unwrap_or("");

    let hostname = headers
        .get("Host")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "localhost".to_string());

    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "Unknown".to_string());

    let body_value = parse_body(&body, content_type);

    /* -------------------------------
       IP OBJECT
    -------------------------------- */
    let mut ip_fields = HashMap::new();

    if let Some(ip) = peer_ip {
        match ip {
            std::net::IpAddr::V4(v4) => {
                ip_fields.insert("v4".into(), Value::String(v4.to_string()));
                ip_fields.insert("v6".into(), Value::Null);
            }
            std::net::IpAddr::V6(v6) => {
                ip_fields.insert("v4".into(), Value::Null);
                ip_fields.insert("v6".into(), Value::String(v6.to_string()));
            }
        }
    } else {
        ip_fields.insert("v4".into(), Value::Null);
        ip_fields.insert("v6".into(), Value::Null);
    }

    let ip_value = Value::Object {
        fields: Rc::new(RefCell::new(ip_fields)),
    };

    /* -------------------------------
       REQUEST OBJECT
    -------------------------------- */
    let mut req_fields = HashMap::new();
    req_fields.insert("method".into(), Value::String(method));
    req_fields.insert("path".into(), Value::String(path.clone()));
    req_fields.insert("url".into(), Value::String(path));
    req_fields.insert("ip".into(), ip_value);
    req_fields.insert("hostname".into(), Value::String(hostname));
    req_fields.insert("userAgent".into(), Value::String(user_agent));

    req_fields.insert(
        "query".into(),
        Value::Object {
            fields: Rc::new(RefCell::new(parse_query(query_str))),
        },
    );

    req_fields.insert(
        "headers".into(),
        Value::Object {
            fields: Rc::new(RefCell::new(headers)),
        },
    );

    req_fields.insert("body".into(), body_value);

    let req = Value::Object {
        fields: Rc::new(RefCell::new(req_fields)),
    };

    /* -------------------------------
       RESPONSE OBJECT + BODY CELL
    -------------------------------- */
    let response_body: Rc<RefCell<Value>> = Rc::new(RefCell::new(Value::Null));
    let res_fields = Rc::new(RefCell::new(HashMap::new()));

    // Copies of shared cells for closures
    let res_fields_for_status = res_fields.clone();
    let res_fields_for_json = res_fields.clone();
    let body_for_json = response_body.clone();

    // --- res.status(code) ---
    {
        let fields = res_fields_for_status.clone();
        res_fields.borrow_mut().insert(
            "status".into(),
            Value::NativeFunction(Arc::new(move |_args| {
                // You can later wire status code into response if you want.
                Value::Object {
                    fields: fields.clone(),
                }
            })),
        );
    }

    // --- res.json(data) ---
    {
        let fields = res_fields_for_json.clone();
        let body_cell = body_for_json.clone();

        res_fields.borrow_mut().insert(
            "json".into(),
            Value::NativeFunction(Arc::new(move |args| {
                // Accept either plain String or any Value
                let json_str = match args.get(0) {
                    // Handler passed a raw string: use it as-is
                    Some(Value::String(s)) => s.clone(),

                    // Handler passed some structured Value: convert ONCE
                    Some(v) => serde_json::to_string(&value_to_json_http(v)).unwrap(),

                    // Nothing: send empty object
                    None => "{}".to_string(),
                };

                *body_cell.borrow_mut() = Value::String(json_str);

                // Return res for chaining: res.status(...).json(...)
                Value::Object {
                    fields: fields.clone(),
                }
            })),
        );
    }

    let res = Value::Object {
        fields: res_fields.clone(),
    };

    (req, res, response_body)
}

/* ============================================================================
 * QUERY + BODY PARSERS
 * ============================================================================
 */

fn value_to_json_http(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),

        Value::Number(n) => {
            serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }

        Value::String(s) => serde_json::Value::String(s.clone()),

        Value::Array { values, .. } => {
            let elems = values
                .borrow()
                .iter()
                .map(|v| value_to_json_http(v))
                .collect();
            serde_json::Value::Array(elems)
        }

        Value::Object { fields } => {
            let mut map = serde_json::Map::new();
            for (k, v) in fields.borrow().iter() {
                map.insert(k.clone(), value_to_json_http(v));
            }
            serde_json::Value::Object(map)
        }

        // For functions, classes, modules, etc – just give a readable marker
        _ => serde_json::Value::String("[non-json]".to_string()),
    }
}

fn split_path_query(path: &str) -> (String, &str) {
    if let Some(i) = path.find('?') {
        (path[..i].to_string(), &path[i + 1..])
    } else {
        (path.to_string(), "")
    }
}

fn parse_query(q: &str) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    for pair in q.split('&') {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");

        if !k.is_empty() {
            map.insert(k.to_string(), Value::String(url_decode(v)));
        }
    }

    map
}

fn parse_body(body: &str, ct: &str) -> Value {
    if ct.contains("application/json") {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(v) => json_to_value(v),
            Err(_) => Value::Null,
        }
    } else if ct.contains("application/x-www-form-urlencoded") {
        Value::Object {
            fields: Rc::new(RefCell::new(parse_query(body))),
        }
    } else {
        Value::String(body.to_string())
    }
}

fn json_to_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,

        serde_json::Value::Bool(b) => Value::Bool(b),

        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Null
            }
        }

        serde_json::Value::String(s) => Value::String(s),

        serde_json::Value::Array(arr) => Value::Array {
            values: Rc::new(RefCell::new(
                arr.into_iter().map(json_to_value).collect()
            )),
            proto: create_array_proto(),
        },

        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_value(v));
            }

            Value::Object {
                fields: Rc::new(RefCell::new(map)),
            }
        }
    }
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hi = chars.next();
            let lo = chars.next();

            if let (Some(hi), Some(lo)) = (hi, lo) {
                let hex = format!("{}{}", hi, lo);
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}