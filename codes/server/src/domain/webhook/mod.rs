//! Webhook handlers for AI automation pipeline
//!
//! This module handles incoming webhooks from:
//! - Discord (slash commands and interactions)
//! - GitHub (issues, pull requests, comments)

#[allow(dead_code)]
pub mod discord_handler;
#[allow(dead_code)]
pub mod dto;
#[allow(dead_code)]
pub mod github_handler;

#[allow(unused_imports)]
pub use discord_handler::handle_discord_webhook;
#[allow(unused_imports)]
pub use github_handler::handle_github_webhook;
