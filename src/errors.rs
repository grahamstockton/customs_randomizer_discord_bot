#[derive(Debug, Clone)]
pub struct InvalidInputError(pub String);

impl std::fmt::Display for InvalidInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for InvalidInputError {}
