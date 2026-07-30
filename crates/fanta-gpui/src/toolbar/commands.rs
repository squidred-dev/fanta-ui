use gpui::{App, KeyBinding, actions};

actions!(
    fanta_toolbar,
    [
        ActivateToolbarControl,
        CloseToolbarOverlay,
        NextToolbarCommand,
        OpenToolbarActions,
        OpenToolbarAgent,
        PreviousToolbarCommand,
        SelectMoveTool,
        SelectHandTool,
        SelectScaleTool,
        SelectFrameTool,
        SelectSectionTool,
        SelectSliceTool,
        SelectRectangleTool,
        SelectLineTool,
        SelectArrowTool,
        SelectEllipseTool,
        SelectImageVideoTool,
        SelectPenTool,
        SelectPencilTool,
        SelectTextTool,
        SelectCommentTool,
        SelectAnnotationTool,
        SelectMeasureTool,
        EnterDevMode
    ]
);

pub(crate) const TOOLBAR_KEY_CONTEXT: &str = "FantaEditorToolbar";
pub(crate) const TOOLBAR_TEXT_ENTRY_KEY_CONTEXT: &str = "FantaEditorToolbarTextEntry";
pub(crate) const TOOLBAR_CONTROL_KEY_CONTEXT: &str = "FantaEditorToolbarControl";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new(
            "enter",
            ActivateToolbarControl,
            Some(TOOLBAR_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "space",
            ActivateToolbarControl,
            Some(TOOLBAR_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new("escape", CloseToolbarOverlay, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("secondary-k", OpenToolbarActions, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("secondary-/", OpenToolbarActions, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new(
            "secondary-enter",
            OpenToolbarAgent,
            Some(TOOLBAR_KEY_CONTEXT),
        ),
        KeyBinding::new("down", NextToolbarCommand, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("up", PreviousToolbarCommand, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new(
            "escape",
            CloseToolbarOverlay,
            Some(TOOLBAR_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "secondary-k",
            OpenToolbarActions,
            Some(TOOLBAR_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "secondary-/",
            OpenToolbarActions,
            Some(TOOLBAR_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "secondary-enter",
            OpenToolbarAgent,
            Some(TOOLBAR_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "down",
            NextToolbarCommand,
            Some(TOOLBAR_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "up",
            PreviousToolbarCommand,
            Some(TOOLBAR_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new("v", SelectMoveTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("h", SelectHandTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("k", SelectScaleTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("f", SelectFrameTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("shift-s", SelectSectionTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("s", SelectSliceTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("r", SelectRectangleTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("l", SelectLineTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("shift-l", SelectArrowTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("o", SelectEllipseTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new(
            "shift-secondary-k",
            SelectImageVideoTool,
            Some(TOOLBAR_KEY_CONTEXT),
        ),
        KeyBinding::new("p", SelectPenTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("shift-p", SelectPencilTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("t", SelectTextTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("c", SelectCommentTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("shift-t", SelectAnnotationTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("shift-m", SelectMeasureTool, Some(TOOLBAR_KEY_CONTEXT)),
        KeyBinding::new("shift-d", EnterDevMode, Some(TOOLBAR_KEY_CONTEXT)),
    ]);
}
