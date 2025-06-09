//! Task 1.2.3: JavaScript Interface Types
//!
//! This module provides JavaScript-friendly interface types that offer
//! idiomatic JavaScript APIs, Promise-based operations, and modern JS patterns.

pub mod core;
pub mod systems;
pub mod utils;
pub mod async_ops;
pub mod events;
pub mod validation;
pub mod i18n;
pub mod performance;
pub mod compat;

// Re-export all public interfaces
pub use core::*;
pub use systems::*;
pub use utils::*;
pub use async_ops::*;
pub use events::*;
pub use validation::*;
pub use i18n::*;
pub use performance::*;
pub use compat::*;