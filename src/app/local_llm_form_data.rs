#[derive(Debug, Clone, PartialEq)]
pub enum FocusState {
    Field(usize),
    SaveButton,
    CancelButton,
}

#[derive(Debug, Clone)]
pub struct LocalLlmFormData {
    pub(crate) llm_model: String,
    pub(crate) llm_api_base: String,
    pub(crate) max_tokens: String,
    pub(crate) embedding_model: String,
    pub(crate) embedding_api_base: String,
    pub(crate) embedding_dim: String,
    pub(crate) focus_state: FocusState,
    pub(crate) error_message: String,
}

impl LocalLlmFormData {
    pub fn new() -> Self {
        Self {
            llm_model: String::new(),
            llm_api_base: String::new(),
            max_tokens: String::from("20480"),
            embedding_model: String::new(),
            embedding_api_base: String::new(),
            embedding_dim: String::from("2560"),
            focus_state: FocusState::Field(0),
            error_message: String::new(),
        }
    }

    pub fn validate(&mut self) -> bool {
        // Validate LLM Model
        if self.llm_model.trim().is_empty() {
            self.error_message = "LLM Model is required!".to_string();
            return false;
        }

        // Validate LLM API Base
        if self.llm_api_base.trim().is_empty() {
            self.error_message = "LLM API Base URL is required!".to_string();
            return false;
        }
        if !self.is_valid_url(&self.llm_api_base) {
            self.error_message = "LLM API Base must start with http:// or https://".to_string();
            return false;
        }

        // Validate Max Tokens
        if self.max_tokens.trim().is_empty() {
            self.error_message = "Max Tokens is required!".to_string();
            return false;
        }
        if self.max_tokens.parse::<u32>().is_err() {
            self.error_message = "Max Tokens must be a valid number!".to_string();
            return false;
        }

        // Validate Embedding Model
        if self.embedding_model.trim().is_empty() {
            self.error_message = "Embedding Model is required!".to_string();
            return false;
        }

        // Validate Embedding API Base
        if self.embedding_api_base.trim().is_empty() {
            self.error_message = "Embedding API Base URL is required!".to_string();
            return false;
        }
        if !self.is_valid_url(&self.embedding_api_base) {
            self.error_message =
                "Embedding API Base must start with http:// or https://".to_string();
            return false;
        }

        // Validate Embedding Dimension
        if self.embedding_dim.trim().is_empty() {
            self.error_message = "Embedding Dimension is required!".to_string();
            return false;
        }
        if self.embedding_dim.parse::<u32>().is_err() {
            self.error_message = "Embedding Dimension must be a valid number!".to_string();
            return false;
        }

        self.error_message.clear();
        true
    }

    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    pub fn get_current_value_mut(&mut self) -> &mut String {
        match &self.focus_state {
            FocusState::Field(idx) => match idx {
                0 => &mut self.llm_model,
                1 => &mut self.llm_api_base,
                2 => &mut self.max_tokens,
                3 => &mut self.embedding_model,
                4 => &mut self.embedding_api_base,
                5 => &mut self.embedding_dim,
                _ => &mut self.llm_model,
            },
            _ => &mut self.llm_model, // Fallback for buttons
        }
    }

    pub fn get_field_name(&self, field_index: usize) -> &str {
        match field_index {
            0 => "LLM Model",
            1 => "LLM API Base",
            2 => "Max Tokens",
            3 => "Embedding Model",
            4 => "Embedding API Base",
            5 => "Embedding Dimension",
            _ => "Unknown",
        }
    }

    pub fn get_total_fields(&self) -> usize {
        6
    }
}
