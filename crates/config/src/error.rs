use thiserror::Error;

/// Errors from the configuration system.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Engine error: {0}")]
    Engine(String),
}

#[cfg(test)]
mod tests {
    use crate::error::ConfigError;

    #[test]
    fn test_io_error_conversion() {
        let e = ConfigError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(format!("{}", e).contains("file not found"));
    }

    #[test]
    fn test_toml_error() {
        let toml_str = "invalid = [[";
        let result: Result<toml::Value, _> = toml_str.parse();
        let e = ConfigError::from(result.unwrap_err());
        assert!(format!("{}", e).contains("TOML"));
    }

    #[test]
    fn test_json_error() {
        let json = "{invalid";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json);
        let e = ConfigError::from(result.unwrap_err());
        assert!(format!("{}", e).contains("JSON"));
    }

    #[test]
    fn test_engine_error() {
        let e = ConfigError::Engine("init failed".into());
        assert_eq!(format!("{}", e), "Engine error: init failed");
    }
}
