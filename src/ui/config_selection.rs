use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::templates::ConfigTemplate;
use crate::ui::{get_orange_accent, get_orange_color};

pub struct ConfigSelectionView<'a> {
    pub templates: &'a [ConfigTemplate],
    pub selected_index: usize,
}

pub fn render_config_selection(frame: &mut Frame, view: &ConfigSelectionView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(area);

    // Header
    let title = Paragraph::new("🧩 Choose a configuration template")
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

    // Grid layout for providers
    let grid_area = chunks[1];

    // Calculate grid dimensions
    let cols = 4; // 4 columns
    let total_items = view.templates.len();
    let _rows = (total_items + cols - 1) / cols; // Ceiling division

    // Calculate card dimensions
    let card_width = (grid_area.width.saturating_sub(2)) / cols as u16; // -2 for borders
    let card_height = 3; // Fixed height for each card

    // Render grid
    let grid_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(get_orange_accent()))
        .title("Model Providers")
        .title_style(
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(grid_block, grid_area);

    // Inner area for cards (inside the border)
    let inner_area = Rect {
        x: grid_area.x + 1,
        y: grid_area.y + 1,
        width: grid_area.width.saturating_sub(2),
        height: grid_area.height.saturating_sub(2),
    };

    // Render each card
    for (index, template) in view.templates.iter().enumerate() {
        let row = index / cols;
        let col = index % cols;

        let x = inner_area.x + (col as u16 * card_width);
        let y = inner_area.y + (row as u16 * card_height);

        // Skip if card would be outside visible area
        if y + card_height > inner_area.y + inner_area.height {
            continue;
        }

        let card_area = Rect {
            x,
            y,
            width: card_width.saturating_sub(1), // -1 for spacing
            height: card_height,
        };

        let is_selected = index == view.selected_index;

        let card_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(get_orange_color())
        };

        let border_style = if is_selected {
            Style::default().fg(get_orange_color())
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Truncate name if too long
        let max_name_len = card_width.saturating_sub(4) as usize;
        let display_name = if template.name.len() > max_name_len {
            format!("{}…", &template.name[..max_name_len.saturating_sub(1)])
        } else {
            template.name.to_string()
        };

        let card = Paragraph::new(display_name)
            .style(card_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .centered();

        frame.render_widget(card, card_area);
    }

    // Details panel
    let detail_lines = if let Some(template) = view.templates.get(view.selected_index) {
        vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    template.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Yellow)),
                Span::styled(template.description, Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().fg(Color::Yellow)),
                Span::raw("←→↑↓ to move | "),
                Span::styled("Enter", Style::default().fg(get_orange_color())),
                Span::raw(" to select | "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(" to go back"),
            ]),
        ]
    } else {
        vec![
            Line::from("No templates available"),
            Line::from(""),
            Line::from("Press Esc to go back, Ctrl+C to exit"),
        ]
    };

    let details = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Details")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(details, chunks[2]);
}
