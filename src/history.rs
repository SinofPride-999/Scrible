use super::buffer::Buffer;
use super::cursor::Cursor;

/// A snapshot of the editor state used for undo/redo.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub buffer: Buffer,
    pub cursor: Cursor,
}

/// Ring-buffer-style undo/redo history.
pub struct History {
    /// Past states (oldest → newest).  Current is at `past[past.len()-1]`.
    past: Vec<Snapshot>,
    /// Future states for redo (most recent undo at the end).
    future: Vec<Snapshot>,
    /// Maximum number of snapshots to keep.
    max_size: usize,
}

impl History {
    pub fn new(initial: Snapshot) -> Self {
        Self {
            past: vec![initial],
            future: Vec::new(),
            max_size: 200,
        }
    }

    /// Push a new snapshot (clears the redo stack).
    pub fn push(&mut self, snapshot: Snapshot) {
        self.future.clear();
        self.past.push(snapshot);
        if self.past.len() > self.max_size {
            self.past.remove(0);
        }
    }

    /// Undo: pop the most recent snapshot, return previous state.
    /// Returns `None` if there is nothing to undo.
    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        if self.past.len() <= 1 {
            return None; // Nothing to undo — the very first snapshot is the baseline.
        }
        self.future.push(current);
        self.past.pop() // returns the state we're reverting TO
                        // (the one before current was pushed)
    }

    /// Redo: re-apply the most recently undone snapshot.
    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let next = self.future.pop()?;
        self.past.push(current);
        Some(next)
    }
}
