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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Param, Stmt};
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Access {
    Public,     // pride
    Private,    // den
    Protected,  // lair
}

#[derive(Debug, Clone)]
pub struct EnvEntry {
    pub value: Value,
    pub access: Access,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub return_type: Option<String>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, EnvEntry>,
    pub functions: HashMap<String, FunctionDef>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            functions: HashMap::new(),
            parent,
        }
    }

    // pride = PUBLIC
    pub fn define_public(&mut self, name: String, value: Value) {
        self.values.insert(
            name,
            EnvEntry {
                value,
                access: Access::Public,
            },
        );
    }

    // den = PRIVATE
    pub fn define_private(&mut self, name: String, value: Value) {
        self.values.insert(
            name,
            EnvEntry {
                value,
                access: Access::Private,
            },
        );
    }

    // lair = PROTECTED
    pub fn define_protected(&mut self, name: String, value: Value) {
        self.values.insert(
            name,
            EnvEntry {
                value,
                access: Access::Protected,
            },
        );
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if let Some(entry) = self.values.get_mut(name) {
            entry.value = value;
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }

        false
    }

    // Public    -> anywhere
    // Private   -> same scope only
    // Protected -> same scope + child scopes
    pub fn get(&self, name: &str, is_child_scope: bool) -> Option<Value> {
        if let Some(entry) = self.values.get(name) {
            match entry.access {
                Access::Public => return Some(entry.value.clone()),

                Access::Private => {
                    if !is_child_scope {
                        return Some(entry.value.clone());
                    } else {
                        panic!("Private access violation: '{}'", name);
                    }
                }

                Access::Protected => {
                    return Some(entry.value.clone());
                }
            }
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name, true);
        }

        None
    }

    pub fn define_function(&mut self, name: String, func: FunctionDef) {
        self.functions.insert(name, func);
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionDef> {
        if let Some(f) = self.functions.get(name) {
            return Some(f.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get_function(name);
        }

        None
    }
}