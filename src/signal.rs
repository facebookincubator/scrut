/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#[cfg(unix)]
pub use nix_signal::KillSignal;
#[cfg(windows)]
pub use win_signal::KillSignal;

#[cfg(unix)]
mod nix_signal {
    use std::fmt;
    use std::fmt::Display;
    use std::str::FromStr;

    use anyhow::Result;
    use anyhow::anyhow;
    use nix::sys::signal;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;
    use serde::de;
    use serde::de::Visitor;

    /// Convinience trait to get the short name of a nix Signal
    trait ShortSignalName {
        fn short_name(&self) -> String;
    }

    impl ShortSignalName for signal::Signal {
        /// Returns the name of the signal without the "SIG" prefix and in lower case ("SIGINT" -> "int")
        fn short_name(&self) -> String {
            let name = self.as_str().to_lowercase();
            match name.strip_prefix("sig") {
                Some(prefix) => prefix.to_string(),
                None => name,
            }
        }
    }

    /// Nix signal container that capture whether and which OS signal to send to
    /// executions of shell expressions that are marked as detached.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum KillSignal {
        Disabled,
        Enabled(signal::Signal),
    }

    impl KillSignal {
        #[cfg(test)]
        pub fn test_default() -> Self {
            Self::Enabled(signal::SIGQUIT)
        }

        pub fn is_off(&self) -> bool {
            matches!(self, KillSignal::Disabled)
        }

        pub fn to_nix(&self) -> Result<signal::Signal> {
            match self {
                KillSignal::Disabled => Err(anyhow!("Cannot convert off to signal")),
                KillSignal::Enabled(signal) => Ok(*signal),
            }
        }
    }

    impl Default for KillSignal {
        fn default() -> Self {
            Self::Enabled(signal::SIGTERM)
        }
    }

    impl Display for KillSignal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    KillSignal::Disabled => "disabled".to_string(),
                    KillSignal::Enabled(signal) => signal.short_name(),
                }
            )
        }
    }

    impl Serialize for KillSignal {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                KillSignal::Disabled => serializer.serialize_str("disabled"),
                KillSignal::Enabled(signal) => serializer.serialize_str(&signal.short_name()),
            }
        }
    }

    impl<'de> Deserialize<'de> for KillSignal {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NixSignalVisitor;

            impl<'de> Visitor<'de> for NixSignalVisitor {
                type Value = KillSignal;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("i32 or str")
                }

                fn visit_i32<E>(self, value: i32) -> Result<KillSignal, E>
                where
                    E: de::Error,
                {
                    signal::Signal::try_from(value).map_or_else(
                        |e| {
                            Err(de::Error::custom(format!(
                                "Invalid integer signal value {}: {}",
                                value, e
                            )))
                        },
                        |s| Ok(KillSignal::Enabled(s)),
                    )
                }

                fn visit_i64<E>(self, value: i64) -> Result<KillSignal, E>
                where
                    E: de::Error,
                {
                    if value < i32::MIN as i64 || value > i32::MAX as i64 {
                        return Err(de::Error::custom(format!(
                            "Value out of range for i32: {}",
                            value
                        )));
                    }
                    self.visit_i32(value as i32)
                }

                fn visit_u64<E>(self, value: u64) -> Result<KillSignal, E>
                where
                    E: de::Error,
                {
                    if value > i32::MAX as u64 {
                        return Err(de::Error::custom(format!(
                            "Value out of range for i32: {}",
                            value
                        )));
                    }
                    self.visit_i32(value as i32)
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match &value.to_lowercase() as &str {
                        "disabled" => Ok(KillSignal::Disabled),
                        _ => match value.parse::<i32>() {
                            Ok(value) => self.visit_i32(value),
                            Err(_) => {
                                let value = value.to_uppercase();
                                let name = match value.strip_prefix("SIG") {
                                    Some(_) => value,
                                    None => format!("SIG{value}"),
                                };
                                signal::Signal::from_str(&name).map_or_else(
                                    |e| {
                                        Err(de::Error::custom(format!(
                                            "Invalid signal name {}: {}",
                                            name, e
                                        )))
                                    },
                                    |s| Ok(KillSignal::Enabled(s)),
                                )
                            }
                        },
                    }
                }
            }

            deserializer.deserialize_any(NixSignalVisitor)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_deserialize_json() {
            let cases = [
                ("1", KillSignal::Enabled(signal::SIGHUP)),
                ("6", KillSignal::Enabled(signal::SIGABRT)),
                ("\"abrt\"", KillSignal::Enabled(signal::SIGABRT)),
                ("\"sigabrt\"", KillSignal::Enabled(signal::SIGABRT)),
                ("\"ABRT\"", KillSignal::Enabled(signal::SIGABRT)),
                ("\"SIGABRT\"", KillSignal::Enabled(signal::SIGABRT)),
                ("\"disabled\"", KillSignal::Disabled),
            ];
            for (input, expected) in cases {
                let value: KillSignal =
                    serde_json::from_str(input).expect(&format!("deserialize from '{input}'"));
                assert_eq!(value, expected);
            }
        }

        #[test]
        fn test_serialize_json() {
            let cases = [
                (KillSignal::Enabled(signal::SIGHUP), "\"hup\""),
                (KillSignal::Enabled(signal::SIGABRT), "\"abrt\""),
                (KillSignal::Disabled, "\"disabled\""),
            ];
            for (input, expected) in cases {
                let value = serde_json::to_string(&input)
                    .expect(&format!("serialize '{}'", input.to_string()));
                assert_eq!(&value as &str, expected);
            }
        }
    }
}

#[cfg(windows)]
mod win_signal {
    use std::fmt;
    use std::fmt::Display;

    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;
    use serde::de;
    use serde::de::Visitor;

    /// List of supported signals on Linux, so that Windows can validate them
    const SUPPORTED_SIGNALS: &[&str] = &[
        "hup", "int", "quit", "ill", "trap", "abrt", "bus", "fpe", "kill", "usr1", "segv", "usr2",
        "pipe", "alrm", "term", "chld", "cont", "stop", "tstp", "ttin", "ttou", "urg", "xcpu",
        "xfsz", "vtalrm", "prof", "winch", "sys",
    ];

    /// Windows "fake signal" container. It is currently only used to support the configuration
    /// type, so that updating or running tests on Windows does not fail with an error about
    /// invalid configuration.
    /// This also makes sure that only (in *nix) valid signal values can be used.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum KillSignal {
        Disabled,
        Enabled(String),
    }

    impl KillSignal {
        #[cfg(test)]
        pub fn test_default() -> Self {
            Self::Enabled("quit".to_string())
        }

        pub fn is_off(&self) -> bool {
            matches!(self, KillSignal::Disabled)
        }
    }

    impl Default for KillSignal {
        fn default() -> Self {
            Self::Enabled("term".to_string())
        }
    }

    impl Display for KillSignal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    KillSignal::Enabled(signal) => signal,
                    KillSignal::Disabled => "disabled",
                }
            )
        }
    }

    impl Serialize for KillSignal {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                KillSignal::Enabled(signal) => serializer.serialize_str(signal),
                KillSignal::Disabled => serializer.serialize_str("disabled"),
            }
        }
    }

    impl<'de> Deserialize<'de> for KillSignal {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct WinSignalVisitor;

            impl<'de> Visitor<'de> for WinSignalVisitor {
                type Value = KillSignal;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("i32 or str")
                }

                fn visit_i64<E>(self, value: i64) -> Result<KillSignal, E>
                where
                    E: de::Error,
                {
                    Ok(KillSignal::Enabled(value.to_string()))
                }

                fn visit_u64<E>(self, value: u64) -> Result<KillSignal, E>
                where
                    E: de::Error,
                {
                    Ok(KillSignal::Enabled(value.to_string()))
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match &value.to_lowercase() as &str {
                        "disabled" => Ok(KillSignal::Disabled),
                        name => {
                            let name = if let Some(suffix) = name.strip_prefix("sig") {
                                suffix.to_string()
                            } else {
                                name.to_string()
                            };
                            if SUPPORTED_SIGNALS.contains(&name.as_str()) {
                                Ok(KillSignal::Enabled(name.clone()))
                            } else {
                                Err(de::Error::custom(format!(
                                    "Invalid signal name {value}: not in list of supported signals",
                                )))
                            }
                        }
                    }
                }
            }

            deserializer.deserialize_any(WinSignalVisitor)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_deserialize_json() {
            let cases = [
                ("\"abrt\"", KillSignal::Enabled("abrt".to_string())),
                ("\"sigabrt\"", KillSignal::Enabled("abrt".to_string())),
                ("\"ABRT\"", KillSignal::Enabled("abrt".to_string())),
                ("\"SIGABRT\"", KillSignal::Enabled("abrt".to_string())),
                ("\"disabled\"", KillSignal::Disabled),
            ];
            for (input, expected) in cases {
                let value: KillSignal =
                    serde_json::from_str(input).expect(&format!("deserialize from '{input}'"));
                assert_eq!(value, expected);
            }
        }

        #[test]
        fn test_serialize_json() {
            let cases = [
                (KillSignal::Enabled("hup".to_string()), "\"hup\""),
                (KillSignal::Enabled("abrt".to_string()), "\"abrt\""),
                (KillSignal::Disabled, "\"disabled\""),
            ];
            for (input, expected) in cases {
                let value = serde_json::to_string(&input).expect(&format!("serialize '{input}'"));
                assert_eq!(value, expected);
            }
        }
    }
}
