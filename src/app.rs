use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};

use super::{
    editor::{Editor, EditorResult},
    file_io,
    input::{self, Action, InputContext},
    renderer,
};

/// Owns the terminal, event loop, and editor state.
pub struct App {
    editor: Editor,
    input_ctx: InputContext,
}

impl App {
    pub fn new(filename: &str) -> Result<Self> {
        let (filepath, content) = file_io::open_or_create(filename)?;
        let editor = Editor::new(filepath, content);
        let input_ctx = InputContext::new();

        Ok(Self { editor, input_ctx })
    }

    pub fn run(&mut self) -> Result<()> {
        // Terminal setup
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        // Always restore terminal, even on error
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(50);
        let mut last_tick = Instant::now();
        // Quit-confirm state: set true when user presses q with unsaved changes.
        let mut confirm_quit = false;

        loop {
            // Draw
            let viewport_height = terminal.size()?.height as usize;
            // Subtract title (1) + status (1) + mode-line (1) = 3
            let editor_height = viewport_height.saturating_sub(3);
            self.editor.adjust_scroll_with_height(editor_height);

            // Sync input_ctx mode
            self.input_ctx.mode = self.editor.mode.clone();

            terminal.draw(|f| {
                renderer::draw(
                    f,
                    &self.editor.buffer,
                    &self.editor.cursor,
                    &self.editor.mode,
                    &self.editor.highlight,
                    &self.editor.status,
                    &self.editor.filepath.display().to_string(),
                    self.editor.modified,
                    self.editor.scroll_offset,
                );
            })?;

            // Poll for events
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout)? {
                let raw_event = event::read()?;

                // If we're waiting for quit confirmation, intercept keys
                if confirm_quit {
                    if let Event::Key(k) = &raw_event {
                        match k.code {
                            crossterm::event::KeyCode::Char('y')
                            | crossterm::event::KeyCode::Char('Y') => break, // confirmed quit
                            crossterm::event::KeyCode::Char('s')
                            | crossterm::event::KeyCode::Char('S') => {
                                // Save then quit
                                self.editor.apply(Action::Save);
                                break;
                            }
                            _ => {
                                // Cancel quit
                                confirm_quit = false;
                                self.editor
                                    .status
                                    .set("Quit cancelled", std::time::Duration::from_secs(2));
                            }
                        }
                        continue;
                    }
                }

                let action = input::handle_event(raw_event, &mut self.input_ctx);

                match self.editor.apply(action) {
                    EditorResult::Continue => {}
                    EditorResult::Quit { modified } => {
                        if modified {
                            self.editor.status.set_persistent(
                                "Unsaved changes! Press y=quit, s=save+quit, any other=cancel",
                            );
                            confirm_quit = true;
                        } else {
                            break;
                        }
                    }
                    EditorResult::ForceQuit => break,
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.editor.status.tick();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }
}
