#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("name must not be blank")]
pub struct BlankName;

pub trait Name {
    fn text(&self) -> Result<String, BlankName>;
}

#[derive(Debug, Clone)]
pub struct Raw {
    origin: String,
}

impl Raw {
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

#[derive(Debug, Clone)]
pub struct Trimmed<N> {
    origin: N,
}

impl<N: Name> Trimmed<N> {
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
