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

#![allow(dead_code, unused_variables, unused_imports)]

mod lexer;
mod parser;
mod ast;
mod interpreter;
mod environment;
mod value;
mod error;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: pawx <file.px>");
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1])
        .expect("Failed to read Pawx source file");

    run(&source);
}

fn run(source: &str) {
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    interpreter::run(ast);
}