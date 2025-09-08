//! Advanced text editor with FHIRPath-aware features

use std::ops::Range;

/// Advanced text editor with FHIRPath-aware features
pub struct TextEditor {
    text: String,
    cursor_position: usize,
    selection: Option<Range<usize>>,
    undo_stack: Vec<EditorState>,
    redo_stack: Vec<EditorState>,
}

/// Editor state for undo/redo
#[derive(Debug, Clone)]
struct EditorState {
    text: String,
    cursor_position: usize,
    selection: Option<Range<usize>>,
}

impl TextEditor {
    /// Create a new text editor
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Get current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set text content
    pub fn set_text(&mut self, text: String) {
        self.save_state();
        self.text = text;
        self.cursor_position = self.cursor_position.min(self.text.len());
        self.selection = None;
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Set cursor position
    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position.min(self.text.len());
        self.selection = None;
    }

    /// Insert text at cursor
    pub fn insert_text(&mut self, text: &str) {
        self.save_state();

        if let Some(selection) = &self.selection {
            // Replace selection
            self.text.replace_range(selection.clone(), text);
            self.cursor_position = selection.start + text.len();
            self.selection = None;
        } else {
            // Insert at cursor
            self.text.insert_str(self.cursor_position, text);
            self.cursor_position += text.len();
        }
    }

    /// Delete character at cursor
    pub fn delete_char(&mut self) {
        if self.cursor_position < self.text.len() {
            self.save_state();
            self.text.remove(self.cursor_position);
        }
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.save_state();
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.text.len();
    }

    /// Move cursor to previous word
    pub fn cursor_word_left(&mut self) {
        if self.cursor_position > 0 {
            let mut pos = self.cursor_position - 1;

            // Skip whitespace
            while pos > 0
                && self
                    .text
                    .chars()
                    .nth(pos)
                    .map_or(false, |c| c.is_whitespace())
            {
                pos -= 1;
            }

            // Skip to start of word
            while pos > 0
                && self
                    .text
                    .chars()
                    .nth(pos - 1)
                    .map_or(false, |c| c.is_alphanumeric())
            {
                pos -= 1;
            }

            self.cursor_position = pos;
        }
    }

    /// Move cursor to next word
    pub fn cursor_word_right(&mut self) {
        if self.cursor_position < self.text.len() {
            let mut pos = self.cursor_position;
            let chars: Vec<char> = self.text.chars().collect();

            // Skip current word
            while pos < chars.len() && chars[pos].is_alphanumeric() {
                pos += 1;
            }

            // Skip whitespace
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            self.cursor_position = pos;
        }
    }

    /// Delete word at cursor
    pub fn delete_word(&mut self) {
        let start = self.cursor_position;
        self.cursor_word_right();
        let end = self.cursor_position;

        if start != end {
            self.save_state();
            self.text.drain(start..end);
            self.cursor_position = start;
        }
    }

    /// Delete to end of line
    pub fn delete_to_end(&mut self) {
        if self.cursor_position < self.text.len() {
            self.save_state();
            self.text.truncate(self.cursor_position);
        }
    }

    /// Clear all text
    pub fn clear(&mut self) {
        if !self.text.is_empty() {
            self.save_state();
            self.text.clear();
            self.cursor_position = 0;
            self.selection = None;
        }
    }

    /// Undo last operation
    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            let current_state = EditorState {
                text: self.text.clone(),
                cursor_position: self.cursor_position,
                selection: self.selection.clone(),
            };
            self.redo_stack.push(current_state);

            self.text = state.text;
            self.cursor_position = state.cursor_position;
            self.selection = state.selection;
        }
    }

    /// Redo last undone operation
    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            let current_state = EditorState {
                text: self.text.clone(),
                cursor_position: self.cursor_position,
                selection: self.selection.clone(),
            };
            self.undo_stack.push(current_state);

            self.text = state.text;
            self.cursor_position = state.cursor_position;
            self.selection = state.selection;
        }
    }

    /// Save current state for undo
    fn save_state(&mut self) {
        let state = EditorState {
            text: self.text.clone(),
            cursor_position: self.cursor_position,
            selection: self.selection.clone(),
        };
        self.undo_stack.push(state);

        // Limit undo stack size
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }

        // Clear redo stack when new changes are made
        self.redo_stack.clear();
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}
