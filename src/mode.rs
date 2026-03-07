/// The two editor modes, inspired by Neovim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Navigate, copy/paste, undo/redo, save, quit.
    Normal,
    /// Type and edit text freely.
    Insert,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
