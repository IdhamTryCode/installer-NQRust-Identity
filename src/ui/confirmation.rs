use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::MenuSelection;
use crate::ui::{ASCII_HEADER, get_orange_accent, get_orange_color};

pub struct ConfirmationView<'a> {
    pub cert_exists: bool,
    pub env_has_ip: bool,
    pub menu_selection: &'a MenuSelection,
    pub menu_options: &'a [MenuSelection],
    /// True when running as airgapped binary (offline mode)
    pub airgapped: bool,
}

pub fn render_confirmation(frame: &mut Frame, view: &ConfirmationView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7), // ASCII header
            Constraint::Min(10),   // status / checklist
            Constraint::Length(6), // menu
            Constraint::Length(2), // help
        ])
        .split(area);

    // ── ASCII header ───────────────────────────────────────────────────────
    let header_lines: Vec<Line> = ASCII_HEADER
        .trim()
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let header = Paragraph::new(header_lines)
        .block(Block::default().borders(Borders::NONE))
        .centered();
    frame.render_widget(header, chunks[0]);

    // ── Status / Checklist ─────────────────────────────────────────────────
    let all_ready = view.cert_exists && view.env_has_ip;

    let mut content_lines = vec![Line::from("")];

    if view.airgapped {
        content_lines.push(Line::from(Span::styled(
            "🔒 Offline / Airgapped mode — images from embedded payload only",
            Style::default().fg(Color::Cyan),
        )));
        content_lines.push(Line::from(""));
    }

    content_lines.push(Line::from(Span::styled(
        "Setup Checklist:",
        Style::default().fg(if all_ready {
            Color::Green
        } else {
            Color::Yellow
        }),
    )));
    content_lines.push(Line::from(""));

    // SSL Cert row
    let cert_icon = if view.cert_exists { "✓" } else { "✗" };
    let cert_color = if view.cert_exists {
        Color::Green
    } else {
        Color::Red
    };
    content_lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(cert_icon, Style::default().fg(cert_color)),
        Span::raw("  SSL Certificate  "),
        Span::styled(
            if view.cert_exists {
                "(certs/server.crt + server.key)"
            } else {
                "(missing — generate below)"
            },
            Style::default().fg(if view.cert_exists {
                Color::DarkGray
            } else {
                Color::Red
            }),
        ),
    ]));

    // SERVER_IP row
    let ip_icon = if view.env_has_ip { "✓" } else { "✗" };
    let ip_color = if view.env_has_ip {
        Color::Green
    } else {
        Color::Red
    };
    content_lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(ip_icon, Style::default().fg(ip_color)),
        Span::raw("  SERVER_IP         "),
        Span::styled(
            if view.env_has_ip {
                "(set in .env)"
            } else {
                "(missing — generate below)"
            },
            Style::default().fg(if view.env_has_ip {
                Color::DarkGray
            } else {
                Color::Red
            }),
        ),
    ]));

    content_lines.push(Line::from(""));

    if all_ready {
        content_lines.push(Line::from(Span::styled(
            "✅ All requirements met — ready to install!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        content_lines.push(Line::from(""));
        content_lines.push(Line::from("Services to be started:"));
        content_lines.push(Line::from(Span::styled(
            "  • identity-db     (PostgreSQL 16 — port 5436)",
            Style::default().fg(Color::White),
        )));
        content_lines.push(Line::from(Span::styled(
            "  • identity        (Keycloak — port 8008)",
            Style::default().fg(Color::White),
        )));
        content_lines.push(Line::from(Span::styled(
            "  • identity-caddy  (HTTPS proxy — port 8008)",
            Style::default().fg(Color::White),
        )));
    } else {
        content_lines.push(Line::from(Span::styled(
            "⚠️  Some requirements are missing.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        content_lines.push(Line::from(
            "Generate the SSL cert & .env before proceeding.",
        ));
    }

    let content = Paragraph::new(content_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title(" Status ")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .centered();
    frame.render_widget(content, chunks[1]);

    // ── Menu ───────────────────────────────────────────────────────────────
    let mut menu_lines = vec![Line::from("")];

    for option in view.menu_options {
        let (label, fg_color, highlight_color) = match option {
            MenuSelection::GenerateSsl => (
                "Generate SSL Cert & write .env",
                get_orange_color(),
                get_orange_color(),
            ),
            MenuSelection::CheckUpdates => ("Check for updates", Color::Cyan, Color::Cyan),
            MenuSelection::UpdateToken => ("Update GHCR token", Color::Yellow, Color::Yellow),
            MenuSelection::Proceed => ("Proceed with installation", Color::Green, Color::Green),
            MenuSelection::Cancel => ("Cancel", Color::Red, Color::Red),
        };

        let style = if option == view.menu_selection {
            Style::default()
                .fg(Color::Black)
                .bg(highlight_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color)
        };

        menu_lines.push(Line::from(Span::styled(format!("  ▶  {}", label), style)));
    }

    let menu = Paragraph::new(menu_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title(" Menu ")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .centered();
    frame.render_widget(menu, chunks[2]);

    let help = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[3]);
}
