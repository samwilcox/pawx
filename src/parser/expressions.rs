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
use crate::value::Value;

impl Parser {
    /// expression → assignment
    pub fn expression(&mut self) -> Expr {
        self.logical_or()
    }

    /// assignment → equality ( "=" assignment )?
    fn assignment(&mut self) -> Expr {
        let expr = self.equality();

        if self.match_symbol('=') {
            let value = self.assignment();

            match expr {
                Expr::Identifier(name) => {
                    return Expr::Assign { name, value: Box::new(value) };
                }

                Expr::Get { object, name } => {
                    return Expr::Set { object, name, value: Box::new(value) };
                }

                Expr::Index { object, index } => {
                    return Expr::IndexAssign {
                        object,
                        index,
                        value: Box::new(value),
                    };
                }

                _ => panic!("Invalid assignment target"),
            }
        }

        expr
    }

    /// equality → comparison ( ( "==" | "!=" | "===" | "!==" ) comparison )*
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_operator("==")
            || self.match_operator("!=")
            || self.match_operator("===")
            || self.match_operator("!==")
        {
            let op = self.previous().lexeme.clone();
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
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
            let op = self.previous().lexeme.clone();
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        expr
    }

    /// term → factor ( ( "+" | "-" ) factor )*
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_operator("+") || self.match_operator("-") {
            let op = self.previous().lexeme.clone();
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
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
            let op = self.previous().lexeme.clone();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        expr
    }

    /// unary → ( "!" | "-" ) unary | call
    fn unary(&mut self) -> Expr {
        if self.match_operator("!") || self.match_operator("-") {
            let op = self.previous().lexeme.clone();
            let right = self.unary();
            return Expr::Unary {
                operator: op,
                right: Box::new(right),
            };
        }

        self.call()
    }

    /// call → primary ( "(" arguments? ")" | "." identifier | "[" expression "]" )*
    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            // function call: f(...)
            if self.match_symbol('(') {
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
                };

                continue;
            }

            // property access: obj.prop
           if self.match_symbol('.') {
                let token = self.advance();

                if token.kind != TokenKind::Identifier && token.kind != TokenKind::Keyword {
                    panic!("Expected property name after '.', got {:?}", token);
                }

                let name = token.lexeme;

                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };

                continue;
            }

            // index access: arr[expr]
            if self.match_symbol('[') {
                let index = self.expression();
                self.consume_symbol(']');
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
                continue;
            }

            break;
        }

        expr
    }

    /// Helper to finish a call if you ever parse `callee` and then see `(`.
    fn finish_call(&mut self, callee: Expr) -> Expr {
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

        Expr::Call {
            callee: Box::new(callee),
            arguments: args,
        }
    }

    fn primary(&mut self) -> Expr {
        // tap expression: tap('./file') OR tap myModule
        if self.match_keyword("tap") {
            let path_expr = if self.check_symbol('(') {
                self.consume_symbol('(');
                let expr = self.expression();
                self.consume_symbol(')');
                expr
            } else {
                let name = self.consume_identifier();
                Expr::Literal(Value::String(name))
            };

            return Expr::Tap {
                path: Box::new(path_expr),
            };
        }

        // Array literal: [a, b, c]
        if self.match_symbol('[') {
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
            return Expr::ArrayLiteral { values };
        }

        // Object literal: { a: 1, b: 2 }
        if self.match_symbol('{') {
            let mut fields = Vec::new();

            // empty object {}
            if self.match_symbol('}') {
                return Expr::ObjectLiteral { fields };
            }

            loop {
                // key = identifier or string
                let name = if !self.is_at_end()
                    && (self.tokens[self.current].kind == TokenKind::Identifier
                        || self.tokens[self.current].kind == TokenKind::String)
                {
                    self.advance().lexeme.clone()
                } else {
                    panic!("Expected property name in object literal");
                };

                self.consume_symbol(':');

                let value = self.expression();
                fields.push((name, value));

                if self.match_symbol('}') {
                    break;
                }

                self.consume_symbol(',');
            }

            return Expr::ObjectLiteral { fields };
        }

        // new Class(...)
        if self.match_keyword("new") {
            let class_name = self.consume_identifier();
            self.consume_symbol('(');

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
            return Expr::New {
                class_name,
                arguments: args,
            };
        }

        // Parenthesized lambda: (a, b) -> { ... }
        if self.check_symbol('(') {
            let saved = self.current;

            self.advance(); // consume '('
            let mut params = Vec::new();
            let mut valid = true;

            if !self.check_symbol(')') {
                loop {
                    if self.is_at_end()
                        || self.tokens[self.current].kind != TokenKind::Identifier
                    {
                        valid = false;
                        break;
                    }

                    params.push(self.advance().lexeme.clone());

                    if !self.match_symbol(',') {
                        break;
                    }
                }
            }

            if !self.match_symbol(')') {
                valid = false;
            }

            // Only lambda if "->" follows
            if valid
                && !self.is_at_end()
                && self.tokens[self.current].lexeme == "->"
            {
                self.advance(); // consume '->'
                self.consume_symbol('{');

                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }

                self.consume_symbol('}');
                return Expr::Lambda { params, body };
            }

            // Not a lambda – rollback
            self.current = saved;
        }

        // Bare lambda: x -> { ... }
        if !self.is_at_end() && self.tokens[self.current].kind == TokenKind::Identifier {
            let saved = self.current;

            let name = self.advance().lexeme.clone();

            // THIS is the important fix
            if self.match_arrow() {
                self.consume_symbol('{');

                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }

                self.consume_symbol('}');

                return Expr::Lambda {
                    params: vec![name],
                    body,
                };
            }

            // rollback if not actually a lambda
            self.current = saved;
        }

        // Normal literals / grouping / tuple
        let token = self.advance();

        match token.kind {
            TokenKind::Number => Expr::Literal(Value::Number(token.lexeme.parse().unwrap())),
            TokenKind::String => Expr::Literal(Value::String(token.lexeme)),

            TokenKind::Keyword if token.lexeme == "true" => {
                Expr::Literal(Value::Bool(true))
            }
            TokenKind::Keyword if token.lexeme == "false" => {
                Expr::Literal(Value::Bool(false))
            }
            TokenKind::Keyword if token.lexeme == "null" => {
                Expr::Literal(Value::Null)
            }

            TokenKind::Identifier | TokenKind::Keyword => {
                let name = token.lexeme;

                // i++
                if self.match_symbol_lexeme("++") {
                    return Expr::PostIncrement { name };
                }

                // i--
                if self.match_symbol_lexeme("--") {
                    return Expr::PostDecrement { name };
                }

                Expr::Identifier(name)
            }

            TokenKind::Symbol if token.lexeme == "(" => {
                let mut values = Vec::new();

                // Empty tuple ()
                if self.check_symbol(')') {
                    self.consume_symbol(')');
                    return Expr::Tuple(vec![]);
                }

                values.push(self.expression());

                // Tuple via commas
                if self.match_symbol(',') {
                    loop {
                        values.push(self.expression());
                        if !self.match_symbol(',') {
                            break;
                        }
                    }

                    self.consume_symbol(')');
                    return Expr::Tuple(values);
                }

                // Normal grouping
                self.consume_symbol(')');
                Expr::Grouping(Box::new(values.remove(0)))
            }

            _ => panic!("Unexpected token: {:?}", token),
        }
    }

    /// Parses a logical OR (`||`) expression.
    ///
    /// This method implements the **lowest-precedence boolean operator layer**
    /// in the PAWX expression grammar. It repeatedly folds left-associative `||`
    /// operators into a single `Expr::Logical` node.
    ///
    /// # Grammar
    /// ```text
    /// logical_or → logical_and ( "||" logical_and )*
    /// ```
    ///
    /// # Behavior
    /// - Begins by parsing the left-hand side using `logical_and()`.
    /// - Repeatedly consumes `||` operators if present.
    /// - Each additional `||` produces a new `Expr::Logical` node.
    /// - Enforces **left associativity**:
    ///   ```pawx
    ///   a || b || c  →  (a || b) || c
    ///   ```
    ///
    /// # Precedence
    /// - This is the **lowest-precedence boolean operator**.
    /// - It binds more loosely than:
    ///   - `&&` (AND)
    ///   - Equality (`==`, `===`, `!=`, `!==`)
    ///
    /// # Used By
    /// - `expression()` (as the primary entry point)
    /// - `if`, `while`, `return`, and all boolean conditions
    ///
    /// # Returns
    /// - `Ok(Expr)` if parsing succeeds
    /// - `Err(ParseError)` on invalid syntax
    fn logical_or(&mut self) -> Expr {
        let mut expr = self.logical_and();

        while self.match_symbol_lexeme("||") {
            let op = self.previous().clone();
            let right = self.logical_and();
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        expr
    }

    /// Parses a logical AND (`&&`) expression.
    ///
    /// This method implements the **higher-precedence boolean operator layer**
    /// in the PAWX expression grammar. It groups chained `&&` operations tightly
    /// before they are combined by `logical_or()`.
    ///
    /// # Grammar
    /// ```text
    /// logical_and → equality ( "&&" equality )*
    /// ```
    ///
    /// # Behavior
    /// - Begins by parsing the left operand via `equality()`.
    /// - Repeatedly folds each `&&` operation into a left-associative tree.
    /// - Produces `Expr::Logical` AST nodes for each operator.
    ///
    /// # Precedence
    /// - Binds more tightly than:
    ///   - `||` (logical OR)
    /// - Binds less tightly than:
    ///   - Equality (`===`, `!==`, `==`, `!=`)
    ///   - Comparison (`<`, `>`, `<=`, `>=`)
    ///
    /// # Evaluation Semantics
    /// This layer enables **short-circuit boolean evaluation** at runtime:
    /// - If the left operand is false, the right is **never evaluated**.
    ///
    /// # Returns
    /// - `Ok(Expr)` if valid
    /// - `Err(ParseError)` if malformed
    fn logical_and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.match_symbol_lexeme("&&") {
            let op = self.previous().clone();
            let right = self.equality();
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        expr
    }
}