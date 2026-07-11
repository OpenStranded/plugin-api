use std::collections::HashMap;

use crate::ServiceError;

/// Dynamic type for Service API arguments and return values.
///
/// Allows passing data between plugins without knowing concrete types.
/// Serialisable via serde for WASM boundary crossing.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// No value / null.
    Null,

    /// Boolean.
    Bool(bool),

    /// Signed 32-bit integer.
    I32(i32),

    /// Unsigned 32-bit integer (`item_id`, `type_id`).
    U32(u32),

    /// 64-bit float (weight, distance).
    F64(f64),

    /// String (name, material, class).
    String(String),

    /// Raw bytes (serialised ECS components, .ron files, etc.).
    Bytes(Vec<u8>),

    /// Array of values.
    Array(Vec<Value>),

    /// Associative map / object with named fields.
    Map(HashMap<String, Value>),
}

// ── Type conversions ───────────────────────────────────────────────

impl Value {
    /// Extract as `u32`.
    ///
    /// Accepts `U32`, `I32` (if >= 0), `F64` (if integer and in range), and
    /// parseable `String`.
    pub fn as_u32(&self) -> Result<u32, ServiceError> {
        match self {
            Value::U32(n) => Ok(*n),
            Value::I32(n) if *n >= 0 => Ok(*n as u32),
            Value::F64(n) if *n >= 0.0 && *n <= f64::from(u32::MAX) => Ok(*n as u32),
            Value::String(s) => s.parse::<u32>().map_err(|e| {
                ServiceError::TypeMismatch {
                    expected: "u32".into(),
                    found: format!("string \"{s}\" ({e})"),
                }
            }),
            other => Err(ServiceError::TypeMismatch {
                expected: "u32".into(),
                found: format!("{other:?}"),
            }),
        }
    }

    /// Extract as `i32`.
    pub fn as_i32(&self) -> Result<i32, ServiceError> {
        match self {
            Value::I32(n) => Ok(*n),
            Value::U32(n) if i32::try_from(*n).is_ok() => Ok(*n as i32),
            Value::F64(n) if *n >= f64::from(i32::MIN) && *n <= f64::from(i32::MAX) => {
                Ok(*n as i32)
            }
            Value::String(s) => s.parse::<i32>().map_err(|e| {
                ServiceError::TypeMismatch {
                    expected: "i32".into(),
                    found: format!("string \"{s}\" ({e})"),
                }
            }),
            other => Err(Self::type_mismatch("i32", other)),
        }
    }

    /// Extract as `f64`.
    pub fn as_f64(&self) -> Result<f64, ServiceError> {
        match self {
            Value::F64(n) => Ok(*n),
            Value::I32(n) => Ok(f64::from(*n)),
            Value::U32(n) => Ok(f64::from(*n)),
            Value::String(s) => s.parse::<f64>().map_err(|e| {
                ServiceError::TypeMismatch {
                    expected: "f64".into(),
                    found: format!("string \"{s}\" ({e})"),
                }
            }),
            other => Err(Self::type_mismatch("f64", other)),
        }
    }

    /// Extract as `&str`.
    pub fn as_str(&self) -> Result<&str, ServiceError> {
        match self {
            Value::String(s) => Ok(s.as_str()),
            other => Err(Self::type_mismatch("string", other)),
        }
    }

    /// Extract as `bool`.
    pub fn as_bool(&self) -> Result<bool, ServiceError> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::U32(n) => Ok(*n != 0),
            Value::I32(n) => Ok(*n != 0),
            Value::String(s) if s == "true" || s == "1" => Ok(true),
            Value::String(s) if s == "false" || s == "0" => Ok(false),
            other => Err(Self::type_mismatch("bool", other)),
        }
    }

    /// Extract as `&[u8]`.
    pub fn as_bytes(&self) -> Result<&[u8], ServiceError> {
        match self {
            Value::Bytes(b) => Ok(b.as_slice()),
            other => Err(Self::type_mismatch("bytes", other)),
        }
    }

    /// Extract as `&[Value]`.
    pub fn as_array(&self) -> Result<&[Value], ServiceError> {
        match self {
            Value::Array(arr) => Ok(arr.as_slice()),
            other => Err(Self::type_mismatch("array", other)),
        }
    }

    /// Extract as `&HashMap<String, Value>`.
    pub fn as_map(&self) -> Result<&HashMap<String, Value>, ServiceError> {
        match self {
            Value::Map(map) => Ok(map),
            other => Err(Self::type_mismatch("map", other)),
        }
    }

    /// Get a value by key from a Map.
    pub fn get(&self, key: &str) -> Result<&Value, ServiceError> {
        match self {
            Value::Map(map) => {
                map.get(key).ok_or_else(|| ServiceError::TypeMismatch {
                    expected: format!("map with key \"{key}\""),
                    found: format!("{self:?}"),
                })
            }
            other => Err(Self::type_mismatch("map", other)),
        }
    }

    /// Helper to create a `TypeMismatch` error with automatic formatting.
    pub(crate) fn type_mismatch(expected: &str, found: &Self) -> ServiceError {
        ServiceError::TypeMismatch {
            expected: expected.into(),
            found: format!("{found:?}"),
        }
    }
}

// ── Conversions from primitives ────────────────────────────────────

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::U32(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I32(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::F64(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

impl<K, V> From<HashMap<K, V>> for Value
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from(map: HashMap<K, V>) -> Self {
        Value::Map(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_u32_from_u32() {
        assert_eq!(Value::U32(42).as_u32().unwrap(), 42);
    }

    #[test]
    fn test_as_u32_from_i32_positive() {
        assert_eq!(Value::I32(42).as_u32().unwrap(), 42);
    }

    #[test]
    fn test_as_u32_from_i32_negative() {
        assert!(Value::I32(-1).as_u32().is_err());
    }

    #[test]
    fn test_as_u32_from_string() {
        assert_eq!(Value::String("42".into()).as_u32().unwrap(), 42);
    }

    #[test]
    fn test_as_u32_from_string_invalid() {
        assert!(Value::String("hello".into()).as_u32().is_err());
    }

    #[test]
    fn test_as_u32_type_mismatch() {
        let err = Value::Bool(true).as_u32().unwrap_err();
        assert!(matches!(err, ServiceError::TypeMismatch { .. }));
    }

    #[test]
    fn test_as_str_ok() {
        assert_eq!(Value::String("hello".into()).as_str().unwrap(), "hello");
    }

    #[test]
    fn test_as_str_type_mismatch() {
        assert!(Value::U32(42).as_str().is_err());
    }

    #[test]
    fn test_as_array_ok() {
        let arr = Value::Array(vec![Value::U32(1), Value::U32(2)]);
        assert_eq!(arr.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_as_array_empty() {
        let arr = Value::Array(vec![]);
        assert!(arr.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_get_from_map() {
        let mut map = HashMap::new();
        map.insert("id".into(), Value::U32(42));
        let val = Value::Map(map);
        assert_eq!(val.get("id").unwrap().as_u32().unwrap(), 42);
    }

    #[test]
    fn test_get_missing_key() {
        let map = Value::Map(HashMap::new());
        assert!(map.get("missing").is_err());
    }

    #[test]
    fn test_get_from_non_map() {
        assert!(Value::U32(42).get("id").is_err());
    }

    #[test]
    fn test_from_primitives() {
        assert_eq!(Value::from(42u32), Value::U32(42));
        assert_eq!(Value::from(-1i32), Value::I32(-1));
        assert_eq!(Value::from(3.14f64), Value::F64(3.14));
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from("hello"), Value::String("hello".into()));
    }

    #[test]
    fn test_as_i32_from_i32() {
        assert_eq!(Value::I32(42).as_i32().unwrap(), 42);
        assert_eq!(Value::I32(-5).as_i32().unwrap(), -5);
    }

    #[test]
    fn test_as_i32_from_u32() {
        assert_eq!(Value::U32(42).as_i32().unwrap(), 42);
        assert!(Value::U32(u32::MAX).as_i32().is_err());
    }

    #[test]
    fn test_as_f64_from_f64() {
        assert!((Value::F64(3.14).as_f64().unwrap() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_as_f64_from_i32() {
        assert!((Value::I32(5).as_f64().unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_as_bytes() {
        let bytes = vec![1u8, 2, 3];
        assert_eq!(Value::Bytes(bytes).as_bytes().unwrap(), &[1, 2, 3]);
        assert!(Value::U32(42).as_bytes().is_err());
    }

    #[test]
    fn test_as_map_ok() {
        let mut map = HashMap::new();
        map.insert("a".into(), Value::U32(1));
        map.insert("b".into(), Value::U32(2));
        let val = Value::Map(map);
        let m = val.as_map().unwrap();
        assert!(m.contains_key("a"));
        assert!(m.contains_key("b"));
    }

    #[test]
    fn test_as_map_type_mismatch() {
        assert!(Value::U32(42).as_map().is_err());
    }

    #[test]
    fn test_as_bool_from_bool() {
        assert_eq!(Value::Bool(true).as_bool().unwrap(), true);
    }

    #[test]
    fn test_as_bool_from_u32() {
        assert_eq!(Value::U32(1).as_bool().unwrap(), true);
        assert_eq!(Value::U32(0).as_bool().unwrap(), false);
    }

    #[test]
    fn test_as_bool_from_string() {
        assert_eq!(Value::String("true".into()).as_bool().unwrap(), true);
        assert_eq!(Value::String("false".into()).as_bool().unwrap(), false);
    }
}
