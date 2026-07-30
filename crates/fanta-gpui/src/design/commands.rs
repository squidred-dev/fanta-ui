use gpui::{App, KeyBinding, actions};

actions!(
    fanta_design,
    [ActivateDesignControl, CancelDesignInteraction]
);

pub(crate) const DESIGN_PANEL_KEY_CONTEXT: &str = "FantaDesignPanel";
pub(crate) const DESIGN_CONTROL_KEY_CONTEXT: &str = "FantaDesignControl";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new(
            "enter",
            ActivateDesignControl,
            Some(DESIGN_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "space",
            ActivateDesignControl,
            Some(DESIGN_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "escape",
            CancelDesignInteraction,
            Some(DESIGN_PANEL_KEY_CONTEXT),
        ),
    ]);
}
