use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::local_llm_form_data::{FocusState, LocalLlmFormData};
use crate::ui::{get_orange_accent, get_orange_color};

pub struct LocalLlmConfigView<'a> {
    pub form_data: &'a LocalLlmFormData,
}

pub fn render_local_llm_config(frame: &mut Frame, view: &LocalLlmConfigView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let title = Paragraph::new("🔧 Local LLM Configuration")
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

    // Form fields
    let mut form_lines = Vec::new();
    let total_fields = view.form_data.get_total_fields();

    for i in 0..total_fields {
        let is_focused = matches!(&view.form_data.focus_state, FocusState::Field(idx) if *idx == i);
        let field_name = view.form_data.get_field_name(i);
        
        let value = match i {
            0 => &view.form_data.llm_model,
            1 => &view.form_data.llm_api_base,
            2 => &view.form_data.max_tokens,
            3 => &view.form_data.embedding_model,
            4 => &view.form_data.embedding_api_base,
            5 => &view.form_data.embedding_dim,
            _ => "",
        };

        let label_style = if is_focused {
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let value_style = if is_focused {
            Style::default()
                .fg(Color::Black)
                .bg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let cursor = if is_focused { "▶" } else { " " };

        form_lines.push(Line::from(vec![
            Span::styled(cursor, label_style),
            Span::raw(" "),
            Span::styled(format!("{:.<25}", field_name), label_style),
            Span::raw(" "),
            Span::styled(
                if value.is_empty() {
                    "<empty>".to_string()
                } else {
                    value.to_string()
                },
                value_style,
            ),
        ]));

        form_lines.push(Line::from(""));
    }

    let form = Paragraph::new(form_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Configuration Fields")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(form, chunks[1]);

    // Help text
    let mut help_lines = vec![];
    
    // Show error message first if present
    if !view.form_data.error_message.is_empty() {
        help_lines.push(Line::from(vec![
            Span::styled("❌ ERROR: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(&view.form_data.error_message, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]));
        help_lines.push(Line::from(""));
    }
    
    help_lines.push(Line::from(vec![
        Span::styled("Navigation: ", Style::default().fg(Color::Yellow)),
        Span::raw("↑↓ or Tab to move | Type to edit"),
    ]));

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Help")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(help, chunks[2]);

    // Buttons
    let save_focused = matches!(&view.form_data.focus_state, FocusState::SaveButton);
    let cancel_focused = matches!(&view.form_data.focus_state, FocusState::CancelButton);

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
        Span::styled(" Save ", save_style),
        Span::raw("  "),
        Span::styled(" Cancel ", cancel_style),
    ]);

    let buttons = Paragraph::new(button_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Actions")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .centered();
    frame.render_widget(buttons, chunks[3]);
}
