/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Factory registry for ManagedResource implementations
//!
//! This module provides a way to register factory functions that can create
//! ManagedResource instances from resource IDs. This is used by the `--with-setup`
//! flag to provision resources that tests declare they need.
//!
//! # Example
//!
//! ```rust,ignore
//! use scrutty::resources::{register_resource_factory, ManagedResource};
//!
//! // Register a factory that handles "mast:*" resource IDs
//! register_resource_factory(|id| {
//!     if id.starts_with("mast:") {
//!         Some(Box::new(MastJobResource::new(id)))
//!     } else {
//!         None
//!     }
//! });
//! ```

use std::sync::Mutex;

use super::ManagedResource;

/// Type for resource factory functions
///
/// A factory receives a resource ID and returns Some(resource) if it can
/// handle that ID, or None otherwise.
pub type ResourceFactory = fn(&str) -> Option<Box<dyn ManagedResource>>;

// Global registry of resource factories
static FACTORIES: Mutex<Vec<ResourceFactory>> = Mutex::new(Vec::new());

/// Register a factory function that can create ManagedResource instances
///
/// The factory receives a resource ID and returns Some(resource) if it can
/// handle that ID, or None otherwise. Factories are tried in registration order.
///
/// # Example
///
/// ```rust,ignore
/// register_resource_factory(|id| {
///     if id.starts_with("mast:") {
///         Some(Box::new(MastJobResource::new(id)))
///     } else {
///         None
///     }
/// });
/// ```
pub fn register_resource_factory(factory: ResourceFactory) {
    FACTORIES.lock().unwrap().push(factory);
}

/// Create a ManagedResource for the given ID using registered factories
///
/// Tries each registered factory in order until one returns Some.
/// Returns None if no factory can handle the resource ID.
pub fn create_resource(resource_id: &str) -> Option<Box<dyn ManagedResource>> {
    FACTORIES
        .lock()
        .unwrap()
        .iter()
        .find_map(|factory| factory(resource_id))
}

/// Create ManagedResources for all given IDs
///
/// Returns a vector of (resource_id, resource) pairs for IDs that have
/// registered factories. Resource IDs without factories are skipped.
pub fn create_resources(resource_ids: &[&str]) -> Vec<(String, Box<dyn ManagedResource>)> {
    resource_ids
        .iter()
        .filter_map(|id| create_resource(id).map(|resource| (id.to_string(), resource)))
        .collect()
}
