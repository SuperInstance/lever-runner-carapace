//! # lever-runner-carapace
//!
//! Native performance layer for lever-runner.
//! Replaces Python hot paths with pure Rust.

pub mod embed;
pub mod gate;
pub mod hash;
pub mod nail;
pub mod search;
pub mod skill;
pub mod trust;

pub use embed::embed_intent;
pub use gate::{GateResult, GatePipeline};
pub use hash::hash_intent;
pub use search::cosine_search;
pub use skill::SkillPack;
pub use trust::TrustTracker;
