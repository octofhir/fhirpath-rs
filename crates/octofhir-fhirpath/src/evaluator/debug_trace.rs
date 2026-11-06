//! Debug trace of an evaluation.
//!
//! The events follow the language-neutral debug trace of the FHIR Mapping Language IG
//! (`debug.md`): each node of the expression has an `enter` and an `exit` with its depth and
//! its position in the source, `trace()` adds a `step` with the message, and the trace says
//! whether it is `complete`, `truncated` or `stopped`.
//!
//! Tracing evaluates with the tree evaluator, as the stack machine has no nodes to report.
//! Without a recorder on the context nothing here runs.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value as JsonValue, json};

use super::context::EvaluationContext;
use super::result::EvaluationResult;
use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathValue, Result};

/// How much a trace may hold. Past either limit evaluation goes on and the trace is `truncated`.
#[derive(Debug, Clone, Copy)]
pub struct DebugLimits {
    pub max_events: usize,
    /// Roughly the size of the values in the trace, as JSON.
    pub max_bytes: usize,
}

impl Default for DebugLimits {
    fn default() -> Self {
        Self {
            max_events: 10_000,
            max_bytes: 4 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugPhase {
    Enter,
    Step,
    Exit,
}

impl DebugPhase {
    pub fn code(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Step => "step",
            Self::Exit => "exit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugStatus {
    /// Evaluation ran to the end and every event was kept.
    Complete,
    /// Evaluation went on past the kept events.
    Truncated,
    /// Evaluation stopped on an error; open scopes have no `exit`.
    Stopped,
}

impl DebugStatus {
    pub fn code(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Truncated => "truncated",
            Self::Stopped => "stopped",
        }
    }
}

/// A place in the expression: 0-based rows, columns in UTF-16 code units, the end exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugRange {
    pub start_row: usize,
    pub start_column: usize,
    pub end_row: usize,
    pub end_column: usize,
}

/// One item of a variable's collection; an empty collection has one entry without a value.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugVariable {
    pub name: String,
    /// `context` for the focus and its siblings, `expression` for what a node returned.
    pub context_type: &'static str,
    pub value: Option<JsonValue>,
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugMessage {
    pub name: String,
    pub text: String,
}

/// One evaluation unit.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugEvent {
    pub phase: DebugPhase,
    pub depth: usize,
    pub range: Option<DebugRange>,
    pub variables: Vec<DebugVariable>,
    pub messages: Vec<DebugMessage>,
}

/// The recorded events of an evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugTrace {
    pub status: DebugStatus,
    /// Why events are missing or evaluation stopped.
    pub note: Option<String>,
    pub events: Vec<DebugEvent>,
}

fn variable_value(value: &JsonValue) -> (&'static str, JsonValue) {
    match value {
        JsonValue::Bool(_) => ("valueBoolean", value.clone()),
        JsonValue::Number(number) if number.is_f64() => ("valueDecimal", value.clone()),
        JsonValue::Number(_) => ("valueInteger", value.clone()),
        JsonValue::String(_) => ("valueString", value.clone()),
        JsonValue::Object(fields) if fields.contains_key("resourceType") => {
            ("resource", value.clone())
        }
        other => ("jsonValue", json!({"valueString": other.to_string()})),
    }
}

impl DebugVariable {
    fn part(&self) -> JsonValue {
        let mut part = vec![
            json!({"name": "name", "valueString": self.name}),
            json!({"name": "contextType", "valueString": self.context_type}),
        ];
        if let Some(value) = &self.value {
            let (element, content) = variable_value(value);
            part.push(match element {
                "resource" => json!({"name": "resource", "resource": content}),
                "jsonValue" => {
                    json!({"name": "jsonValue", "valueString": content["valueString"]})
                }
                _ => json!({"name": "value", element: content}),
            });
        }
        if let Some(data_type) = &self.data_type {
            part.push(json!({"name": "dataType", "valueString": data_type}));
        }
        json!({"name": "variable", "part": part})
    }
}

impl DebugEvent {
    fn parameter(&self) -> JsonValue {
        let mut part = Vec::with_capacity(self.variables.len() + 6);
        if let Some(range) = &self.range {
            part.push(json!({"name": "startRow", "valueInteger": range.start_row}));
            part.push(json!({"name": "startColumn", "valueInteger": range.start_column}));
            part.push(json!({"name": "endRow", "valueInteger": range.end_row}));
            part.push(json!({"name": "endColumn", "valueInteger": range.end_column}));
        }
        part.push(json!({"name": "phase", "valueCode": self.phase.code()}));
        part.push(json!({"name": "depth", "valueInteger": self.depth}));
        part.extend(self.variables.iter().map(DebugVariable::part));
        part.extend(self.messages.iter().map(|message| {
            json!({"name": "message", "part": [
                {"name": "name", "valueString": message.name},
                {"name": "text", "valueString": message.text},
            ]})
        }));
        json!({"name": "trace", "part": part})
    }
}

impl DebugTrace {
    /// The output parameters of the format: `traceStatus` and a `trace` per event.
    pub fn parameters(&self) -> Vec<JsonValue> {
        let mut parameters = Vec::with_capacity(self.events.len() + 1);
        parameters.push(json!({"name": "traceStatus", "valueCode": self.status.code()}));
        parameters.extend(self.events.iter().map(DebugEvent::parameter));
        parameters
    }
}

#[derive(Debug, Default)]
struct Recording {
    events: Vec<DebugEvent>,
    bytes: usize,
    truncated: bool,
}

/// The HTML entities the parser decodes.
const ENTITIES: [(&str, char); 5] = [
    ("&lt;", '<'),
    ("&gt;", '>'),
    ("&amp;", '&'),
    ("&quot;", '"'),
    ("&apos;", '\''),
];

/// Where the bytes of the text the parser reads sit in the expression as written. The parser
/// leaves out comments, collapses whitespace and decodes entities, and its positions are in
/// what remains.
#[derive(Debug)]
struct SourceMap {
    /// For each byte of the parsed text, the range of the character it came from.
    ranges: Vec<(usize, usize)>,
}

/// The offset past whitespace and comments at `from`.
fn skip_blanks(source: &str, mut from: usize) -> usize {
    loop {
        let rest = &source[from..];
        let trimmed = rest.trim_start();
        from += rest.len() - trimmed.len();
        if trimmed.starts_with("//") {
            from += trimmed.find(['\n', '\r']).unwrap_or(trimmed.len());
        } else if trimmed.starts_with("/*") {
            from += trimmed.find("*/").map_or(trimmed.len(), |end| end + 2);
        } else {
            return from;
        }
    }
}

impl SourceMap {
    fn new(source: &str, parsed: &str) -> Self {
        let mut ranges = Vec::with_capacity(parsed.len());
        let mut at = 0;
        let mut delimiter = None;
        let mut escaped = false;
        for character in parsed.chars() {
            if delimiter.is_none() {
                if character == ' ' {
                    let end = skip_blanks(source, at);
                    ranges.push((at, end));
                    at = end;
                    continue;
                }
                at = skip_blanks(source, at);
            }
            let rest = &source[at..];
            let length = ENTITIES
                .iter()
                .find(|(entity, decoded)| {
                    delimiter.is_none() && *decoded == character && rest.starts_with(entity)
                })
                .map_or_else(
                    || rest.chars().next().map_or(0, char::len_utf8),
                    |(entity, _)| entity.len(),
                );
            ranges.extend(std::iter::repeat_n((at, at + length), character.len_utf8()));
            at += length;
            match delimiter {
                Some(_) if escaped => escaped = false,
                Some(_) if character == '\\' => escaped = true,
                Some(open) if open == character => delimiter = None,
                None if matches!(character, '\'' | '"' | '`') => delimiter = Some(character),
                _ => {}
            }
        }
        Self { ranges }
    }

    /// The written range of a parsed one.
    fn written(&self, offset: usize, length: usize) -> Option<(usize, usize)> {
        let start = self.ranges.get(offset)?.0;
        let end = self.ranges.get((offset + length).checked_sub(1)?)?.1;
        Some((start, end))
    }
}

/// Collects the events of one evaluation.
#[derive(Debug)]
pub struct DebugRecorder {
    source: String,
    map: SourceMap,
    limits: DebugLimits,
    depth: AtomicUsize,
    recording: Mutex<Recording>,
}

fn items(name: &str, context_type: &'static str, values: &Collection) -> Vec<DebugVariable> {
    if values.is_empty() {
        return vec![DebugVariable {
            name: name.to_owned(),
            context_type,
            value: None,
            data_type: None,
        }];
    }
    values
        .iter()
        .map(|value| item(name, context_type, value))
        .collect()
}

fn item(name: &str, context_type: &'static str, value: &FhirPathValue) -> DebugVariable {
    DebugVariable {
        name: name.to_owned(),
        context_type,
        value: Some(value.to_json_value()),
        data_type: Some(value.display_type_name()),
    }
}

fn size(variables: &[DebugVariable]) -> usize {
    variables
        .iter()
        .map(|variable| {
            variable.name.len()
                + variable
                    .value
                    .as_ref()
                    .map_or(0, |value| value.to_string().len())
        })
        .sum()
}

impl DebugRecorder {
    pub fn new(source: impl Into<String>, limits: DebugLimits) -> Self {
        let source = source.into();
        let map = SourceMap::new(
            &source,
            &crate::parser::normalized_source(&source).unwrap_or_else(|_| source.clone()),
        );
        Self {
            source,
            map,
            limits,
            depth: AtomicUsize::new(0),
            recording: Mutex::new(Recording::default()),
        }
    }

    /// The row and UTF-16 column of a byte offset in the expression.
    fn position(&self, offset: usize) -> (usize, usize) {
        let (mut row, mut column) = (0, 0);
        for (index, character) in self.source.char_indices() {
            if index >= offset {
                break;
            }
            if character == '\n' {
                row += 1;
                column = 0;
            } else {
                column += character.len_utf16();
            }
        }
        (row, column)
    }

    fn range(&self, node: &ExpressionNode) -> Option<DebugRange> {
        let location = node.location()?;
        let (start, end) = self.map.written(location.offset, location.length)?;
        let (start_row, start_column) = self.position(start);
        let (end_row, end_column) = self.position(end);
        Some(DebugRange {
            start_row,
            start_column,
            end_row,
            end_column,
        })
    }

    fn record(&self, event: DebugEvent) {
        let mut recording = self.recording.lock().unwrap_or_else(|e| e.into_inner());
        let bytes = recording.bytes + size(&event.variables);
        if recording.events.len() >= self.limits.max_events || bytes > self.limits.max_bytes {
            recording.truncated = true;
            return;
        }
        recording.bytes = bytes;
        recording.events.push(event);
    }

    /// The focus of a node, about to be evaluated.
    pub(super) fn enter(&self, node: &ExpressionNode, context: &EvaluationContext) {
        let depth = self.depth.fetch_add(1, Ordering::Relaxed);
        let mut variables = items("$this", "context", context.input_collection());
        if let Some(index) = context.get_variable("$index") {
            variables.push(item("$index", "context", &index));
        }
        self.record(DebugEvent {
            phase: DebugPhase::Enter,
            depth,
            range: self.range(node),
            variables,
            messages: Vec::new(),
        });
    }

    /// What a node returned. A failing node has no `exit`.
    pub(super) fn exit(&self, node: &ExpressionNode, result: &Result<EvaluationResult>) {
        let depth = self.depth.fetch_sub(1, Ordering::Relaxed) - 1;
        if let Ok(result) = result {
            self.record(DebugEvent {
                phase: DebugPhase::Exit,
                depth,
                range: self.range(node),
                variables: items("result", "expression", &result.value),
                messages: Vec::new(),
            });
        }
    }

    /// A `trace()` output.
    pub(crate) fn message(&self, name: &str, text: String) {
        self.record(DebugEvent {
            phase: DebugPhase::Step,
            depth: self.depth.load(Ordering::Relaxed),
            range: None,
            variables: Vec::new(),
            messages: vec![DebugMessage {
                name: name.to_owned(),
                text,
            }],
        });
    }

    /// The trace of a finished evaluation; `failure` is what stopped it.
    pub fn finish(&self, failure: Option<String>) -> DebugTrace {
        let mut recording = self.recording.lock().unwrap_or_else(|e| e.into_inner());
        let truncated = recording.truncated;
        let events = std::mem::take(&mut recording.events);
        let (status, note) = match (failure, truncated) {
            (Some(failure), _) => (DebugStatus::Stopped, Some(failure)),
            (None, true) => (
                DebugStatus::Truncated,
                Some("The trace reached its limits".to_owned()),
            ),
            (None, false) => (DebugStatus::Complete, None),
        };
        DebugTrace {
            status,
            note,
            events,
        }
    }
}
