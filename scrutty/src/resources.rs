/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Resources module - Declarative test resource requirements
//!
//! This module provides types for declaring and checking test dependencies:
//!
//! - `TestResource` trait - Interface for resources that tests depend on
//! - `ResourceCheck` - Result of checking resource availability
//! - `ManagedResource` trait - Interface for resources with full lifecycle management
//! - `ResourceManager` - Orchestrates resource setup/teardown with state persistence
//!
//! # Example
//!
//! ```rust,ignore
//! use scrutty::resources::{TestResource, ResourceCheck};
//!
//! struct DatabaseConnection {
//!     connection_string: String,
//! }
//!
//! impl TestResource for DatabaseConnection {
//!     fn resource_type(&self) -> &'static str {
//!         "database"
//!     }
//!
//!     fn description(&self) -> String {
//!         format!("Database at {}", self.connection_string)
//!     }
//!
//!     fn check(&self) -> ResourceCheck {
//!         if can_connect(&self.connection_string) {
//!             ResourceCheck::available()
//!         } else {
//!             ResourceCheck::unavailable("Cannot connect to database")
//!         }
//!     }
//! }
//! ```

mod factory;
mod manager;

pub use factory::create_resource;
pub use factory::create_resources;
pub use factory::register_resource_factory;
pub use manager::ResourceManager;
pub use manager::ResourceState;
pub use manager::STATE_FILE_NAME;
pub use manager::resource_handle;

/// Result of checking whether a resource is available
#[derive(Debug, Clone)]
pub struct ResourceCheck {
    /// Whether the resource is available
    pub available: bool,
    /// Human-readable message about the resource status
    pub message: String,
}

impl ResourceCheck {
    /// Create a check result indicating the resource is available
    pub fn available() -> Self {
        Self {
            available: true,
            message: String::new(),
        }
    }

    /// Create a check result indicating the resource is available with a message
    pub fn available_with_message(message: impl Into<String>) -> Self {
        Self {
            available: true,
            message: message.into(),
        }
    }

    /// Create a check result indicating the resource is unavailable
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            available: false,
            message: reason.into(),
        }
    }

    /// Check if the resource is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Get the message (empty for available resources without a message)
    pub fn get_message(&self) -> &str {
        &self.message
    }
}

/// A resource that a test depends on
///
/// Implement this trait to define custom resource types that tests can depend on.
/// The test runner will check all required resources before running a test and
/// skip tests whose resources are not available.
///
/// # Example
///
/// ```rust,ignore
/// struct FileResource {
///     path: String,
/// }
///
/// impl TestResource for FileResource {
///     fn resource_type(&self) -> &'static str {
///         "file"
///     }
///
///     fn description(&self) -> String {
///         format!("File at {}", self.path)
///     }
///
///     fn check(&self) -> ResourceCheck {
///         if std::path::Path::new(&self.path).exists() {
///             ResourceCheck::available()
///         } else {
///             ResourceCheck::unavailable(format!("File not found: {}", self.path))
///         }
///     }
/// }
/// ```
pub trait TestResource: Send + Sync {
    /// Unique identifier for this resource type (e.g., "file", "job", "database")
    fn resource_type(&self) -> &'static str;

    /// Human-readable description of this specific resource instance
    fn description(&self) -> String;

    /// Check if the resource is currently available
    fn check(&self) -> ResourceCheck;

    /// Check if the resource is available (convenience method)
    fn is_available(&self) -> bool {
        self.check().available
    }

    /// Get the reason why the resource is unavailable (if it is)
    fn unavailable_reason(&self) -> Option<String> {
        let check = self.check();
        if check.available {
            None
        } else {
            Some(check.message)
        }
    }
}

/// A resource with full lifecycle management (setup, teardown, check)
///
/// Unlike `TestResource` which only checks availability, `ManagedResource`
/// can provision and clean up resources. Use this for resources that need
/// CI orchestration (e.g., MAST jobs, Docker containers, cloud resources).
pub trait ManagedResource: Send + Sync {
    /// Unique identifier for this resource (used for deduplication and state lookup)
    fn resource_id(&self) -> &str;

    /// Provision the resource and return a handle as JSON
    fn setup(&self) -> anyhow::Result<serde_json::Value>;

    /// Clean up the resource
    fn teardown(&self) -> anyhow::Result<()>;

    /// Check if the resource is currently available
    fn check(&self) -> ResourceCheck;

    /// Check if the resource is available (convenience method)
    fn is_available(&self) -> bool {
        self.check().available
    }
}

/// A simple file-based resource that checks if a file exists
///
/// This is a common resource type for tests that depend on configuration files,
/// state files, or other filesystem resources.
#[derive(Debug, Clone)]
pub struct FileResource {
    /// Path to the file
    pub path: String,
    /// Description of what this file is for
    pub description: String,
}

impl FileResource {
    /// Create a new file resource
    pub fn new(path: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            description: description.into(),
        }
    }
}

impl TestResource for FileResource {
    fn resource_type(&self) -> &'static str {
        "file"
    }

    fn description(&self) -> String {
        format!("{} ({})", self.description, self.path)
    }

    fn check(&self) -> ResourceCheck {
        if std::path::Path::new(&self.path).exists() {
            ResourceCheck::available_with_message(format!("File exists: {}", self.path))
        } else {
            ResourceCheck::unavailable(format!("File not found: {}", self.path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockManagedResource {
        id: &'static str,
        setup_called: std::sync::atomic::AtomicBool,
        teardown_called: std::sync::atomic::AtomicBool,
    }

    impl MockManagedResource {
        fn new(id: &'static str) -> Self {
            Self {
                id,
                setup_called: std::sync::atomic::AtomicBool::new(false),
                teardown_called: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    impl ManagedResource for MockManagedResource {
        fn resource_id(&self) -> &str {
            self.id
        }

        fn setup(&self) -> anyhow::Result<serde_json::Value> {
            self.setup_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(serde_json::json!({"mock": "handle"}))
        }

        fn teardown(&self) -> anyhow::Result<()> {
            self.teardown_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn check(&self) -> ResourceCheck {
            ResourceCheck::available()
        }
    }

    #[test]
    fn test_managed_resource_lifecycle() {
        let resource = MockManagedResource::new("test:resource");

        assert_eq!(resource.resource_id(), "test:resource");
        assert!(
            !resource
                .setup_called
                .load(std::sync::atomic::Ordering::SeqCst)
        );

        let handle = resource.setup().unwrap();
        assert!(
            resource
                .setup_called
                .load(std::sync::atomic::Ordering::SeqCst)
        );
        assert_eq!(handle, serde_json::json!({"mock": "handle"}));

        resource.teardown().unwrap();
        assert!(
            resource
                .teardown_called
                .load(std::sync::atomic::Ordering::SeqCst)
        );
    }
}
