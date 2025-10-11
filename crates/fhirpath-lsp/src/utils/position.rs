//! LSP position ↔ source offset utilities

use lsp_types::Position;

/// Convert LSP position to byte offset
pub fn position_to_offset(_text: &str, _position: Position) -> Option<usize> {
    // TODO: Implement
    None
}

/// Convert byte offset to LSP position
pub fn offset_to_position(_text: &str, _offset: usize) -> Position {
    // TODO: Implement
    Position::new(0, 0)
}
