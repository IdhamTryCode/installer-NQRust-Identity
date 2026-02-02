mod airgapped;
mod app;
mod templates;
mod ui;
mod utils;

use app::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    
    // Check if running as airgapped binary and setup if needed
    if airgapped::is_airgapped_binary()? {
        airgapped::setup().await?;
    }
    
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
