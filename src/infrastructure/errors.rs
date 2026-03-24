#![allow(dead_code)]

use std::fmt;

use anyhow::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorContext {
    Generic,
    Launch,
    Download,
    Settings,
    Java,
    Persistence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Network,
    Permission,
    NotFound,
    Validation,
    Integrity,
    Java,
    Archive,
    Database,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserFacingError {
    pub kind: ErrorKind,
    pub title_key: &'static str,
    pub message_key: &'static str,
    pub fallback_message: String,
    pub recoverable: bool,
}

impl UserFacingError {
    pub fn from_error(error: &Error) -> Self {
        Self::from_error_with_context(error, ErrorContext::Generic)
    }

    pub fn from_error_with_context(error: &Error, context: ErrorContext) -> Self {
        let message = error.to_string();
        let lower = message.to_lowercase();
        let kind = classify_error(&lower);
        let (title_key, message_key, recoverable) = resolve_keys(&kind, &context);

        Self {
            kind,
            title_key,
            message_key,
            fallback_message: message,
            recoverable,
        }
    }

    pub fn message(&self) -> &str {
        &self.fallback_message
    }
}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.fallback_message)
    }
}

impl std::error::Error for UserFacingError {}

fn classify_error(message: &str) -> ErrorKind {
    if message.contains("network")
        || message.contains("dns")
        || message.contains("http")
        || message.contains("timeout")
        || message.contains("connection refused")
        || message.contains("failed to open external url")
    {
        return ErrorKind::Network;
    }

    if message.contains("permission denied")
        || message.contains("access is denied")
        || message.contains("operation not permitted")
    {
        return ErrorKind::Permission;
    }

    if message.contains("not found")
        || message.contains("no such file")
        || message.contains("missing")
        || message.contains("does not exist")
    {
        return ErrorKind::NotFound;
    }

    if message.contains("invalid")
        || message.contains("incompatible")
        || message.contains("mismatch")
        || message.contains("unexpected")
    {
        return ErrorKind::Validation;
    }

    if message.contains("sha1")
        || message.contains("corrupt")
        || message.contains("integrity")
        || message.contains("manifest.json")
    {
        return ErrorKind::Integrity;
    }

    if message.contains("java")
        || message.contains("jvm")
        || message.contains("archive")
        || message.contains("unzip")
        || message.contains("zip")
    {
        if message.contains("archive") || message.contains("zip") || message.contains("unzip") {
            return ErrorKind::Archive;
        }
        return ErrorKind::Java;
    }

    if message.contains("database")
        || message.contains("sqlite")
        || message.contains("sqlx")
        || message.contains("constraint")
    {
        return ErrorKind::Database;
    }

    ErrorKind::Unknown
}

fn resolve_keys(kind: &ErrorKind, context: &ErrorContext) -> (&'static str, &'static str, bool) {
    match (kind, context) {
        (ErrorKind::Network, ErrorContext::Download) => (
            "error.download.network.title",
            "error.download.network.body",
            true,
        ),
        (ErrorKind::Network, ErrorContext::Launch) => (
            "error.launch.network.title",
            "error.launch.network.body",
            true,
        ),
        (ErrorKind::Permission, ErrorContext::Settings) => (
            "error.settings.permission.title",
            "error.settings.permission.body",
            true,
        ),
        (ErrorKind::Integrity, ErrorContext::Download) => (
            "error.download.integrity.title",
            "error.download.integrity.body",
            true,
        ),
        (ErrorKind::Java, ErrorContext::Java) => {
            ("error.java.runtime.title", "error.java.runtime.body", true)
        }
        (ErrorKind::Archive, ErrorContext::Java) => {
            ("error.java.archive.title", "error.java.archive.body", true)
        }
        (ErrorKind::Database, ErrorContext::Persistence) => (
            "error.persistence.database.title",
            "error.persistence.database.body",
            false,
        ),
        (ErrorKind::Validation, ErrorContext::Launch) => (
            "error.launch.validation.title",
            "error.launch.validation.body",
            true,
        ),
        _ => ("error.generic.title", "error.generic.body", true),
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorContext, ErrorKind, UserFacingError, classify_error};

    #[test]
    fn classifies_network_errors() {
        assert_eq!(classify_error("dns lookup failed"), ErrorKind::Network);
    }

    #[test]
    fn maps_launch_validation_errors() {
        let error = anyhow::anyhow!("current java may be incompatible");
        let mapped = UserFacingError::from_error_with_context(&error, ErrorContext::Launch);
        assert_eq!(mapped.kind, ErrorKind::Validation);
        assert_eq!(mapped.title_key, "error.launch.validation.title");
        assert!(mapped.recoverable);
    }
}
