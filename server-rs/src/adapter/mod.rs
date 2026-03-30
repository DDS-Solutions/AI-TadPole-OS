//! External infrastructure adapters and service bridges.
//!
//! Adapters provide standardized interfaces for the engine to interact with
//! external resources such as the local filesystem, secret vaults, and
//! notification platforms like Discord.

pub mod discord;
pub mod filesystem;
#[cfg(test)]
mod tests_filesystem;
pub mod vault;
