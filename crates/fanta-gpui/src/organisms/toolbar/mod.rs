//! Headless, host-controlled editor toolbar with Figma-like tool flyouts,
//! mode switching, Actions search, and contextual drawing controls.

mod commands;
mod draw_model;
pub use draw_model::{
    DrawBrushCapabilities, DrawSelectionCapabilities, DrawSelectionOperation, DrawToolbarAction,
    DrawToolbarOptions,
};
mod component;
mod icons;
mod model;

pub use commands::{
    CloseToolbarOverlay, ConfirmToolbarTextEntry, DecrementToolbarControl, EnterDevMode,
    FirstToolbarCommand, IncrementToolbarControl, LastToolbarCommand, NextToolbarCommand,
    OpenToolbarActions, PreviousToolbarCommand, SelectAnnotationTool, SelectArrowTool,
    SelectBrushTool, SelectCommentTool, SelectEllipseTool, SelectEraserTool, SelectFrameTool,
    SelectHandTool, SelectImageVideoTool, SelectLineTool, SelectMarqueeTool, SelectMeasureTool,
    SelectMoveTool, SelectPenTool, SelectPencilTool, SelectRectangleTool, SelectSectionTool,
    SelectSliceTool, SelectTextTool, SelectWandTool, ZoomCanvasTo100, ZoomCanvasToFit,
    ZoomCanvasToSelection,
};
pub use component::EditorToolbar;
pub use model::{
    DevToolbarOptions, MotionToolbarOptions, ToolbarAction, ToolbarChromeControl, ToolbarCommand,
    ToolbarControlValue, ToolbarItem, ToolbarMode, ToolbarSecondaryControl, ToolbarTool,
    ToolbarToolGroup,
};

pub(crate) fn init(cx: &mut gpui::App) {
    commands::init(cx);
}

pub(crate) use commands::TOOLBAR_TEXT_ENTRY_KEY_CONTEXT;
pub(crate) use icons::render_tool_icon;
