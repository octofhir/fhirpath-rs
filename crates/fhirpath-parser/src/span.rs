// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Source location tracking for parser

use nom_locate::LocatedSpan;
use std::fmt;

/// Type alias for located spans in the input
pub type Span<'a> = LocatedSpan<&'a str>;

/// A value with source location information
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    /// The value
    pub value: T,
    /// Start position in the input
    pub start: usize,
    /// End position in the input
    pub end: usize,
}

impl<T> Spanned<T> {
    /// Create a new spanned value
    pub fn new(value: T, start: usize, end: usize) -> Self {
        Self { value, start, end }
    }

    /// Get the span length
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if span is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Map the value while preserving the span
    pub fn map<U, F>(self, f: F) -> Spanned<U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned {
            value: f(self.value),
            start: self.start,
            end: self.end,
        }
    }

    /// Get a reference to the inner value
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            value: &self.value,
            start: self.start,
            end: self.end,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Helper functions for working with spans
pub mod helpers {
    use super::*;

    /// Get the position offset from a span
    pub fn position<'a>(span: &Span<'a>) -> usize {
        span.location_offset()
    }

    /// Get the line number from a span (1-indexed)
    pub fn line<'a>(span: &Span<'a>) -> u32 {
        span.location_line()
    }

    /// Get the column number from a span (1-indexed)
    pub fn column<'a>(span: &Span<'a>) -> usize {
        span.get_utf8_column()
    }

    /// Create a spanned value from a span and value
    pub fn spanned<'a, T>(start: &Span<'a>, end: &Span<'a>, value: T) -> Spanned<T> {
        Spanned::new(value, position(start), position(end))
    }
}
