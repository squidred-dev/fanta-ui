//! Shared reference-fixture chrome: the story navigation row and the
//! descriptive two-column shell used by panel stories.

use crate::*;
use fanta_gpui::atoms::TypographyExt as _;

use super::intent;

/// The reference fixtures keep their historical navigation order; the
/// entries themselves come from the registry.
const REFERENCE_NAV_ORDER: [StoryKind; 9] = [
    StoryKind::Toolbar,
    StoryKind::Pages,
    StoryKind::Layers,
    StoryKind::FileInspector,
    StoryKind::Design,
    StoryKind::Variables,
    StoryKind::Prototype,
    StoryKind::Timeline,
    StoryKind::PseudoEditor,
];

/// Static copy shown beside a story inside the reference shell.
pub(crate) struct ReferenceStoryCopy {
    pub(crate) eyebrow: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) adapter_description: &'static str,
}

impl Storybook {
    fn render_navigation_item(
        &self,
        kind: StoryKind,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active = self.active_story == kind;
        fanta_gpui::atoms::ui_button(SharedString::from(format!(
            "storybook-nav-{}",
            label.to_lowercase()
        )))
        .label(label)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(cx.theme().transparent)
                .foreground(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
                .border(cx.theme().transparent)
                .hover(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                .active(
                    fanta_gpui::atoms::SemanticColor::BackgroundSelected
                        .resolve(cx)
                        .opacity(0.32),
                ),
        )
        .selected(active)
        .h(px(32.))
        .px_3()
        .rounded(px(6.))
        .border_0()
        .cursor_pointer()
        .when(active, |item| {
            item.font_semibold().hover(|style| {
                style.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
            })
        })
        .on_click(cx.listener(move |this, _, window, cx| {
            this.activate_gallery_story(kind, window, cx);
        }))
        .into_any_element()
    }

    /// The top navigation row shared by every chrome-bearing reference
    /// fixture.
    pub(crate) fn render_reference_nav(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut nav = h_flex()
            .h(px(56.))
            .w_full()
            .px_5()
            .gap_2()
            .border_b_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .child(div().mr_3().font_semibold().child("Fanta GPUI"));
        for kind in REFERENCE_NAV_ORDER {
            nav = nav.child(self.render_navigation_item(kind, kind.descriptor().nav_label, cx));
        }
        nav.into_any_element()
    }

    /// The descriptive reference shell: navigation, the story at panel
    /// width, and the adapter copy with the shared intent section.
    pub(crate) fn render_reference_story_shell(
        &self,
        id: &'static str,
        copy: ReferenceStoryCopy,
        last_action: SharedString,
        component: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .id(SharedString::from(id))
            .debug_selector(move || id.to_owned())
            .size_full()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(self.render_reference_nav(cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_start()
                    .child(
                        v_flex()
                            .w(px(360.))
                            .h_full()
                            .p_4()
                            .gap_3()
                            .child(
                                div()
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(copy.eyebrow),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(0.))
                                    .w_full()
                                    .overflow_hidden()
                                    .child(component),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .h_full()
                            .border_l_1()
                            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                            .p_8()
                            .gap_4()
                            .child(div().text_2xl().font_semibold().child(copy.title))
                            .child(
                                div()
                                    .max_w(px(620.))
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(copy.description),
                            )
                            .child(intent::intent_section(last_action, cx))
                            .child(
                                div()
                                    .mt_4()
                                    .max_w(px(620.))
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(copy.adapter_description),
                            ),
                    ),
            )
            .into_any_element()
    }
}
