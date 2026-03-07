use std::path::PathBuf;
use std::time::Duration;

use super::{
    buffer::Buffer,
    clipboard::Clipboard,
    cursor::Cursor,
    file_io,
    highlight::Highlight,
    history::{History, Snapshot},
    input::Action,
    mode::Mode,
    status::StatusBar,
};

/// Return value from `Editor::apply`.
pub enum EditorResult {
    Continue,
    /// User requested quit.  `bool` = needs save confirmation.
    Quit {
        modified: bool,
    },
    /// Confirmed quit.
    ForceQuit,
}

/// All mutable editor state (except TUI/terminal).
pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub mode: Mode,
    pub highlight: Highlight,
    pub clipboard: Clipboard,
    pub status: StatusBar,
    pub history: History,
    pub filepath: PathBuf,
    pub modified: bool,
    /// Tracks scroll offset for the viewport.
    pub scroll_offset: usize,
}

impl Editor {
    pub fn new(filepath: PathBuf, content: String) -> Self {
        let buffer = Buffer::from_str(&content);
        let cursor = Cursor::default();
        let snapshot = Snapshot {
            buffer: buffer.clone(),
            cursor: cursor.clone(),
        };
        Self {
            buffer,
            cursor,
            mode: Mode::Normal,
            highlight: Highlight::default(),
            clipboard: Clipboard::default(),
            status: StatusBar::default(),
            history: History::new(snapshot),
            filepath,
            modified: false,
            scroll_offset: 0,
        }
    }

    /// Apply an `Action` and return what the app loop should do next.
    pub fn apply(&mut self, action: Action) -> EditorResult {
        match action {
            Action::None => {}

            // ── Mode ────────────────────────────────────────────────────
            Action::EnterInsert => {
                self.mode = Mode::Insert;
                self.status.set("-- INSERT --", Duration::from_secs(2));
            }
            Action::ExitInsert => {
                self.mode = Mode::Normal;
                self.status.set("-- NORMAL --", Duration::from_secs(2));
            }

            // ── Cursor movement ─────────────────────────────────────────
            Action::MoveLeft => {
                self.cursor.move_left();
                self.update_highlight();
            }
            Action::MoveRight => {
                // Use line length as max, allowing cursor after last character
                let max = self.current_line_len();
                self.cursor.move_right(max);
                self.update_highlight();
            }
            Action::MoveUp => {
                self.cursor.move_up();
                // When moving to a shorter line, clamp to that line's length (allow after last char)
                let max = self.current_line_len();
                self.cursor.clamp_col(max);
                self.adjust_scroll();
                self.update_highlight();
            }
            Action::MoveDown => {
                let max_row = self.buffer.len().saturating_sub(1);
                self.cursor.move_down(max_row);
                // When moving to a shorter line, clamp to that line's length (allow after last char)
                let max = self.current_line_len();
                self.cursor.clamp_col(max);
                self.adjust_scroll();
                self.update_highlight();
            }
            Action::GotoFirstLine => {
                self.cursor = Cursor::new(0, 0);
                self.scroll_offset = 0;
            }
            Action::GotoLastLine => {
                let last = self.buffer.len().saturating_sub(1);
                self.cursor = Cursor::new(last, 0);
                self.adjust_scroll();
            }

            // ── Text insertion ──────────────────────────────────────────
            Action::InsertChar(c) => {
                self.insert_char_at_cursor(c);
            }
            Action::InsertNewline => {
                self.push_history();
                let (new_row, new_col) =
                    self.buffer.insert_newline(self.cursor.row, self.cursor.col);
                self.cursor = Cursor::new(new_row, new_col);
                self.modified = true;
                self.adjust_scroll();
            }
            Action::Backspace => {
                self.push_history();
                let (new_row, new_col) = self.buffer.backspace(self.cursor.row, self.cursor.col);
                self.cursor = Cursor::new(new_row, new_col);
                self.modified = true;
            }
            Action::Delete => {
                self.push_history();
                self.buffer.delete_char(self.cursor.row, self.cursor.col);
                // After delete, clamp to line length (allow after last char)
                let max = self.current_line_len();
                self.cursor.clamp_col(max);
                self.modified = true;
            }

            // ── Clipboard ───────────────────────────────────────────────
            Action::Copy => {
                if let Some((start, end)) = self.highlight.ordered_range() {
                    let text = self.buffer.get_range(start, end);
                    self.clipboard.copy(text);
                    self.highlight.clear();
                    self.status.set("Copied selection", Duration::from_secs(2));
                } else {
                    // Copy current line
                    let text = self.buffer.lines[self.cursor.row].clone();
                    self.clipboard.copy(text);
                    self.status.set("Copied line", Duration::from_secs(2));
                }
            }
            Action::Paste => {
                if self.clipboard.is_empty() {
                    self.status
                        .set("Clipboard is empty", Duration::from_secs(2));
                } else {
                    self.push_history();
                    let text = self.clipboard.paste().to_string();
                    self.buffer
                        .insert_str(self.cursor.row, self.cursor.col, &text);
                    self.modified = true;
                    self.status.set("Pasted", Duration::from_secs(2));
                }
            }
            Action::Cut => {
                if let Some((start, end)) = self.highlight.ordered_range() {
                    let text = self.buffer.get_range(start, end);
                    self.clipboard.copy(text);
                    self.push_history();
                    self.buffer.delete_range(start, end);
                    self.cursor = Cursor::new(start.0, start.1);
                    self.highlight.clear();
                    self.modified = true;
                    self.status.set("Cut selection", Duration::from_secs(2));
                } else {
                    // Cut current line
                    let text = self.buffer.lines[self.cursor.row].clone();
                    self.clipboard.copy(text);
                    self.push_history();
                    if self.buffer.len() > 1 {
                        self.buffer.lines.remove(self.cursor.row);
                        if self.cursor.row >= self.buffer.len() {
                            self.cursor.row = self.buffer.len().saturating_sub(1);
                        }
                    } else {
                        self.buffer.lines[0].clear();
                        self.cursor.col = 0;
                    }
                    self.modified = true;
                    self.status.set("Cut line", Duration::from_secs(2));
                }
            }

            // ── Highlight ───────────────────────────────────────────────
            Action::ToggleHighlight => {
                if self.highlight.active {
                    self.highlight.clear();
                    self.status.set("Highlight cleared", Duration::from_secs(2));
                } else {
                    self.highlight.start(&self.cursor);
                    self.status.set(
                        "Highlight ON — move cursor to select",
                        Duration::from_secs(3),
                    );
                }
            }

            // ── Undo / Redo ─────────────────────────────────────────────
            Action::Undo => {
                let current = self.snapshot();
                if let Some(prev) = self.history.undo(current) {
                    self.buffer = prev.buffer;
                    self.cursor = prev.cursor;
                    self.highlight.clear();
                    self.modified = true;
                    self.status.set("Undone", Duration::from_secs(2));
                } else {
                    self.status.set("Nothing to undo", Duration::from_secs(2));
                }
            }
            Action::Redo => {
                let current = self.snapshot();
                if let Some(next) = self.history.redo(current) {
                    self.buffer = next.buffer;
                    self.cursor = next.cursor;
                    self.highlight.clear();
                    self.modified = true;
                    self.status.set("Redone", Duration::from_secs(2));
                } else {
                    self.status.set("Nothing to redo", Duration::from_secs(2));
                }
            }

            // ── File I/O ────────────────────────────────────────────────
            Action::Save => {
                let content = self.buffer.to_string();
                match file_io::save(&self.filepath, &content) {
                    Ok(_) => {
                        self.modified = false;
                        let name = self.filepath.display().to_string();
                        self.status
                            .set(format!("Saved {}", name), Duration::from_secs(3));
                    }
                    Err(e) => {
                        self.status
                            .set(format!("Error saving: {}", e), Duration::from_secs(5));
                    }
                }
            }

            // ── Quit ────────────────────────────────────────────────────
            Action::Quit => {
                return EditorResult::Quit {
                    modified: self.modified,
                };
            }
            Action::QuitForce => {
                return EditorResult::ForceQuit;
            }

            // ── Mouse ───────────────────────────────────────────────────
            Action::MouseClick { row, col } => {
                // row 0 = title, row 1+ = editor (offset by 1)
                let editor_row = (row as usize)
                    .saturating_sub(1)
                    .saturating_add(self.scroll_offset);
                let editor_col = col as usize;
                let clamped_row = editor_row.min(self.buffer.len().saturating_sub(1));
                // Allow cursor after last character (use line length as max, not length-1)
                let max_col = self.buffer.line_len(clamped_row);
                self.cursor = Cursor::new(clamped_row, editor_col.min(max_col));
            }
            Action::MouseDrag { row, col } => {
                let editor_row = (row as usize)
                    .saturating_sub(1)
                    .saturating_add(self.scroll_offset);
                let clamped_row = editor_row.min(self.buffer.len().saturating_sub(1));
                // Allow cursor after last character
                let max_col = self.buffer.line_len(clamped_row);
                self.cursor = Cursor::new(clamped_row, (col as usize).min(max_col));
                if !self.highlight.active {
                    self.highlight.start(&self.cursor);
                }
                self.highlight.update(&self.cursor);
            }
            Action::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(3);
            }
            Action::ScrollDown => {
                let max = self.buffer.len().saturating_sub(1);
                self.scroll_offset = (self.scroll_offset + 3).min(max);
            }
        }

        EditorResult::Continue
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn insert_char_at_cursor(&mut self, c: char) {
        self.push_history();
        self.buffer.insert_char(self.cursor.row, self.cursor.col, c);
        self.cursor.col += 1;
        self.modified = true;
    }

    fn current_line_len(&self) -> usize {
        self.buffer.line_len(self.cursor.row)
    }

    fn update_highlight(&mut self) {
        self.highlight.update(&self.cursor);
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            buffer: self.buffer.clone(),
            cursor: self.cursor.clone(),
        }
    }

    fn push_history(&mut self) {
        self.history.push(self.snapshot());
    }

    /// Keep the cursor visible within the viewport.
    fn adjust_scroll(&mut self) {
        // This is called with an approximate viewport height; App will refine.
        if self.cursor.row < self.scroll_offset {
            self.scroll_offset = self.cursor.row;
        }
    }

    /// Called from App with the actual viewport height.
    pub fn adjust_scroll_with_height(&mut self, height: usize) {
        if self.cursor.row < self.scroll_offset {
            self.scroll_offset = self.cursor.row;
        }
        if self.cursor.row >= self.scroll_offset + height {
            self.scroll_offset = self.cursor.row.saturating_sub(height - 1);
        }
    }
}
