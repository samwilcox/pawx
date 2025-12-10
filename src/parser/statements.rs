/*
 * ==========================================================================
 * PAWX - Code with Claws!
 * ==========================================================================
 * 
 * Statement-Level Parsing Logic
 * 
 * This file contains all grammar rules responsible for parsing **PAWX
 * statements** into their corresponding Abstract Syntax Tree (AST) forms.
 * 
 * It handles:
 * - Function declarations (`purr`, `zoom purr`)
 * - Variables (`den`, `lair`, `snuggle`, `pride`)
 * - Control flow (`if`, `while`, `try`, `catch`, `finally`)
 * - Classes (`clowder`)
 * - Interfaces (`instinct`)
 * - Module exports
 * - Expression-backed statements
 * 
 * This module forms the **top layer of the recursive-descent grammar** and
 * drives overall program structure.
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

use crate::parser::parser::Parser;
use crate::ast::{Stmt, Param, ClassMember, AccessLevel, InstinctMember, InstinctMemberKind};

impl  Parser {
    /// Parses a single top-level PAWX statement.
    ///
    /// This is the **main dispatcher** for all statement grammar forms.
    /// It inspects the leading token and routes to the appropriate parser.
    ///
    /// # Responsibilities
    /// - Function declarations
    /// - Variable declarations
    /// - Control flow statements
    /// - Class & interface declarations
    /// - Export statements
    /// - Expression statements as a fallback
    pub fn statement(&mut self) -> Stmt {
        // ------------------------------------------------------------
        // ASYNC FUNCTION:
        // zoom purr name -> (...) -> [:type ->] { body }
        // ------------------------------------------------------------
        if self.check_keyword("zoom")
            && self.tokens.get(self.current + 1).is_some_and(|t| t.lexeme == "purr")
            && self.tokens.get(self.current + 3).is_some_and(|t| t.lexeme == "->")
        {
            self.advance(); // zoom
            self.advance(); // purr
            return self.function_declaration_with_async(true);
        }

        // ------------------------------------------------------------
        // NORMAL FUNCTION:
        // purr name -> (...) -> [:type ->] { body }
        // ------------------------------------------------------------
        if self.check_keyword("purr")
            && self.tokens.get(self.current + 2).is_some_and(|t| t.lexeme == "->")
        {
            self.advance(); // purr
            return self.function_declaration_with_async(false);
        }

        // ------------------------------------------------------------
        // VARIABLE DECLARATIONS
        // ------------------------------------------------------------
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
            return Stmt::PublicVar { name, value };
        }

        // ------------------------------------------------------------
        // PRIDE BLOCK OR VARIABLE
        // ------------------------------------------------------------
        if self.match_keyword("pride") {
            return self.pride_dispatch();
        }

        // ------------------------------------------------------------
        // TRY / CATCH / FINALLY
        // ------------------------------------------------------------
        if self.match_keyword("try") {
            return self.try_statement();
        }

        // ------------------------------------------------------------
        // EXPORT DECLARATIONS
        // ------------------------------------------------------------
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

        // ------------------------------------------------------------
        // NON-EXPORTED clowder / instinct
        // ------------------------------------------------------------
        if self.match_keyword("clowder") {
            return self.clowder_declaration(false, false);
        }

        if self.match_keyword("instinct") {
            return self.instinct_declaration(false, false);
        }

        // ------------------------------------------------------------
        // CONTROL FLOW
        // ------------------------------------------------------------
        if self.match_keyword("if") {
            return self.if_statement();
        }

        if self.match_keyword("while") {
            return self.while_statement();
        }

        // ------------------------------------------------------------
        // FLOW CONTROL
        // ------------------------------------------------------------
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

        // ------------------------------------------------------------
        // FALLBACK: EXPRESSION STATEMENT
        // ------------------------------------------------------------
        self.expression_statement()
    }

    /// Parses a function declaration with optional async support.
    ///
    /// This handles both:
    /// - `purr name -> (...) { ... }`
    /// - `zoom purr name -> (...) { ... }`
    pub fn function_declaration_with_async(&mut self, is_async: bool) -> Stmt {
        // Optional redundant purr after zoom
        let name = self.consume_identifier();

        self.consume_arrow();     // name ->
        self.consume_symbol('(');

        let mut params = Vec::new();

        if !self.check_symbol(')') {
            loop {
                let param_name = self.consume_identifier();
                let mut default = None;

                if self.match_symbol('=') {
                    default = Some(self.expression());
                }

                params.push(Param {
                    name: param_name,
                    default,
                    type_annotation: None,
                });

                if !self.match_symbol(',') {
                    break;
                }
            }
        }

        self.consume_symbol(')');
        self.consume_arrow();

        let mut return_type = None;
        if self.match_symbol(':') {
            let t = self.advance();
            return_type = Some(t.lexeme);
            self.consume_arrow();
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

    /// Parses a full PAWX `clowder` declaration (class definition).
    ///
    /// A `clowder` represents an object blueprint similar to a class in
    /// traditional OOP languages (Java, C++, TS, etc).
    ///
    /// This function supports:
    /// - Constructors (`purr new -> (...)`)
    /// - Static members (`static pride x`)
    /// - Access control (`pride`, `den`, `lair`)
    /// - Fields
    /// - Methods
    /// - Getters & setters
    /// - Inheritance (`inherits Base`)
    /// - Interfaces (`practices A, B`)
    /// - Module exports (`exports`, `default`)
    ///
    /// # Grammar (Simplified)
    /// ```pawx
    /// clowder Name inherits Base practices A, B {
    ///     pride x: Number = 10;
    ///     den y: String;
    ///     static pride purr foo -> (a) -> { }
    ///     get value -> :Number { }
    ///     set value -> (v:Number) -> { }
    ///     purr new -> (x) -> { }
    /// }
    /// ```
    ///
    /// # Parameters
    /// - `is_exported`: Set when preceded by `exports`
    /// - `is_default`: Set when preceded by `exports default`
    ///
    /// # Returns
    /// A fully constructed `Stmt::Clowder` AST node.
    ///
    /// # Panics
    /// - If invalid class syntax is detected
    /// - If getters/setters use illegal modifiers
    /// - If malformed inheritance or method blocks occur
    pub fn clowder_declaration(&mut self, is_exported: bool, is_default: bool) -> Stmt {
        // ---------------------------------------------
        // Class Name
        // ---------------------------------------------
        // Supports:
        //   clowder Cat { ... }
        //   clowder new { ... }   // constructor-only class
        let name = if self.match_keyword("new") {
            "new".to_string()
        } else {
            self.consume_identifier()
        };

        // ---------------------------------------------
        // Optional Inheritance
        // ---------------------------------------------
        // Example:
        //   clowder Dog inherits Animal { ... }
        let mut base = None;
        if self.match_keyword("inherits") {
            base = Some(self.consume_identifier());
        }

        // ---------------------------------------------
        // Optional Interfaces (Multiple Supported)
        // ---------------------------------------------
        // Example:
        //   clowder Cat practices Hunter, Pet { ... }
        let mut interfaces = Vec::new();
        if self.match_keyword("practices") {
            loop {
                interfaces.push(self.consume_identifier());
                if !self.match_symbol(',') {
                    break;
                }
            }
        }

        // ---------------------------------------------
        // Begin Class Body
        // ---------------------------------------------
        self.consume_symbol('{');
        let mut members = Vec::new();

        while !self.check_symbol('}') && !self.is_at_end() {
            // ---------------------------------------------
            // Optional Static Modifier
            // ---------------------------------------------
            let is_static = self.match_keyword("static");

            // ---------------------------------------------
            // Optional Access Modifier
            // ---------------------------------------------
            let access = if self.match_keyword("pride") {
                Some(AccessLevel::Public)
            } else if self.match_keyword("den") {
                Some(AccessLevel::Private)
            } else if self.match_keyword("lair") {
                Some(AccessLevel::Protected)
            } else {
                None
            };

            // ---------------------------------------------
            // Getter Declaration
            // ---------------------------------------------
            if self.match_keyword("get") {
                if is_static {
                    panic!("static getters not supported yet");
                }
                if access.is_some() {
                    panic!("getters cannot use access modifiers (pride/den/lair)");
                }

                let prop_name = self.consume_identifier();
                self.consume_arrow();

                let mut return_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    return_type = Some(t);
                    self.consume_arrow();
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

            // ---------------------------------------------
            // Setter Declaration
            // ---------------------------------------------
            if self.match_keyword("set") {
                if is_static {
                    panic!("static setters not supported yet");
                }
                if access.is_some() {
                    panic!("setters cannot use access modifiers (pride/den/lair)");
                }

                let prop_name = self.consume_identifier();
                self.consume_arrow();
                self.consume_symbol('(');

                let param_name = self.consume_identifier();
                let mut param_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    param_type = Some(t);
                }

                self.consume_symbol(')');
                self.consume_arrow();

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

            // ---------------------------------------------
            // Field or Method With Access Modifier
            // ---------------------------------------------
            if let Some(access_level) = access {
                if self.match_keyword("purr") {
                    // ✅ Method (or constructor)
                    let name = if self.match_keyword("new") {
                        "new".to_string()
                    } else {
                        self.consume_identifier()
                    };

                    self.consume_arrow();
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
                    self.consume_arrow();

                    let mut return_type = None;
                    if self.match_symbol(':') {
                        let t = self.consume_identifier();
                        return_type = Some(t);
                        self.consume_arrow();
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

            // ---------------------------------------------
            // Public Method (No Access Modifier)
            // ---------------------------------------------
            if self.match_keyword("purr") {
                let name = if self.match_keyword("new") {
                    "new".to_string()
                } else {
                    self.consume_identifier()
                };

                self.consume_arrow();
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
                self.consume_arrow();

                let mut return_type = None;
                if self.match_symbol(':') {
                    let t = self.consume_identifier();
                    return_type = Some(t);
                    self.consume_arrow();
                }

                self.consume_symbol('{');
                let mut body = Vec::new();
                while !self.check_symbol('}') {
                    body.push(self.statement());
                }
                self.consume_symbol('}');

                members.push(ClassMember::Method {
                    name,
                    access: AccessLevel::Public,
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

    /// Parses a full PAWX `instinct` declaration (interface definition).
    ///
    /// Example:
    /// ```pawx
    /// instinct Animal {
    ///     purr speak -> () -> :String;
    /// }
    /// ```
    pub fn instinct_declaration(&mut self, is_exported: bool, is_default: bool) -> Stmt {
        // ---------------------------------------------
        // Interface Name
        // ---------------------------------------------
        let name = self.consume_identifier();

        // ---------------------------------------------
        // Begin Interface Body
        // ---------------------------------------------
        self.consume_symbol('{');
        let mut members = Vec::new();

        while !self.check_symbol('}') && !self.is_at_end() {
            // Only method signatures are allowed in instincts
            let name = if self.match_keyword("purr") {
                self.consume_identifier()
            } else {
                panic!("Only 'purr' method signatures allowed in instinct bodies");
            };

            self.consume_arrow();
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
            self.consume_arrow();

            let mut return_type = None;
            if self.match_symbol(':') {
                let t = self.consume_identifier();
                return_type = Some(t);
                self.consume_arrow();
            }

            self.match_symbol(';');

            members.push(InstinctMember {
                name,
                params,
                return_type,
                kind: InstinctMemberKind::Method,
            });
        }

        self.consume_symbol('}');

        Stmt::Instinct {
            name,
            members,
            is_exported,
            is_default,
        }
    }

    /// Resolves a `pride` declaration into either:
    /// - A public variable
    /// - A named scoped block (namespace-style)
    ///
    /// Supported Forms:
    /// ```pawx
    /// pride cats = 10;
    /// pride Config { ... }
    /// ```
    pub fn pride_dispatch(&mut self) -> Stmt {
        let name = self.consume_identifier();

        // Variable assignment form
        if self.match_symbol('=') {
            let value = self.expression();
            self.match_symbol(';');
            return Stmt::PublicVar { name, value };
        }

        // Named scope form
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

    /// Parses a `den` private variable declaration.
    pub fn private_var(&mut self) -> Stmt {
        let name = self.consume_identifier();
        self.consume_symbol('=');
        let value = self.expression();
        self.match_symbol(';');
        Stmt::PrivateVar { name, value }
    }

    /// Parses a `lair` protected variable declaration.
    pub fn protected_var(&mut self) -> Stmt {
        let name = self.consume_identifier();
        self.consume_symbol('=');
        let value = self.expression();
        self.match_symbol(';');
        Stmt::ProtectedVar { name, value }
    }

    /// Parses an `if / else if / else` control-flow construct.
    ///
    /// Supported Forms:
    /// ```pawx
    /// if x > 5 { ... }
    /// if (x > 5) { ... }
    /// else { ... }
    /// else if x < 3 { ... }
    /// ```
    pub fn if_statement(&mut self) -> Stmt {
        let condition = if self.match_symbol('(') {
            let cond = self.expression();
            self.consume_symbol(')');
            cond
        } else {
            self.expression()
        };

        self.consume_symbol('{');
        let mut then_branch = Vec::new();
        while !self.check_symbol('}') {
            then_branch.push(self.statement());
        }
        self.consume_symbol('}');

        let mut else_branch = None;

        if self.match_keyword("else") {
            if self.match_keyword("if") {
                let nested_if = self.if_statement();
                else_branch = Some(vec![nested_if]);
            } else {
                self.consume_symbol('{');
                let mut else_body = Vec::new();
                while !self.check_symbol('}') {
                    else_body.push(self.statement());
                }
                self.consume_symbol('}');
                else_branch = Some(else_body);
            }
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    /// Parses a `while` loop.
    pub fn while_statement(&mut self) -> Stmt {
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

    /// Parses a function `return` statement.
    pub fn return_statement(&mut self) -> Stmt {
        if self.match_symbol(';') {
            return Stmt::Return(None);
        }

        let expr = self.expression();
        self.match_symbol(';');
        Stmt::Return(Some(expr))
    }

    /// Parses a standalone expression used as a statement.
    pub fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.match_symbol(';');
        Stmt::Expression(expr)
    }

    /// Parses a full exception-handling block:
    /// - `try {}`
    /// - optional `catch(e) {}`
    /// - optional `finally {}`
    pub fn try_statement(&mut self) -> Stmt {
        self.consume_symbol('{');

        let mut try_block = Vec::new();
        while !self.check_symbol('}') {
            try_block.push(self.statement());
        }
        self.consume_symbol('}');

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