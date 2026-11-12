//! Names of the people to greet.

/// Name that has no visible characters.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("name must not be blank")]
pub struct BlankName;

/// Name of a person.
pub trait Name {
    /// Returns the name, or an error when it is blank.
    fn text(&self) -> Result<String, BlankName>;
}

/// Name exactly as it was received.
#[derive(Debug, Clone)]
pub struct Raw {
    /// Received text.
    origin: String,
}

impl Raw {
    /// Wraps the received text.
    #[must_use]
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
        }
    }
}

impl Name for Raw {
    fn text(&self) -> Result<String, BlankName> {
        Ok(self.origin.clone())
    }
}

/// Decorator: trims another name and refuses a blank one.
#[derive(Debug, Clone)]
pub struct Trimmed<N> {
    /// Name to trim.
    origin: N,
}

impl<N: Name> Trimmed<N> {
    /// Trims the given name.
    #[must_use]
    pub const fn new(origin: N) -> Self {
        Self { origin }
    }
}

impl<N: Name> Name for Trimmed<N> {
    fn text(&self) -> Result<String, BlankName> {
        let text = self.origin.text()?;
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(BlankName);
        }
        Ok(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{BlankName, Name, Raw, Trimmed};

    #[test]
    fn keeps_raw_text_as_is() {
        assert_eq!(Raw::new(" Ann ").text(), Ok(String::from(" Ann ")));
    }

    #[test]
    fn trims_spaces() {
        assert_eq!(
            Trimmed::new(Raw::new("  Ann ")).text(),
            Ok(String::from("Ann"))
        );
    }

    #[test]
    fn refuses_blank_names() {
        assert_eq!(Trimmed::new(Raw::new("   ")).text(), Err(BlankName));
    }
}
