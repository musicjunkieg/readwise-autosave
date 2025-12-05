//! Readwise API module
//!
//! Handles saving highlights and documents to Readwise.

pub mod client;
pub mod highlights;
pub mod reader;

pub use client::ReadwiseClient;
