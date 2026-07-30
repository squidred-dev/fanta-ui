#![forbid(unsafe_code)]
//! Engine-decoupled, host-controlled GPUI components for Fanta applications.
//!
//! This crate owns presentation and typed UI intents only. A host supplies
//! immutable view data, keeps domain state, and maps intents to Fanta
//! operations. See the workspace `ARCHITECTURE.md` §2–§4.

pub mod assets;
mod color;
mod controls;
pub mod design;
pub mod layers;
pub mod pages;
pub mod prototype;
pub mod pseudo_editor;
pub mod timeline;
pub mod toolbar;
pub mod variables;

pub use controls::ActivateControl;

/// Registers Fanta GPUI commands and their default key bindings.
///
/// Call this once after `gpui_component::init`.
pub fn init(cx: &mut gpui::App) {
    controls::init(cx);
    design::init(cx);
    layers::init(cx);
    pages::init(cx);
    toolbar::init(cx);
}

/// Common imports for Fanta GPUI hosts.
pub mod prelude {
    pub use crate::ActivateControl;
    pub use crate::assets::{
        AssetsIconAssets, AssetsLibrary, AssetsLibraryKind, AssetsPanel, AssetsPanelAction,
        AssetsRailItem, AssetsViewData,
    };
    pub use crate::design::*;
    pub use crate::layers::{
        CloseLayersOverlay, LayersPanel, LayersPanelAction, LayersPanelContextAction,
        LayersPanelDropPosition, LayersPanelItem, LayersPanelNodeKind, LayersPanelSelectionMode,
        OpenLayerContextMenu,
    };
    pub use crate::pages::{
        AddPage, ClosePagesSearch, FindInPages, NextSearchResult, OpenPageContextMenu, PagesPanel,
        PagesPanelAction, PagesPanelElementCount, PagesPanelElementKind, PagesPanelItem,
        PagesPanelResultDirection, PagesPanelSearchRequest, PagesPanelSearchResult,
        PagesPanelSearchResults, PagesPanelSearchScope, PreviousSearchResult, ReplaceAllResults,
        ReplaceCurrentResult, TogglePagesPanel, ToggleSearchSettings,
    };
    pub use crate::prototype::{
        PrototypeHint, PrototypePanel, PrototypePanelAction, PrototypePanelSurface,
        PrototypeViewData,
    };
    pub use crate::pseudo_editor::{
        PseudoEditor, PseudoEditorAction, PseudoEditorChildren, PseudoEditorLeftSurface,
        PseudoEditorRightSurface,
    };
    pub use crate::timeline::{Timeline, TimelineAction, TimelineViewData};
    pub use crate::toolbar::{
        ActivateToolbarControl, AgentToolbarOptions, CloseToolbarOverlay, DevToolbarOptions,
        DrawToolbarOptions, EditorToolbar, EnterDevMode, MotionToolbarOptions, NextToolbarCommand,
        OpenToolbarActions, OpenToolbarAgent, PreviousToolbarCommand, SelectAnnotationTool,
        SelectArrowTool, SelectCommentTool, SelectEllipseTool, SelectFrameTool, SelectHandTool,
        SelectImageVideoTool, SelectLineTool, SelectMeasureTool, SelectMoveTool, SelectPenTool,
        SelectPencilTool, SelectRectangleTool, SelectScaleTool, SelectSectionTool, SelectSliceTool,
        SelectTextTool, ToolbarAction, ToolbarCommand, ToolbarControlValue, ToolbarItem,
        ToolbarMode, ToolbarSecondaryControl, ToolbarTool, ToolbarToolGroup,
    };
    pub use crate::variables::{
        VariableKind, VariableModeValue, VariableRow, VariablesAction, VariablesCollection,
        VariablesGroup, VariablesMode, VariablesPage, VariablesViewData,
    };
}
