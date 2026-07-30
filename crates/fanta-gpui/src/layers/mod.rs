//! Figma-like document layer navigation components.

mod commands;
mod model;
mod panel;

pub use commands::{CloseLayersOverlay, OpenLayerContextMenu};
pub use model::{
    LayersPanelAction, LayersPanelContextAction, LayersPanelDropPosition, LayersPanelItem,
    LayersPanelNodeKind, LayersPanelSelectionMode,
};
pub use panel::LayersPanel;

pub(crate) use commands::{
    ActivateLayersControl, LAYERS_CONTROL_KEY_CONTEXT, LAYERS_PANEL_KEY_CONTEXT,
};

pub(crate) fn init(cx: &mut gpui::App) {
    commands::init(cx);
}
