//! Shared serde helpers that reproduce Go's `encoding/json` behavior.

use std::fmt;

use serde::{Deserialize, Deserializer};

/// Go's `omitempty`: leave the field out when it holds its zero value.
pub(crate) fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// Go writes a nil slice or map as `null`. Read that as an empty collection.
pub(crate) fn null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// A payload that breaks its wire contract. The text matches the Go package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(String);

impl ValidationError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub(crate) fn field(self, name: &str) -> Self {
        Self(format!("{name}: {}", self.0))
    }

    pub(crate) fn invalid(self, name: &str) -> Self {
        Self(format!("invalid {name}: {}", self.0))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

/// Bounds one text field: required or not, a byte limit, no control characters.
pub(crate) fn validate_wire_string(
    value: &str,
    max_bytes: usize,
    required: bool,
) -> Result<(), ValidationError> {
    if required && value.trim().is_empty() {
        return Err(ValidationError::new("is required"));
    }
    if value.len() > max_bytes {
        return Err(ValidationError::new(format!("exceeds {max_bytes} bytes")));
    }
    if value.chars().any(|c| c < '\u{20}' || c == '\u{7f}') {
        return Err(ValidationError::new("contains a control character"));
    }
    Ok(())
}

/// Declares a string type with a closed set of wire values. Like its Go
/// counterpart it can hold any text, so decoding never fails on a new value;
/// `is_valid` reports whether the value is in the contract.
macro_rules! wire_string {
    (
        $(#[$meta:meta])*
        $name:ident $($or_empty:ident)? {
            $($(#[$value_meta:meta])* $constant:ident = $value:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub std::borrow::Cow<'static, str>);

        impl $name {
            $(
                $(#[$value_meta])*
                pub const $constant: Self = Self(std::borrow::Cow::Borrowed($value));
            )+

            /// Every value in the wire contract.
            pub const VALUES: &'static [&'static str] = &[$($value),+];

            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Reports whether the value is part of the wire contract.
            pub fn is_valid(&self) -> bool {
                $(
                    if stringify!($or_empty) == "or_empty" && self.0.is_empty() {
                        return true;
                    }
                )?
                Self::VALUES.contains(&self.as_str())
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(std::borrow::Cow::Owned(value.to_owned()))
            }
        }
    };
}

pub(crate) use wire_string;
