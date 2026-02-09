#[derive(Debug, Clone, PartialEq)]
pub enum FocusState {
    Field(usize),
    SaveButton,
    CancelButton,
}

#[derive(Debug)]
pub struct RegistryForm {
    pub token: String,
    pub focus_state: FocusState,
    pub error_message: String,
}

impl RegistryForm {
    pub fn new() -> Self {
        Self {
            token: String::new(),
            focus_state: FocusState::Field(0),
            error_message: String::new(),
        }
    }

    pub fn get_current_value_mut(&mut self) -> &mut String {
        &mut self.token
    }

    pub fn validate(&mut self) -> bool {
        if self.token.trim().is_empty() {
            self.error_message = "Personal access token is required".to_string();
            return false;
        }

        self.error_message.clear();
        true
    }
}

impl Default for RegistryForm {
    fn default() -> Self {
        Self::new()
    }
}
