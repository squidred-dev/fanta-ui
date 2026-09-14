//! Shared reference-fixture chrome: the story navigation row and the
//! descriptive two-column shell used by panel stories.

use crate::*;

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
        Button::new(SharedString::from(format!(
            "storybook-nav-{}",
            label.to_lowercase()
        )))
        .label(label)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(cx.theme().transparent)
                .foreground(cx.theme().foreground)
                .border(cx.theme().transparent)
                .hover(cx.theme().accent)
                .active(cx.theme().selection.opacity(0.32)),
        )
        .selected(active)
        .h(px(32.))
        .px_3()
        .rounded(px(6.))
        .border_0()
        .cursor_pointer()
        .when(active, |item| {
            item.font_semibold()
                .hover(|style| style.bg(cx.theme().accent))
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
            .border_color(cx.theme().border)
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
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
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
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
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
                            .border_color(cx.theme().border)
                            .p_8()
                            .gap_4()
                            .child(div().text_2xl().font_semibold().child(copy.title))
                            .child(
                                div()
                                    .max_w(px(620.))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(copy.description),
                            )
                            .child(intent::intent_section(last_action, cx))
                            .child(
                                div()
                                    .mt_4()
                                    .max_w(px(620.))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(copy.adapter_description),
                            ),
                    ),
            )
            .into_any_element()
    }
}
