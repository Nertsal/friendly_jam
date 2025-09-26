use super::*;

pub struct TextWidget {
    pub state: WidgetState,
    pub text: Text,
}

impl TextWidget {
    pub fn new(text: impl Into<Text>) -> Self {
        Self {
            state: WidgetState::new(),
            text: text.into(),
        }
    }
}
