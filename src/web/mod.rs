//! Web layer module
//!
//! Handles HTTP routes, OAuth flow, and dashboard.

pub mod handlers;
pub mod routes;

pub use routes::create_router;
