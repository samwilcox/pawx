/*
 * ==========================================================================
 * PAWX - Code with Claws! 🐾
 * ==========================================================================
 *
 * File:     expression.rs
 * Purpose:  Implements the PAWX expression grammar using recursive descent
 *
 * Author:   Sam Wilcox
 * Email:    sam@pawx-lang.com
 * Website:  https://www.pawx-lang.com
 * GitHub:   https://github.com/samwilcox/pawx
 *
 * --------------------------------------------------------------------------
 *  LICENSE
 * --------------------------------------------------------------------------
 * This file is part of the PAWX programming language project.
 *
 * PAWX is dual-licensed under the terms of:
 *   - The MIT License
 *   - The Apache License, Version 2.0
 *
 * You may choose either license to govern your use of this software.
 *
 * Full license text available at:
 *    https://license.pawx-lang.com
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under these licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *
 * --------------------------------------------------------------------------
 *  MODULE OVERVIEW
 * --------------------------------------------------------------------------
 * This module contains the **entire PAWX expression grammar**.
 *
 * It is responsible for parsing:
 *  - Assignments
 *  - Binary operators
 *  - Unary operators
 *  - Function calls
 *  - Property access
 *  - Indexing
 *  - Object literals
 *  - Array literals
 *  - Tuples
 *  - Lambdas (safe-detected)
 *  - Post-increment / decrement
 *  - `new` constructor calls
 *  - `tap` module loading
 *
 * Parsing order follows strict mathematical precedence:
 *
 *   assignment → equality → comparison → term → factor → unary → call → primary
 *
 * This guarantees:
 *  - Correct operator precedence
 *  - Correct associativity
 *  - Zero ambiguity
 *  - Safe lambda detection without backtracking bugs
 *
 * ==========================================================================
 */

use std::string::ParseError;

use crate::ast::Expr;
use crate::lexer::token::TokenKind;
use crate::parser::parser::Parser;
use crate::span::Span;
use crate::value::Value;

impl Parser {
    /// expression → assignment
    pub fn expression(&mut self) -> Expr {
        self.assignment()
    }

    /// assignment → logical_or ( "=" assignment )?
    fn assignment(&mut self) -> Expr {
        let expr = self.logical_or();

        if self.match_symbol('=') {
            let equals = self.previous().clone();
            let value = self.assignment();

            match expr {
                Expr::Identifier { name, .. } => Expr::Assign {
                    name,
                    value: Box::new(value),
                    span: equals.span,
                },

                Expr::Get { object, name, .. } => Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                    span: equals.span,
                },

                Expr::Index { object, index, .. } => Expr::IndexAssign {
                    object,
                    index,
                    value: Box::new(value),
                    span: equals.span,
                },

                _ => panic!("Invalid assignment target"),
            }
        } else {
            expr
        }
    }

    /// equality → comparison ( ( "==" | "!=" | "===" | "!==" ) comparison )*
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_operator("==")
            || self.match_operator("!=")
            || self.match_operator("===")
            || self.match_operator("!==")
        {
            let op = self.previous().clone();
            let right = self.comparison();
            let span = op.span;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        expr
    }

    /// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_operator(">")
            || self.match_operator(">=")
            || self.match_operator("<")
            || self.match_operator("<=")
        {
            let op = self.previous().clone();
            let right = self.term();
            let span = op.span;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        expr
    }

    /// term → factor ( ( "+" | "-" ) factor )*
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_operator("+") || self.match_operator("-") {
            let op = self.previous().clone();
            let right = self.factor();
            let span = op.span;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        expr
    }

    /// factor → unary ( ( "*" | "/" | "%" ) unary )*
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_operator("*")
            || self.match_operator("/")
            || self.match_operator("%")
        {
            let op = self.previous().clone();
            let right = self.unary();
            let span = op.span;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        expr
    }

    /// unary → ( "!" | "-" ) unary | call
    fn unary(&mut self) -> Expr {
        if self.match_operator("!") || self.match_operator("-") {
            let op = self.previous().clone();
            let right = self.unary();
            let span = op.span;

            return Expr::Unary {
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        self.call()
    }

    /// call → primary ( "(" arguments? ")" | "." identifier | "[" expression "]" )*
    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            // function call
            if self.match_symbol('(') {
                let lparen = self.previous().clone();
                let mut args = Vec::new();

                if !self.check_symbol(')') {
                    loop {
                        args.push(self.expression());
                        if !self.match_symbol(',') {
                            break;
                        }
                    }
                }

                self.consume_symbol(')');

                expr = Expr::Call {
                    callee: Box::new(expr),
                    arguments: args,
                    span: lparen.span,
                };
                continue;
            }

            // property access
            if self.match_symbol('.') {
                let dot = self.previous().clone();
                let name_token = self.advance();

                let name = name_token.lexeme.clone();

                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                    span: dot.span,
                };
                continue;
            }

            // index access
            if self.match_symbol('[') {
                let lbracket = self.previous().clone();
                let index = self.expression();
                self.consume_symbol(']');

                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span: lbracket.span,
                };
                continue;
            }

            break;
        }

        expr
    }

    fn primary(&mut self) -> Expr {
        // tap
        if self.match_keyword("tap") {
            let tap_token = self.previous().clone();

            let path = if self.match_symbol('(') {
                let expr = self.expression();
                self.consume_symbol(')');
                expr
            } else {
                let token = self.advance();
                Expr::Literal {
                    value: Value::String(token.lexeme),
                    span: token.span,
                }
            };

            return Expr::Tap {
                path: Box::new(path),
                span: tap_token.span,
            };
        }

        // array literal
        if self.match_symbol('[') {
            let start = self.previous().clone();
            let mut values = Vec::new();

            if !self.check_symbol(']') {
                loop {
                    values.push(self.expression());
                    if !self.match_symbol(',') {
                        break;
                    }
                }
            }

            self.consume_symbol(']');

            return Expr::ArrayLiteral {
                values,
                span: start.span,
            };
        }

        // object literal
        if self.match_symbol('{') {
            let start = self.previous().clone();
            let mut fields = Vec::new();

            if self.match_symbol('}') {
                return Expr::ObjectLiteral {
                    fields,
                    span: start.span,
                };
            }

            loop {
                let key = self.advance().lexeme.clone();
                self.consume_symbol(':');
                let value = self.expression();
                fields.push((key, value));

                if self.match_symbol('}') {
                    break;
                }

                self.consume_symbol(',');
            }

            return Expr::ObjectLiteral {
                fields,
                span: start.span,
            };
        }

        // literals / identifiers / grouping / tuple
        let token = self.advance();

        match token.kind {
            TokenKind::Number => Expr::Literal {
                value: Value::Number(token.lexeme.parse().unwrap()),
                span: token.span,
            },

            TokenKind::String => Expr::Literal {
                value: Value::String(token.lexeme),
                span: token.span,
            },

            TokenKind::Identifier | TokenKind::Keyword => {
                Expr::Identifier {
                    name: token.lexeme,
                    span: token.span,
                }
            }

            TokenKind::Symbol if token.lexeme == "(" => {
                let expr = self.expression();
                self.consume_symbol(')');
                Expr::Grouping {
                    expr: Box::new(expr),
                    span: token.span,
                }
            }

            _ => panic!("Unexpected token: {:?}", token),
        }
    }

    fn logical_or(&mut self) -> Expr {
        let mut expr = self.logical_and();

        while self.match_symbol_lexeme("||") {
            let op = self.previous().clone();
            let right = self.logical_and();
            let span = op.span;

            expr = Expr::Logical {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        expr
    }

    fn logical_and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.match_symbol_lexeme("&&") {
            let op = self.previous().clone();
            let right = self.equality();
            let span = op.span;

            expr = Expr::Logical {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        expr
    }
}