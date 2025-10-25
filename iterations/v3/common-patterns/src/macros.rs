//! Common macro patterns and utilities

/// Macro for implementing common traits for configuration structs
#[macro_export]
macro_rules! impl_config_traits {
    ($struct_name:ident) => {
        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Redact sensitive fields in debug output
                f.debug_struct(stringify!($struct_name))
                    .field("redacted", &"[REDACTED]")
                    .finish()
            }
        }

        impl Clone for $struct_name {
            fn clone(&self) -> Self {
                // Implement custom clone that handles sensitive data appropriately
                Self {
                    ..*self
                }
            }
        }
    };
}

/// Macro for creating error types with common patterns
#[macro_export]
macro_rules! define_error {
    ($error_name:ident, $($variant:ident => $message:expr),* $(,)?) => {
        #[derive(Debug, Clone, thiserror::Error)]
        pub enum $error_name {
            $(
                #[error($message)]
                $variant,
            )*
            #[error("Unknown error: {0}")]
            Unknown(String),
        }

        impl $error_name {
            /// Create an unknown error variant
            pub fn unknown(message: impl Into<String>) -> Self {
                Self::Unknown(message.into())
            }
        }
    };
}

/// Macro for implementing common component traits
#[macro_export]
macro_rules! impl_component_traits {
    ($struct_name:ident) => {
        impl crate::traits::StatusReporter for $struct_name {
            fn status(&self) -> crate::types::ComponentStatus {
                crate::types::ComponentStatus {
                    name: stringify!($struct_name).to_string(),
                    health: crate::types::HealthStatus::Healthy,
                    last_checked: chrono::Utc::now(),
                    details: std::collections::HashMap::new(),
                }
            }

            fn status_details(&self) -> anyhow::Result<serde_json::Value> {
                Ok(serde_json::json!({
                    "component": stringify!($struct_name),
                    "status": "operational"
                }))
            }
        }
    };
}

/// Macro for implementing validation traits
#[macro_export]
macro_rules! impl_validation_traits {
    ($struct_name:ident) => {
        impl crate::traits::Validatable for $struct_name {
            fn validate(&self) -> anyhow::Result<crate::types::ValidationResult> {
                // Basic validation - override in implementation
                Ok(crate::types::ValidationResult::success())
            }
        }
    };
}

/// Macro for creating builder patterns
#[macro_export]
macro_rules! builder_pattern {
    ($struct_name:ident, $($field:ident: $type:ty),* $(,)?) => {
        impl $struct_name {
            /// Create a new builder
            pub fn builder() -> paste::paste! { [<$struct_name Builder>] } {
                paste::paste! { [<$struct_name Builder>] } {
                    $(
                        $field: None,
                    )*
                }
            }
        }

        paste::paste! {
            /// Builder for $struct_name
            pub struct [<$struct_name Builder>] {
                $(
                    $field: Option<$type>,
                )*
            }

            impl [<$struct_name Builder>] {
                $(
                    pub fn $field(mut self, value: $type) -> Self {
                        self.$field = Some(value);
                        self
                    }
                )*

                pub fn build(self) -> anyhow::Result<$struct_name> {
                    Ok($struct_name {
                        $(
                            $field: self.$field.ok_or_else(|| anyhow::anyhow!(concat!("Missing required field: ", stringify!($field))))?,
                        )*
                    })
                }
            }
        }
    };
}

/// Macro for implementing cache key generation
#[macro_export]
macro_rules! impl_cache_key {
    ($struct_name:ident, $key_expr:expr) => {
        impl crate::traits::Cacheable for $struct_name {
            fn cache_key(&self) -> String {
                $key_expr.to_string()
            }
        }
    };
}

/// Macro for implementing metrics traits
#[macro_export]
macro_rules! impl_metrics_provider {
    ($struct_name:ident) => {
        impl crate::traits::MetricsProvider for $struct_name {
            fn get_metrics(&self) -> anyhow::Result<serde_json::Value> {
                Ok(serde_json::json!({
                    "component": stringify!($struct_name),
                    "metrics": {}
                }))
            }

            fn get_metrics_snapshot(&self) -> anyhow::Result<crate::types::MetricsSnapshot> {
                Ok(crate::types::MetricsSnapshot {
                    component: stringify!($struct_name).to_string(),
                    timestamp: chrono::Utc::now(),
                    metrics: std::collections::HashMap::new(),
                })
            }
        }
    };
}

/// Macro for creating typed IDs
#[macro_export]
macro_rules! define_id_type {
    ($id_name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $id_name(pub uuid::Uuid);

        impl $id_name {
            /// Generate a new random ID
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            /// Create from UUID
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// Create from string (with validation)
            pub fn from_string(s: &str) -> anyhow::Result<Self> {
                let uuid = uuid::Uuid::parse_str(s)?;
                Ok(Self(uuid))
            }

            /// Get as string
            pub fn as_string(&self) -> String {
                self.0.to_string()
            }

            /// Get inner UUID
            pub fn inner(&self) -> uuid::Uuid {
                self.0
            }
        }

        impl std::fmt::Display for $id_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl Default for $id_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<uuid::Uuid> for $id_name {
            fn from(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$id_name> for uuid::Uuid {
            fn from(id: $id_name) -> Self {
                id.0
            }
        }
    };
}

/// Macro for creating event types
#[macro_export]
macro_rules! define_event {
    ($event_name:ident { $($variant:ident $({ $($field:ident: $type:ty),* $(,)? })?),* $(,)? }) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub enum $event_name {
            $(
                $variant $({
                    $($field: $type,)*
                })?,
            )*
        }

        impl $event_name {
            /// Get event type name
            pub fn event_type(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }
        }
    };
}

/// Macro for creating result types with common patterns
#[macro_export]
macro_rules! define_result_type {
    ($result_name:ident<$data:ident>) => {
        pub type $result_name = crate::types::OperationResult<$data>;
    };
}
