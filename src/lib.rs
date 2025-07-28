pub mod core;
pub mod domain;
pub mod rest_api;
pub mod services;

// Re-exporting so they are available directly from the crate root
pub use domain::*;
