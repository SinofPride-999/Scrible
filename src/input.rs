use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

/// High-level editor actions parsed from raw input events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Mode
    EnterInsert,
    ExitInsert,

    // Normal-mode commands
    Save,
    Quit,
    QuitForce,
    Copy,
    Paste,
    Cut,
    Undo,
    Redo,
    ToggleHighlight,
    GotoFirstLine,
    GotoLastLine,

    // Cursor movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    // Insert-mode text input
    InsertChar(char),
    InsertNewline,
    Backspace,
    Delete,

    // Mouse
    MouseClick { row: u16, col: u16 },
    MouseDrag { row: u16, col: u16 },
    ScrollUp,
    ScrollDown,

    // No-op
    None,
}

/// Context passed to the input parser so it can make mode-aware decisions.
pub struct InputContext {
    pub mode: crate::mode::Mode,
    /// Pending key buffer for multi-key sequences (e.g. "hh" for highlight)
    pub pending: String,
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            mode: crate::mode::Mode::Normal,
            pending: String::new(),
        }
    }
}

/// Convert a crossterm `Event` into an `Action` given the current context.
pub fn handle_event(event: Event, ctx: &mut InputContext) -> Action {
    match event {
        Event::Key(key) => handle_key(key, ctx),
        Event::Mouse(mouse) => handle_mouse(mouse),
        _ => Action::None,
    }
}

fn handle_key(key: KeyEvent, ctx: &mut InputContext) -> Action {
    use crate::mode::Mode;

    // Ctrl+C / Ctrl+Q always quit
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => return Action::QuitForce,
            KeyCode::Char('s') => return Action::Save,
            _ => {}
        }
    }

    match &ctx.mode {
        Mode::Insert => handle_insert_key(key, ctx),
        Mode::Normal => handle_normal_key(key, ctx),
    }
}

fn handle_insert_key(key: KeyEvent, ctx: &mut InputContext) -> Action {
    match key.code {
        // Use ESC to exit insert mode (RightShift isn't a separate key code)
        KeyCode::Esc => {
            ctx.pending.clear();
            Action::ExitInsert
        }
        KeyCode::Enter => {
            ctx.pending.clear();
            Action::InsertNewline
        }
        KeyCode::Backspace => {
            ctx.pending.clear();
            Action::Backspace
        }
        KeyCode::Delete => {
            ctx.pending.clear();
            Action::Delete
        }
        KeyCode::Left => Action::MoveLeft,
        KeyCode::Right => Action::MoveRight,
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Char(c) => {
            ctx.pending.clear();
            Action::InsertChar(c)
        }
        _ => Action::None,
    }
}

fn handle_normal_key(key: KeyEvent, ctx: &mut InputContext) -> Action {
    match key.code {
        // Use space to enter insert mode
        KeyCode::Char(' ') => {
            ctx.pending.clear();
            Action::EnterInsert
        }
        KeyCode::Char('h') => {
            if ctx.pending == "h" {
                ctx.pending.clear();
                Action::ToggleHighlight
            } else {
                ctx.pending = "h".to_string();
                // Ambiguous: might be "move left" or start of "hh".
                // We return MoveLeft immediately and rely on the next 'h'
                // to trigger ToggleHighlight.
                Action::MoveLeft
            }
        }
        KeyCode::Char('j') => {
            ctx.pending.clear();
            Action::MoveDown
        }
        KeyCode::Char('k') => {
            ctx.pending.clear();
            Action::MoveUp
        }
        KeyCode::Char('l') => {
            ctx.pending.clear();
            Action::MoveRight
        }
        KeyCode::Char('c') => {
            ctx.pending.clear();
            Action::Copy
        }
        KeyCode::Char('v') => {
            ctx.pending.clear();
            Action::Paste
        }
        KeyCode::Char('x') => {
            ctx.pending.clear();
            Action::Cut
        }
        KeyCode::Char('s') => {
            ctx.pending.clear();
            Action::Save
        }
        KeyCode::Char('u') => {
            ctx.pending.clear();
            Action::Undo
        }
        KeyCode::Char('r') => {
            ctx.pending.clear();
            Action::Redo
        }
        KeyCode::Char('t') => {
            ctx.pending.clear();
            Action::GotoFirstLine
        }
        KeyCode::Char('b') => {
            ctx.pending.clear();
            Action::GotoLastLine
        }
        KeyCode::Char('q') => {
            ctx.pending.clear();
            Action::Quit
        }
        KeyCode::Left => Action::MoveLeft,
        KeyCode::Right => Action::MoveRight,
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        _ => {
            ctx.pending.clear();
            Action::None
        }
    }
}

fn handle_mouse(mouse: MouseEvent) -> Action {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => Action::MouseClick {
            row: mouse.row,
            col: mouse.column,
        },
        MouseEventKind::Drag(MouseButton::Left) => Action::MouseDrag {
            row: mouse.row,
            col: mouse.column,
        },
        MouseEventKind::ScrollUp => Action::ScrollUp,
        MouseEventKind::ScrollDown => Action::ScrollDown,
        _ => Action::None,
    }
}
