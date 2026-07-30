use gpui::{App, Div, ElementId, InteractiveElement as _, Pixels, Stateful, Styled as _, div};
use gpui_component::ActiveTheme as _;

use super::CONTROL_KEY_CONTEXT;

/// Shared baseline for compact square icon or glyph controls.
///
/// Consumers retain control-specific selected, focus, listener, selector, and
/// child styling so this helper owns presentation only.
pub(crate) fn compact_icon_control(
    id: impl Into<ElementId>,
    size: Pixels,
    radius: Pixels,
    cx: &App,
) -> Stateful<Div> {
    div()
        .id(id)
        .key_context(CONTROL_KEY_CONTEXT)
        .tab_index(0)
        .size(size)
        .items_center()
        .justify_center()
        .rounded(radius)
        .cursor_pointer()
        .hover(|style| style.bg(cx.theme().accent))
}
