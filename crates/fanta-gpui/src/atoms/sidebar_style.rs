//! Zed panel styling shared by the File Inspector shell and its two sections.
//!
//! A standalone Fanta host can still mount these components with only the
//! gpui-component theme initialized. When it also installs Zed's theme, the
//! panel reads Zed's canonical file-tree roles directly.

use gpui::{App, Hsla, Rems};

#[derive(Clone, Copy)]
pub(crate) struct SidebarStyle {
    pub background: Hsla,
    pub border: Hsla,
    pub hover: Hsla,
    pub selected: Hsla,
    pub selected_border: Hsla,
    pub focused_border: Hsla,
    pub text: Hsla,
    pub muted_text: Hsla,
    pub disabled_text: Hsla,
    pub icon: Hsla,
    pub muted_icon: Hsla,
    pub menu_background: Hsla,
}

pub(crate) fn sidebar_style(cx: &App) -> SidebarStyle {
    if cx.try_global::<theme::GlobalTheme>().is_some() {
        let colors = theme::GlobalTheme::theme(cx).colors();
        SidebarStyle {
            background: colors.panel_background,
            border: colors.border_variant,
            hover: colors.element_hover,
            // TreeViewItem is the closest Zed primitive to the inspector tree.
            selected: colors.element_active.opacity(0.5),
            selected_border: colors.border.opacity(0.4),
            focused_border: colors.border_focused,
            text: colors.text,
            muted_text: colors.text_muted,
            disabled_text: colors.text_disabled,
            icon: colors.icon,
            muted_icon: colors.icon_muted,
            menu_background: colors.elevated_surface_background,
        }
    } else {
        let theme = gpui_component::Theme::global(cx);
        SidebarStyle {
            background: theme.sidebar,
            border: theme.sidebar_border,
            hover: theme.sidebar_accent,
            selected: theme.list_active,
            selected_border: theme.list_active_border,
            focused_border: theme.ring,
            text: theme.sidebar_foreground,
            muted_text: theme.muted_foreground,
            disabled_text: theme.muted_foreground,
            icon: theme.sidebar_foreground,
            muted_icon: theme.muted_foreground,
            menu_background: theme.popover,
        }
    }
}

/// Zed's default UI text size, shared by the inspector's panel chrome.
pub(crate) fn sidebar_text_size() -> Rems {
    ui::rems_from_px(super::tokens::TypeScale::TITLE)
}
