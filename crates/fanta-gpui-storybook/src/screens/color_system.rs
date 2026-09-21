//! Live semantic color reference derived from the active storybook theme.

use fanta_gpui::atoms::TypographyExt as _;
use fanta_gpui::atoms::{SemanticColor, SemanticColorGroup};
use gpui::{AnyElement, Hsla, IntoElement as _, ParentElement as _, Styled as _, div, px};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use super::specimen::specimen_story_root;
use crate::*;

fn color_hex(color: Hsla) -> String {
    let h = color.h.rem_euclid(1.);
    let s = color.s.clamp(0., 1.);
    let l = color.l.clamp(0., 1.);
    let c = (1. - (2. * l - 1.).abs()) * s;
    let x = c * (1. - ((h * 6.).rem_euclid(2.) - 1.).abs());
    let m = l - c / 2.;
    let (r, g, b) = match (h * 6.).floor() as u8 {
        0 => (c, x, 0.),
        1 => (x, c, 0.),
        2 => (0., c, x),
        3 => (0., x, c),
        4 => (x, 0., c),
        _ => (c, 0., x),
    };
    let channel = |value: f32| ((value + m) * 255.).round().clamp(0., 255.) as u8;
    if color.a < 0.999 {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            channel(r),
            channel(g),
            channel(b),
            channel(color.a)
        )
    } else {
        format!("#{:02X}{:02X}{:02X}", channel(r), channel(g), channel(b))
    }
}

fn role_note(role: SemanticColor) -> &'static str {
    use SemanticColor::*;
    match role {
        Border => "Default divider and input border",
        BorderSelected => "Focused or selected control",
        BorderSelectedStrong => "Selected border on a selected fill",
        BorderToolbar | BorderMenu => "Border on dark editor chrome",
        Background => "Default application surface",
        BackgroundSecondary => "Container surface",
        BackgroundTertiary => "Nested container surface",
        BackgroundHover => "Hover fill",
        BackgroundActive => "Pressed fill",
        BackgroundSelected => "Selected item fill",
        BackgroundDisabled => "Non-interactive fill",
        BackgroundBrand | BackgroundBrandHover | BackgroundBrandActive => "Primary brand action",
        BackgroundDanger | BackgroundDangerHover | BackgroundDangerActive => {
            "Destructive action or error banner"
        }
        BackgroundWarning | BackgroundWarningHover => "Warning surface",
        BackgroundSuccess | BackgroundSuccessHover | BackgroundSuccessActive => {
            "Confirmation surface"
        }
        BackgroundAssistive => "Assistive UI surface",
        BackgroundToolbar => "Editor toolbar",
        BackgroundToolbarHover => "Toolbar hover",
        BackgroundToolbarSelected => "Toolbar selection",
        BackgroundMenu => "Menu surface",
        BackgroundTooltip => "Tooltip surface",
        Icon => "Default icon",
        IconSecondary => "Inactive icon",
        IconTertiary => "Low-emphasis caret",
        IconDisabled => "Non-interactive icon",
        IconBrand => "Link or brand icon",
        IconDanger => "Error icon",
        IconWarning => "Warning icon",
        IconSuccess => "Confirmation icon",
        IconAssistive => "Assistive UI icon",
        IconOnBrand => "Icon on brand fill",
        IconOnDanger => "Icon on danger fill",
        IconOnWarning => "Icon on warning fill",
        IconOnSuccess => "Icon on success fill",
        IconOnLightCanvas => "Icon over light content",
        IconOnDarkCanvas => "Icon over dark content",
        Text => "Default title and body text",
        TextSecondary => "Labels, timestamps, inactive tabs",
        TextTertiary => "Placeholder text",
        TextDisabled => "Non-interactive text",
        TextBrand => "Link or brand text",
        TextDanger => "Error text",
        TextWarning => "Warning text",
        TextSuccess => "Confirmation text",
        TextAssistive => "Assistive UI text",
        TextOnBrand => "Text on brand fill",
        TextOnDanger => "Text on danger fill",
        TextOnWarning => "Text on warning fill",
        TextOnSuccess => "Text on success fill",
    }
}

fn swatch(role: SemanticColor, cx: &Context<Storybook>) -> AnyElement {
    let color = role.resolve(cx);
    let name = role.name();
    h_flex()
        .id(name)
        .debug_selector(move || format!("color-system-{name}"))
        .w(px(330.))
        .p_2()
        .gap_3()
        .rounded(cx.theme().radius)
        .hover(|style| style.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx)))
        .child(
            div()
                .size(px(44.))
                .flex_none()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                .bg(color),
        )
        .child(
            v_flex()
                .min_w_0()
                .gap_1()
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(name),
                        )
                        .child(
                            div()
                                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                                .text_color(
                                    fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                )
                                .child(color_hex(color)),
                        ),
                )
                .child(
                    div()
                        .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                        .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(role_note(role)),
                ),
        )
        .into_any_element()
}

impl Storybook {
    pub(crate) fn render_color_system_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut root = specimen_story_root("storybook-color-system")
            .debug_selector(|| "storybook-color-system".to_owned()).min_w(px(720.))
            .child(v_flex().gap_1()
                .child(div().typography(fanta_gpui::atoms::TypographyToken::HeadingLarge).font_weight(gpui::FontWeight::SEMIBOLD).child("Semantic color system"))
                .child(div().typography(fanta_gpui::atoms::TypographyToken::BodyLarge).text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)).child(format!(
                    "{} · {} live utility roles · Figma UI3 taxonomy",
                    cx.theme().theme_name(), SemanticColor::ALL.len())))
                .child(div().typography(fanta_gpui::atoms::TypographyToken::BodyLarge).text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)).child(
                    "Use SemanticColor::resolve(cx) at render time. Switching the Storybook theme remaps every swatch.")));
        for group in SemanticColorGroup::ALL {
            let mut grid = h_flex()
                .w_full()
                .items_start()
                .content_start()
                .gap_2()
                .flex_wrap();
            for role in SemanticColor::ALL
                .into_iter()
                .filter(|role| role.group() == group)
            {
                grid = grid.child(swatch(role, cx));
            }
            root = root.child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .p_3()
                    .rounded(cx.theme().radius_lg)
                    .border_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
                    .child(
                        div()
                            .typography(fanta_gpui::atoms::TypographyToken::HeadingMedium)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(group.label()),
                    )
                    .child(grid),
            );
        }
        root.into_any_element()
    }

    pub(crate) fn render_color_system_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.spec_reference(
            "storybook-reference-color-system",
            self.render_color_system_story(cx),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hex_formatter_keeps_alpha_when_translucent() {
        assert_eq!(color_hex(gpui::hsla(0., 1., 0.5, 0.5)), "#FF000080");
        assert_eq!(color_hex(gpui::hsla(0., 1., 0.5, 1.)), "#FF0000");
    }
}
