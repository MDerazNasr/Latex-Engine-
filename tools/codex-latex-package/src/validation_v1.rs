//! Validates bounded package labels and relative manifest paths.

use std::path::Component;
use std::path::Path;

use crate::error_v1::PackageErrorV1;

pub(crate) fn validate_label_v1(value: &str, label: &str) -> Result<(), PackageErrorV1> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(PackageErrorV1::new(format!("bundle {label} is invalid")));
    }
    Ok(())
}

pub(crate) fn validate_relative_path_v1(path: &str) -> Result<(), PackageErrorV1> {
    if path.is_empty() || path.len() > 512 || path.contains('\\') {
        return Err(PackageErrorV1::new("manifest path is invalid"));
    }
    if Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(())
    } else {
        Err(PackageErrorV1::new("manifest path is not safely relative"))
    }
}

#[cfg(test)]
#[path = "validation_v1_tests.rs"]
mod tests;
