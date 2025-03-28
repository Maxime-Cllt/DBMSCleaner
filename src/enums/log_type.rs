
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
