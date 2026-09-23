//! Controlled motion editor: synchronized ruler, tracks, keyframes and transport.
mod controls;
mod interaction;
mod model;
mod overlays;
mod tracks;
pub use model::*;

use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _, TypographyToken,
    icon_button, render_lucide_icon, tokens, track_bounds, truncating_label,
};
use crate::molecules::{Slider, SliderAction, SliderPhase};
use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    h_flex,
    input::{InputEvent, InputState},
    tooltip::Tooltip,
    v_flex,
};

// One geometry source for headers, labels, lanes and their hit targets.
const RAIL: f32 = tokens::InputGeometry::TEXT_WIDTH + tokens::Space::LG;
const ROW: f32 = tokens::RowHeight::PAGE;
const HEADER: f32 = tokens::RowHeight::SECTION_HEADER + tokens::Space::SM;
const GUTTER: f32 = tokens::Space::LG;
pub const TIMELINE_MIN_WIDTH: f32 = RAIL + tokens::InputGeometry::COMBO_WIDTH;

#[derive(Clone, Debug, PartialEq)]
enum Overlay {
    Playback,
    Easing(TimelineEasingTarget),
    Presets,
}
#[derive(Clone, Copy)]
enum Trim {
    Move,
    Start,
    End,
}
#[derive(Clone)]
enum Drag {
    Seek,
    Keys {
        origin: Point<Pixels>,
        times: Vec<TimelineKeyframeTime>,
        delta: i64,
    },
    Span {
        origin: Point<Pixels>,
        track_id: SharedString,
        clip_id: Option<SharedString>,
        start: u32,
        end: u32,
        trim: Trim,
        delta: i64,
    },
    Duration {
        time: u32,
    },
    Height {
        origin: Pixels,
        height: u16,
        draft: u16,
    },
    Marquee {
        origin: Point<Pixels>,
        current: Point<Pixels>,
        additive: bool,
    },
}
pub struct Timeline {
    id: SharedString,
    focus_handle: FocusHandle,
    view_data: TimelineViewData,
    auto_keyframe_available: bool,
    comments_available: bool,
    show_empty_state: bool,
    collapsed: bool,
    overlay: Option<Overlay>,
    drag: Option<Drag>,
    ruler_bounds: Bounds<Pixels>,
    body_bounds: Bounds<Pixels>,
    surface_bounds: Bounds<Pixels>,
    key_bounds: Vec<(SharedString, Bounds<Pixels>)>,
    scroll_x: f32,
    rows_scroll: ScrollHandle,
    zoom_slider: Entity<Slider>,
    time_input: Option<Entity<InputState>>,
    duration_input: Option<Entity<InputState>>,
    easing_input: Option<Entity<InputState>>,
    easing_error: Option<SharedString>,
    rename_input: Option<Entity<InputState>>,
    renaming_track: Option<SharedString>,
    subscriptions: Vec<Subscription>,
}
impl EventEmitter<TimelineAction> for Timeline {}
impl Focusable for Timeline {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl Timeline {
    pub fn new(
        id: impl Into<SharedString>,
        view_data: TimelineViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let view_data = view_data.normalized();
        let zoom_slider =
            cx.new(|cx| Slider::new("timeline-zoom", Self::zoom_to_slider(view_data.zoom), cx));
        let subscription = cx.subscribe(&zoom_slider, |this, _, event: &SliderAction, cx| {
            if event.phase != SliderPhase::Begin {
                cx.emit(TimelineAction::ZoomChangeRequested {
                    zoom: Self::slider_to_zoom(event.value),
                });
            }
            this.overlay = None;
        });
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            view_data,
            auto_keyframe_available: true,
            comments_available: true,
            show_empty_state: true,
            collapsed: false,
            overlay: None,
            drag: None,
            ruler_bounds: Bounds::default(),
            body_bounds: Bounds::default(),
            surface_bounds: Bounds::default(),
            key_bounds: vec![],
            scroll_x: 0.,
            rows_scroll: ScrollHandle::new(),
            zoom_slider,
            time_input: None,
            duration_input: None,
            easing_input: None,
            easing_error: None,
            rename_input: None,
            renaming_track: None,
            subscriptions: vec![subscription],
        }
    }
    pub fn view_data(&self) -> &TimelineViewData {
        &self.view_data
    }
    pub fn set_auto_keyframe_available(&mut self, available: bool, cx: &mut Context<Self>) {
        if self.auto_keyframe_available != available {
            self.auto_keyframe_available = available;
            cx.notify();
        }
    }
    pub fn set_comments_available(&mut self, available: bool, cx: &mut Context<Self>) {
        if self.comments_available != available {
            self.comments_available = available;
            cx.notify();
        }
    }
    pub fn set_view_data(&mut self, data: TimelineViewData, cx: &mut Context<Self>) {
        self.view_data = data.normalized();
        if self.renaming_track.as_ref().is_some_and(|id| {
            self.view_data.read_only
                || !self
                    .view_data
                    .tracks
                    .iter()
                    .any(|t| &t.id == id && !t.locked)
        }) {
            self.renaming_track = None;
        }
        self.scroll_x = self.scroll_x.min(self.max_scroll());
        let zoom = Self::zoom_to_slider(self.view_data.zoom);
        self.zoom_slider
            .update(cx, |slider, cx| slider.set_value(zoom, cx));
        if self.view_data.read_only {
            if matches!(
                self.drag,
                Some(Drag::Keys { .. } | Drag::Span { .. } | Drag::Duration { .. })
            ) {
                self.drag = None;
            }
            if !matches!(self.overlay, Some(Overlay::Playback)) {
                self.overlay = None;
            }
        }
        cx.notify();
    }
    pub fn restore_empty_state(&mut self, cx: &mut Context<Self>) {
        self.show_empty_state = true;
        cx.notify();
    }
    fn zoom_to_slider(zoom: f32) -> f32 {
        (zoom.log2() + 2.) / 6.
    }
    fn slider_to_zoom(value: f32) -> f32 {
        2_f32.powf(value * 6. - 2.)
    }
    fn pixels_per_ms(&self) -> f32 {
        (self.ruler_bounds.size.width.as_f32() - 2. * GUTTER).max(1.)
            / self.view_data.duration_ms as f32
            * self.view_data.zoom
    }
    fn time_x(&self, time: u32) -> f32 {
        GUTTER + time as f32 * self.pixels_per_ms() - self.scroll_x
    }
    fn max_scroll(&self) -> f32 {
        ((self.ruler_bounds.size.width.as_f32() - 2. * GUTTER).max(1.) * (self.view_data.zoom - 1.))
            .max(0.)
    }
    fn time_at(&self, position: Point<Pixels>) -> u32 {
        (((position.x - self.ruler_bounds.left()).as_f32() + self.scroll_x - GUTTER)
            / self.pixels_per_ms())
        .round()
        .clamp(0., self.view_data.duration_ms as f32) as u32
    }
    fn step_ms(&self) -> u32 {
        let target = 72. / self.pixels_per_ms();
        let magnitude = 10_f32.powf(target.max(1.).log10().floor());
        [1., 2., 5., 10.]
            .into_iter()
            .map(|v| (v * magnitude) as u32)
            .find(|v| *v as f32 >= target)
            .unwrap_or(1000)
            .max(1)
    }
    fn ensure_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.rename_input.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("Layer name"));
            self.subscriptions.push(cx.subscribe_in(
                &input,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        this.commit_rename(cx);
                        if matches!(event, InputEvent::PressEnter { .. }) {
                            this.focus_handle.focus(window, cx);
                        }
                    }
                },
            ));
            self.rename_input = Some(input);
        }
        if self.time_input.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx));
            self.subscriptions.push(cx.subscribe_in(
                &input,
                window,
                |this, input, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. })
                        && let Some(time) = this.view_data.time_unit.parse(&input.read(cx).value())
                    {
                        cx.emit(TimelineAction::SeekRequested {
                            time_ms: time.min(this.view_data.duration_ms),
                        });
                        this.focus_handle.focus(window, cx);
                    }
                },
            ));
            self.time_input = Some(input);
            let input = cx.new(|cx| InputState::new(window, cx));
            self.subscriptions.push(cx.subscribe_in(
                &input,
                window,
                |this, input, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. })
                        && !this.view_data.read_only
                        && let Some(time) = this.view_data.time_unit.parse(&input.read(cx).value())
                    {
                        cx.emit(TimelineAction::DurationChangeRequested {
                            duration_ms: time.max(1),
                        });
                        this.focus_handle.focus(window, cx);
                    }
                },
            ));
            self.duration_input = Some(input);
            self.easing_input =
                Some(cx.new(|cx| InputState::new(window, cx).placeholder("0.25, 0.1, 0.25, 1")));
        }
        for (input, time) in [
            (
                self.time_input.as_ref().unwrap(),
                self.view_data.current_time_ms,
            ),
            (
                self.duration_input.as_ref().unwrap(),
                self.view_data.duration_ms,
            ),
        ] {
            let value = self.view_data.time_unit.format(time);
            if !input.focus_handle(cx).is_focused(window)
                && input.read(cx).value().as_ref() != value
            {
                input.update(cx, |input, cx| input.set_value(value, window, cx));
            }
        }
    }
    fn begin_rename(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let Some(track) = self
            .view_data
            .tracks
            .iter()
            .find(|t| t.id == id && !t.locked)
        else {
            return;
        };
        if self.view_data.read_only {
            return;
        }
        let name = track.name.clone();
        self.ensure_inputs(window, cx);
        self.renaming_track = Some(id);
        self.overlay = None;
        self.drag = None;
        let input = self.rename_input.as_ref().unwrap();
        input.update(cx, |input, cx| input.set_value(name, window, cx));
        input.focus_handle(cx).focus(window, cx);
        window.on_next_frame(|window, cx| {
            window.dispatch_action(Box::new(gpui_component::input::SelectAll), cx)
        });
        cx.notify();
    }
    fn commit_rename(&mut self, cx: &mut Context<Self>) {
        let Some(track_id) = self.renaming_track.take() else {
            return;
        };
        let name: SharedString = self
            .rename_input
            .as_ref()
            .unwrap()
            .read(cx)
            .value()
            .trim()
            .to_owned()
            .into();
        if !self.view_data.read_only
            && !name.is_empty()
            && self
                .view_data
                .tracks
                .iter()
                .any(|t| t.id == track_id && !t.locked && t.name != name)
        {
            cx.emit(TimelineAction::TrackRenameRequested { track_id, name });
        }
        cx.notify();
    }
    fn input_focused(&self, window: &Window, cx: &App) -> bool {
        [
            &self.time_input,
            &self.duration_input,
            &self.easing_input,
            &self.rename_input,
        ]
        .into_iter()
        .flatten()
        .any(|input| input.focus_handle(cx).is_focused(window))
    }
    fn selected_tracks(&self) -> Vec<SharedString> {
        self.view_data
            .tracks
            .iter()
            .filter(|t| t.selected && !t.locked)
            .map(|t| t.id.clone())
            .collect()
    }
}
impl Render for Timeline {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_inputs(window, cx);
        self.key_bounds.clear();
        let height = if self.collapsed {
            HEADER
        } else if let Some(Drag::Height { draft, .. }) = self.drag {
            draft as f32
        } else {
            self.view_data.height as f32
        };
        v_flex()
            .id(self.id.clone())
            .debug_selector(|| "timeline".to_owned())
            .track_focus(&self.focus_handle)
            .key_context("FantaTimeline")
            .tab_index(0)
            .relative()
            .w_full()
            .h(px(height))
            .max_h_full()
            .min_h(px(0.))
            .min_w(px(0.))
            .overflow_hidden()
            .bg(Color::Background.resolve(cx))
            .text_color(Color::Text.resolve(cx))
            .typography(TypographyToken::BodyMedium)
            .border_t_1()
            .border_color(if self.view_data.auto_keyframe {
                Color::BackgroundDanger.resolve(cx)
            } else {
                Color::Border.resolve(cx)
            })
            .occlude()
            .on_scroll_wheel(cx.listener(|this, e, _, cx| this.wheel(e, cx)))
            .on_pinch(cx.listener(|this, event: &PinchEvent, _, cx| {
                cx.stop_propagation();
                if this.overlay.is_none() && event.position.y >= this.ruler_bounds.top() {
                    cx.emit(TimelineAction::ZoomChangeRequested {
                        zoom: (this.view_data.zoom * (1. + event.delta)).clamp(0.25, 16.),
                    });
                }
            }))
            .on_key_down(cx.listener(|this, e, window, cx| this.key_down(e, window, cx)))
            .on_mouse_move(
                cx.listener(|this, e: &MouseMoveEvent, _, cx| this.drag_move(e.position, cx)),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.finish_drag(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.finish_drag(cx)),
            )
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.surface_bounds = bounds
            }))
            .child(self.render_controls(window, cx))
            .when(!self.collapsed, |root| {
                root.child(self.render_ruler(cx))
                    .child(self.render_tracks(window, cx))
            })
            .child(
                div()
                    .id("timeline-resize")
                    .debug_selector(|| "timeline-resize".to_owned())
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(tokens::Space::XS))
                    .cursor_row_resize()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, e: &MouseDownEvent, window, cx| {
                            window.prevent_default();
                            this.drag = Some(Drag::Height {
                                origin: e.position.y,
                                height: this.view_data.height,
                                draft: this.view_data.height,
                            });
                            cx.stop_propagation();
                        }),
                    ),
            )
            .when(self.overlay.is_some(), |root| {
                root.child(self.render_overlay(window, cx))
            })
    }
}
#[cfg(test)]
mod interaction_tests;
