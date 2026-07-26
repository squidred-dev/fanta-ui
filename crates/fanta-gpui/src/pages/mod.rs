//! Pages navigation components.

mod commands;
mod model;
mod panel;

pub use commands::{
    AddPage, ClosePagesSearch, FindInPages, NextSearchResult, OpenPageContextMenu,
    PreviousSearchResult, ReplaceAllResults, ReplaceCurrentResult, TogglePagesPanel,
    ToggleSearchSettings,
};
pub use model::{
    PagesPanelAction, PagesPanelElementCount, PagesPanelElementKind, PagesPanelItem,
    PagesPanelResultDirection, PagesPanelSearchRequest, PagesPanelSearchResult,
    PagesPanelSearchResults, PagesPanelSearchScope,
};
pub use panel::PagesPanel;

pub(crate) use commands::{
    ActivatePagesControl, PAGES_CONTROL_KEY_CONTEXT, PAGES_PANEL_KEY_CONTEXT,
};

pub(crate) fn init(cx: &mut gpui::App) {
    commands::init(cx);
}
