/// In-process clipboard (shared via the App struct).
#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    content: String,
}

impl Clipboard {
    pub fn copy(&mut self, text: String) {
        self.content = text;
    }

    pub fn paste(&self) -> &str {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}
