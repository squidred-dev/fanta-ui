//! The shared "last typed intent" affordance.
//!
//! Every story surfaces the most recent typed intent its mock host
//! received. The three historical renderings — the Gallery footer text,
//! the floating reference-fixture chip, and the labeled rail section —
//! all read the registry's `last_action` hook; the chip and section
//! renderers live here so screens never re-implement them.

use crate::*;
use fanta_gpui::atoms::TypographyExt as _;

/// The labeled "LAST TYPED INTENT" section used inside story rails and
/// reference shells.
pub(crate) fn intent_section(last_action: SharedString, cx: &mut Context<Storybook>) -> AnyElement {
    v_flex()
        .gap_1()
        .child(
            div()
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child("LAST TYPED INTENT"),
        )
        .child(
            div()
                .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                .child(last_action),
        )
        .into_any_element()
}

impl Storybook {
    /// The floating chip reference fixtures overlay once the story has
    /// received a real intent.
    pub(crate) fn render_reference_last_action(
        &self,
        last_action: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .absolute()
            .left(px(12.))
            .bottom(px(12.))
            .max_w(px(460.))
            .px(px(10.))
            .py(px(7.))
            .rounded(px(6.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundMenu
                .resolve(cx)
                .opacity(0.96))
            .shadow_lg()
            .text_size(px(11.))
            .child(last_action)
            .into_any_element()
    }

    /// The full-size reference fixture wrapper shared by component-only
    /// stories: the component fills the window and the intent chip
    /// appears after the first non-Ready intent.
    pub(crate) fn render_reference_component_fixture(
        &self,
        id: &'static str,
        component: AnyElement,
        last_action: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let show_last_action = !last_action.as_ref().starts_with("Ready");
        v_flex()
            .id(SharedString::from(id))
            .debug_selector(move || id.to_owned())
            .relative()
            .size_full()
            .min_h(px(0.))
            .overflow_hidden()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .child(component)
            .when(show_last_action, |fixture| {
                fixture.child(self.render_reference_last_action(last_action, cx))
            })
            .into_any_element()
    }
}
