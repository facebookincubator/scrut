/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Resource manager for lifecycle orchestration and state persistence

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::ManagedResource;

/// State file name for persisted resource handles
pub const STATE_FILE_NAME: &str = ".tty_test_resources.json";

/// Persisted state for all managed resources
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ResourceState {
    /// Map of resource_id -> handle (as JSON value)
    pub handles: HashMap<String, serde_json::Value>,
    /// Map of resource_id -> error message for resources that failed setup
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub failures: HashMap<String, String>,
}

impl ResourceState {
    /// Load state from file, or return empty state if file doesn't exist
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get a handle for a resource, deserializing to the expected type
    pub fn get_handle<T: for<'de> Deserialize<'de>>(&self, resource_id: &str) -> Result<Option<T>> {
        match self.handles.get(resource_id) {
            Some(value) => Ok(Some(serde_json::from_value(value.clone())?)),
            None => Ok(None),
        }
    }
}

/// Manager for resource lifecycle orchestration
pub struct ResourceManager {
    resources: Vec<Box<dyn ManagedResource>>,
    state_path: std::path::PathBuf,
}

impl ResourceManager {
    /// Create a new resource manager with the given state file path
    pub fn new(state_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            resources: Vec::new(),
            state_path: state_path.into(),
        }
    }

    /// Create a resource manager using the default state file in current directory
    pub fn with_default_state_file() -> Self {
        Self::new(STATE_FILE_NAME)
    }

    /// Register a resource (deduplicates by resource_id)
    pub fn register(&mut self, resource: Box<dyn ManagedResource>) {
        let id = resource.resource_id();
        if !self.resources.iter().any(|r| r.resource_id() == id) {
            self.resources.push(resource);
        }
    }

    /// Get list of all resource IDs
    pub fn resource_ids(&self) -> Vec<&str> {
        self.resources.iter().map(|r| r.resource_id()).collect()
    }

    /// Setup all registered resources in parallel, persisting handles to state file
    ///
    /// Resources are set up concurrently using threads. Each resource's setup()
    /// is called in its own thread, and results are collected.
    ///
    /// This method is resilient to partial failures: if some resources succeed
    /// and others fail, the successful handles are saved to state and the method
    /// returns Ok with those handles. This allows tests to proceed with whatever
    /// resources were successfully set up. Errors are logged to stderr.
    pub fn setup_all(&self) -> Result<ResourceState> {
        use std::thread;

        let mut state = ResourceState::default();

        if self.resources.is_empty() {
            state.save(&self.state_path)?;
            return Ok(state);
        }

        // Spawn threads for each resource setup
        let handles: Vec<_> = self
            .resources
            .iter()
            .map(|resource| {
                let resource_id = resource.resource_id().to_string();
                // Use scoped threads to allow borrowing resources without moving them
                (resource_id, resource)
            })
            .collect();

        // Use scoped threads to allow borrowing self.resources
        let results: Vec<(String, Result<serde_json::Value>)> = thread::scope(|s| {
            let thread_handles: Vec<_> = handles
                .iter()
                .map(|(resource_id, resource)| {
                    let id = resource_id.clone();
                    s.spawn(move || {
                        let result = resource.setup();
                        (id, result)
                    })
                })
                .collect();

            thread_handles
                .into_iter()
                .map(|h| h.join().expect("Thread panicked during resource setup"))
                .collect()
        });

        // Collect successful results and record failures
        for (resource_id, result) in results {
            match result {
                Ok(handle) => {
                    state.handles.insert(resource_id, handle);
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    eprintln!(
                        "Warning: Failed to setup resource '{}': {}",
                        resource_id, error_msg
                    );
                    state.failures.insert(resource_id, error_msg);
                }
            }
        }

        state.save(&self.state_path)?;
        Ok(state)
    }

    /// Teardown all registered resources and remove state file
    ///
    /// This method is resilient to partial failures: if some resources fail
    /// to teardown, it continues attempting to teardown the remaining resources.
    /// Errors are logged to stderr. The state file is removed regardless of
    /// individual teardown failures.
    pub fn teardown_all(&self) -> Result<()> {
        for resource in &self.resources {
            if let Err(e) = resource.teardown() {
                eprintln!(
                    "Warning: Failed to teardown resource '{}': {:?}",
                    resource.resource_id(),
                    e
                );
            }
        }

        if self.state_path.exists() {
            std::fs::remove_file(&self.state_path)?;
        }

        Ok(())
    }

    /// Load existing state (for reading handles in tests)
    pub fn load_state(&self) -> Result<ResourceState> {
        ResourceState::load(&self.state_path)
    }
}

/// Get a resource handle by ID, deserializing from the state file
///
/// This is the primary way tests access resource handles that were set up
/// by `--with-setup` or `--setup-resources`.
///
/// # State File Location
///
/// Looks for the state file in this order:
/// 1. `TTY_TEST_RESOURCES_FILE` environment variable (for testing)
/// 2. `.tty_test_resources.json` in current directory
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct MastHandle {
///     job_name: String,
///     launched_at: String,
/// }
///
/// let handle: MastHandle = resource_handle("mast:gtn_infra")?;
/// println!("Using job: {}", handle.job_name);
/// ```
pub fn resource_handle<T: for<'de> Deserialize<'de>>(resource_id: &str) -> Result<T> {
    let state_path = std::env::var("TTY_TEST_RESOURCES_FILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(STATE_FILE_NAME));

    let state = ResourceState::load(&state_path)?;

    state
        .get_handle(resource_id)?
        .ok_or_else(|| anyhow::anyhow!("Resource '{}' not found in state file", resource_id))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use tempfile::tempdir;

    use super::*;
    use crate::prelude::ResourceCheck;

    struct CountingResource {
        id: &'static str,
        setup_count: AtomicUsize,
        teardown_count: AtomicUsize,
    }

    impl CountingResource {
        fn new(id: &'static str) -> Self {
            Self {
                id,
                setup_count: AtomicUsize::new(0),
                teardown_count: AtomicUsize::new(0),
            }
        }
    }

    impl ManagedResource for CountingResource {
        fn resource_id(&self) -> &str {
            self.id
        }

        fn setup(&self) -> Result<serde_json::Value> {
            self.setup_count.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!({"id": self.id, "count": self.setup_count.load(Ordering::SeqCst)}))
        }

        fn teardown(&self) -> Result<()> {
            self.teardown_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn check(&self) -> ResourceCheck {
            ResourceCheck::available()
        }
    }

    #[test]
    fn test_resource_manager_deduplicates() {
        let mut manager = ResourceManager::with_default_state_file();

        manager.register(Box::new(CountingResource::new("res:a")));
        manager.register(Box::new(CountingResource::new("res:a"))); // duplicate
        manager.register(Box::new(CountingResource::new("res:b")));

        let ids = manager.resource_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"res:a"));
        assert!(ids.contains(&"res:b"));
    }

    #[test]
    fn test_resource_manager_setup_teardown() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join(STATE_FILE_NAME);

        let mut manager = ResourceManager::new(&state_path);
        manager.register(Box::new(CountingResource::new("res:test")));

        // Setup
        let state = manager.setup_all().unwrap();
        assert!(state_path.exists());
        assert!(state.handles.contains_key("res:test"));

        // Verify state file content
        let loaded = ResourceState::load(&state_path).unwrap();
        assert!(loaded.handles.contains_key("res:test"));

        // Teardown
        manager.teardown_all().unwrap();
        assert!(!state_path.exists());
    }

    #[test]
    fn test_resource_state_get_handle() {
        let mut state = ResourceState::default();
        state.handles.insert(
            "test:resource".to_string(),
            serde_json::json!({"job_name": "job-123", "port": 8080}),
        );

        #[derive(Debug, Deserialize, PartialEq)]
        struct JobHandle {
            job_name: String,
            port: u16,
        }

        let handle: Option<JobHandle> = state.get_handle("test:resource").unwrap();
        assert_eq!(
            handle,
            Some(JobHandle {
                job_name: "job-123".to_string(),
                port: 8080
            })
        );

        let missing: Option<JobHandle> = state.get_handle("nonexistent").unwrap();
        assert!(missing.is_none());
    }

    struct FailingResource {
        id: &'static str,
    }

    impl FailingResource {
        fn new(id: &'static str) -> Self {
            Self { id }
        }
    }

    impl ManagedResource for FailingResource {
        fn resource_id(&self) -> &str {
            self.id
        }

        fn setup(&self) -> Result<serde_json::Value> {
            Err(anyhow::anyhow!("setup failed for {}", self.id))
        }

        fn teardown(&self) -> Result<()> {
            Ok(())
        }

        fn check(&self) -> ResourceCheck {
            ResourceCheck::available()
        }
    }

    #[test]
    fn test_setup_all_records_failures() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join(STATE_FILE_NAME);

        let mut manager = ResourceManager::new(&state_path);
        manager.register(Box::new(FailingResource::new("res:broken")));

        let state = manager.setup_all().unwrap();

        // The failing resource should be recorded in failures, not handles
        assert!(
            state.failures.contains_key("res:broken"),
            "expected failures to contain 'res:broken'"
        );
        assert!(
            !state.handles.contains_key("res:broken"),
            "expected handles NOT to contain 'res:broken'"
        );

        // Verify the error message is preserved
        let error = &state.failures["res:broken"];
        assert!(
            error.contains("setup failed"),
            "expected error to contain 'setup failed', got: {}",
            error
        );

        // Verify state file was written with failures
        let loaded = ResourceState::load(&state_path).unwrap();
        assert!(loaded.failures.contains_key("res:broken"));
    }

    #[test]
    fn test_setup_all_partial_failure() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join(STATE_FILE_NAME);

        let mut manager = ResourceManager::new(&state_path);
        manager.register(Box::new(CountingResource::new("res:good")));
        manager.register(Box::new(FailingResource::new("res:bad")));

        let state = manager.setup_all().unwrap();

        // Successful resource should be in handles
        assert!(
            state.handles.contains_key("res:good"),
            "expected handles to contain 'res:good'"
        );
        // Failed resource should be in failures
        assert!(
            state.failures.contains_key("res:bad"),
            "expected failures to contain 'res:bad'"
        );
        // Neither should be cross-contaminated
        assert!(!state.handles.contains_key("res:bad"));
        assert!(!state.failures.contains_key("res:good"));
    }

    #[test]
    fn test_resource_state_failures_serialization() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");

        // Create state with both handles and failures
        let mut state = ResourceState::default();
        state
            .handles
            .insert("res:ok".to_string(), serde_json::json!({"value": "good"}));
        state
            .failures
            .insert("res:broken".to_string(), "connection refused".to_string());
        state.save(&state_path).unwrap();

        // Reload and verify round-trip
        let loaded = ResourceState::load(&state_path).unwrap();
        assert!(loaded.handles.contains_key("res:ok"));
        assert_eq!(
            loaded.failures.get("res:broken").map(String::as_str),
            Some("connection refused")
        );
    }

    #[test]
    fn test_resource_state_backward_compat_no_failures_field() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("legacy_state.json");

        // Simulate a legacy state file that has no "failures" field
        let legacy_json = r#"{"handles":{"res:old":{"value":"legacy"}}}"#;
        std::fs::write(&state_path, legacy_json).unwrap();

        let loaded = ResourceState::load(&state_path).unwrap();
        assert!(loaded.handles.contains_key("res:old"));
        assert!(
            loaded.failures.is_empty(),
            "missing 'failures' field should deserialize as empty map"
        );
    }

    #[test]
    fn test_resource_handle_helper() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join(STATE_FILE_NAME);

        // Create state file manually
        let mut state = ResourceState::default();
        state.handles.insert(
            "mast:gtn_infra".to_string(),
            serde_json::json!({"job_name": "job-abc", "launched_at": "2026-02-04T10:00:00Z"}),
        );
        state.save(&state_path).unwrap();

        // Set env var to point to our state file
        // SAFETY: This test runs in isolation and we clean up afterwards
        unsafe {
            std::env::set_var("TTY_TEST_RESOURCES_FILE", state_path.to_str().unwrap());
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct MastHandle {
            job_name: String,
            launched_at: String,
        }

        let handle: MastHandle = resource_handle("mast:gtn_infra").unwrap();
        assert_eq!(handle.job_name, "job-abc");

        // Cleanup
        // SAFETY: Restoring environment to original state
        unsafe {
            std::env::remove_var("TTY_TEST_RESOURCES_FILE");
        }
    }
}
