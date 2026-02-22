#[derive(Debug)]
pub struct ChatSession {
    pub history: Vec<String>,
    pub draft: String,
}

impl ChatSession {
    pub fn new() -> Self {
        Self {
            history: vec!["Welcome to AI Chat!".into()],
            draft: String::new(),
        }
    }
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new()
    }
}
