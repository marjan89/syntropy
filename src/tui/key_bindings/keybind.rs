use std::{
    error::Error,
    fmt::{self, Formatter},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A parsed key binding that can match against key events
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBind {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBind {
    /// Parse a user-friendly key binding string into a KeyBind
    ///
    /// # Supported Formats
    ///
    /// ## Simple keys
    /// - Single character: `"v"`, `"D"`, `"1"`
    /// - Case-sensitive for letters
    ///
    /// ## Special keys (case-insensitive)
    /// - `"<space>"` - Space bar
    /// - `"<enter>"` or `"<return>"` - Enter key
    /// - `"<esc>"` or `"<escape>"` - Escape key
    /// - `"<tab>"` - Tab key
    /// - `"<backspace>"` or `"<bs>"` - Backspace
    /// - `"<delete>"` or `"<del>"` - Delete key
    /// - `"<up>"`, `"<down>"`, `"<left>"`, `"<right>"` - Arrow keys
    /// - `"<home>"`, `"<end>"` - Home/End keys
    /// - `"<pageup>"` or `"<pgup>"` - Page Up
    /// - `"<pagedown>"` or `"<pgdn>"` - Page Down
    ///
    /// ## Modifiers
    /// - `"<C-k>"` - Ctrl + k
    /// - `"<S-a>"` - Shift + a
    /// - `"<A-x>"` - Alt + x
    /// - `"<C-S-k>"` - Ctrl + Shift + k (combine multiple)
    ///
    /// Modifier aliases: `C`/`Ctrl`, `S`/`Shift`, `A`/`Alt` (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use syntropy::tui::key_bindings::keybind::KeyBind;
    ///
    /// let binding = KeyBind::parse("v").unwrap();
    /// let binding = KeyBind::parse("<C-k>").unwrap();
    /// let binding = KeyBind::parse("<space>").unwrap();
    /// ```
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(ParseError::Empty);
        }

        // Handle bracketed notation: <C-k>, <space>, <C-S-k>
        if input.starts_with('<') && input.ends_with('>') {
            Self::parse_bracketed(&input[1..input.len() - 1])
        }
        // Handle simple single character: v, D, 1
        else if input.len() == 1 {
            let ch = input
                .chars()
                .next()
                .expect("input.len() == 1 guarantees at least one character");
            // Uppercase letters implicitly mean Shift+lowercase
            // e.g., "K" is treated as "<S-k>"
            if ch.is_ascii_uppercase() {
                Ok(KeyBind {
                    code: KeyCode::Char(ch.to_ascii_lowercase()),
                    modifiers: KeyModifiers::SHIFT,
                })
            } else {
                Ok(KeyBind {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::empty(),
                })
            }
        }
        // Invalid format
        else {
            Err(ParseError::InvalidFormat(input.to_string()))
        }
    }

    /// Parse bracketed notation (content between < and >)
    fn parse_bracketed(inner: &str) -> Result<Self, ParseError> {
        let parts: Vec<&str> = inner.split('-').collect();

        if parts.is_empty() {
            return Err(ParseError::Empty);
        }

        let mut modifiers = KeyModifiers::empty();

        // All parts except the last are modifiers
        for modifier in &parts[..parts.len() - 1] {
            match modifier.to_lowercase().as_str() {
                "c" | "ctrl" => modifiers |= KeyModifiers::CONTROL,
                "s" | "shift" => modifiers |= KeyModifiers::SHIFT,
                "a" | "alt" => modifiers |= KeyModifiers::ALT,
                _ => return Err(ParseError::UnknownModifier(modifier.to_string())),
            }
        }

        // Last part is the key itself
        let key_str = parts[parts.len() - 1];

        // Bracketed notation for simple characters is invalid
        // Use "k" not "<k>" - brackets are for special keys and modifiers only
        if modifiers.is_empty() && key_str.len() == 1 {
            return Err(ParseError::UnknownKey(key_str.to_string()));
        }

        let code = Self::parse_key_name(key_str)?;

        // If the key is a single uppercase letter, add SHIFT modifier
        // This makes <C-K> equivalent to <C-S-k>
        if key_str.len() == 1
            && let Some(ch) = key_str.chars().next()
            && ch.is_ascii_uppercase()
        {
            modifiers |= KeyModifiers::SHIFT;
        }

        Ok(KeyBind { code, modifiers })
    }

    /// Parse a key name into a KeyCode
    fn parse_key_name(name: &str) -> Result<KeyCode, ParseError> {
        match name.to_lowercase().as_str() {
            "space" => Ok(KeyCode::Char(' ')),
            "enter" | "return" => Ok(KeyCode::Enter),
            "esc" | "escape" => Ok(KeyCode::Esc),
            "tab" => Ok(KeyCode::Tab),
            "backspace" | "bs" => Ok(KeyCode::Backspace),
            "delete" | "del" => Ok(KeyCode::Delete),
            "up" => Ok(KeyCode::Up),
            "down" => Ok(KeyCode::Down),
            "left" => Ok(KeyCode::Left),
            "right" => Ok(KeyCode::Right),
            "home" => Ok(KeyCode::Home),
            "end" => Ok(KeyCode::End),
            "pageup" | "pgup" => Ok(KeyCode::PageUp),
            "pagedown" | "pgdn" => Ok(KeyCode::PageDown),
            // F-keys
            "f1" => Ok(KeyCode::F(1)),
            "f2" => Ok(KeyCode::F(2)),
            "f3" => Ok(KeyCode::F(3)),
            "f4" => Ok(KeyCode::F(4)),
            "f5" => Ok(KeyCode::F(5)),
            "f6" => Ok(KeyCode::F(6)),
            "f7" => Ok(KeyCode::F(7)),
            "f8" => Ok(KeyCode::F(8)),
            "f9" => Ok(KeyCode::F(9)),
            "f10" => Ok(KeyCode::F(10)),
            "f11" => Ok(KeyCode::F(11)),
            "f12" => Ok(KeyCode::F(12)),
            // Single character (preserves original case)
            s if s.len() == 1 => Ok(KeyCode::Char(
                s.chars()
                    .next()
                    .expect("s.len() == 1 guarantees at least one character"),
            )),
            _ => Err(ParseError::UnknownKey(name.to_string())),
        }
    }

    /// Check if this binding matches a KeyEvent
    pub fn matches(&self, event: &KeyEvent) -> bool {
        // Normalize event: if it has uppercase char + SHIFT, convert to lowercase + SHIFT
        // This makes "K" equivalent to "<S-k>" and handles terminal behavior
        let (normalized_code, normalized_mods) = if let KeyCode::Char(ch) = event.code {
            if ch.is_ascii_uppercase() && event.modifiers.contains(KeyModifiers::SHIFT) {
                (KeyCode::Char(ch.to_ascii_lowercase()), event.modifiers)
            } else {
                (event.code, event.modifiers)
            }
        } else {
            (event.code, event.modifiers)
        };

        self.code == normalized_code && self.modifiers == normalized_mods
    }
}

/// Errors that can occur when parsing key bindings
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Empty string provided
    Empty,
    /// Invalid format (not bracketed and not single char)
    InvalidFormat(String),
    /// Unknown modifier key (not C, S, or A)
    UnknownModifier(String),
    /// Unknown key name
    UnknownKey(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "Empty key binding"),
            ParseError::InvalidFormat(s) => {
                write!(f, "Invalid key binding format: '{}'", s)
            }
            ParseError::UnknownModifier(m) => {
                write!(f, "Unknown modifier: '{}' (use C, S, or A)", m)
            }
            ParseError::UnknownKey(k) => write!(f, "Unknown key: '{}'", k),
        }
    }
}

impl Error for ParseError {}
