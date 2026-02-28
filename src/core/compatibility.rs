use crate::error::Result;
use std::collections::HashMap;

/// Compatibility matrix mapping compiler versions to compatible Frame versions
#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    mappings: HashMap<String, Vec<String>>,
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        let mut mappings = HashMap::new();

        // Frame 1.0.0 requires compiler >= 0.14.0
        mappings.insert("0.14.0".to_string(), vec!["1.0.0".to_string()]);
        mappings.insert("0.15.0".to_string(), vec!["1.0.0".to_string()]);

        // Frame 2.0.0 for future compiler versions
        mappings.insert("0.16.0".to_string(), vec!["2.0.0".to_string()]);

        Self { mappings }
    }
}

impl CompatibilityMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    /// Find compatible Frame version for a given compiler version
    pub fn find_compatible_frame_version(&self, compiler_version: &str) -> Option<String> {
        // Normalize version (remove 'v' prefix if present)
        let normalized = compiler_version.trim_start_matches('v');

        // Check exact match first
        if let Some(versions) = self.mappings.get(normalized) {
            return versions.first().cloned();
        }

        // Check if compiler version is greater than any minimum required version
        // For compiler >= 0.16.0, use Frame 2.0.0
        if is_version_gte(normalized, "0.16.0") {
            return Some("2.0.0".to_string());
        }
        // For compiler >= 0.14.0, use Frame 1.0.0
        if is_version_gte(normalized, "0.14.0") {
            return Some("1.0.0".to_string());
        }

        None
    }

    /// Get minimum required compiler version for a Frame version
    pub fn get_required_compiler_version(&self, frame_version: &str) -> Option<String> {
        let normalized = frame_version.trim_start_matches('v');

        // Use version ranges: any Frame >= 2.0.0 requires compiler >= 0.16.0
        if is_version_gte(normalized, "2.0.0") {
            Some("0.16.0".to_string())
        } else if is_version_gte(normalized, "0.1.0") {
            // All Frame versions >= 0.1.0 (including 1.x.x) require compiler >= 0.14.0
            Some("0.14.0".to_string())
        } else {
            None
        }
    }

    /// Check if a compiler version is compatible with a Frame version
    pub fn is_compatible(&self, compiler_version: &str, frame_version: &str) -> bool {
        let required = match self.get_required_compiler_version(frame_version) {
            Some(req) => req,
            None => return false,
        };

        let normalized_compiler = compiler_version.trim_start_matches('v');
        is_version_gte(normalized_compiler, &required)
    }
}

/// Check if version `a` is greater than or equal to version `b`
pub fn is_version_gte(a: &str, b: &str) -> bool {
    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    for i in 0..3 {
        if a_parts[i] > b_parts[i] {
            return true;
        } else if a_parts[i] < b_parts[i] {
            return false;
        }
    }

    true // Equal versions
}

/// Parse version string into [major, minor, patch]
fn parse_version(version: &str) -> [u32; 3] {
    let normalized = version.trim_start_matches('v');
    let parts: Vec<&str> = normalized.split('.').collect();

    [
        parts.first().and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
    ]
}

/// Validate that a compiler version is compatible with the given Frame version
pub fn check_frame_compatibility(compiler_version: &str, frame_version: &str) -> Result<()> {
    let matrix = CompatibilityMatrix::new();

    if !matrix.is_compatible(compiler_version, frame_version) {
        let required = matrix
            .get_required_compiler_version(frame_version)
            .unwrap_or_else(|| "unknown".to_string());

        return Err(crate::error::CleenError::FrameIncompatible {
            frame_version: frame_version.to_string(),
            required_compiler: required,
            current_compiler: compiler_version.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_version_gte("0.14.0", "0.14.0"));
        assert!(is_version_gte("0.15.0", "0.14.0"));
        assert!(is_version_gte("1.0.0", "0.14.0"));
        assert!(!is_version_gte("0.13.0", "0.14.0"));
        assert!(!is_version_gte("0.13.9", "0.14.0"));
    }

    #[test]
    fn test_version_parsing() {
        assert_eq!(parse_version("0.14.0"), [0, 14, 0]);
        assert_eq!(parse_version("v1.2.3"), [1, 2, 3]);
        assert_eq!(parse_version("2.0"), [2, 0, 0]);
    }

    #[test]
    fn test_compatibility_matrix() {
        let matrix = CompatibilityMatrix::new();

        // Frame 1.0.0 compatibility
        assert!(matrix.is_compatible("0.14.0", "1.0.0"));
        assert!(matrix.is_compatible("0.15.0", "1.0.0"));
        assert!(!matrix.is_compatible("0.13.0", "1.0.0"));

        // Frame 2.0.0 compatibility
        assert!(matrix.is_compatible("0.16.0", "2.0.0"));
        assert!(!matrix.is_compatible("0.15.0", "2.0.0"));
    }

    #[test]
    fn test_find_compatible_frame() {
        let matrix = CompatibilityMatrix::new();

        // Compiler 0.14.x and 0.15.x should use Frame 1.0.0
        assert_eq!(
            matrix.find_compatible_frame_version("0.14.0"),
            Some("1.0.0".to_string())
        );
        assert_eq!(
            matrix.find_compatible_frame_version("0.15.0"),
            Some("1.0.0".to_string())
        );

        // Compiler 0.16.x and above should use Frame 2.0.0
        assert_eq!(
            matrix.find_compatible_frame_version("0.16.0"),
            Some("2.0.0".to_string())
        );

        // Compiler below 0.14.0 should have no compatible Frame version
        assert_eq!(matrix.find_compatible_frame_version("0.13.0"), None);
    }
}
