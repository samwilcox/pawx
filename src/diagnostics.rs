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

use crate::error::PawxError;
use crate::span::Span;
use std::fs;

/// Responsible for rendering human-friendly, compiler-style diagnostics
/// for PAWX errors.
///
/// This printer:
/// - Formats errors with file/line/column information
/// - Displays the offending source line
/// - Highlights the exact error position using a caret (`^`)
/// - Optionally shows a helpful follow-up hint
///
/// The output is intentionally inspired by `rustc` diagnostics, but
/// simplified for PAWX and designed to remain readable without color.
pub struct DiagnosticPrinter {
    /// Full source code of the file being interpreted.
    ///
    /// Stored as a single string so we can easily extract specific
    /// lines for error reporting.
    source: String,

    /// Name of the source file (e.g. `main.px`).
    ///
    /// Used only for display purposes in diagnostics.
    file_name: String,
}

impl DiagnosticPrinter {
    /// Creates a new diagnostic printer for a given source file.
    ///
    /// # Arguments
    /// - `file_name` → The name of the file being executed
    /// - `source` → The full source text of that file
    ///
    /// Both parameters accept any type convertible into `String`
    /// for ergonomic call-sites.
    pub fn new(file_name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            source: source.into(),
        }
    }

    /// Prints a formatted error diagnostic to stderr.
    ///
    /// This function:
    /// 1. Extracts line/column information from the error span
    /// 2. Locates the corresponding line of source code
    /// 3. Prints a compiler-style error header
    /// 4. Renders the source line with a caret pointing at the error
    /// 5. Optionally prints a helpful suggestion
    ///
    /// # Output Example
    /// ```text
    /// error[P0004]: invalid binary operation
    ///   --> example.px:12:10
    ///    |
    /// 12 | let x = 5 + true
    ///    |          ^
    /// help: Check operand types or use a conversion.
    /// ```
    pub fn print(&self, error: &PawxError) {
        // Destructure the span to get precise location data
        let Span { line, column } = error.span;

        // Split the source into individual lines so we can fetch
        // the exact line where the error occurred.
        let lines: Vec<&str> = self.source.lines().collect();

        // Lines are 1-indexed in diagnostics, but vectors are 0-indexed.
        // `saturating_sub` prevents underflow if line == 0.
        let src_line = lines.get(line.saturating_sub(1)).unwrap_or(&"");

        // Print the main error header, including:
        // - Stable error code
        // - Human-readable message
        // - File name + line + column
        eprintln!(
            "error[{}]: {}\n  --> {}:{}:{}",
            error.code,
            error.message,
            self.file_name,
            line,
            column + 1
        );

        // Visual separator (matches rustc style)
        eprintln!("   |");

        // Print the offending source line with its line number
        eprintln!("{:>3} | {}", line, src_line);

        // Build a caret underline pointing exactly to the column
        // where the error occurred.
        let mut underline = String::new();
        for _ in 0..column {
            underline.push(' ');
        }
        underline.push('^');

        // Render the underline beneath the source line
        eprintln!("   | {}", underline);

        // If the error includes an optional help message,
        // display it as a follow-up suggestion.
        if let Some(help) = &error.help {
            eprintln!("\nhelp: {}", help);
        }
    }
}