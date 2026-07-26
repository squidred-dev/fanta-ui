#![forbid(unsafe_code)]
//! Headless, host-controlled GPUI components for Fanta applications.
//!
//! This crate owns presentation and typed UI intents only. A host supplies
//! immutable view data, keeps domain state, and maps intents to Fanta
//! operations. See the workspace `ARCHITECTURE.md` §2–§4.

pub mod pages;

/// Common imports for Fanta GPUI hosts.
pub mod prelude {
    pub use crate::pages::{
        PagesPanel, PagesPanelAction, PagesPanelElementCount, PagesPanelElementKind,
        PagesPanelItem, PagesPanelResultDirection, PagesPanelSearchRequest, PagesPanelSearchResult,
        PagesPanelSearchResults, PagesPanelSearchScope,
    };
}
