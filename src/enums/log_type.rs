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
        assert_eq!(LogType::Info.to_string(), "INFO");
        assert_eq!(LogType::Warning.to_string(), "WARNING");
        assert_eq!(LogType::Error.to_string(), "ERROR");
        assert_eq!(LogType::Critical.to_string(), "CRITICAL");
    }
}
