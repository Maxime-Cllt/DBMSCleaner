pub enum LogType {
    Info,
    Warning,
    Error,
    Critical,
}

impl LogType {
    pub fn as_str(&self) -> &str {
        match self {
            LogType::Info => "INFO",
            LogType::Warning => "WARNING",
            LogType::Error => "ERROR",
            LogType::Critical => "CRITICAL",
        }
    }
}

impl std::fmt::Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_type_to_string() {
        assert_eq!(LogType::Info.as_str(), "INFO");
        assert_eq!(LogType::Warning.as_str(), "WARNING");
        assert_eq!(LogType::Error.as_str(), "ERROR");
        assert_eq!(LogType::Critical.as_str(), "CRITICAL");
    }
}
