use serde::Deserializer;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
