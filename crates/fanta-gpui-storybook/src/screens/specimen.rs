//! Shared specimen chrome for the Atoms and Molecules catalog stories.
//!
//! Every specimen story frames its demos the same way: a titled card with a
//! short contract description; a grid vocabulary of fixed-size labeled
//! [`specimen_cell`]s arranged into titled [`specimen_row`]s (12 px between
//! cells, 16 px between rows); and an optional fixed-size framed panel for
//! mounting a component at a deliberate width. The helpers started life in
//! the retired Foundations story and are shared here so the per-atom and
//! per-molecule screens never re-implement the card chrome.

use crate::*;
use fanta_gpui::atoms::TypographyExt as _;

/// The shared side of the square specimen cell.
pub(crate) const SPECIMEN_CELL_SIZE: f32 = 96.;

/// A titled specimen card: heading, contract copy, then the demo content.
pub(crate) fn specimen_card(
    id: &'static str,
    title: &'static str,
    copy: &'static str,
    content: AnyElement,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .w_full()
        .p_3()
        .gap_2()
        .rounded(px(8.))
        .border_1()
        .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
        .bg(fanta_gpui::atoms::SemanticColor::BackgroundToolbar.resolve(cx))
        .child(
            div()
                .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                .font_semibold()
                .child(title),
        )
        .child(
            div()
                .max_w(px(720.))
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(copy),
        )
        .child(content)
        .into_any_element()
}

/// One fixed-size specimen cell: a bordered square with a subtle fill
/// distinct from the card behind it, the specimen centered inside, and the
/// state label — plus an optional shortcut or instruction hint — under the
/// cell in muted small text.
pub(crate) fn specimen_cell(
    label: impl Into<SharedString>,
    hint: Option<&'static str>,
    content: AnyElement,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .w(px(SPECIMEN_CELL_SIZE))
        .flex_none()
        .items_center()
        .gap_1p5()
        .child(
            div()
                .size(px(SPECIMEN_CELL_SIZE))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .border_1()
                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                    .resolve(cx)
                    .opacity(0.55))
                .child(content),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(label.into()),
        )
        .when_some(hint, |cell, hint| {
            cell.child(
                div()
                    .w_full()
                    .text_center()
                    .text_size(px(10.))
                    .text_color(
                        fanta_gpui::atoms::SemanticColor::TextTertiary
                            .resolve(cx)
                            .opacity(0.75),
                    )
                    .child(hint),
            )
        })
        .into_any_element()
}

/// A wider specimen cell for labeled controls such as checkboxes and
/// dropdown triggers whose natural width exceeds the square icon cell.
pub(crate) fn specimen_control_cell(
    label: impl Into<SharedString>,
    content: AnyElement,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .w(px(160.))
        .flex_none()
        .gap_1p5()
        .child(
            div()
                .w_full()
                .h(px(72.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .border_1()
                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                    .resolve(cx)
                    .opacity(0.55))
                .child(content),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(label.into()),
        )
        .into_any_element()
}

/// A wide specimen cell for text fields and compound controls.
pub(crate) fn specimen_wide_control_cell(
    label: impl Into<SharedString>,
    content: AnyElement,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .w(px(240.))
        .flex_none()
        .gap_1p5()
        .child(
            div()
                .w_full()
                .h(px(96.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .border_1()
                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                    .resolve(cx)
                    .opacity(0.55))
                .child(content),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(label.into()),
        )
        .into_any_element()
}

/// A titled specimen row: a left-aligned section title over a wrapping run
/// of cells (or any demo content) with the shared 12 px gap, so rows
/// reflow instead of clipping when the story viewport narrows.
pub(crate) fn specimen_row(
    id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    cells: Vec<AnyElement>,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    let id = id.into();
    let selector = id.to_string();
    v_flex()
        .id(id)
        .debug_selector(move || selector.clone())
        .w_full()
        .gap_2()
        .child(
            div()
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .font_semibold()
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(title.into()),
        )
        .child(
            h_flex()
                .w_full()
                .items_start()
                .content_start()
                .gap_3()
                .flex_wrap()
                .children(cells),
        )
        .into_any_element()
}

/// The stack that separates [`specimen_row`]s inside one card with the
/// shared 16 px row gap.
pub(crate) fn specimen_rows(rows: Vec<AnyElement>) -> AnyElement {
    v_flex().w_full().gap_4().children(rows).into_any_element()
}

/// A fixed-size bordered frame for mounting a component at a deliberate
/// specimen width and height.
pub(crate) fn framed_panel(
    width: f32,
    height: f32,
    content: AnyElement,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    div()
        .w(px(width))
        .h(px(height))
        .flex_none()
        .overflow_hidden()
        .rounded(px(6.))
        .border_1()
        .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
        .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
        .child(content)
        .into_any_element()
}

/// The scrollable story root shared by the specimen catalog stories.
pub(crate) fn specimen_story_root(id: &'static str) -> gpui::Stateful<gpui::Div> {
    v_flex()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .size_full()
        .min_h(px(0.))
        .overflow_y_scroll()
        .p_4()
        .gap_4()
}
