use std::collections::HashMap;

use crate::ServiceError;

#[cfg(feature = "parse")]
use crate::Value;
#[cfg(feature = "parse")]
use ron::value::Value as RonValue;

/// A single file from a Content Pack, indexed in the Registry.
#[derive(Clone, Debug)]
pub struct RegistryEntry {
    /// Filename (e.g. "`items_edible.ron`").
    pub filename: String,

    /// Raw file bytes.
    pub data: Vec<u8>,
}

/// All data from a Content Pack, grouped by domain (from manifest.toml).
///
/// Registry is an "in-memory filesystem". It does not know the structure of
/// the data inside files. Each plugin decides how to parse the bytes it receives.
///
/// # Structure
///
/// ```text
/// domains: {
///     "items"   → { "items_edible.ron": [...bytes...], "items_material.ron": [...bytes...] },
///     "recipes" → { "combinations.ron": [...bytes...] },
/// }
/// ```
#[derive(Clone, Debug, Default)]
pub struct Registry {
    /// Domain → { filename → raw bytes }
    pub domains: HashMap<String, HashMap<String, Vec<u8>>>,
}

impl Registry {
    /// Create an empty Registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
        }
    }

    /// Get all files in a domain.
    pub fn domain(&self, name: &str) -> Result<&HashMap<String, Vec<u8>>, ServiceError> {
        self.domains
            .get(name)
            .ok_or_else(|| ServiceError::RegistryDomainNotFound(name.into()))
    }

    /// Get raw bytes of a specific file within a domain.
    pub fn file(&self, domain: &str, filename: &str) -> Result<&[u8], ServiceError> {
        self.domains
            .get(domain)
            .and_then(|files| files.get(filename))
            .map(std::vec::Vec::as_slice)
            .ok_or_else(|| ServiceError::FileNotFound {
                domain: domain.into(),
                filename: filename.into(),
            })
    }

    /// Get all entries in a domain as `Vec<RegistryEntry>` (filename + data).
    pub fn domain_entries(&self, name: &str) -> Result<Vec<RegistryEntry>, ServiceError> {
        let files = self.domain(name)?;
        Ok(files
            .iter()
            .map(|(filename, data)| RegistryEntry {
                filename: filename.clone(),
                data: data.clone(),
            })
            .collect())
    }

    /// Add a file to a domain.
    pub fn add_file(&mut self, domain: &str, filename: &str, data: Vec<u8>) {
        self.domains
            .entry(domain.to_owned())
            .or_default()
            .insert(filename.to_owned(), data);
    }

    /// Check whether a domain exists.
    #[must_use]
    pub fn has_domain(&self, name: &str) -> bool {
        self.domains.contains_key(name)
    }

    /// Iterate over all domain names.
    pub fn domain_names(&self) -> impl Iterator<Item = &str> {
        self.domains.keys().map(std::string::String::as_str)
    }

    /// Number of domains.
    #[must_use]
    pub fn len(&self) -> usize {
        self.domains.len()
    }

    /// Whether the Registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.domains.is_empty()
    }
}

// ── Registry data parsing ──────────────────────────────────────────

/// Parse registry data bytes into a Value tree.
///
/// The parsing backend is version-dependent (currently RON).
/// This function is the single point of parsing: plugins that use it
/// automatically get support for new data formats when the plugin-api
/// crate is upgraded.
///
/// # Examples
///
/// ```rust
/// use openstranded_plugin_api::parse_registry_data;
///
/// // Named struct (common in data files)
/// let bytes = b"[ItemDef(id: 1, name: \"Wood\")]";
/// let data = parse_registry_data(bytes).unwrap();
/// ```
#[cfg(feature = "parse")]
pub fn parse_registry_data(bytes: &[u8]) -> Result<Value, ServiceError> {
    let s = std::str::from_utf8(bytes).map_err(|e| {
        ServiceError::ParseError(format!("registry data is not valid UTF-8: {e}"))
    })?;
    let ron_value: RonValue = ron::from_str(s).map_err(|e| {
        ServiceError::ParseError(format!("failed to parse registry data: {e}"))
    })?;
    Ok(convert_ron_value(ron_value))
}

/// Convenience wrapper: parse bytes as an array and return elements.
///
/// Errors if the root value is not an array.
///
/// # Examples
///
/// ```rust
/// use openstranded_plugin_api::parse_registry_list;
///
/// let bytes = b"[ItemDef(id: 1), ItemDef(id: 2)]";
/// let items = parse_registry_list(bytes).unwrap();
/// assert_eq!(items.len(), 2);
/// ```
#[cfg(feature = "parse")]
pub fn parse_registry_list(bytes: &[u8]) -> Result<Vec<Value>, ServiceError> {
    let root = parse_registry_data(bytes)?;
    match root {
        Value::Array(arr) => Ok(arr),
        other => Err(ServiceError::TypeMismatch {
            expected: "array".into(),
            found: format!("{other:?}"),
        }),
    }
}

// ── RON value conversion ───────────────────────────────────────────

/// Convert a RON dynamic value to our plugin-api Value.
#[cfg(feature = "parse")]
fn convert_ron_value(v: RonValue) -> Value {
    match v {
        RonValue::Unit => Value::Null,
        RonValue::Bool(b) => Value::Bool(b),
        RonValue::Number(n) => {
            match n {
                ron::value::Number::I32(i) => Value::I32(i),
                ron::value::Number::I64(i) => {
                    if let Ok(n) = i32::try_from(i) {
                        Value::I32(n)
                    } else {
                        Value::F64(i as f64)
                    }
                }
                ron::value::Number::F32(f) => Value::F64(f64::from(f.0)),
                ron::value::Number::F64(f) => Value::F64(f.0),
                ron::value::Number::U32(u) => Value::U32(u),
                ron::value::Number::U64(u) => {
                    if let Ok(n) = u32::try_from(u) {
                        Value::U32(n)
                    } else {
                        Value::F64(u as f64)
                    }
                }
                // Handle remaining integer types
                ron::value::Number::I8(i) => Value::I32(i32::from(i)),
                ron::value::Number::I16(i) => Value::I32(i32::from(i)),
                ron::value::Number::U8(u) => Value::U32(u32::from(u)),
                ron::value::Number::U16(u) => Value::U32(u32::from(u)),
                #[allow(unreachable_patterns)]
                _ => Value::Null,
            }
        }
        RonValue::Char(c) => Value::String(c.to_string()),
        RonValue::String(s) => Value::String(s),
        RonValue::Bytes(b) => Value::Bytes(b),
        RonValue::Seq(arr) => {
            Value::Array(arr.into_iter().map(convert_ron_value).collect())
        }
        RonValue::Map(map) => {
            Value::Map(
                map.into_iter()
                    .map(|(k, v)| (convert_ron_key(k), convert_ron_value(v)))
                    .collect(),
            )
        }
        RonValue::Option(Some(v)) => convert_ron_value(*v),
        RonValue::Option(None) => Value::Null,
    }
}

/// Convert a RON map key (which can be any Value) to a String.
#[cfg(feature = "parse")]
fn convert_ron_key(k: RonValue) -> String {
    match k {
        RonValue::String(s) => s,
        RonValue::Char(c) => c.to_string(),
        other => format!("{other:?}"),
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(feature = "parse")]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registry_data_object() {
        let ron = br#"ItemDef(id: 1, name: "Wood")"#;
        let value = parse_registry_data(ron).unwrap();
        assert_eq!(value.get("id").unwrap().as_u32().unwrap(), 1);
        assert_eq!(value.get("name").unwrap().as_str().unwrap(), "Wood");
    }

    #[test]
    fn test_parse_registry_data_array() {
        let ron = br#"[1, 2, 3]"#;
        let value = parse_registry_data(ron).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_parse_registry_list_ok() {
        let ron = br#"[ItemDef(id: 1), ItemDef(id: 2)]"#;
        let items = parse_registry_list(ron).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].get("id").unwrap().as_u32().unwrap(), 1);
    }

    #[test]
    fn test_parse_registry_list_not_array() {
        let ron = br#"ItemDef(id: 1)"#;
        let err = parse_registry_list(ron).unwrap_err();
        assert!(matches!(err, ServiceError::TypeMismatch { .. }));
    }

    #[test]
    fn test_parse_invalid_ron() {
        let ron = b"not valid ron {{{";
        assert!(parse_registry_data(ron).is_err());
    }

    #[test]
    fn test_registry_add_and_get() {
        let mut reg = Registry::new();
        reg.add_file("items", "items_edible.ron", vec![1, 2, 3]);
        reg.add_file("items", "items_material.ron", vec![4, 5, 6]);

        assert_eq!(reg.file("items", "items_edible.ron").unwrap(), &[1, 2, 3]);
        assert_eq!(reg.domain_entries("items").unwrap().len(), 2);
    }

    #[test]
    fn test_registry_file_not_found() {
        let reg = Registry::new();
        let err = reg.file("nonexistent", "foo.ron").unwrap_err();
        assert!(matches!(err, ServiceError::FileNotFound { .. }));
    }

    #[test]
    fn test_registry_domain_not_found() {
        let reg = Registry::new();
        let err = reg.domain("nonexistent").unwrap_err();
        assert!(matches!(err, ServiceError::RegistryDomainNotFound(..)));
    }
}
