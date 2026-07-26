//! Pages navigation components.

mod model;
mod panel;

pub use model::{
    PagesPanelAction, PagesPanelElementCount, PagesPanelElementKind, PagesPanelItem,
    PagesPanelResultDirection, PagesPanelSearchRequest, PagesPanelSearchResult,
    PagesPanelSearchResults, PagesPanelSearchScope,
};
pub use panel::PagesPanel;
