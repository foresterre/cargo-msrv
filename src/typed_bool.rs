use serde::Deserializer;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct True;

impl serde::Serialize for True {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

impl<'de> serde::Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<True, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(TrueVisitor)
    }
}

struct TrueVisitor;

impl<'de> serde::de::Visitor<'de> for TrueVisitor {
    type Value = True;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("true")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value {
            Ok(True)
        } else {
            Err(E::custom(format!("Value '{}' must be 'true'", value)))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct False;

impl serde::Serialize for False {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(false)
    }
}

impl<'de> serde::Deserialize<'de> for False {
    fn deserialize<D>(deserializer: D) -> Result<False, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(FalseVisitor)
    }
}

struct FalseVisitor;

impl<'de> serde::de::Visitor<'de> for FalseVisitor {
    type Value = False;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("false")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if !value {
            Ok(False)
        } else {
            Err(E::custom(format!("Value '{}' must be 'false'", value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_false() {
        let serialized = serde_json::to_value(False).unwrap();

        assert!(matches!(serialized, serde_json::Value::Bool(false)));
    }

    #[test]
    fn serialize_true() {
        let serialized = serde_json::to_value(True).unwrap();

        assert!(matches!(serialized, serde_json::Value::Bool(true)));
    }

    #[test]
    fn deserialize_false() {
        let value = serde_json::from_str("false").unwrap();

        assert!(matches!(value, False));
    }

    #[test]
    fn deserialize_true() {
        let value = serde_json::from_str("true").unwrap();

        assert!(matches!(value, True));
    }

    #[test]
    fn deserialize_false_failed() {
        let error = serde_json::from_str::<False>("true").unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("Value 'true' must be 'false'"),)
    }

    #[test]
    fn deserialize_true_failed() {
        let error = serde_json::from_str::<True>("false").unwrap_err();
        let error_message = format!("{}", error);

        assert!(error_message.contains("Value 'false' must be 'true'"),)
    }
}
