//! Terminal ANSI color codes for formatted console output.
//!
//! This module provides constants for ANSI escape codes used to colorize
//! terminal output, making logs and messages more readable.

/// ANSI escape code for red text
pub const RED: &str = "\x1b[31m";

/// ANSI escape code for green text
pub const GREEN: &str = "\x1b[32m";

/// ANSI escape code for yellow text
pub const YELLOW: &str = "\x1b[33m";

/// ANSI escape code for blue text
pub const BLUE: &str = "\x1b[34m";

/// ANSI escape code to reset text formatting
pub const RESET: &str = "\x1b[0m";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_codes() {
        assert_eq!(RED, "\x1b[31m");
        assert_eq!(GREEN, "\x1b[32m");
        assert_eq!(YELLOW, "\x1b[33m");
        assert_eq!(BLUE, "\x1b[34m");
        assert_eq!(RESET, "\x1b[0m");
    }
}
