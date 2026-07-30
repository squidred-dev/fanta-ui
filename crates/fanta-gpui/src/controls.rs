//! Shared interaction contracts for custom Fanta controls.

use gpui::{App, KeyBinding, actions};

mod chrome;

pub(crate) use chrome::compact_icon_control;

actions!(fanta_controls, [ActivateControl]);

/// Key context used by focusable custom controls that behave like buttons.
pub(crate) const CONTROL_KEY_CONTEXT: &str = "FantaControl";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", ActivateControl, Some(CONTROL_KEY_CONTEXT)),
        KeyBinding::new("space", ActivateControl, Some(CONTROL_KEY_CONTEXT)),
    ]);
}
