use gpui::{App, KeyBinding, actions};

actions!(
    fanta_layers,
    [
        ActivateLayersControl,
        OpenLayerContextMenu,
        CloseLayersOverlay
    ]
);

pub(crate) const LAYERS_PANEL_KEY_CONTEXT: &str = "FantaLayersPanel";
pub(crate) const LAYERS_CONTROL_KEY_CONTEXT: &str = "FantaLayersControl";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new(
            "enter",
            ActivateLayersControl,
            Some(LAYERS_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "space",
            ActivateLayersControl,
            Some(LAYERS_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "ctrl-enter",
            OpenLayerContextMenu,
            Some(LAYERS_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new("escape", CloseLayersOverlay, Some(LAYERS_PANEL_KEY_CONTEXT)),
    ]);
}
