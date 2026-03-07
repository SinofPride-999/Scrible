use std::time::{Duration, Instant};

/// Manages a transient status message shown at the bottom of the screen.
#[derive(Debug, Clone)]
pub struct StatusBar {
    pub message: String,
    pub expires_at: Option<Instant>,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            message: String::new(),
            expires_at: None,
        }
    }
}

impl StatusBar {
    /// Set a message that auto-clears after `duration`.
    pub fn set(&mut self, msg: impl Into<String>, duration: Duration) {
        self.message = msg.into();
        self.expires_at = Some(Instant::now() + duration);
    }

    /// Set a persistent message (no expiry).
    pub fn set_persistent(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.expires_at = None;
    }

    /// Tick: clear the message if it has expired.
    pub fn tick(&mut self) {
        if let Some(exp) = self.expires_at {
            if Instant::now() >= exp {
                self.message.clear();
                self.expires_at = None;
            }
        }
    }

    pub fn current(&self) -> &str {
        &self.message
    }
}
