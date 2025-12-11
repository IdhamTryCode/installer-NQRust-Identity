use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::registry_form::RegistryForm;
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
            Constraint::Length(5),
            Constraint::Length(7),
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

    let fields = [
        ("GitHub username", view.form.username.as_str(), false),
        ("Personal access token", &"*".repeat(view.form.token.chars().count()), true),
    ];

    let mut field_lines = Vec::new();
    field_lines.push(Line::from("Please supply credentials with `read:packages` scope."));
    field_lines.push(Line::from("Use ↑/↓ to navigate, Enter to edit, Ctrl+S or click Submit."));
    field_lines.push(Line::from("Press Esc to skip (you can authenticate later)."));
    field_lines.push(Line::from(""));

    for (index, (label, value, masked)) in fields.iter().enumerate() {
        let is_selected = view.form.current_field == index;
        let display = if *masked && value.is_empty() {
            "<hidden>"
        } else if value.is_empty() {
            "<empty>"
        } else {
            value
        };

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        field_lines.push(Line::from(vec![
            Span::styled("  ▶  ", style),
            Span::styled(*label, style),
            Span::raw(": "),
            Span::styled(display, style),
        ]));
    }

    let submit_style = if view.form.current_field == 2 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    field_lines.push(Line::from(""));
    field_lines.push(Line::from(Span::styled("  ▶  Submit and login", submit_style)));

    let form_block = Paragraph::new(field_lines)
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

    let help = Paragraph::new("Enter credentials to pull GHCR images. Esc to skip.")
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[3]);
}
