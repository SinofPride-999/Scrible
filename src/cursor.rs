/// Tracks the cursor position within the text buffer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cursor {
    /// Zero-indexed row (line number).
    pub row: usize,
    /// Zero-indexed column (character offset within the line).
    /// Can be from 0 to line_len (inclusive), where line_len means after the last character.
    pub col: usize,
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Move cursor left, clamped to 0.
    pub fn move_left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }

    /// Move cursor right, clamped to `max_col` (which should be line length, allowing position after last char)
    pub fn move_right(&mut self, max_col: usize) {
        if self.col < max_col {
            self.col += 1;
        }
    }

    /// Move cursor up, clamped to 0.
    pub fn move_up(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    /// Move cursor down, clamped to `max_row`.
    pub fn move_down(&mut self, max_row: usize) {
        if self.row < max_row {
            self.row += 1;
        }
    }

    /// Clamp the column to `max_col` (line length, allowing position after last char)
    pub fn clamp_col(&mut self, max_col: usize) {
        if self.col > max_col {
            self.col = max_col;
        }
    }
}
