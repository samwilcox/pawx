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

use crate::span::Span;

#[derive(Debug, Clone)]
pub struct PawxError {
    /// Stable error code (P0001, P0002, …)
    pub code: &'static str,

    /// Human-readable error message
    pub message: String,

    /// Primary source location
    pub span: Span,

    /// Optional note / help text
    pub help: Option<String>,
}

impl PawxError {
    /// Generic constructor
    pub fn new(
        code: &'static str,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            help: None,
        }
    }

    /// Runtime error (during evaluation)
    pub fn runtime_error(
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::new("E_RUNTIME", message, span)
    }

    /// Type error (invalid operation / operand types)
    pub fn type_error(
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::new("E_TYPE", message, span)
    }

    /// Reference error (undefined variable, property, etc.)
    pub fn reference_error(
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::new("E_REFERENCE", message, span)
    }

    /// Attach a help message to the error (builder-style).
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}