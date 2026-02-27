#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    SslSetup,
    RegistrySetup,
    Confirmation,
    UpdateList,
    UpdatePulling,
    Installing,
    Success,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuSelection {
    GenerateSsl,
    Proceed,
    UpdateToken,
    CheckUpdates,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SslSetupMenuSelection {
    Generate,
    Skip,
    Cancel,
}
