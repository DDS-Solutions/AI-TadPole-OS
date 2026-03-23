//! Autonomous agent orchestration and reasoning system.
//!
//! This module serves as the primary namespace for all agent-related logic,
//! including LLM provider integrations, mission lifecycle management, 
//! memory persistence, and the high-concurrency swarm runner.

pub mod audio;
pub mod audio_cache;
pub mod benchmarks;
pub mod script_skills;
pub mod continuity;
pub mod gemini;
pub mod groq;
pub mod hooks;
pub mod mcp;
#[cfg(test)]
mod mcp_tests;
pub mod memory;
pub mod mission;
pub mod null_provider;
pub mod openai;
pub mod persistence;
pub mod provider_trait;
pub mod rate_limiter;
pub mod rates;
pub mod registry;
pub mod runner;
pub mod sanitizer;
pub mod skill_manifest;
#[cfg(test)]
mod test_oversight;
#[cfg(test)]
mod test_sanitizer;
#[cfg(test)]
mod tests_rate_limiter;
#[cfg(test)]
mod tests_skills;
pub mod types;
