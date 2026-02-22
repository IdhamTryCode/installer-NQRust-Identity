use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::form_data::{FocusState, FormData};
use crate::ui::{get_orange_accent, get_orange_color};

pub struct EnvSetupView<'a> {
    pub form_data: &'a FormData,
}

pub fn render_env_setup(frame: &mut Frame, view: &EnvSetupView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    let data = view.form_data;

    let title_text = if !data.selected_provider.is_empty() {
        format!("🔧 Generate .env File - {}", data.get_api_key_name())
    } else {
        "🔧 Generate .env File".to_string()
    };

    let title = Paragraph::new(title_text)
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
    frame.render_widget(title, chunks[0]);

    let mut form_lines = vec![];

    let needs_openai = data.needs_openai_embedding();

    // Field 0: Provider API Key
    let is_field0_focused = matches!(&data.focus_state, FocusState::Field(0));

    let field0_style = if is_field0_focused {
        Style::default()
            .fg(Color::Black)
            .bg(get_orange_color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let api_key_name = data.get_api_key_name();
    let key_display = if data.api_key.is_empty() {
        "<empty>".to_string()
    } else {
        let masked = if data.api_key.len() > 8 {
            format!(
                "{}...{}",
                &data.api_key[..4],
                &data.api_key[data.api_key.len() - 4..]
            )
        } else {
            "*".repeat(data.api_key.len())
        };
        masked
    };

    let cursor0 = if is_field0_focused { "▶" } else { " " };

    form_lines.push(Line::from(vec![
        Span::styled(cursor0, field0_style),
        Span::raw(" "),
        Span::styled(format!("{} API Key", api_key_name), field0_style),
        Span::raw(": "),
        Span::styled(key_display, field0_style),
    ]));
    form_lines.push(Line::from(""));

    // Field 1: OpenAI API Key (if needed for embedding)
    if needs_openai {
        let is_field1_focused = matches!(&data.focus_state, FocusState::Field(1));

        let field1_style = if is_field1_focused {
            Style::default()
                .fg(Color::Black)
                .bg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let openai_key_display = if data.openai_api_key.is_empty() {
            "<empty>".to_string()
        } else {
            let masked = if data.openai_api_key.len() > 8 {
                format!(
                    "{}...{}",
                    &data.openai_api_key[..4],
                    &data.openai_api_key[data.openai_api_key.len() - 4..]
                )
            } else {
                "*".repeat(data.openai_api_key.len())
            };
            masked
        };

        let cursor1 = if is_field1_focused { "▶" } else { " " };

        form_lines.push(Line::from(vec![
            Span::styled(cursor1, field1_style),
            Span::raw(" "),
            Span::styled("OpenAI API Key (embedding)", field1_style),
            Span::raw(": "),
            Span::styled(openai_key_display, field1_style),
        ]));
        form_lines.push(Line::from(""));
        form_lines.push(Line::from(Span::styled(
            "ℹ️  This provider uses OpenAI embedding model",
            Style::default().fg(Color::Yellow),
        )));
        form_lines.push(Line::from(""));
    }

    if data.selected_provider == "lm_studio" || data.selected_provider == "ollama" {
        form_lines.push(Line::from(Span::styled(
            "ℹ️  No API key required for local services",
            Style::default().fg(Color::Yellow),
        )));
        form_lines.push(Line::from(""));
    }

    if !data.error_message.is_empty() {
        form_lines.push(Line::from(Span::styled(
            &data.error_message,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        form_lines.push(Line::from(""));
    }

    let form = Paragraph::new(form_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(get_orange_accent()))
            .title("API Keys")
            .title_style(
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ),
    );
    frame.render_widget(form, chunks[1]);

    // Buttons
    let save_focused = matches!(&data.focus_state, FocusState::SaveButton);
    let cancel_focused = matches!(&data.focus_state, FocusState::CancelButton);

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
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let button_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(" Save ", save_style),
        Span::raw("  "),
        Span::styled(" Cancel ", cancel_style),
        Span::raw("  "),
        Span::styled("↑↓ Tab to navigate", Style::default().fg(Color::DarkGray)),
    ]);

    let buttons = Paragraph::new(button_line).centered();
    frame.render_widget(buttons, chunks[2]);
}
