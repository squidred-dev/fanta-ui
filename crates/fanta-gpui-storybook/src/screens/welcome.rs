//! The Getting-started Welcome story: the atomic-design map of the library.
//!
//! One card per library tier — Atoms, Molecules, Organisms, Layouts — plus
//! the Screens card explaining that the storybook itself is the screens
//! tier. Each card carries its story count, a one-line description of what
//! the tier owns, and that tier's stories as plain labels sourced from the
//! registry, so the sidebar taxonomy and this overview can never drift
//! apart: registering a story adds it to both. The map is a map, not a
//! launcher — the sidebar owns navigation, so the labels link nothing.

use gpui::FocusHandle;

use crate::*;

pub(crate) struct WelcomeScreen {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) last_action: SharedString,
}

impl WelcomeScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            last_action: "Ready — the sidebar opens any story on this map".into(),
        }
    }
}

/// The one-line copy shown on each tier card, keyed by sidebar section.
fn tier_copy(section: screens::StorySection) -> &'static str {
    use screens::StorySection as Section;
    match section {
        Section::GettingStarted => "",
        Section::Atoms => {
            "The smallest shared interaction units — the single activation path, \
             icon_button, the stroke-path vector icons, and truncating labels — \
             imported from fanta_gpui::atoms."
        }
        Section::Molecules => {
            "Composite chrome assembled from the atoms — clamped context menus, \
             anchored popups, selectable list rows, and scroll edge fades — \
             imported from fanta_gpui::molecules."
        }
        Section::Organisms => {
            "The host-facing feature surfaces: each renders host-supplied view \
             data and emits typed intents a host applies and echoes back."
        }
        Section::Layouts => {
            "Composition shells that arrange organisms into a working surface \
             without owning any domain state."
        }
    }
}

impl Storybook {
    fn render_welcome_tier_card(
        &self,
        section: screens::StorySection,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let stories: Vec<&'static str> = screens::registry()
            .iter()
            .filter(|descriptor| descriptor.section == section)
            .map(|descriptor| descriptor.title)
            .collect();
        let count_label = if stories.len() == 1 {
            "1 story".to_owned()
        } else {
            format!("{} stories", stories.len())
        };
        let mut chips = h_flex().gap_2().flex_wrap();
        for title in stories {
            chips = chips.child(
                div()
                    .flex_none()
                    .px_2()
                    .py_0p5()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary.opacity(0.55))
                    .text_xs()
                    .text_color(cx.theme().foreground)
                    .child(title),
            );
        }
        v_flex()
            .w_full()
            .p_3()
            .gap_2()
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().font_semibold().child(section.label()))
                    .child(
                        div()
                            .flex_none()
                            .px_1p5()
                            .rounded(px(4.))
                            .bg(cx.theme().secondary)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(count_label),
                    ),
            )
            .child(
                div()
                    .max_w(px(720.))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(tier_copy(section)),
            )
            .child(chips)
            .into_any_element()
    }

    pub(crate) fn render_welcome_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut root = v_flex()
            .id("storybook-welcome")
            .debug_selector(|| "storybook-welcome".to_owned())
            .track_focus(&self.welcome_screen.focus_handle)
            .size_full()
            .min_h(px(0.))
            .overflow_y_scroll()
            .p_4()
            .gap_4()
            .child(
                v_flex()
                    .w_full()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .child("Fanta GPUI is organized by atomic design"),
                    )
                    .child(
                        div()
                            .max_w(px(760.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                "The sidebar mirrors the library source tree: Atoms and \
                                 Molecules are the shared control layer every surface is \
                                 built from, Organisms are the host-facing feature panels, \
                                 and Layouts compose organisms into whole surfaces. Each \
                                 tier card below lists its stories; the sidebar opens them.",
                            ),
                    ),
            );
        for section in [
            screens::StorySection::Atoms,
            screens::StorySection::Molecules,
            screens::StorySection::Organisms,
            screens::StorySection::Layouts,
        ] {
            root = root.child(self.render_welcome_tier_card(section, cx));
        }
        root.child(
            v_flex()
                .w_full()
                .p_3()
                .gap_2()
                .rounded(px(8.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().sidebar)
                .child(div().text_sm().font_semibold().child("Screens"))
                .child(
                    div()
                        .max_w(px(720.))
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(
                            "The fifth tier lives outside the library: a screen is an \
                             application surface that feeds real data into the tiers \
                             above. This storybook is itself the screens tier — every \
                             story is a screen module owning mock host state and a \
                             reducer for the typed intents its component emits.",
                        ),
                ),
        )
        .into_any_element()
    }

    pub(crate) fn render_welcome_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-welcome",
            self.render_welcome_story(cx),
            self.welcome_screen.last_action.clone(),
            cx,
        )
    }
}
