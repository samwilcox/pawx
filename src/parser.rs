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

use crate::ast::{AccessLevel, ClassMember, Expr, InstinctMember, InstinctMemberKind, Param, Stmt};
use crate::lexer::{Token, TokenKind};
use crate::value::Value;

pub fn parse(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut parser = Parser { tokens, current: 0 };
    parser.parse()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            stmts.push(self.statement());
        }

        stmts
    }

    fn statement(&mut self) -> Stmt {
        // Function declarations:
        // zoom? purr? name -> (args) -> [:type ->] { body }
        if self.check_keyword("zoom")
            || self.check_keyword("purr")
            || self.is_function_start()
        {
            return self.function_declaration();
        }

        if self.match_keyword("den") {
            return self.private_var();
        }

        if self.match_keyword("lair") {
            return self.protected_var();
        }

        if self.match_keyword("snuggle") {
            let name = self.consume_identifier();
            self.consume_symbol('=');
            let value = self.expression();
            self.match_symbol(';');
            return Stmt::PublicVar { name, value }; // ✅ Temporarily treat as public const
        }

        if self.match_keyword("pride") {
            return self.pride_dispatch();
        }

        if self.match_keyword("try") {
            return self.try_statement();
        }

        if self.match_keyword("exports") {
            let mut is_default = false;
            if self.match_keyword("default") {
                is_default = true;
            }

            if self.match_keyword("clowder") {
                return self.clowder_declaration(true, is_default);
            }

            if self.match_keyword("instinct") {
                return self.instinct_declaration(true, is_default);
            }

            panic!("Expected 'clowder' or 'instinct' after 'exports'");
        }

        // plain clowder / instinct (non-exported)
        if self.match_keyword("clowder") {
            return self.clowder_declaration(false, false);
        }

        if self.match_keyword("instinct") {
            return self.instinct_declaration(false, false);
        }

        if self.match_keyword("if") {
            return self.if_statement();
        }

        if self.match_keyword("while") {
            return self.while_statement();
        }

        if self.match_keyword("nap") {
            let expr = self.expression();
            self.match_symbol(';');
            return Stmt::Nap(expr);
        }

        if self.match_keyword("throw") {
            let expr = self.expression();
            self.match_symbol(';');
            return Stmt::Throw(expr);
        }

        if self.match_keyword("return") {
            return self.return_statement();
        }

        self.expression_statement()
    }

    fn clowder_declaration(&mut self, is_exported: bool, is_default: bool) -> Stmt {
        let name = if self.match_keyword("new") {
            "new".to_string()
        } else {
            self.consume_identifier()
        };

        // Optional: inherits Base
        let mut base = None;
        if self.match_keyword("inherits") {
            base = Some(self.consume_identifier());
        }

        // Optional: practices Interface[, Interface]*
        let mut interfaces = Vec::new();
        if self.match_keyword("practices") {
            loop {
                interfaces.push(self.consume_identifier());
                if !self.match_symbol(',') {
                    break;
                }
            }
        }

        self.consume_symbol('{');
        let mut members = Vec::new();

        while !self.check_symbol('}') && !self.is_at_end() {
            // static?
            let is_static = self.match_keyword("static");

            // access keyword? (for fields/methods)
            let access = if self.match_keyword("pride") {
                Some(AccessLevel::Public)
            } else if self.match_keyword("den") {
                Some(AccessLevel::Private)
            } else if self.match_keyword("lair") {
                Some(AccessLevel::Protected)
            } else {
                None
            };

            // getter
            if self.match_keyword("get") {
                if is_static {
                    panic!("static getters not supported yet");
                }
                if access.is_some() {
                    panic!("getters cannot use access modifiers (pride/den/lair)");
                }

                let prop_name = self.consume_identifier();
                self.consume_symbol('-');
                self.consume_symbol('>');

                let mut return_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    return_type = Some(t);
                    self.consume_symbol('-');
                    self.consume_symbol('>');
                }

                self.consume_symbol('{');
                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }
                self.consume_symbol('}');

                members.push(ClassMember::Getter {
                    name: prop_name,
                    return_type,
                    body,
                });

                continue;
            }

            // setter
            if self.match_keyword("set") {
                if is_static {
                    panic!("static setters not supported yet");
                }
                if access.is_some() {
                    panic!("setters cannot use access modifiers (pride/den/lair)");
                }

                let prop_name = self.consume_identifier();

                self.consume_symbol('-');
                self.consume_symbol('>');

                self.consume_symbol('(');
                let param_name = self.consume_identifier();
                let mut param_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    param_type = Some(t);
                }
                self.consume_symbol(')');

                self.consume_symbol('-');
                self.consume_symbol('>');

                self.consume_symbol('{');
                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }
                self.consume_symbol('}');

                members.push(ClassMember::Setter {
                    name: prop_name,
                    param_name,
                    param_type,
                    body,
                });

                continue;
            }

            // If we have access, this is either a field or a method
            if let Some(access_level) = access {
                // Could be: [static] pride name :type? = expr? ;
                // or:      [static] pride purr name -> (...) -> [:type ->]? { ... }

                // Look ahead for "purr" to detect method
                if self.match_keyword("purr") {
                    // ✅ allow constructor: purr new -> (...)
                    let name = if self.match_keyword("new") {
                        "new".to_string()
                    } else {
                        self.consume_identifier()
                    };

                    self.consume_symbol('-');
                    self.consume_symbol('>');

                    // params
                    self.consume_symbol('(');
                    let mut params = Vec::new();
                    if !self.check_symbol(')') {
                        loop {
                            let param_name = self.consume_identifier();
                            let mut type_annotation = None;
                            if self.match_symbol(':') {
                                let t = self.consume_identifier();
                                type_annotation = Some(t);
                            }
                            params.push(Param {
                                name: param_name,
                                default: None,
                                type_annotation,
                            });
                            if !self.match_symbol(',') {
                                break;
                            }
                        }
                    }
                    self.consume_symbol(')');

                    self.consume_symbol('-');
                    self.consume_symbol('>');

                    // Optional return type: :type ->
                    let mut return_type = None;
                    if self.match_symbol(':') {
                        let t = self.consume_identifier();
                        return_type = Some(t);
                        self.consume_symbol('-');
                        self.consume_symbol('>');
                    }

                    self.consume_symbol('{');
                    let mut body = Vec::new();
                    while !self.check_symbol('}') {
                        body.push(self.statement());
                    }
                    self.consume_symbol('}');

                    members.push(ClassMember::Method {
                        name,
                        access: access_level,
                        is_static,
                        params,
                        return_type,
                        body,
                    });
                } else {
                    // Field
                    let field_name = self.consume_identifier();
                    let mut type_annotation = None;
                    if self.match_symbol(':') {
                        let t = self.consume_identifier();
                        type_annotation = Some(t);
                    }

                    let mut value = None;
                    if self.match_symbol('=') {
                        value = Some(self.expression());
                    }
                    self.match_symbol(';');

                    members.push(ClassMember::Field {
                        name: field_name,
                        access: access_level,
                        is_static,
                        type_annotation,
                        value,
                    });
                }

                continue;
            }

            // If we reach here with no access / get / set, this might be
            // a plain method: [static] purr name -> ...
            if self.match_keyword("purr") {
                // ✅ also allow constructor in the "no access" form
                let name = if self.match_keyword("new") {
                    "new".to_string()
                } else {
                    self.consume_identifier()
                };

                self.consume_symbol('-');
                self.consume_symbol('>');

                self.consume_symbol('(');
                let mut params = Vec::new();
                if !self.check_symbol(')') {
                    loop {
                        let param_name = self.consume_identifier();
                        let mut type_annotation = None;
                        if self.match_symbol(':') {
                            let t = self.consume_identifier();
                            type_annotation = Some(t);
                        }
                        params.push(Param {
                            name: param_name,
                            default: None,
                            type_annotation,
                        });
                        if !self.match_symbol(',') {
                            break;
                        }
                    }
                }
                self.consume_symbol(')');

                self.consume_symbol('-');
                self.consume_symbol('>');

                let mut return_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    return_type = Some(t);
                    self.consume_symbol('-');
                    self.consume_symbol('>');
                }

                self.consume_symbol('{');
                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }
                self.consume_symbol('}');

                members.push(ClassMember::Method {
                    name,
                    access: AccessLevel::Public, // default
                    is_static,
                    params,
                    return_type,
                    body,
                });

                continue;
            }

            panic!("Unexpected token in clowder body");
        }

        self.consume_symbol('}');

        Stmt::Clowder {
            name,
            base,
            interfaces,
            members,
            is_exported,
            is_default,
        }
    }

    fn pride_dispatch(&mut self) -> Stmt {
        let name = self.consume_identifier();

        if self.match_symbol('=') {
            let value = self.expression();
            self.match_symbol(';');
            return Stmt::PublicVar { name, value };
        }

        if self.match_symbol('{') {
            let mut body = Vec::new();
            while !self.check_symbol('}') {
                body.push(self.statement());
            }
            self.consume_symbol('}');
            return Stmt::Pride { name, body };
        }

        panic!("Expected '=' or '{{' after pride {}", name);
    }

    fn private_var(&mut self) -> Stmt {
        let name = self.consume_identifier();
        self.consume_symbol('=');
        let value = self.expression();
        self.match_symbol(';');
        Stmt::PrivateVar { name, value }
    }

    fn protected_var(&mut self) -> Stmt {
        let name = self.consume_identifier();
        self.consume_symbol('=');
        let value = self.expression();
        self.match_symbol(';');
        Stmt::ProtectedVar { name, value }
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume_symbol('(');
        let condition = self.expression();
        self.consume_symbol(')');

        self.consume_symbol('{');
        let mut then_branch = Vec::new();
        while !self.check_symbol('}') {
            then_branch.push(self.statement());
        }
        self.consume_symbol('}');

        let mut else_branch = None;
        if self.match_keyword("else") {
            self.consume_symbol('{');
            let mut else_body = Vec::new();
            while !self.check_symbol('}') {
                else_body.push(self.statement());
            }
            self.consume_symbol('}');
            else_branch = Some(else_body);
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume_symbol('(');
        let condition = self.expression();
        self.consume_symbol(')');

        self.consume_symbol('{');
        let mut body = Vec::new();
        while !self.check_symbol('}') {
            body.push(self.statement());
        }
        self.consume_symbol('}');

        Stmt::While { condition, body }
    }

    fn return_statement(&mut self) -> Stmt {
        if self.match_symbol(';') {
            return Stmt::Return(None);
        }

        let expr = self.expression();
        self.match_symbol(';');
        Stmt::Return(Some(expr))
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.match_symbol(';');
        Stmt::Expression(expr)
    }

        fn instinct_declaration(&mut self, is_exported: bool, is_default: bool) -> Stmt {
        let name = self.consume_identifier();

        self.consume_symbol('{');
        let mut members = Vec::new();

        while !self.check_symbol('}') && !self.is_at_end() {
            // get name -> :type ;
            if self.match_keyword("get") {
                let prop_name = self.consume_identifier();
                self.consume_symbol('-');
                self.consume_symbol('>');

                let mut return_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    return_type = Some(t);
                }
                self.match_symbol(';');

                members.push(InstinctMember {
                    name: prop_name,
                    kind: InstinctMemberKind::Getter { return_type },
                });

                continue;
            }

            // set name -> (param[:type]) ;
            if self.match_keyword("set") {
                let prop_name = self.consume_identifier();
                self.consume_symbol('-');
                self.consume_symbol('>');

                self.consume_symbol('(');
                let param_name = self.consume_identifier();
                let mut param_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    param_type = Some(t);
                }
                self.consume_symbol(')');
                self.match_symbol(';');

                members.push(InstinctMember {
                    name: prop_name,
                    kind: InstinctMemberKind::Setter {
                        param_name,
                        param_type,
                    },
                });

                continue;
            }

            // method signature: purr name -> (...) -> [:type]? ;
            if self.match_keyword("purr") {
                let method_name = self.consume_identifier();
                self.consume_symbol('-');
                self.consume_symbol('>');

                self.consume_symbol('(');
                let mut params = Vec::new();
                if !self.check_symbol(')') {
                    loop {
                        let param_name = self.consume_identifier();
                        let mut type_annotation = None;
                        if self.match_symbol(':') {
                            let t = self.consume_identifier();
                            type_annotation = Some(t);
                        }
                        params.push(Param {
                            name: param_name,
                            default: None,
                            type_annotation,
                        });
                        if !self.match_symbol(',') {
                            break;
                        }
                    }
                }
                self.consume_symbol(')');

                self.consume_symbol('-');
                self.consume_symbol('>');

                let mut return_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    return_type = Some(t);
                }
                self.match_symbol(';');

                members.push(InstinctMember {
                    name: method_name,
                    kind: InstinctMemberKind::Method { params, return_type },
                });

                continue;
            }

            panic!("Unexpected token in instinct body");
        }

        self.consume_symbol('}');

        Stmt::Instinct {
            name,
            members,
            is_exported,
            is_default,
        }
    }

    // ---------- Functions ----------

    fn function_declaration(&mut self) -> Stmt {
        let mut is_async = false;

        if self.match_keyword("zoom") {
            is_async = true;
        }

        // Optional "purr"
        let _ = self.match_keyword("purr");

        let name = self.consume_identifier();

        // name -> ( ... ) -> [:type ->] { ... }

        self.consume_symbol('-');
        self.consume_symbol('>');

        self.consume_symbol('(');
        let mut params = Vec::new();

        if !self.check_symbol(')') {
            loop {
                let param_name = self.consume_identifier();
                let mut default = None;

                if self.match_symbol('=') {
                    let def_expr = self.expression();
                    default = Some(def_expr);
                }

                params.push(Param { name: param_name, default, type_annotation: None });

                if !self.match_symbol(',') {
                    break;
                }
            }
        }

        self.consume_symbol(')');

        self.consume_symbol('-');
        self.consume_symbol('>');

        // Optional return type: :string ->
        let mut return_type = None;
        if self.match_symbol(':') {
            let t = self.advance();
            match t.kind {
                TokenKind::Identifier | TokenKind::Keyword => {
                    return_type = Some(t.lexeme);
                }
                _ => panic!("Expected type name after ':'"),
            }

            self.consume_symbol('-');
            self.consume_symbol('>');
        }

        self.consume_symbol('{');
        let mut body = Vec::new();
        while !self.check_symbol('}') {
            body.push(self.statement());
        }
        self.consume_symbol('}');

        Stmt::Function {
            name,
            params,
            body,
            return_type,
            is_async,
        }
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.equality();

        if self.match_symbol('=') {
            let value = self.assignment();

            match expr {
                // ✅ Variable assignment: x = 5
                Expr::Identifier(name) => {
                    return Expr::Assign {
                        name,
                        value: Box::new(value),
                    };
                }

                // ✅ PROPERTY assignment: obj.prop = 5
                Expr::Get { object, name } => {
                    return Expr::Set {
                        object,
                        name,
                        value: Box::new(value),
                    };
                }

                _ => panic!("Invalid assignment target"),
            }
        }

        expr
    }

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

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_operator(">") || self.match_operator(">=")
            || self.match_operator("<") || self.match_operator("<=")
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

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_operator("*") || self.match_operator("/") {
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

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        // ✅ Lambda: (params) -> { body }
        if let Expr::Grouping(inner) = expr.clone() {
            if self.match_symbol('-') {
                self.consume_symbol('>');

                // Extract params
                let mut params = Vec::new();
                match *inner {
                    Expr::Identifier(name) => params.push(name),
                    _ => panic!("Invalid lambda parameter"),
                }

                self.consume_symbol('{');
                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }
                self.consume_symbol('}');

                return Expr::Lambda { params, body };
            }
        }

        loop {
            if self.match_symbol('(') {
                expr = self.finish_call(expr);
            } else if self.match_symbol('.') {
                let name = self.consume_identifier();
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        expr
    }

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
        // ✅ NEW EXPRESSION SUPPORT
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

        let token = self.advance();

        match token.kind {
            TokenKind::Number => Expr::Literal(Value::Number(token.lexeme.parse().unwrap())),
            TokenKind::String => Expr::Literal(Value::String(token.lexeme)),

            // ✅ FIX: allow BOTH identifiers AND keywords like "this"
            TokenKind::Identifier | TokenKind::Keyword => {
                Expr::Identifier(token.lexeme)
            }

            TokenKind::Symbol if token.lexeme == "(" => {
                let expr = self.expression();
                self.consume_symbol(')');
                Expr::Grouping(Box::new(expr))
            }

            _ => panic!("Unexpected token: {:?}", token),
        }
    }

    // ---------- helpers ----------

    fn match_keyword(&mut self, kw: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.tokens[self.current].kind == TokenKind::Keyword
            && self.tokens[self.current].lexeme == kw
        {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_keyword(&self, kw: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current].kind == TokenKind::Keyword
            && self.tokens[self.current].lexeme == kw
    }

    fn is_function_start(&self) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.tokens[self.current].kind == TokenKind::Identifier {
            if self.current + 2 < self.tokens.len() {
                let t1 = &self.tokens[self.current + 1];
                let t2 = &self.tokens[self.current + 2];
                return t1.kind == TokenKind::Symbol
                    && t1.lexeme == "-"
                    && t2.kind == TokenKind::Symbol
                    && t2.lexeme == ">";
            }
        }

        false
    }

    fn match_operator(&mut self, op: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.tokens[self.current].kind == TokenKind::Symbol
            && self.tokens[self.current].lexeme == op
        {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_symbol(&mut self, ch: char) -> bool {
        if self.check_symbol(ch) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_symbol(&self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current].kind == TokenKind::Symbol
            && self.tokens[self.current].lexeme == ch.to_string()
    }

    fn consume_symbol(&mut self, ch: char) {
        if self.check_symbol(ch) {
            self.advance();
        } else {
            panic!("Expected '{}'", ch);
        }
    }

    fn consume_identifier(&mut self) -> String {
        let token = self.advance();
        if token.kind != TokenKind::Identifier {
            panic!("Expected identifier");
        }
        token.lexeme
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.current].clone();
        self.current += 1;
        t
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.current].kind == TokenKind::Eof
    }

    fn try_statement(&mut self) -> Stmt {
        // try { ... }
        self.consume_symbol('{');

        let mut try_block = Vec::new();
        while !self.check_symbol('}') {
            try_block.push(self.statement());
        }
        self.consume_symbol('}');

        // optional catch (e) { ... }
        let mut catch_param = None;
        let mut catch_block = None;

        if self.match_keyword("catch") {
            self.consume_symbol('(');
            let name = if self.match_keyword("new") {
                "new".to_string()
            } else {
                self.consume_identifier()
            };
            self.consume_symbol(')');
            catch_param = Some(name);

            self.consume_symbol('{');
            let mut body = Vec::new();
            while !self.check_symbol('}') {
                body.push(self.statement());
            }
            self.consume_symbol('}');
            catch_block = Some(body);
        }

        // optional finally { ... }
        let mut finally_block = None;
        if self.match_keyword("finally") {
            self.consume_symbol('{');
            let mut body = Vec::new();
            while !self.check_symbol('}') {
                body.push(self.statement());
            }
            self.consume_symbol('}');
            finally_block = Some(body);
        }

        Stmt::Try {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        }
    }
}