use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use super::{buffer::Buffer, cursor::Cursor, highlight::Highlight, mode::Mode, status::StatusBar};

/// Draw the entire UI.
pub fn draw(
    frame: &mut Frame,
    buffer: &Buffer,
    cursor: &Cursor,
    mode: &Mode,
    highlight: &Highlight,
    status: &StatusBar,
    filename: &str,
    modified: bool,
    scroll_offset: usize,
) {
    let size = frame.area();

    // Layout: title bar | editor | status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(1),    // editor body
            Constraint::Length(1), // status / keybindings
            Constraint::Length(1), // mode line
        ])
        .split(size);

    draw_title(frame, chunks[0], filename, modified, mode);
    draw_editor(frame, chunks[1], buffer, cursor, highlight, scroll_offset);
    draw_status(frame, chunks[2], status);
    draw_mode_line(frame, chunks[3], mode, cursor);
}

fn draw_title(frame: &mut Frame, area: Rect, filename: &str, modified: bool, mode: &Mode) {
    let mod_indicator = if modified { " [+]" } else { "" };
    let title = format!(" Scribble  {}{}  ", filename, mod_indicator);

    let mode_hint = match mode {
        Mode::Normal => "  SPACE=Insert  q=Quit  s=Save  ?=Help",
        Mode::Insert => "  ESC=Normal Mode",
    };

    let line = Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            mode_hint,
            Style::default().fg(Color::DarkGray).bg(Color::Black),
        ),
    ]);
    let para = Paragraph::new(line);
    frame.render_widget(para, area);
}

fn draw_editor(
    frame: &mut Frame,
    area: Rect,
    buffer: &Buffer,
    cursor: &Cursor,
    highlight: &Highlight,
    scroll_offset: usize,
) {
    let visible_height = area.height as usize;

    let mut lines: Vec<Line> = Vec::with_capacity(visible_height);

    for row_idx in scroll_offset..(scroll_offset + visible_height) {
        if row_idx >= buffer.len() {
            // Empty filler line with tilde (nano/vim style)
            lines.push(Line::from(Span::styled(
                "~",
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        let line_str = &buffer.lines[row_idx];
        let graphemes: Vec<&str> = line_str.graphemes(true).collect();

        if graphemes.is_empty() {
            lines.push(Line::from(Span::raw(" ")));
            continue;
        }

        let mut spans: Vec<Span> = Vec::new();

        for (col_idx, g) in graphemes.iter().enumerate() {
            let is_highlighted = highlight.contains(row_idx, col_idx);
            let style = if is_highlighted {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(g.to_string(), style));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));

    frame.render_widget(para, area);

    // Position the terminal cursor
    let screen_row = (cursor.row.saturating_sub(scroll_offset)) as u16 + area.y;
    let screen_col = cursor.col as u16 + area.x;
    if screen_row < area.y + area.height && screen_col < area.x + area.width {
        frame.set_cursor_position((screen_col, screen_row));
    }
}

fn draw_status(frame: &mut Frame, area: Rect, status: &StatusBar) {
    let msg = status.current();
    let style = if msg.starts_with("Error") || msg.starts_with("Cannot") {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let para = Paragraph::new(Span::styled(format!(" {}", msg), style));
    frame.render_widget(para, area);
}

fn draw_mode_line(frame: &mut Frame, area: Rect, mode: &Mode, cursor: &Cursor) {
    let mode_str = mode.name();
    let pos_str = format!("  Ln {}, Col {}  ", cursor.row + 1, cursor.col + 1);

    let (mode_fg, mode_bg) = match mode {
        Mode::Normal => (Color::Black, Color::Green),
        Mode::Insert => (Color::Black, Color::Yellow),
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", mode_str),
            Style::default()
                .fg(mode_fg)
                .bg(mode_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(pos_str, Style::default().fg(Color::DarkGray)),
    ]);
    let para = Paragraph::new(line);
    frame.render_widget(para, area);
}
