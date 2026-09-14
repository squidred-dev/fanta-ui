use std::rc::Rc;

use gpui::App;
use gpui_component::{Theme, ThemeConfig, ThemeMode};
use serde_json::{Map, Value};

const ZED_THEME_FILES: [&str; 3] = [
    include_str!("../assets/themes/one.json"),
    include_str!("../assets/themes/ayu.json"),
    include_str!("../assets/themes/gruvbox.json"),
];

/// The gallery order deliberately alternates light and dark first, preserving
/// the old one-click light/dark smoke test while exposing every bundled variant.
const THEME_ORDER: [&str; 11] = [
    "One Light",
    "One Dark",
    "Ayu Light",
    "Ayu Dark",
    "Ayu Mirage",
    "Gruvbox Light",
    "Gruvbox Dark",
    "Gruvbox Light Hard",
    "Gruvbox Dark Hard",
    "Gruvbox Light Soft",
    "Gruvbox Dark Soft",
];

pub(super) fn zed_themes() -> Vec<Rc<ThemeConfig>> {
    let mut themes = ZED_THEME_FILES
        .iter()
        .flat_map(|source| parse_zed_theme_family(source))
        .collect::<Vec<_>>();
    themes.sort_by_key(|theme| {
        THEME_ORDER
            .iter()
            .position(|name| *name == theme.name.as_ref())
            .unwrap_or(usize::MAX)
    });
    themes.into_iter().map(Rc::new).collect()
}

pub(super) fn initial_zed_theme_index(mode: ThemeMode) -> usize {
    usize::from(mode.is_dark())
}

pub(super) fn apply_zed_theme(theme: &Rc<ThemeConfig>, cx: &mut App) {
    Theme::global_mut(cx).apply_config(theme);
    cx.refresh_windows();
}

fn parse_zed_theme_family(source: &str) -> Vec<ThemeConfig> {
    let family: Value = serde_json::from_str(source).expect("bundled Zed theme must be valid JSON");
    family["themes"]
        .as_array()
        .expect("bundled Zed theme family must contain themes")
        .iter()
        .map(convert_zed_theme)
        .collect()
}

fn convert_zed_theme(theme: &Value) -> ThemeConfig {
    let style = theme["style"]
        .as_object()
        .expect("bundled Zed theme must contain a style object");
    let mut colors = Map::new();

    let mappings = [
        ("background", "background"),
        ("foreground", "text"),
        ("border", "border"),
        ("ring", "border.focused"),
        ("drag_border", "border.focused"),
        ("drop_target.background", "drop_target.background"),
        ("accent.background", "ghost_element.selected"),
        ("accent.foreground", "text"),
        ("secondary.background", "element.background"),
        ("secondary.hover.background", "element.hover"),
        ("secondary.active.background", "element.active"),
        ("secondary.foreground", "text"),
        ("muted.background", "surface.background"),
        ("muted.foreground", "text.muted"),
        ("popover.background", "elevated_surface.background"),
        ("popover.foreground", "text"),
        ("input.border", "border.variant"),
        ("primary.background", "text.accent"),
        ("primary.foreground", "editor.background"),
        ("primary.hover.background", "link_text.hover"),
        ("primary.active.background", "text.accent"),
        (
            "selection.background",
            "editor.document_highlight.read_background",
        ),
        ("sidebar.background", "panel.background"),
        ("sidebar.accent.background", "ghost_element.hover"),
        ("sidebar.accent.foreground", "text"),
        ("sidebar.border", "border.variant"),
        ("sidebar.foreground", "text"),
        ("sidebar.primary.background", "text.accent"),
        ("sidebar.primary.foreground", "editor.background"),
        ("tab.background", "tab.inactive_background"),
        ("tab.active.background", "tab.active_background"),
        ("tab.active.foreground", "text"),
        ("tab_bar.background", "tab_bar.background"),
        ("title_bar.background", "title_bar.background"),
        ("title_bar.border", "border.variant"),
        ("scrollbar.background", "scrollbar.track.background"),
        ("scrollbar.thumb.background", "scrollbar.thumb.background"),
        (
            "scrollbar.thumb.hover.background",
            "scrollbar.thumb.hover_background",
        ),
        ("success.background", "success"),
        ("danger.background", "error"),
        ("warning.background", "warning"),
        ("info.background", "info"),
        ("link.foreground", "text.accent"),
        ("link.hover.foreground", "link_text.hover"),
    ];
    for (target, source) in mappings {
        if let Some(value) = style.get(source)
            && !value.is_null()
        {
            colors.insert(target.into(), value.clone());
        }
    }

    let converted = serde_json::json!({
        "name": theme["name"],
        "mode": theme["appearance"],
        "colors": colors,
        "highlight": theme["style"],
    });
    serde_json::from_value(converted).expect("bundled Zed theme must map to gpui-component")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_zed_themes_parse_in_gallery_order() {
        let themes = zed_themes();
        assert_eq!(themes.len(), THEME_ORDER.len());
        assert_eq!(
            themes
                .iter()
                .map(|theme| theme.name.as_ref())
                .collect::<Vec<_>>(),
            THEME_ORDER
        );
        assert_eq!(themes[0].mode, ThemeMode::Light);
        assert_eq!(themes[1].mode, ThemeMode::Dark);
    }
}
