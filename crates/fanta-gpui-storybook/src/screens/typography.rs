//! Figma UI3 semantic typography reference.

use fanta_gpui::atoms::{TypographyExt as _, TypographyToken};
use gpui::{AnyElement, IntoElement as _, ParentElement as _, Styled as _, div, px};
use gpui_component::{h_flex, v_flex};

use super::specimen::specimen_story_root;
use crate::*;

fn specimen(token: TypographyToken, cx: &Context<Storybook>) -> AnyElement {
    let style = token.style();
    h_flex()
        .w_full()
        .items_start()
        .gap_4()
        .py_3()
        .border_b_1()
        .border_color(SemanticColor::Border.resolve(cx))
        .child(
            v_flex()
                .w(px(220.))
                .flex_none()
                .gap_1()
                .child(
                    div()
                        .typography(TypographyToken::BodyLargeStrong)
                        .child(token.name()),
                )
                .child(
                    div()
                        .typography(TypographyToken::BodySmall)
                        .text_color(SemanticColor::TextTertiary.resolve(cx))
                        .child(format!(
                            "{} / {} · weight {}",
                            style.font_size, style.line_height, style.font_weight
                        )),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .typography(token)
                .child("Design clearly, build consistently."),
        )
        .into_any_element()
}

impl Storybook {
    pub(crate) fn render_typography_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut list = v_flex().w_full();
        for token in TypographyToken::ALL {
            list = list.child(specimen(token, cx));
        }
        specimen_story_root("storybook-typography")
            .min_w(px(720.))
            .child(v_flex().gap_1()
                .child(div().typography(TypographyToken::HeadingLarge).child("Typography tokens"))
                .child(div().typography(TypographyToken::BodyLarge)
                    .text_color(SemanticColor::TextSecondary.resolve(cx))
                    .child("UI3 hierarchy mapped to the active GPUI font family. Components apply roles instead of raw text sizes.")))
            .child(v_flex().w_full().p_4().rounded(px(8.)).border_1()
                .border_color(SemanticColor::Border.resolve(cx))
                .bg(SemanticColor::Background.resolve(cx)).child(list))
            .into_any_element()
    }

    pub(crate) fn render_typography_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.spec_reference(
            "storybook-reference-typography",
            self.render_typography_story(cx),
            cx,
        )
    }
}
