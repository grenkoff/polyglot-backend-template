//! Greetings built from names.

use crate::name::{BlankName, Name};

/// Text that greets someone.
pub trait Greeting {
    /// Returns the greeting, or an error when the name is blank.
    fn text(&self) -> Result<String, BlankName>;
}

/// Greeting that says hello to a name.
#[derive(Debug, Clone)]
pub struct Plain<N> {
    /// Name of the person to greet.
    name: N,
}

impl<N: Name> Plain<N> {
    /// Greets the given name.
    #[must_use]
    pub const fn new(name: N) -> Self {
        Self { name }
    }
}

impl<N: Name> Greeting for Plain<N> {
    fn text(&self) -> Result<String, BlankName> {
        Ok(format!("Hello, {}!", self.name.text()?))
    }
}

/// Decorator: adds a polite ending to another greeting.
#[derive(Debug, Clone)]
pub struct Polite<G> {
    /// Greeting to extend.
    origin: G,
}

impl<G: Greeting> Polite<G> {
    /// Makes the given greeting polite.
    #[must_use]
    pub const fn new(origin: G) -> Self {
        Self { origin }
    }
}

impl<G: Greeting> Greeting for Polite<G> {
    fn text(&self) -> Result<String, BlankName> {
        Ok(format!("{} Nice to meet you.", self.origin.text()?))
    }
}

#[cfg(test)]
mod tests {
    use super::{Greeting, Plain, Polite};
    use crate::name::{BlankName, Raw, Trimmed};

    #[test]
    fn greets_by_name() {
        assert_eq!(
            Plain::new(Raw::new("Ann")).text(),
            Ok(String::from("Hello, Ann!"))
        );
    }

    #[test]
    fn greets_politely() {
        assert_eq!(
            Polite::new(Plain::new(Raw::new("Ann"))).text(),
            Ok(String::from("Hello, Ann! Nice to meet you."))
        );
    }

    #[test]
    fn does_not_greet_nobody() {
        assert_eq!(
            Polite::new(Plain::new(Trimmed::new(Raw::new(" ")))).text(),
            Err(BlankName)
        );
    }
}
