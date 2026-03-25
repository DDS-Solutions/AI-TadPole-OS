pub mod agent;
pub mod audio;
pub mod deploy;
pub mod engine_control;
pub mod error;
pub mod health;
pub mod memory;
pub mod model_manager;
pub mod oversight;
pub mod pagination;
pub mod templates;
pub mod ws;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod auth_tests;
pub mod benchmarks;
pub mod continuity;
pub mod docs;
pub mod env_schema;
pub mod mcp;
#[cfg(test)]
mod mcp_test;
pub mod nodes;
pub mod skills;
#[cfg(test)]
mod ws_tests;
