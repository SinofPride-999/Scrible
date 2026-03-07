use unicode_segmentation::UnicodeSegmentation;

/// The primary text storage.  Each element is one line (no trailing newline).
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    /// Create from a raw file string.
    pub fn from_str(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        // Ensure at least one line
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self { lines }
    }

    /// Serialize back to a file string (lines joined with newline).
    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    /// Number of lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Length (in grapheme clusters) of a given line.
    pub fn line_len(&self, row: usize) -> usize {
        self.lines
            .get(row)
            .map(|l| l.graphemes(true).count())
            .unwrap_or(0)
    }

    /// Insert a single character at (row, col).
    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if row >= self.lines.len() {
            self.lines.resize(row + 1, String::new());
        }
        let line = &mut self.lines[row];
        // Convert grapheme col to byte offset
        let byte_idx = grapheme_to_byte(line, col);
        line.insert(byte_idx, ch);
    }

    /// Delete the character at (row, col).  Returns the deleted char if any.
    pub fn delete_char(&mut self, row: usize, col: usize) -> Option<char> {
        let line = self.lines.get_mut(row)?;
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        if col >= graphemes.len() {
            return None;
        }
        let byte_start = grapheme_to_byte(line, col);
        let ch = line.chars().nth(col);
        let grapheme_len = graphemes[col].len();
        line.drain(byte_start..byte_start + grapheme_len);
        ch
    }

    /// Delete char before (row, col) — backspace semantics.
    /// Returns new (row, col) position.
    pub fn backspace(&mut self, row: usize, col: usize) -> (usize, usize) {
        if col > 0 {
            self.delete_char(row, col - 1);
            (row, col - 1)
        } else if row > 0 {
            // Merge this line with the previous one
            let cur_line = self.lines.remove(row);
            let prev_len = self.lines[row - 1].graphemes(true).count();
            self.lines[row - 1].push_str(&cur_line);
            (row - 1, prev_len)
        } else {
            (row, col)
        }
    }

    /// Insert a newline at (row, col), splitting the line.
    /// Returns the new cursor position (row+1, 0).
    pub fn insert_newline(&mut self, row: usize, col: usize) -> (usize, usize) {
        if row >= self.lines.len() {
            self.lines.push(String::new());
            return (row + 1, 0);
        }
        let line = &self.lines[row];
        let byte_idx = grapheme_to_byte(line, col);
        let rest = self.lines[row][byte_idx..].to_string();
        self.lines[row].truncate(byte_idx);
        self.lines.insert(row + 1, rest);
        (row + 1, 0)
    }

    /// Get a slice of text between two positions (for copy/cut).
    /// Positions are (row, col) pairs; start must be <= end.
    pub fn get_range(&self, start: (usize, usize), end: (usize, usize)) -> String {
        let (sr, sc) = start;
        let (er, ec) = end;

        if sr == er {
            let line = &self.lines[sr];
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let sc = sc.min(graphemes.len());
            let ec = ec.min(graphemes.len());
            graphemes[sc..ec].join("")
        } else {
            let mut result = String::new();
            // First partial line
            let first = &self.lines[sr];
            let fg: Vec<&str> = first.graphemes(true).collect();
            let sc = sc.min(fg.len());
            result.push_str(&fg[sc..].join(""));
            // Middle full lines
            for r in (sr + 1)..er {
                result.push('\n');
                result.push_str(&self.lines[r]);
            }
            // Last partial line
            result.push('\n');
            let last = &self.lines[er];
            let lg: Vec<&str> = last.graphemes(true).collect();
            let ec = ec.min(lg.len());
            result.push_str(&lg[..ec].join(""));
            result
        }
    }

    /// Delete text between start and end (inclusive start, exclusive end).
    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) {
        let (sr, sc) = start;
        let (er, ec) = end;

        if sr == er {
            let line = &mut self.lines[sr];
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let sc = sc.min(graphemes.len());
            let ec = ec.min(graphemes.len());
            let byte_start = grapheme_to_byte(line, sc);
            let byte_end = grapheme_to_byte(line, ec);
            line.drain(byte_start..byte_end);
        } else {
            // Collect the tail of the last line
            let tail = {
                let last = &self.lines[er];
                let lg: Vec<&str> = last.graphemes(true).collect();
                let ec = ec.min(lg.len());
                lg[ec..].join("")
            };
            // Truncate first line at sc
            {
                let first = &mut self.lines[sr];
                let byte_start = grapheme_to_byte(first, sc);
                first.truncate(byte_start);
            }
            // Append tail to first line
            self.lines[sr].push_str(&tail);
            // Remove intermediate + last lines
            self.lines.drain((sr + 1)..=er);
        }
    }

    /// Insert a multi-line string at (row, col).
    pub fn insert_str(&mut self, row: usize, col: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        let parts: Vec<&str> = text.split('\n').collect();
        if parts.len() == 1 {
            for ch in parts[0].chars() {
                let c = self.line_len(row);
                let col_clamped = col.min(c);
                self.insert_char(row, col_clamped, ch);
            }
            return;
        }
        // Split current line at col
        let (_, new_row) = self.insert_newline(row, col);
        // Insert first part at end of row
        let first = parts[0];
        let insert_col = self.line_len(row);
        for (i, ch) in first.chars().enumerate() {
            self.insert_char(row, insert_col + i, ch);
        }
        // Insert middle lines
        for (i, part) in parts[1..parts.len() - 1].iter().enumerate() {
            self.lines.insert(new_row + i, part.to_string());
        }
        // Prepend last part to new_row line
        let last_part = parts[parts.len() - 1];
        let target_row = new_row + parts.len() - 2;
        for (i, ch) in last_part.chars().enumerate() {
            self.insert_char(target_row, i, ch);
        }
    }
}

/// Convert a grapheme-cluster index to a byte offset in `s`.
pub fn grapheme_to_byte(s: &str, grapheme_idx: usize) -> usize {
    s.grapheme_indices(true)
        .nth(grapheme_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
