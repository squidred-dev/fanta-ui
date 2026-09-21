//! Semantic UI colors resolved from the active `gpui-component` theme.
//!
//! Names follow Figma UI3's color taxonomy. Consumers choose a purpose and
//! leave palette selection to the current theme.

use gpui::{App, Hsla};
use gpui_component::ActiveTheme as _;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticColorGroup {
    Border,
    Background,
    Icon,
    Text,
}

impl SemanticColorGroup {
    pub const ALL: [Self; 4] = [Self::Border, Self::Background, Self::Icon, Self::Text];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Border => "Border colors",
            Self::Background => "Background colors",
            Self::Icon => "Icon colors",
            Self::Text => "Text colors",
        }
    }
}

/// A purpose-based application UI color. Resolve it during rendering so a
/// theme change is reflected immediately.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticColor {
    Border,
    BorderSelected,
    BorderSelectedStrong,
    BorderToolbar,
    BorderMenu,
    Background,
    BackgroundSecondary,
    BackgroundTertiary,
    BackgroundHover,
    BackgroundActive,
    BackgroundSelected,
    BackgroundDisabled,
    BackgroundBrand,
    BackgroundBrandHover,
    BackgroundBrandActive,
    BackgroundDanger,
    BackgroundDangerHover,
    BackgroundDangerActive,
    BackgroundWarning,
    BackgroundWarningHover,
    BackgroundSuccess,
    BackgroundSuccessHover,
    BackgroundSuccessActive,
    BackgroundAssistive,
    BackgroundToolbar,
    BackgroundToolbarHover,
    BackgroundToolbarSelected,
    BackgroundMenu,
    BackgroundTooltip,
    Icon,
    IconSecondary,
    IconTertiary,
    IconDisabled,
    IconBrand,
    IconDanger,
    IconWarning,
    IconSuccess,
    IconAssistive,
    IconOnBrand,
    IconOnDanger,
    IconOnWarning,
    IconOnSuccess,
    IconOnLightCanvas,
    IconOnDarkCanvas,
    Text,
    TextSecondary,
    TextTertiary,
    TextDisabled,
    TextBrand,
    TextDanger,
    TextWarning,
    TextSuccess,
    TextAssistive,
    TextOnBrand,
    TextOnDanger,
    TextOnWarning,
    TextOnSuccess,
}

impl SemanticColor {
    pub const ALL: [Self; 57] = [
        Self::Border,
        Self::BorderSelected,
        Self::BorderSelectedStrong,
        Self::BorderToolbar,
        Self::BorderMenu,
        Self::Background,
        Self::BackgroundSecondary,
        Self::BackgroundTertiary,
        Self::BackgroundHover,
        Self::BackgroundActive,
        Self::BackgroundSelected,
        Self::BackgroundDisabled,
        Self::BackgroundBrand,
        Self::BackgroundBrandHover,
        Self::BackgroundBrandActive,
        Self::BackgroundDanger,
        Self::BackgroundDangerHover,
        Self::BackgroundDangerActive,
        Self::BackgroundWarning,
        Self::BackgroundWarningHover,
        Self::BackgroundSuccess,
        Self::BackgroundSuccessHover,
        Self::BackgroundSuccessActive,
        Self::BackgroundAssistive,
        Self::BackgroundToolbar,
        Self::BackgroundToolbarHover,
        Self::BackgroundToolbarSelected,
        Self::BackgroundMenu,
        Self::BackgroundTooltip,
        Self::Icon,
        Self::IconSecondary,
        Self::IconTertiary,
        Self::IconDisabled,
        Self::IconBrand,
        Self::IconDanger,
        Self::IconWarning,
        Self::IconSuccess,
        Self::IconAssistive,
        Self::IconOnBrand,
        Self::IconOnDanger,
        Self::IconOnWarning,
        Self::IconOnSuccess,
        Self::IconOnLightCanvas,
        Self::IconOnDarkCanvas,
        Self::Text,
        Self::TextSecondary,
        Self::TextTertiary,
        Self::TextDisabled,
        Self::TextBrand,
        Self::TextDanger,
        Self::TextWarning,
        Self::TextSuccess,
        Self::TextAssistive,
        Self::TextOnBrand,
        Self::TextOnDanger,
        Self::TextOnWarning,
        Self::TextOnSuccess,
    ];

    pub const fn group(self) -> SemanticColorGroup {
        match self {
            Self::Border
            | Self::BorderSelected
            | Self::BorderSelectedStrong
            | Self::BorderToolbar
            | Self::BorderMenu => SemanticColorGroup::Border,
            Self::Background
            | Self::BackgroundSecondary
            | Self::BackgroundTertiary
            | Self::BackgroundHover
            | Self::BackgroundActive
            | Self::BackgroundSelected
            | Self::BackgroundDisabled
            | Self::BackgroundBrand
            | Self::BackgroundBrandHover
            | Self::BackgroundBrandActive
            | Self::BackgroundDanger
            | Self::BackgroundDangerHover
            | Self::BackgroundDangerActive
            | Self::BackgroundWarning
            | Self::BackgroundWarningHover
            | Self::BackgroundSuccess
            | Self::BackgroundSuccessHover
            | Self::BackgroundSuccessActive
            | Self::BackgroundAssistive
            | Self::BackgroundToolbar
            | Self::BackgroundToolbarHover
            | Self::BackgroundToolbarSelected
            | Self::BackgroundMenu
            | Self::BackgroundTooltip => SemanticColorGroup::Background,
            Self::Icon
            | Self::IconSecondary
            | Self::IconTertiary
            | Self::IconDisabled
            | Self::IconBrand
            | Self::IconDanger
            | Self::IconWarning
            | Self::IconSuccess
            | Self::IconAssistive
            | Self::IconOnBrand
            | Self::IconOnDanger
            | Self::IconOnWarning
            | Self::IconOnSuccess
            | Self::IconOnLightCanvas
            | Self::IconOnDarkCanvas => SemanticColorGroup::Icon,
            _ => SemanticColorGroup::Text,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Border => "color-border",
            Self::BorderSelected => "color-border-selected",
            Self::BorderSelectedStrong => "color-border-selected-strong",
            Self::BorderToolbar => "color-border-toolbar",
            Self::BorderMenu => "color-border-menu",
            Self::Background => "color-bg",
            Self::BackgroundSecondary => "color-bg-secondary",
            Self::BackgroundTertiary => "color-bg-tertiary",
            Self::BackgroundHover => "color-bg-hover",
            Self::BackgroundActive => "color-bg-active",
            Self::BackgroundSelected => "color-bg-selected",
            Self::BackgroundDisabled => "color-bg-disabled",
            Self::BackgroundBrand => "color-bg-brand",
            Self::BackgroundBrandHover => "color-bg-brand-hover",
            Self::BackgroundBrandActive => "color-bg-brand-active",
            Self::BackgroundDanger => "color-bg-danger",
            Self::BackgroundDangerHover => "color-bg-danger-hover",
            Self::BackgroundDangerActive => "color-bg-danger-active",
            Self::BackgroundWarning => "color-bg-warning",
            Self::BackgroundWarningHover => "color-bg-warning-hover",
            Self::BackgroundSuccess => "color-bg-success",
            Self::BackgroundSuccessHover => "color-bg-success-hover",
            Self::BackgroundSuccessActive => "color-bg-success-active",
            Self::BackgroundAssistive => "color-bg-assistive",
            Self::BackgroundToolbar => "color-bg-toolbar",
            Self::BackgroundToolbarHover => "color-bg-toolbar-hover",
            Self::BackgroundToolbarSelected => "color-bg-toolbar-selected",
            Self::BackgroundMenu => "color-bg-menu",
            Self::BackgroundTooltip => "color-bg-tooltip",
            Self::Icon => "color-icon",
            Self::IconSecondary => "color-icon-secondary",
            Self::IconTertiary => "color-icon-tertiary",
            Self::IconDisabled => "color-icon-disabled",
            Self::IconBrand => "color-icon-brand",
            Self::IconDanger => "color-icon-danger",
            Self::IconWarning => "color-icon-warning",
            Self::IconSuccess => "color-icon-success",
            Self::IconAssistive => "color-icon-assistive",
            Self::IconOnBrand => "color-icon-onbrand",
            Self::IconOnDanger => "color-icon-ondanger",
            Self::IconOnWarning => "color-icon-onwarning",
            Self::IconOnSuccess => "color-icon-onsuccess",
            Self::IconOnLightCanvas => "color-icon-onlightcanvas",
            Self::IconOnDarkCanvas => "color-icon-ondarkcanvas",
            Self::Text => "color-text",
            Self::TextSecondary => "color-text-secondary",
            Self::TextTertiary => "color-text-tertiary",
            Self::TextDisabled => "color-text-disabled",
            Self::TextBrand => "color-text-brand",
            Self::TextDanger => "color-text-danger",
            Self::TextWarning => "color-text-warning",
            Self::TextSuccess => "color-text-success",
            Self::TextAssistive => "color-text-assistive",
            Self::TextOnBrand => "color-text-onbrand",
            Self::TextOnDanger => "color-text-ondanger",
            Self::TextOnWarning => "color-text-onwarning",
            Self::TextOnSuccess => "color-text-onsuccess",
        }
    }

    pub fn resolve(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        match self {
            Self::Border => theme.border,
            Self::BorderSelected => theme.ring,
            Self::BorderSelectedStrong => theme.primary,
            Self::BorderToolbar => theme.title_bar_border,
            Self::BorderMenu => theme.input,
            Self::Background => theme.background,
            Self::BackgroundSecondary => theme.secondary,
            Self::BackgroundTertiary => theme.muted,
            Self::BackgroundHover => theme.secondary_hover,
            Self::BackgroundActive => theme.secondary_active,
            Self::BackgroundSelected => theme.selection,
            Self::BackgroundDisabled => theme.muted,
            Self::BackgroundBrand => theme.primary,
            Self::BackgroundBrandHover => theme.primary_hover,
            Self::BackgroundBrandActive => theme.primary_active,
            Self::BackgroundDanger => theme.danger,
            Self::BackgroundDangerHover => theme.danger_hover,
            Self::BackgroundDangerActive => theme.danger_active,
            Self::BackgroundWarning => theme.warning,
            Self::BackgroundWarningHover => theme.warning_hover,
            Self::BackgroundSuccess => theme.success,
            Self::BackgroundSuccessHover => theme.success_hover,
            Self::BackgroundSuccessActive => theme.success_active,
            Self::BackgroundAssistive => theme.magenta,
            Self::BackgroundToolbar => theme.title_bar,
            Self::BackgroundToolbarHover => theme.sidebar_accent,
            Self::BackgroundToolbarSelected => theme.sidebar_primary,
            Self::BackgroundMenu | Self::BackgroundTooltip => theme.popover,
            Self::Icon | Self::Text => theme.foreground,
            Self::IconSecondary | Self::TextSecondary => theme.secondary_foreground,
            Self::IconTertiary | Self::IconDisabled | Self::TextTertiary | Self::TextDisabled => {
                theme.muted_foreground
            }
            Self::IconBrand | Self::TextBrand => theme.link,
            Self::IconDanger | Self::TextDanger => theme.danger,
            Self::IconWarning | Self::TextWarning => theme.warning,
            Self::IconSuccess | Self::TextSuccess => theme.success,
            Self::IconAssistive | Self::TextAssistive => theme.magenta,
            Self::IconOnBrand | Self::TextOnBrand => theme.primary_foreground,
            Self::IconOnDanger | Self::TextOnDanger => theme.danger_foreground,
            Self::IconOnWarning | Self::TextOnWarning => theme.warning_foreground,
            Self::IconOnSuccess | Self::TextOnSuccess => theme.success_foreground,
            Self::IconOnLightCanvas => gpui::black(),
            Self::IconOnDarkCanvas => gpui::white(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_role_has_a_unique_name_and_matching_group_prefix() {
        let mut names = std::collections::HashSet::new();
        for role in SemanticColor::ALL {
            assert!(names.insert(role.name()), "duplicate role {}", role.name());
            let prefix = match role.group() {
                SemanticColorGroup::Border => "color-border",
                SemanticColorGroup::Background => "color-bg",
                SemanticColorGroup::Icon => "color-icon",
                SemanticColorGroup::Text => "color-text",
            };
            assert!(role.name().starts_with(prefix));
        }
    }
}
