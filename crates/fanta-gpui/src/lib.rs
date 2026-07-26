#![forbid(unsafe_code)]
//! Headless, host-controlled GPUI components for Fanta applications.
//!
//! This crate owns presentation and typed UI intents only. A host supplies
//! immutable view data, keeps domain state, and maps intents to Fanta
//! operations. See the workspace `ARCHITECTURE.md` §2–§4.

pub mod pages;

/// Registers Fanta GPUI commands and their default key bindings.
///
/// Call this once after `gpui_component::init`.
pub fn init(cx: &mut gpui::App) {
    pages::init(cx);
}

/// Common imports for Fanta GPUI hosts.
pub mod prelude {
    pub use crate::pages::{
        AddPage, ClosePagesSearch, FindInPages, NextSearchResult, OpenPageContextMenu, PagesPanel,
        PagesPanelAction, PagesPanelElementCount, PagesPanelElementKind, PagesPanelItem,
        PagesPanelResultDirection, PagesPanelSearchRequest, PagesPanelSearchResult,
        PagesPanelSearchResults, PagesPanelSearchScope, PreviousSearchResult, ReplaceAllResults,
        ReplaceCurrentResult, TogglePagesPanel, ToggleSearchSettings,
    };
}
