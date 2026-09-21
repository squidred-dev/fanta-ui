//! Theme-aware slider drawing primitives. These own no values or transactions.
use super::tokens;
use gpui::{prelude::FluentBuilder, *};

/// A slider's visual state; interaction owners supply it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderState {
    #[default]
    Default,
    Focused,
    Disabled,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderHandleVariant {
    #[default]
    Fill,
    Stroke,
    Chit,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SliderBackground {
    #[default]
    Default,
    Hue,
    Opacity(Hsla),
    Gradient(Vec<(f32, Hsla)>),
}
impl SliderBackground {
    pub fn render(&self, state: SliderState, cx: &App) -> Div {
        let radius = tokens::SliderGeometry::TRACK_HEIGHT / 2.;
        let base = div()
            .relative()
            .w_full()
            .h(px(tokens::SliderGeometry::TRACK_HEIGHT))
            .rounded(px(radius))
            .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
            .border_1()
            .border_color(if state == SliderState::Focused {
                crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)
            } else {
                crate::atoms::SemanticColor::Border.resolve(cx)
            });
        let stops = match self {
            Self::Default => Vec::new(),
            Self::Hue => (0..=6)
                .map(|i| (i as f32 / 6., hsla(i as f32 / 6., 1., 0.5, 1.)))
                .collect(),
            Self::Opacity(color) => vec![(0., color.alpha(0.)), (1., color.alpha(1.))],
            Self::Gradient(stops) => stops.clone(),
        };
        let mut stops: Vec<_> = stops
            .into_iter()
            .filter(|(p, _)| p.is_finite())
            .map(|(p, c)| (p.clamp(0., 1.), c))
            .collect();
        stops.sort_by(|a, b| a.0.total_cmp(&b.0));
        if let Some(&(p, c)) = stops.first()
            && p > 0.
        {
            stops.insert(0, (0., c));
        }
        if let Some(&(p, c)) = stops.last()
            && p < 1.
        {
            stops.push((1., c));
        }
        let len = stops.len();
        base.when(!matches!(self, Self::Default | Self::Hue), |base| {
            base.bg(pattern_slash(
                crate::atoms::SemanticColor::TextTertiary
                    .resolve(cx)
                    .opacity(0.22),
                0.35,
                0.35,
            ))
        })
        .children(stops.windows(2).enumerate().map(|(i, pair)| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(relative(pair[0].0))
                .w(relative(pair[1].0 - pair[0].0))
                .when(i == 0, |d| d.rounded_tl(px(radius)).rounded_bl(px(radius)))
                .when(i + 2 == len, |d| {
                    d.rounded_tr(px(radius)).rounded_br(px(radius))
                })
                .bg(linear_gradient(
                    90.,
                    linear_color_stop(pair[0].1, 0.),
                    linear_color_stop(pair[1].1, 1.),
                ))
        }))
        .child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left_0()
                .right_0()
                .rounded(px(radius))
                .border_1()
                .border_color(if state == SliderState::Focused {
                    crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)
                } else {
                    crate::atoms::SemanticColor::Border.resolve(cx)
                }),
        )
        .when(state == SliderState::Disabled, |d| d.opacity(0.5))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SliderHandle {
    pub variant: SliderHandleVariant,
    pub state: SliderState,
    pub color: Option<Hsla>,
}
impl SliderHandle {
    pub fn render(self, cx: &App) -> Div {
        let color = self
            .color
            .unwrap_or(crate::atoms::SemanticColor::BackgroundBrand.resolve(cx));
        div()
            .flex_none()
            .size(px(tokens::SliderGeometry::HANDLE_SIZE))
            .rounded_full()
            .border_2()
            .border_color(if self.state == SliderState::Focused {
                crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)
            } else {
                crate::atoms::SemanticColor::Background.resolve(cx)
            })
            .bg(crate::atoms::SemanticColor::Background.resolve(cx))
            .flex()
            .items_center()
            .justify_center()
            .when(self.variant != SliderHandleVariant::Fill, |d| {
                d.child(
                    div()
                        .flex_none()
                        .rounded_full()
                        .size(px(if self.variant == SliderHandleVariant::Stroke {
                            tokens::Space::XS
                        } else {
                            tokens::Space::SM
                        }))
                        .bg(color),
                )
            })
            .when(self.state != SliderState::Disabled, |d| d.shadow_sm())
            .when(self.state == SliderState::Disabled, |d| {
                d.bg(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .opacity(0.5)
            })
    }
}

/// A selectable gradient color stop. Positioning and dragging belong to its owner.
#[derive(Clone, Copy, Debug)]
pub struct SliderGradientStop {
    pub color: Hsla,
    pub selected: bool,
    pub disabled: bool,
}
impl SliderGradientStop {
    pub fn render(self, id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
        div()
            .id(id)
            .key_context(super::CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .flex()
            .items_center()
            .justify_center()
            .size(px(tokens::SliderGeometry::STOP_SIZE))
            .rounded(px(tokens::Radius::CONTROL))
            .border_1()
            .border_color(if self.selected {
                crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)
            } else {
                crate::atoms::SemanticColor::Border.resolve(cx)
            })
            .bg(if self.selected {
                crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)
            } else {
                crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx)
            })
            .focus(|d| d.border_color(crate::atoms::SemanticColor::BorderSelected.resolve(cx)))
            .when(!self.disabled, |d| {
                d.hover(|d| d.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx)))
                    .cursor_pointer()
            })
            .when(self.disabled, |d| d.opacity(0.5).tab_stop(false))
            .child(
                div()
                    .size(px(tokens::ControlSize::INLINE))
                    .rounded(px(tokens::Radius::CONTROL))
                    .bg(pattern_slash(
                        crate::atoms::SemanticColor::TextTertiary
                            .resolve(cx)
                            .opacity(0.22),
                        0.35,
                        0.35,
                    ))
                    .child(
                        div()
                            .size_full()
                            .rounded(px(tokens::Radius::CONTROL))
                            .bg(self.color),
                    ),
            )
    }
}
