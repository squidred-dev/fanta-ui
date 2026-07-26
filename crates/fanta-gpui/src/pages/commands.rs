use gpui::{App, KeyBinding, actions};

actions!(
    fanta_pages,
    [
        TogglePagesPanel,
        FindInPages,
        AddPage,
        ToggleSearchSettings,
        ClosePagesSearch,
        PreviousSearchResult,
        NextSearchResult,
        ReplaceCurrentResult,
        ReplaceAllResults,
        ActivatePagesControl,
        OpenPageContextMenu
    ]
);

pub(crate) const PAGES_PANEL_KEY_CONTEXT: &str = "FantaPagesPanel";
pub(crate) const PAGES_CONTROL_KEY_CONTEXT: &str = "FantaPagesControl";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-f", FindInPages, Some(PAGES_PANEL_KEY_CONTEXT)),
        KeyBinding::new(
            "shift-secondary-d",
            PreviousSearchResult,
            Some(PAGES_PANEL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "shift-secondary-f",
            NextSearchResult,
            Some(PAGES_PANEL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "enter",
            ActivatePagesControl,
            Some(PAGES_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "space",
            ActivatePagesControl,
            Some(PAGES_CONTROL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "ctrl-enter",
            OpenPageContextMenu,
            Some(PAGES_CONTROL_KEY_CONTEXT),
        ),
    ]);
}
