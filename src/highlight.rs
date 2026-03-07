use super::cursor::Cursor;

/// Represents an active text selection (highlight).
#[derive(Debug, Clone, Default)]
pub struct Highlight {
    /// Whether a selection is currently active.
    pub active: bool,
    /// The anchor point (where the selection started).
    pub anchor: Option<(usize, usize)>,
    /// The current end of the selection (tracks cursor).
    pub end: Option<(usize, usize)>,
}

impl Highlight {
    /// Start a selection at the current cursor position.
    pub fn start(&mut self, cursor: &Cursor) {
        self.active = true;
        self.anchor = Some((cursor.row, cursor.col));
        self.end = Some((cursor.row, cursor.col));
    }

    /// Update the selection end as the cursor moves.
    pub fn update(&mut self, cursor: &Cursor) {
        if self.active {
            self.end = Some((cursor.row, cursor.col));
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.active = false;
        self.anchor = None;
        self.end = None;
    }

    /// Returns (start, end) in document order, or None if no selection.
    pub fn ordered_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.anchor?;
        let end = self.end?;
        if anchor <= end {
            Some((anchor, end))
        } else {
            Some((end, anchor))
        }
    }

    /// Returns true if (row, col) falls within the highlighted region.
    pub fn contains(&self, row: usize, col: usize) -> bool {
        if let Some((start, end)) = self.ordered_range() {
            let pos = (row, col);
            pos >= start && pos < end
        } else {
            false
        }
    }
}
