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

 #![allow(dead_code, unused_variables, unused_imports)]

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number,
    String,
    Identifier,
    Keyword,
    Symbol,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    tokens.push(Token {
        kind: TokenKind::Eof,
        lexme: "".to_string(),
        line: 1,
    });

    tokens
}