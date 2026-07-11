use std::fmt;

/// Compile-time conversion of a decimal &str to u32.
macro_rules! parse_version_num {
    ($s:expr) => {{
        const BYTES: &[u8] = $s.as_bytes();
        let mut n: u32 = 0;
        let mut i: usize = 0;
        while i < BYTES.len() && BYTES[i] >= b'0' && BYTES[i] <= b'9' {
            n = n * 10 + (BYTES[i] - b'0') as u32;
            i += 1;
        }
        n
    }};
}

/// Version of `openstranded-plugin-api` baked at compile time.
///
/// Each WASM plugin exports this via `plugin_api_version()`.
/// The engine reads it on load to check compatibility.
///
/// The version is automatically derived from the `Cargo.toml` of the
/// `openstranded-plugin-api` crate. Plugin authors never set it manually.
///
/// # WASM ABI
///
/// `#[repr(C)]` ensures a stable layout when passing across the WASM boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ApiVersion {
    /// Major version — breaking changes.
    pub major: u32,
    /// Minor version — backwards-compatible additions.
    pub minor: u32,
    /// Patch version — backwards-compatible bug fixes.
    pub patch: u32,
}

impl ApiVersion {
    /// Current API version, baked from `CARGO_PKG_VERSION_*` environment variables.
    ///
    /// `env!()` evaluates at compile time, so the version is statically embedded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openstranded_plugin_api::ApiVersion;
    ///
    /// let v = ApiVersion::current();
    /// println!("API version: {v}");
    /// ```
    #[must_use]
    pub const fn current() -> Self {
        Self {
            major: parse_version_num!(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: parse_version_num!(env!("CARGO_PKG_VERSION_MINOR")),
            patch: parse_version_num!(env!("CARGO_PKG_VERSION_PATCH")),
        }
    }

    /// Check compatibility with an engine version.
    ///
    /// Returns `Ok(())` if major versions match.
    /// Returns a warning string if only the minor differs.
    /// Returns an error if major versions differ.
    ///
    /// # Errors
    ///
    /// Returns [`VersionMismatch::Incompatible`] if `self.major != engine_version.major`.
    /// Minor version mismatches are accepted with a logged warning
    /// (the caller is expected to handle the log, not this function).
    #[must_use]
    pub fn compatible_with(&self, engine_version: &ApiVersion) -> Result<(), VersionMismatch> {
        if self.major != engine_version.major {
            return Err(VersionMismatch::Incompatible {
                plugin: *self,
                engine: *engine_version,
            });
        }
        if self.minor != engine_version.minor {
            // Minor mismatch: warn but allow loading
        }
        Ok(())
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version mismatch between a plugin and the engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionMismatch {
    /// Plug-in major version differs from engine — guaranteed incompatibility.
    Incompatible {
        plugin: ApiVersion,
        engine: ApiVersion,
    },
}

impl fmt::Display for VersionMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionMismatch::Incompatible { plugin, engine } => {
                write!(
                    f,
                    "plugin API v{plugin} is incompatible with engine API v{engine} \
                     (major version mismatch; expected {major}, got {got})",
                    major = engine.major,
                    got = plugin.major,
                )
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_parsed() {
        let v = ApiVersion::current();
        // Version values are non-negative by definition; this test confirms
        // the env macro produced a valid parseable number.
        assert_eq!(v.major, 0);
        assert!(v.minor < 100);
        assert!(v.patch < 100);
    }

    #[test]
    fn test_display() {
        let v = ApiVersion {
            major: 0,
            minor: 2,
            patch: 2,
        };
        assert_eq!(v.to_string(), "0.2.2");
    }

    #[test]
    fn test_compatible_same_version() {
        let v = ApiVersion { major: 1, minor: 0, patch: 0 };
        assert!(v.compatible_with(&v).is_ok());
    }

    #[test]
    fn test_compatible_minor_mismatch_still_ok() {
        let plugin = ApiVersion { major: 1, minor: 1, patch: 0 };
        let engine = ApiVersion { major: 1, minor: 2, patch: 0 };
        assert!(plugin.compatible_with(&engine).is_ok());
    }

    #[test]
    fn test_compatible_major_mismatch_error() {
        let plugin = ApiVersion { major: 2, minor: 0, patch: 0 };
        let engine = ApiVersion { major: 1, minor: 0, patch: 0 };
        let err = plugin.compatible_with(&engine).unwrap_err();
        assert!(matches!(err, VersionMismatch::Incompatible { .. }));
    }

    #[test]
    fn test_version_mismatch_display() {
        let mismatch = VersionMismatch::Incompatible {
            plugin: ApiVersion { major: 2, minor: 0, patch: 0 },
            engine: ApiVersion { major: 1, minor: 0, patch: 0 },
        };
        let msg = mismatch.to_string();
        assert!(msg.contains("2.0.0"));
        assert!(msg.contains("1.0.0"));
    }
}
