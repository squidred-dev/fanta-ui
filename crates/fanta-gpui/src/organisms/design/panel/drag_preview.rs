use super::*;

fn drag_preview(
    width: f32,
    label: impl IntoElement,
    swatch: Option<AnyElement>,
    cx: &mut App,
) -> AnyElement {
    h_flex()
        .h(px(ROW_HEIGHT))
        .w(px(width))
        .px_2()
        .gap_2()
        .rounded(px(6.))
        .border_1()
        .border_color(crate::atoms::SemanticColor::BackgroundSelected.resolve(cx))
        .bg(crate::atoms::SemanticColor::BackgroundMenu
            .resolve(cx)
            .opacity(0.96))
        .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
        .shadow_lg()
        .child(
            div()
                .w(px(14.))
                .typography(crate::atoms::TypographyToken::BodyMedium)
                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child("⠿"),
        )
        .children(swatch)
        .child(
            div()
                .flex_1()
                .truncate()
                .typography(crate::atoms::TypographyToken::BodyMedium)
                .child(label),
        )
        .into_any_element()
}

impl Render for EffectDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(180., self.drag.label.clone(), None, cx)
    }
}

impl Render for PaintDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let swatch = div()
            .size(px(14.))
            .rounded(px(3.))
            .border_1()
            .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
            .bg(color_hsla(self.drag.paint.color))
            .into_any_element();
        drag_preview(
            196.,
            format!("{} · {}", self.drag.collection.label(), self.drag.label),
            Some(swatch),
            cx,
        )
    }
}

impl Render for LocalStyleDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(196., self.drag.label.clone(), None, cx)
    }
}

impl Render for ComponentPropertyDefinitionDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(196., self.drag.property_name.clone(), None, cx)
    }
}

impl Render for ComponentVariantOptionDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_preview(176., self.drag.option_name.clone(), None, cx)
    }
}
