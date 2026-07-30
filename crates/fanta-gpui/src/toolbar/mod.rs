//! Headless, host-controlled editor toolbar with Figma-like tool flyouts,
//! mode switching, Actions search, and contextual Agent prompting.

mod commands;
mod component;
mod model;

pub use commands::{
    ActivateToolbarControl, CloseToolbarOverlay, EnterDevMode, NextToolbarCommand,
    OpenToolbarActions, OpenToolbarAgent, PreviousToolbarCommand, SelectAnnotationTool,
    SelectArrowTool, SelectCommentTool, SelectEllipseTool, SelectFrameTool, SelectHandTool,
    SelectImageVideoTool, SelectLineTool, SelectMeasureTool, SelectMoveTool, SelectPenTool,
    SelectPencilTool, SelectRectangleTool, SelectScaleTool, SelectSectionTool, SelectSliceTool,
    SelectTextTool,
};
pub use component::EditorToolbar;
pub use model::{
    AgentToolbarOptions, DevToolbarOptions, DrawToolbarOptions, MotionToolbarOptions,
    ToolbarAction, ToolbarCommand, ToolbarControlValue, ToolbarItem, ToolbarMode,
    ToolbarSecondaryControl, ToolbarTool, ToolbarToolGroup,
};

pub(crate) fn init(cx: &mut gpui::App) {
    commands::init(cx);
}
