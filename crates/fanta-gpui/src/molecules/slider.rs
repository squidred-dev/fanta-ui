//! Controlled normalized slider with balanced pointer and keyboard edit intents.
use crate::atoms::{
    SliderBackground, SliderHandle, SliderHandleVariant, SliderState, tokens, track_bounds,
};
use gpui::{prelude::FluentBuilder, *};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderVariant {
    Range,
    Stepper { intervals: u16 },
    Centered,
    CornerRadius { marker: f32 },
    ColorRange,
    Fill,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderPhase {
    Begin,
    Preview,
    Commit,
    Cancel,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderAction {
    pub value: f32,
    pub phase: SliderPhase,
}

pub struct Slider {
    id: SharedString,
    focus: FocusHandle,
    value: f32,
    draft: Option<f32>,
    origin: f32,
    bounds: Bounds<Pixels>,
    variant: SliderVariant,
    background: SliderBackground,
    disabled: bool,
}
impl EventEmitter<SliderAction> for Slider {}
impl Focusable for Slider {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
fn normalized(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., 1.)
    } else {
        0.
    }
}
impl Slider {
    pub fn new(id: impl Into<SharedString>, value: f32, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            focus: cx.focus_handle(),
            value: normalized(value),
            draft: None,
            origin: 0.,
            bounds: Bounds::default(),
            variant: SliderVariant::Range,
            background: SliderBackground::Default,
            disabled: false,
        }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    /// Echo accepted values without interrupting a current pointer draft.
    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        let value = normalized(value);
        if self.value != value {
            self.value = value;
            cx.notify();
        }
    }
    pub fn configure(
        &mut self,
        variant: SliderVariant,
        background: SliderBackground,
        disabled: bool,
        cx: &mut Context<Self>,
    ) {
        if self.variant == variant && self.background == background && self.disabled == disabled {
            return;
        }
        if disabled && !self.disabled {
            self.finish(false, cx);
        }
        self.variant = variant;
        self.background = background;
        self.disabled = disabled;
        cx.notify();
    }
    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        self.finish(false, cx);
    }
    fn snap(&self, value: f32) -> f32 {
        let value = normalized(value);
        match self.variant {
            SliderVariant::Stepper { intervals } if intervals > 0 => {
                (value * f32::from(intervals)).round() / f32::from(intervals)
            }
            _ => value,
        }
    }
    fn pointer(&self, p: Point<Pixels>) -> f32 {
        let inset = px(tokens::SliderGeometry::INSET);
        let width = f32::from(self.bounds.size.width - inset * 2.);
        if width <= 0. {
            return self.value;
        }
        self.snap(f32::from(p.x - self.bounds.left() - inset) / width)
    }
    fn begin(&mut self, p: Point<Pixels>, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.origin = self.value;
        cx.emit(SliderAction {
            value: self.value,
            phase: SliderPhase::Begin,
        });
        self.preview(p, cx);
    }
    fn preview(&mut self, p: Point<Pixels>, cx: &mut Context<Self>) {
        let value = self.pointer(p);
        self.draft = Some(value);
        cx.emit(SliderAction {
            value,
            phase: SliderPhase::Preview,
        });
        cx.notify();
    }
    fn finish(&mut self, commit: bool, cx: &mut Context<Self>) {
        if let Some(value) = self.draft.take() {
            cx.emit(SliderAction {
                value: if commit { value } else { self.origin },
                phase: if commit {
                    SliderPhase::Commit
                } else {
                    SliderPhase::Cancel
                },
            });
            cx.notify();
        }
    }
    fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if event.keystroke.key == "escape" {
            if self.draft.is_some() {
                self.finish(false, cx);
                window.prevent_default();
                cx.stop_propagation();
            }
            return;
        }
        let step = match self.variant {
            SliderVariant::Stepper { intervals } if intervals > 0 => 1. / f32::from(intervals),
            _ => 0.01,
        } * if event.keystroke.modifiers.shift {
            10.
        } else {
            1.
        };
        let value = match event.keystroke.key.as_str() {
            "left" | "down" => self.value - step,
            "right" | "up" => self.value + step,
            "home" => 0.,
            "end" => 1.,
            _ => return,
        };
        self.finish(false, cx);
        cx.emit(SliderAction {
            value: self.value,
            phase: SliderPhase::Begin,
        });
        cx.emit(SliderAction {
            value: self.snap(value),
            phase: SliderPhase::Commit,
        });
        window.prevent_default();
        cx.stop_propagation();
    }
}
impl Render for Slider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = self.draft.unwrap_or(self.value);
        let state = if self.disabled {
            SliderState::Disabled
        } else if self.focus.is_focused(window) {
            SliderState::Focused
        } else {
            SliderState::Default
        };
        let geometry = tokens::SliderGeometry::TRACK_HEIGHT;
        let colored = matches!(
            self.variant,
            SliderVariant::ColorRange | SliderVariant::Fill
        );
        let background = if self.variant == SliderVariant::ColorRange {
            SliderBackground::Hue
        } else {
            self.background.clone()
        };
        let id = self.id.clone();
        let thumb_id = SharedString::from(format!("{}-thumb", self.id));
        let origin = if self.variant == SliderVariant::Centered {
            0.5
        } else {
            0.
        };
        let fill = (!colored && !self.disabled).then(|| {
            div()
                .debug_selector(|| "slider-fill".to_owned())
                .absolute()
                .left(relative(value.min(origin)))
                .w(relative((value - origin).abs()))
                .top_0()
                .bottom_0()
                .rounded_full()
                .bg(crate::atoms::SemanticColor::BackgroundBrand.resolve(cx))
        });
        let mut travel = div()
            .absolute()
            .left(px(tokens::SliderGeometry::INSET))
            .right(px(tokens::SliderGeometry::INSET))
            .top_0()
            .bottom_0();
        let ticks = match self.variant {
            SliderVariant::Stepper { intervals } => (0..=intervals.min(100))
                .map(|i| f32::from(i) / f32::from(intervals.clamp(1, 100)))
                .collect::<Vec<_>>(),
            SliderVariant::Centered => vec![0.5],
            SliderVariant::CornerRadius { marker } => vec![normalized(marker)],
            _ => vec![],
        };
        travel = travel.children(ticks.into_iter().map(|p| {
            div()
                .absolute()
                .left(relative(p))
                .ml(px(-tokens::Space::XS / 2.))
                .top(px((geometry - tokens::Space::XS) / 2.))
                .size(px(tokens::Space::XS))
                .rounded_full()
                .bg(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
        }));
        let handle = SliderHandle {
            variant: if colored {
                SliderHandleVariant::Fill
            } else {
                SliderHandleVariant::Stroke
            },
            state,
            color: None,
        }
        .render(cx)
        .debug_selector(move || thumb_id.to_string())
        .absolute()
        .left(relative(value))
        .ml(px(-tokens::SliderGeometry::HANDLE_SIZE / 2.))
        .top(px((geometry - tokens::SliderGeometry::HANDLE_SIZE) / 2.));
        div()
            .id(self.id.clone())
            .debug_selector(move || id.to_string())
            .track_focus(&self.focus.clone().tab_index(0).tab_stop(!self.disabled))
            .relative()
            .w_full()
            .min_w(px(tokens::SliderGeometry::MIN_WIDTH))
            .h(px(geometry))
            .when(!self.disabled, |d| d.cursor_col_resize())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    if !this.disabled {
                        this.focus.focus(window, cx);
                        this.begin(e.position, cx);
                    }
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                if this.draft.is_some() && e.dragging() {
                    this.preview(e.position, cx);
                    cx.stop_propagation();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| this.finish(true, cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| this.finish(true, cx)),
            )
            .on_key_down(cx.listener(Self::key))
            .child(background.render(state, cx))
            .children(fill)
            .child(travel.child(handle))
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.bounds = bounds
            }))
    }
}

#[cfg(test)]
mod tests;
