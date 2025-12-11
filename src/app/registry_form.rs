#[derive(Debug, Default)]
pub struct RegistryForm {
    pub username: String,
    pub token: String,
    pub current_field: usize,
    pub editing: bool,
    pub error_message: String,
}

impl RegistryForm {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            token: String::new(),
            current_field: 0,
            editing: false,
            error_message: String::new(),
        }
    }

    pub fn total_items(&self) -> usize {
        3
    }

    pub fn is_input_field(index: usize) -> bool {
        index < 2
    }

    pub fn get_current_value_mut(&mut self) -> &mut String {
        if self.current_field == 0 {
            &mut self.username
        } else {
            &mut self.token
        }
    }

    pub fn validate(&mut self) -> bool {
        if self.username.trim().is_empty() {
            self.error_message = "Username is required".to_string();
            return false;
        }

        if self.token.trim().is_empty() {
            self.error_message = "Personal access token is required".to_string();
            return false;
        }

        self.error_message.clear();
        true
    }
}
