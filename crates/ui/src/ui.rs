//! # UI – Zed UI Primitives & Components
//!
//! This crate provides a set of UI primitives and components that are used to build all of the elements in Zed's UI.
//!
//! ## Related Crates:
//!
//! - [`ui_macros`] - proc_macros support for this crate
//! - `ui_input` - the single line input component

// Keep the imported Zed component API unchanged during extraction; these lints
// describe its existing public shapes and implementation, not new code here.
#![allow(
    clippy::assign_op_pattern,
    clippy::collapsible_if,
    clippy::from_over_into,
    clippy::items_after_test_module,
    clippy::large_enum_variant,
    clippy::module_inception,
    clippy::new_ret_no_self,
    clippy::new_without_default,
    clippy::obfuscated_if_else,
    clippy::redundant_closure,
    clippy::redundant_pattern_matching,
    clippy::too_many_arguments,
    clippy::type_complexity
)]

pub mod component_prelude;
mod components;
pub mod prelude;
mod styles;
mod traits;
pub mod utils;

pub use components::*;
pub use prelude::*;
pub use styles::*;
pub use traits::animation_ext::*;
