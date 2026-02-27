use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::state::SslSetupMenuSelection;
use crate::ui::{get_orange_accent, get_orange_color};

pub struct SslSetupView<'a> {
    pub detected_ip: &'a str,
    pub cert_exists: bool,
    pub env_has_ip: bool,
    pub menu_selection: &'a SslSetupMenuSelection,
    pub status: Option<&'a str>,
}

pub fn render_ssl_setup(frame: &mut Frame, view: &SslSetupView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // title
            Constraint::Length(8),  // info block
            Constraint::Length(3),  // status / spacer
            Constraint::Min(3),     // menu
        ])
        .split(area);

    // ── Title ──────────────────────────────────────────────────────────────
    let title = Paragraph::new("🔐  SSL Certificate Setup")
        .style(
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent())),
        );
    frame.render_widget(title, chunks[0]);

    // ── Info Block ─────────────────────────────────────────────────────────
    let cert_icon = if view.cert_exists { "✅" } else { "⚠️ " };
    let cert_label = if view.cert_exists {
        "SSL cert found (certs/server.crt + server.key)"
    } else {
        "SSL cert NOT found — will be generated"
    };

    let env_icon = if view.env_has_ip { "✅" } else { "⚠️ " };
    let env_label = if view.env_has_ip {
        "SERVER_IP already set in .env"
    } else {
        "SERVER_IP not set — will be written to .env"
    };

    let info_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  🌐  Detected IP   : "),
            Span::styled(
                view.detected_ip,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("  {}  ", cert_icon)),
            Span::styled(cert_label, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::raw(format!("  {}  ", env_icon)),
            Span::styled(env_label, Style::default().fg(Color::White)),
        ]),
    ];

    let info = Paragraph::new(info_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(get_orange_accent()))
            .title(" Status ")
            .title_style(
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ),
    );
    frame.render_widget(info, chunks[1]);

    // ── Status line ────────────────────────────────────────────────────────
    if let Some(status) = view.status {
        let status_widget = Paragraph::new(status)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(status_widget, chunks[2]);
    }

    // ── Menu ───────────────────────────────────────────────────────────────
    let make_item = |label: &str, selected: bool| -> Line<'static> {
        let label = label.to_string();
        if selected {
            Line::from(Span::styled(
                format!("  ▶  {}  ", label),
                Style::default()
                    .fg(Color::Black)
                    .bg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                format!("     {}  ", label),
                Style::default().fg(Color::White),
            ))
        }
    };

    let menu_lines = vec![
        make_item(
            "Generate SSL Cert & Write .env",
            view.menu_selection == &SslSetupMenuSelection::Generate,
        ),
        make_item(
            "Skip (use existing / no SSL)",
            view.menu_selection == &SslSetupMenuSelection::Skip,
        ),
        make_item("Cancel", view.menu_selection == &SslSetupMenuSelection::Cancel),
        Line::from(""),
        Line::from(Span::styled(
            "  ↑↓ to move   Enter to select   Ctrl+C to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let menu = Paragraph::new(menu_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(get_orange_accent()))
            .title(" Action ")
            .title_style(
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ),
    );
    frame.render_widget(menu, chunks[3]);
}
