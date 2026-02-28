use color_eyre::{Result, eyre::eyre};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use rcgen::{Certificate, CertificateParams, SanType};
use reqwest::Client;
use std::net::IpAddr as StdIpAddr;
use std::process::Stdio;
use std::{env, fs};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::app::state::SslSetupMenuSelection;
use crate::ui::{
    self, ConfirmationView, ErrorView, InstallingView, RegistrySetupView, SslSetupView,
    SuccessView, UpdateListView,
};
use crate::utils;

pub mod form_data;
pub mod local_llm_form_data;
pub mod registry_form;
pub mod state;
mod updates;

use registry_form::RegistryForm;
pub use state::{AppState, MenuSelection};
pub use updates::UpdateInfo;
use updates::{collect_update_infos, fetch_latest_identity_tag, get_local_image_created};

enum UpdateListAction {
    Pull,
    Refresh,
    Back,
}

enum RegistryAction {
    Submit,
    Skip,
}

#[derive(Debug)]
pub struct App {
    running: bool,
    pub(crate) state: AppState,
    logs: Vec<String>,
    progress: f64,
    current_service: String,
    total_services: usize,
    completed_services: usize,
    pub(crate) cert_exists: bool,
    pub(crate) env_has_ip: bool,
    pub(crate) menu_selection: MenuSelection,
    update_infos: Vec<UpdateInfo>,
    update_selection_index: usize,
    update_message: Option<String>,
    registry_form: RegistryForm,
    registry_status: Option<String>,
    ghcr_token: Option<String>,
    /// True when running as nqrust-identity-airgapped (offline mode, no image pull)
    pub(crate) airgapped: bool,
    // SSL setup screen state
    pub(crate) ssl_detected_ip: String,
    pub(crate) ssl_menu_selection: SslSetupMenuSelection,
    pub(crate) ssl_status: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let token_from_env = env::var("GHCR_TOKEN")
            .or_else(|_| env::var("GITHUB_TOKEN"))
            .or_else(|_| env::var("GH_TOKEN"))
            .ok();
        let token_from_disk = App::load_token_from_disk();
        let initial_token = token_from_env.clone().or(token_from_disk.clone());

        let mut registry_form = RegistryForm::new();
        if let Some(token) = initial_token.clone() {
            registry_form.token = token;
        }

        let airgapped = crate::airgapped::is_airgapped_binary().unwrap_or(false);

        // Detect IP for SSL setup
        let ssl_detected_ip = App::detect_ip();

        // Check file status for checklist
        let root = utils::project_root();
        let cert_exists =
            root.join("certs/server.crt").exists() && root.join("certs/server.key").exists();
        let env_has_ip = fs::read_to_string(root.join(".env"))
            .map(|c| c.lines().any(|l| l.starts_with("SERVER_IP=")))
            .unwrap_or(false);

        // Always start at Confirmation (or RegistrySetup if no token)
        let initial_state = if initial_token.is_some() || airgapped {
            AppState::Confirmation
        } else {
            AppState::RegistrySetup
        };

        let mut app = Self {
            running: true,
            state: initial_state,
            logs: Vec::new(),
            progress: 0.0,
            current_service: String::new(),
            // Identity stack: identity-db + identity + identity-caddy
            total_services: 3,
            completed_services: 0,
            cert_exists,
            env_has_ip,
            menu_selection: MenuSelection::Proceed,
            update_infos: Vec::new(),
            update_selection_index: 0,
            update_message: None,
            registry_form,
            registry_status: None,
            ghcr_token: initial_token,
            airgapped,
            ssl_detected_ip,
            ssl_menu_selection: SslSetupMenuSelection::Generate,
            ssl_status: None,
        };

        app.ensure_menu_selection();
        app
    }

    /// Build the adaptive menu based on current file status.
    fn menu_options(&self) -> Vec<MenuSelection> {
        let mut options = Vec::new();

        // If cert or SERVER_IP is missing, show generate option
        if !self.cert_exists || !self.env_has_ip {
            options.push(MenuSelection::GenerateSsl);
        }

        if !self.airgapped {
            if self.ghcr_token.is_some() {
                options.push(MenuSelection::UpdateToken);
            }
            options.push(MenuSelection::CheckUpdates);
        }

        // Proceed only available when cert + SERVER_IP are both ready
        if self.cert_exists && self.env_has_ip {
            options.push(MenuSelection::Proceed);
        }

        options.push(MenuSelection::Cancel);
        options
    }

    /// Ensure current menu_selection is valid for current state.
    /// Prefers Proceed if available (so after cert generation the cursor lands there),
    /// otherwise falls back to the first available option.
    fn ensure_menu_selection(&mut self) {
        let options = self.menu_options();
        if !options.contains(&self.menu_selection) {
            // Prefer Proceed > GenerateSsl > first option
            if options.contains(&MenuSelection::Proceed) {
                self.menu_selection = MenuSelection::Proceed;
            } else if options.contains(&MenuSelection::GenerateSsl) {
                self.menu_selection = MenuSelection::GenerateSsl;
            } else if let Some(first) = options.first() {
                self.menu_selection = first.clone();
            }
        }
    }

    /// Detect the VM's outbound IP by opening a UDP-like socket toward 8.8.8.8.
    /// Falls back to 127.0.0.1 if detection fails.
    fn detect_ip() -> String {
        use std::net::UdpSocket;
        UdpSocket::bind("0.0.0.0:0")
            .and_then(|s| {
                s.connect("8.8.8.8:80")?;
                s.local_addr()
            })
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Generate a self-signed TLS cert using rcgen (no openssl required).
    /// Writes certs/server.crt and certs/server.key, then updates SERVER_IP in .env.
    fn generate_ssl_cert(ip: &str) -> Result<()> {
        let root = utils::project_root();
        let certs_dir = root.join("certs");
        fs::create_dir_all(&certs_dir)?;

        let ip_addr: StdIpAddr = ip
            .parse()
            .map_err(|_| eyre!("Invalid IP address: {}", ip))?;

        let mut params = CertificateParams::default();
        params.subject_alt_names = vec![SanType::IpAddress(ip_addr)];
        // Valid for ~100 years
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2124, 1, 1);

        let cert = Certificate::from_params(params).map_err(|e| eyre!("rcgen cert error: {e}"))?;

        let cert_pem = cert
            .serialize_pem()
            .map_err(|e| eyre!("cert serialize error: {e}"))?;
        let key_pem = cert.serialize_private_key_pem();

        fs::write(certs_dir.join("server.crt"), cert_pem)?;
        fs::write(certs_dir.join("server.key"), key_pem)?;

        // Write SERVER_IP to .env
        App::write_server_ip_to_env(ip)?;

        Ok(())
    }

    /// Upsert SERVER_IP=<ip> in .env (create file if missing).
    fn write_server_ip_to_env(ip: &str) -> Result<()> {
        let root = utils::project_root();
        let env_path = root.join(".env");
        let entry = format!("SERVER_IP={}", ip);

        let existing = fs::read_to_string(&env_path).unwrap_or_default();
        let has_entry = existing.lines().any(|l| l.starts_with("SERVER_IP="));

        let new_content = if has_entry {
            existing
                .lines()
                .map(|l| {
                    if l.starts_with("SERVER_IP=") {
                        entry.as_str()
                    } else {
                        l
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        } else {
            format!("{}{}", existing, format!("{}\n", entry))
        };

        fs::write(&env_path, new_content)?;
        Ok(())
    }

    fn load_token_from_disk() -> Option<String> {
        let token_path = utils::project_root().join(".ghcr_token");
        fs::read_to_string(&token_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn save_token_to_disk(token: &str) {
        let token_path = utils::project_root().join(".ghcr_token");
        let _ = fs::write(&token_path, token);
    }

    fn add_log(&mut self, message: &str) {
        self.logs.push(message.to_string());
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            match &self.state.clone() {
                AppState::SslSetup => {
                    if let Some(action) = self.handle_ssl_setup_events()? {
                        match action {
                            SslSetupMenuSelection::Generate => {
                                self.ssl_status = Some("⏳ Generating SSL cert...".to_string());
                                terminal.draw(|frame| self.render(frame))?;
                                let ip = self.ssl_detected_ip.clone();
                                match App::generate_ssl_cert(&ip) {
                                    Ok(()) => {
                                        self.ssl_status = None;
                                        // Update checklist state
                                        self.cert_exists = true;
                                        self.env_has_ip = true;
                                        self.state = AppState::Confirmation;
                                        self.ensure_menu_selection();
                                    }
                                    Err(e) => {
                                        self.ssl_status = None;
                                        self.state = AppState::Error(format!(
                                            "SSL cert generation failed: {e}"
                                        ));
                                    }
                                }
                            }
                            SslSetupMenuSelection::Skip => {
                                self.state = AppState::Confirmation;
                                self.ensure_menu_selection();
                            }
                            SslSetupMenuSelection::Cancel => {
                                self.running = false;
                            }
                        }
                    }
                }

                AppState::RegistrySetup => {
                    if let Some(action) = self.handle_registry_events()? {
                        match action {
                            RegistryAction::Submit => {
                                let token = self.registry_form.token.trim().to_string();
                                if token.is_empty() {
                                    self.registry_form.error_message =
                                        "Token cannot be empty. Press Esc to skip.".to_string();
                                } else {
                                    // Validate token by running docker login
                                    self.registry_status =
                                        Some("🔐 Validating token...".to_string());
                                    terminal.draw(|frame| self.render(frame))?;
                                    match self.login_to_ghcr(&token).await {
                                        Ok(()) => {
                                            Self::save_token_to_disk(&token);
                                            self.ghcr_token = Some(token);
                                            self.registry_status = None;
                                            self.registry_form.error_message.clear();
                                            self.state = AppState::Confirmation;
                                            self.ensure_menu_selection();
                                        }
                                        Err(e) => {
                                            self.registry_form.error_message = format!(
                                                "❌ Login failed: {}",
                                                e.to_string()
                                                    .lines()
                                                    .next()
                                                    .unwrap_or("unknown error")
                                            );
                                            self.registry_status = None;
                                            // Stay on RegistrySetup
                                        }
                                    }
                                }
                            }
                            RegistryAction::Skip => {
                                self.state = AppState::Confirmation;
                                // Refresh checklist status after returning from registry
                                let root = utils::project_root();
                                self.cert_exists = root.join("certs/server.crt").exists()
                                    && root.join("certs/server.key").exists();
                                self.env_has_ip = fs::read_to_string(root.join(".env"))
                                    .map(|c| c.lines().any(|l| l.starts_with("SERVER_IP=")))
                                    .unwrap_or(false);
                                self.ensure_menu_selection();
                            }
                        }
                    }
                }

                AppState::Confirmation => {
                    if let Some(action) = self.handle_confirmation_events()? {
                        let options = self.menu_options();
                        match action {
                            MenuSelection::GenerateSsl => {
                                self.ssl_menu_selection = SslSetupMenuSelection::Generate;
                                self.ssl_status = None;
                                self.state = AppState::SslSetup;
                            }
                            MenuSelection::Proceed => {
                                // Only reachable when cert_exists && env_has_ip
                                let root = utils::project_root();
                                if let Err(e) = utils::ensure_compose_bundle(&root) {
                                    self.state = AppState::Error(format!(
                                        "Failed to write compose file: {e}"
                                    ));
                                } else {
                                    self.state = AppState::Installing;
                                    self.logs.clear();
                                    terminal.draw(|frame| self.render(frame))?;
                                    if let Err(e) = self.run_docker_compose(terminal).await {
                                        self.state =
                                            AppState::Error(format!("Installation failed: {e}"));
                                    }
                                }
                            }
                            MenuSelection::CheckUpdates => {
                                self.state = AppState::UpdateList;
                                self.update_message = Some("Fetching update info...".to_string());
                                let client = Client::new();
                                match collect_update_infos(&client, self.ghcr_token.as_deref())
                                    .await
                                {
                                    Ok(infos) => {
                                        for mut info in infos {
                                            match get_local_image_created(
                                                &info.image,
                                                &info.current_tag,
                                            )
                                            .await
                                            {
                                                Ok(created) => info.apply_local_created(created),
                                                Err(_) => info.apply_local_created(None),
                                            }
                                            self.update_infos.push(info);
                                        }
                                        self.update_message = None;
                                    }
                                    Err(e) => {
                                        self.update_message = Some(format!("Error: {e}"));
                                    }
                                }
                            }
                            MenuSelection::UpdateToken => {
                                self.registry_form = RegistryForm::new();
                                self.registry_status = None;
                                self.state = AppState::RegistrySetup;
                            }
                            MenuSelection::Cancel => {
                                self.running = false;
                            }
                        }
                        let _ = options;
                    }
                }

                AppState::UpdateList => {
                    if let Some(action) = self.handle_update_list_events()? {
                        match action {
                            UpdateListAction::Back => {
                                self.state = AppState::Confirmation;
                            }
                            UpdateListAction::Refresh => {
                                self.update_infos.clear();
                                self.update_message = Some("Fetching update info...".to_string());
                                let client = Client::new();
                                match collect_update_infos(&client, self.ghcr_token.as_deref())
                                    .await
                                {
                                    Ok(infos) => {
                                        self.update_infos = infos;
                                        self.update_message = None;
                                    }
                                    Err(e) => {
                                        self.update_message = Some(format!("Error: {e}"));
                                    }
                                }
                            }
                            UpdateListAction::Pull => {
                                self.state = AppState::UpdatePulling;
                                if let Err(e) = self.pull_selected_update().await {
                                    self.add_log(&format!("❌ Error: {e}"));
                                    self.state = AppState::UpdateList;
                                } else {
                                    self.state = AppState::UpdateList;
                                }
                            }
                        }
                    }
                }

                AppState::UpdatePulling => {
                    // Rendering only; handled in UpdateList branch above
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }

                AppState::Installing => {
                    // Installing is driven via run_docker_compose above;
                    // just keep rendering while we wait.
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }

                AppState::Success | AppState::Error(_) => {
                    if event::poll(std::time::Duration::from_millis(200))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press
                                && (key.code == KeyCode::Char('q')
                                    || (key.code == KeyCode::Char('c')
                                        && key.modifiers.contains(KeyModifiers::CONTROL)))
                            {
                                self.running = false;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        match &self.state {
            AppState::SslSetup => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let view = SslSetupView {
                    detected_ip: &self.ssl_detected_ip,
                    cert_exists: self.cert_exists,
                    env_has_ip: self.env_has_ip,
                    menu_selection: &self.ssl_menu_selection,
                    status: self.ssl_status.as_deref(),
                };
                ui::render_ssl_setup(frame, &view);
            }
            AppState::RegistrySetup => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let view = RegistrySetupView {
                    form: &self.registry_form,
                    status: self.registry_status.as_deref(),
                };
                ui::render_registry_setup(frame, &view);
            }
            AppState::Confirmation => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let options = self.menu_options();
                let view = ConfirmationView {
                    cert_exists: self.cert_exists,
                    env_has_ip: self.env_has_ip,
                    menu_selection: &self.menu_selection,
                    menu_options: &options,
                    airgapped: self.airgapped,
                };
                ui::render_confirmation(frame, &view);
            }
            AppState::UpdateList | AppState::UpdatePulling => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let view = UpdateListView {
                    updates: &self.update_infos,
                    selected_index: self.update_selection_index,
                    message: self.update_message.as_deref(),
                    logs: &self.logs,
                    pulling: matches!(self.state, AppState::UpdatePulling),
                    progress: None,
                };
                ui::render_update_list(frame, &view);
            }
            AppState::Installing => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let view = InstallingView {
                    progress: self.progress,
                    current_service: &self.current_service,
                    completed_services: self.completed_services,
                    total_services: self.total_services,
                    logs: &self.logs,
                    airgapped: self.airgapped,
                };
                ui::render_installing(frame, &view);
            }
            AppState::Success => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let view = SuccessView { logs: &self.logs };
                ui::render_success(frame, &view);
            }
            AppState::Error(msg) => {
                frame.render_widget(ratatui::widgets::Clear, frame.area());
                let view = ErrorView {
                    error: msg,
                    logs: &self.logs,
                };
                ui::render_error(frame, &view);
            }
        }
    }

    fn handle_ssl_setup_events(&mut self) -> Result<Option<SslSetupMenuSelection>> {
        if !event::poll(std::time::Duration::from_millis(200))? {
            return Ok(None);
        }
        let Event::Key(key) = event::read()? else {
            return Ok(None);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        let options = [
            SslSetupMenuSelection::Generate,
            SslSetupMenuSelection::Skip,
            SslSetupMenuSelection::Cancel,
        ];
        let current_idx = options
            .iter()
            .position(|o| o == &self.ssl_menu_selection)
            .unwrap_or(0);

        match key.code {
            KeyCode::Up => {
                if current_idx > 0 {
                    self.ssl_menu_selection = options[current_idx - 1].clone();
                }
            }
            KeyCode::Down => {
                if current_idx + 1 < options.len() {
                    self.ssl_menu_selection = options[current_idx + 1].clone();
                }
            }
            KeyCode::Enter => {
                return Ok(Some(self.ssl_menu_selection.clone()));
            }
            KeyCode::Esc => {
                return Ok(Some(SslSetupMenuSelection::Skip));
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_registry_events(&mut self) -> Result<Option<RegistryAction>> {
        if !event::poll(std::time::Duration::from_millis(200))? {
            return Ok(None);
        }
        let Event::Key(key) = event::read()? else {
            return Ok(None);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        use crate::app::registry_form::FocusState;

        match key.code {
            // Ctrl+C → exit
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }

            // Esc → skip (always)
            KeyCode::Esc => {
                return Ok(Some(RegistryAction::Skip));
            }

            // Tab / Down → move focus forward (Field → Submit → Cancel → Field)
            KeyCode::Tab | KeyCode::Down => {
                self.registry_form.focus_state = match &self.registry_form.focus_state {
                    FocusState::Field(_) => FocusState::SaveButton,
                    FocusState::SaveButton => FocusState::CancelButton,
                    FocusState::CancelButton => FocusState::Field(0),
                };
            }

            // Shift+Tab / Up → move focus backward
            KeyCode::BackTab | KeyCode::Up => {
                self.registry_form.focus_state = match &self.registry_form.focus_state {
                    FocusState::Field(_) => FocusState::CancelButton,
                    FocusState::SaveButton => FocusState::Field(0),
                    FocusState::CancelButton => FocusState::SaveButton,
                };
            }

            // Enter → action depends on current focus
            KeyCode::Enter => {
                match &self.registry_form.focus_state {
                    FocusState::Field(_) | FocusState::SaveButton => {
                        // Save token and proceed
                        return Ok(Some(RegistryAction::Submit));
                    }
                    FocusState::CancelButton => {
                        // Skip without saving
                        return Ok(Some(RegistryAction::Skip));
                    }
                }
            }

            // Typing only works when field is focused
            KeyCode::Char(c) => {
                if matches!(self.registry_form.focus_state, FocusState::Field(_)) {
                    self.registry_form.token.push(c);
                }
            }
            KeyCode::Backspace => {
                if matches!(self.registry_form.focus_state, FocusState::Field(_)) {
                    self.registry_form.token.pop();
                }
            }

            _ => {}
        }
        Ok(None)
    }

    fn handle_confirmation_events(&mut self) -> Result<Option<MenuSelection>> {
        if !event::poll(std::time::Duration::from_millis(200))? {
            return Ok(None);
        }
        let Event::Key(key) = event::read()? else {
            return Ok(None);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        let options = self.menu_options();
        let current_idx = options
            .iter()
            .position(|o| o == &self.menu_selection)
            .unwrap_or(0);

        match key.code {
            KeyCode::Up => {
                if current_idx > 0 {
                    self.menu_selection = options[current_idx - 1].clone();
                }
            }
            KeyCode::Down => {
                if current_idx + 1 < options.len() {
                    self.menu_selection = options[current_idx + 1].clone();
                }
            }
            KeyCode::Enter => {
                return Ok(Some(self.menu_selection.clone()));
            }
            KeyCode::Esc => {
                return Ok(Some(MenuSelection::Cancel));
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_update_list_events(&mut self) -> Result<Option<UpdateListAction>> {
        if !event::poll(std::time::Duration::from_millis(200))? {
            return Ok(None);
        }
        let Event::Key(key) = event::read()? else {
            return Ok(None);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => return Ok(Some(UpdateListAction::Back)),
            KeyCode::Char('r') => return Ok(Some(UpdateListAction::Refresh)),
            KeyCode::Enter => {
                // Pull the selected image update
                if !self.update_infos.is_empty() {
                    return Ok(Some(UpdateListAction::Pull));
                }
            }
            KeyCode::Up => {
                if self.update_selection_index > 0 {
                    self.update_selection_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.update_selection_index + 1 < self.update_infos.len() {
                    self.update_selection_index += 1;
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            _ => {}
        }
        Ok(None)
    }

    async fn pull_selected_update(&mut self) -> Result<()> {
        let Some(info) = self.update_infos.get(self.update_selection_index).cloned() else {
            return Ok(());
        };

        if info.is_self {
            self.add_log("ℹ️  Self-update: please download the new installer from:");
            self.add_log("    https://github.com/NexusQuantum/installer-NQRust-Identity/releases");
            return Ok(());
        }

        // Use the latest release tag if available (e.g. "v0.0.1"), otherwise fall back
        let tag = info
            .latest_release_tag
            .as_deref()
            .unwrap_or(info.current_tag.as_str());
        let reference = format!("{}:{}", info.image, tag);

        self.add_log(&format!("⬇️  Pulling {}...", reference));

        // Login first if token is available
        if let Some(token) = self.ghcr_token.clone() {
            self.add_log("🔐 Logging into GHCR...");
            if let Err(e) = self.login_to_ghcr(&token).await {
                self.add_log(&format!("⚠️  GHCR login warning: {e}"));
            }
        }

        // Run docker pull and stream output to logs
        let mut child = Command::new("docker")
            .arg("pull")
            .arg(&reference)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Stream stderr (docker pull progress goes to stderr)
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                self.add_log(&line);
            }
        }

        let status = child.wait().await?;
        if status.success() {
            self.add_log(&format!("✅ Successfully pulled {}", reference));
            // Update local_created timestamp in the stored info
            if let Some(stored) = self.update_infos.get_mut(self.update_selection_index) {
                use updates::get_local_image_created;
                if let Ok(created) = get_local_image_created(&info.image, tag).await {
                    stored.apply_local_created(created);
                }
            }
        } else {
            self.add_log(&format!(
                "❌ Failed to pull {} — check token and image name",
                reference
            ));
        }

        Ok(())
    }

    // ─── Docker Compose ────────────────────────────────────────────────────────

    async fn detect_compose_command(&self) -> Result<Vec<String>> {
        // Try `docker compose` (plugin, Docker 20.10+)
        let result = Command::new("docker")
            .args(["compose", "version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        if result.map(|s| s.success()).unwrap_or(false) {
            return Ok(vec!["docker".to_string(), "compose".to_string()]);
        }

        // Fallback to standalone docker-compose
        let result = Command::new("docker-compose")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        if result.map(|s| s.success()).unwrap_or(false) {
            return Ok(vec!["docker-compose".to_string()]);
        }

        Err(eyre!(
            "Neither 'docker compose' nor 'docker-compose' found.\n\
             Please install Docker 20.10+ (includes Compose plugin)\n\
             or install docker-compose separately."
        ))
    }

    async fn run_docker_compose(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let root = utils::project_root();
        let compose_file = root.join("docker-compose.yaml");

        if !compose_file.exists() {
            return Err(eyre!("docker-compose.yaml not found in {}", root.display()));
        }

        let compose_file_str = compose_file.to_string_lossy().to_string();
        let compose_cmd = self.detect_compose_command().await?;

        // --- Registry login (if token available) ---
        // Non-fatal: Docker may already be authenticated via credentials helper
        if let Some(token) = self.ghcr_token.clone() {
            self.add_log("🔐 Logging into GHCR...");
            if let Err(e) = self.login_to_ghcr(&token).await {
                self.add_log(&format!(
                    "⚠️  GHCR login warning (will try pull anyway): {e}"
                ));
            }
        }

        // --- Resolve latest image tag from GitHub Releases ---
        let identity_tag = if !self.airgapped {
            let client = Client::new();
            self.add_log("🔍 Checking latest nqrust-identity release tag...");
            match fetch_latest_identity_tag(&client, self.ghcr_token.as_deref()).await {
                Some(tag) => {
                    self.add_log(&format!("✅ Using image tag: {tag}"));
                    tag
                }
                None => {
                    self.add_log("⚠️ Could not resolve latest tag, falling back to 'latest'");
                    "latest".to_string()
                }
            }
        } else {
            "latest".to_string()
        };

        // --- Step 1: Pull images (skip in airgapped mode) ---
        if !self.airgapped {
            self.add_log("⬇️  Step 1/2: Pulling images...");
            self.progress = 10.0;

            let mut cmd = Command::new(&compose_cmd[0]);
            for arg in compose_cmd.iter().skip(1) {
                cmd.arg(arg);
            }
            cmd.args(["-f", &compose_file_str, "pull"])
                .env("IDENTITY_TAG", &identity_tag)
                .current_dir(&root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd.spawn()?;

            // Stream stderr with Ctrl+C support
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    self.process_log_line(&line);
                    let _ = terminal.draw(|frame| self.render(frame));
                    // Allow Ctrl+C to cancel during streaming
                    if event::poll(std::time::Duration::ZERO)? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press
                                && key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                self.running = false;
                                return Ok(());
                            }
                        }
                    }
                }
            }

            let status = child.wait().await?;
            if !status.success() {
                return Err(eyre!("docker compose pull failed"));
            }
            self.add_log("✅ Images pulled successfully");
            self.progress = 50.0;
        } else {
            self.add_log("🔒 Airgapped mode — skipping pull (using local images)");
            self.progress = 50.0;
        }

        // --- Step 2: Start services ---
        self.add_log("🚀 Step 2/2: Starting services...");

        let mut cmd = Command::new(&compose_cmd[0]);
        for arg in compose_cmd.iter().skip(1) {
            cmd.arg(arg);
        }
        cmd.args(["-f", &compose_file_str, "up", "-d"])
            .env("IDENTITY_TAG", &identity_tag)
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Stream stderr with Ctrl+C support
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                self.process_log_line(&line);
                let _ = terminal.draw(|frame| self.render(frame));
                // Allow Ctrl+C to cancel during streaming
                if event::poll(std::time::Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press
                            && key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            self.running = false;
                            return Ok(());
                        }
                    }
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            return Err(eyre!("docker compose up failed"));
        }

        self.add_log("✅ All services started!");
        self.add_log("ℹ️  Keycloak warms up in ~30-60s. Access: https://localhost:8008");
        self.progress = 100.0;
        self.completed_services = self.total_services;
        self.state = AppState::Success;

        Ok(())
    }

    async fn login_to_ghcr(&self, token: &str) -> Result<()> {
        let mut child = Command::new("docker")
            .args(["login", "ghcr.io", "-u", "token", "--password-stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(token.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(eyre!("GHCR login failed: {}", stderr.trim()));
        }
        Ok(())
    }

    fn process_log_line(&mut self, line: &str) {
        self.add_log(line);

        // Track service start events for progress
        let service_name = self.extract_service_name(line);
        if let Some(name) = service_name {
            if line.contains("Started") || line.contains("Running") || line.contains("Created") {
                self.current_service = name;
                self.completed_services = (self.completed_services + 1).min(self.total_services);
                self.progress =
                    (self.completed_services as f64 / self.total_services as f64) * 50.0 + 50.0;
            }
        }
    }

    fn extract_service_name(&self, line: &str) -> Option<String> {
        // Matches lines like: " ✔ Container identity-db  Started"
        let service_names = ["identity-db", "identity-caddy", "identity"];
        for name in &service_names {
            if line.contains(name) {
                return Some(name.to_string());
            }
        }
        None
    }
}
