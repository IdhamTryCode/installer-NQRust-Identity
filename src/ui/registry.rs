use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::registry_form::{FocusState, RegistryForm};
use crate::ui::{get_orange_accent, get_orange_color};

pub struct RegistrySetupView<'a> {
    pub form: &'a RegistryForm,
    pub status: Option<&'a str>,
}

pub fn render_registry_setup(frame: &mut Frame, view: &RegistrySetupView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(area);

    let header = Paragraph::new("🔐 GitHub Container Registry Login")
        .style(
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent())),
        )
        .centered();
    frame.render_widget(header, chunks[0]);

    // Token field
    let is_field_focused = matches!(&view.form.focus_state, FocusState::Field(_));
    let raw_value = view.form.token.as_str();

    let display = if raw_value.is_empty() {
        "<paste token here>".to_string()
    } else {
        "*".repeat(raw_value.chars().count())
    };

    let field_style = if is_field_focused {
        Style::default()
            .fg(Color::Black)
            .bg(get_orange_color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let cursor = if is_field_focused { "▶" } else { " " };

    let field_line = Line::from(vec![
        Span::styled(cursor, field_style),
        Span::raw(" "),
        Span::styled("Personal access token: ", field_style),
        Span::styled(display, field_style),
    ]);

    let form_block = Paragraph::new(vec![
        Line::from("Provide a GitHub token with `read:packages` scope."),
        Line::from(""),
        field_line,
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(get_orange_accent()))
            .title("Credentials")
            .title_style(
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ),
    )
    .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(form_block, chunks[1]);

    // Status
    let status_message = if let Some(message) = view.status {
        message.to_string()
    } else if !view.form.error_message.is_empty() {
        view.form.error_message.clone()
    } else {
        "Awaiting input...".to_string()
    };

    let status_style = if status_message.contains("success") {
        Style::default().fg(Color::Green)
    } else if status_message.contains("failed") || status_message.contains("error") {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let status_block = Paragraph::new(status_message)
        .style(status_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Status")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(status_block, chunks[2]);

    // Buttons
    let save_focused = matches!(&view.form.focus_state, FocusState::SaveButton);
    let cancel_focused = matches!(&view.form.focus_state, FocusState::CancelButton);

    let save_style = if save_focused {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    };

    let cancel_style = if cancel_focused {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD)
    };

    let button_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(" Submit ", save_style),
        Span::raw("  "),
        Span::styled(" Skip ", cancel_style),
    ]);

    let buttons = Paragraph::new(button_line).centered();
    frame.render_widget(buttons, chunks[3]);
}
