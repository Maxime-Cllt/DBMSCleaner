/// Enum representing different types of logs
#[repr(u8)]
pub enum LogType {
    Info,
    Warning,
    Error,
    Critical,
}

impl LogType {
    /// Returns the string representation of the log type
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
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

    #[tokio::test]
    async fn test_log_type_display() {
        assert_eq!(format!("{}", LogType::Info), "INFO");
        assert_eq!(format!("{}", LogType::Warning), "WARNING");
        assert_eq!(format!("{}", LogType::Error), "ERROR");
        assert_eq!(format!("{}", LogType::Critical), "CRITICAL");
    }

}
