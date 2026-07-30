//! Host-controlled animation timeline with Figma Motion empty-state chrome.

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, prelude::FluentBuilder as _,
    px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, v_flex,
};

use crate::controls::{ActivateControl, CONTROL_KEY_CONTEXT, compact_icon_control};

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineViewData {
    pub duration_ms: u32,
    pub current_time_ms: u32,
    pub playing: bool,
    pub looping: bool,
    pub zoom: f32,
}

impl Default for TimelineViewData {
    fn default() -> Self {
        Self {
            duration_ms: 2_000,
            current_time_ms: 0,
            playing: false,
            looping: true,
            zoom: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimelineAction {
    PlayStateChangeRequested { playing: bool },
    LoopChangeRequested { looping: bool },
    AddKeyframeRequested { time_ms: u32 },
    SeekRequested { time_ms: u32 },
    ZoomChangeRequested { zoom: f32 },
    AskAgentRequested,
    EmptyStateDismissed,
}

pub struct Timeline {
    id: SharedString,
    focus_handle: FocusHandle,
    view_data: TimelineViewData,
    show_empty_state: bool,
    ruler_bounds: Option<Bounds<Pixels>>,
}

impl EventEmitter<TimelineAction> for Timeline {}

impl Timeline {
    pub fn new(
        id: impl Into<SharedString>,
        view_data: TimelineViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            view_data,
            show_empty_state: true,
            ruler_bounds: None,
        }
    }

    pub fn set_view_data(&mut self, view_data: TimelineViewData, cx: &mut Context<Self>) {
        self.view_data = view_data;
        cx.notify();
    }

    pub fn restore_empty_state(&mut self, cx: &mut Context<Self>) {
        self.show_empty_state = true;
        cx.notify();
    }

    fn render_transport(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .w(px(296.))
            .h(px(50.))
            .flex_none()
            .px(px(8.))
            .gap(px(8.))
            .border_r_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                compact_icon_control(
                    SharedString::from(format!("{}-play", self.id)),
                    px(22.),
                    px(4.),
                    cx,
                )
                .debug_selector(|| "timeline-play".to_owned())
                .focus(|style| style.border_1().border_color(cx.theme().selection))
                .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                    cx.emit(TimelineAction::PlayStateChangeRequested {
                        playing: !this.view_data.playing,
                    });
                }))
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.emit(TimelineAction::PlayStateChangeRequested {
                        playing: !this.view_data.playing,
                    });
                }))
                .child(if self.view_data.playing { "Ⅱ" } else { "▷" }),
            )
            .child(
                compact_icon_control(
                    SharedString::from(format!("{}-keyframe", self.id)),
                    px(22.),
                    px(4.),
                    cx,
                )
                .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                    cx.emit(TimelineAction::AddKeyframeRequested {
                        time_ms: this.view_data.current_time_ms,
                    });
                }))
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.emit(TimelineAction::AddKeyframeRequested {
                        time_ms: this.view_data.current_time_ms,
                    });
                }))
                .child("◇"),
            )
            .child(
                h_flex()
                    .h(px(24.))
                    .rounded(px(4.))
                    .bg(cx.theme().secondary)
                    .text_size(px(10.))
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        div()
                            .px(px(7.))
                            .child(format!("{:04}", self.view_data.current_time_ms)),
                    )
                    .child(
                        div()
                            .h_full()
                            .px(px(7.))
                            .items_center()
                            .border_l_1()
                            .border_color(cx.theme().border)
                            .child(format!("{:04}", self.view_data.duration_ms)),
                    )
                    .child(
                        div()
                            .h_full()
                            .px(px(6.))
                            .items_center()
                            .border_l_1()
                            .border_color(cx.theme().border)
                            .child("ms"),
                    ),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{}-loop", self.id)))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .size(px(22.))
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .when(self.view_data.looping, |button| {
                        button.bg(cx.theme().secondary)
                    })
                    .hover(|style| style.bg(cx.theme().accent))
                    .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                        cx.emit(TimelineAction::LoopChangeRequested {
                            looping: !this.view_data.looping,
                        });
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        cx.emit(TimelineAction::LoopChangeRequested {
                            looping: !this.view_data.looping,
                        });
                    }))
                    .child("⟳"),
            )
            .into_any_element()
    }

    fn render_ruler(&self, cx: &mut Context<Self>) -> AnyElement {
        let segment_width = 280. * self.view_data.zoom.clamp(0.25, 2.);
        let timeline = cx.entity();
        let mut ruler = h_flex()
            .id(SharedString::from(format!("{}-ruler", self.id)))
            .relative()
            .h(px(50.))
            .flex_1()
            .overflow_hidden()
            .border_b_1()
            .border_color(cx.theme().border)
            .cursor_pointer()
            .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
                let Some(bounds) = this.ruler_bounds else {
                    return;
                };
                let Some(position) = event.mouse_position() else {
                    cx.emit(TimelineAction::SeekRequested {
                        time_ms: this.view_data.current_time_ms,
                    });
                    return;
                };
                let available_width = f32::from(bounds.size.width - px(18.)).max(1.);
                let offset = f32::from(position.x - bounds.left() - px(18.));
                let ratio = (offset / available_width).clamp(0., 1.);
                cx.emit(TimelineAction::SeekRequested {
                    time_ms: (this.view_data.duration_ms as f32 * ratio).round() as u32,
                });
            }))
            .child(
                canvas(
                    move |bounds, _, app| {
                        timeline.update(app, |this, _| {
                            this.ruler_bounds = Some(bounds);
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .left_0()
                .top_0()
                .size_full(),
            )
            .child(div().w(px(18.)).h_full().flex_none());
        for index in 0..=10 {
            let label = self
                .view_data
                .duration_ms
                .saturating_mul(index)
                .checked_div(10)
                .unwrap_or_default();
            ruler = ruler.child(
                v_flex()
                    .w(px(segment_width))
                    .h_full()
                    .flex_none()
                    .items_start()
                    .child(
                        div()
                            .ml(px(segment_width - 1.))
                            .w(px(1.))
                            .h(px(8.))
                            .bg(cx.theme().border),
                    )
                    .child(
                        div()
                            .mt(px(13.))
                            .text_size(px(10.))
                            .text_color(cx.theme().muted_foreground)
                            .child(label.to_string()),
                    ),
            );
        }
        let playhead_left = 18.
            + if self.view_data.duration_ms == 0 {
                0.
            } else {
                self.view_data
                    .current_time_ms
                    .min(self.view_data.duration_ms) as f32
                    / self.view_data.duration_ms as f32
                    * segment_width
                    * 10.
            };
        ruler
            .child(
                div()
                    .absolute()
                    .left(px(playhead_left))
                    .top_0()
                    .bottom_0()
                    .w(px(1.))
                    .bg(cx.theme().selection),
            )
            .child(
                div()
                    .absolute()
                    .left(px(playhead_left - 8.))
                    .top_0()
                    .w(px(17.))
                    .h(px(14.))
                    .rounded_b(px(4.))
                    .bg(cx.theme().selection),
            )
            .into_any_element()
    }

    fn render_zoom(&self, cx: &mut Context<Self>) -> AnyElement {
        let thumb_left = self.view_data.zoom.clamp(0., 1.) * 62.;
        let fill_width = thumb_left + 6.;
        h_flex()
            .w(px(160.))
            .h(px(50.))
            .flex_none()
            .gap(px(9.))
            .justify_center()
            .border_l_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .id(SharedString::from(format!("{}-zoom", self.id)))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .relative()
                    .w(px(92.))
                    .h(px(18.))
                    .cursor_pointer()
                    .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                        let zoom = if this.view_data.zoom >= 2. {
                            0.25
                        } else {
                            (this.view_data.zoom + 0.25).min(2.)
                        };
                        cx.emit(TimelineAction::ZoomChangeRequested { zoom });
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        let zoom = if this.view_data.zoom >= 2. {
                            0.25
                        } else {
                            (this.view_data.zoom + 0.25).min(2.)
                        };
                        cx.emit(TimelineAction::ZoomChangeRequested { zoom });
                    }))
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(8.))
                            .h(px(4.))
                            .rounded(px(2.))
                            .bg(cx.theme().secondary),
                    )
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top(px(8.))
                            .w(px(fill_width))
                            .h(px(4.))
                            .rounded(px(2.))
                            .bg(cx.theme().selection),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(thumb_left))
                            .top(px(4.))
                            .size(px(12.))
                            .rounded(px(6.))
                            .bg(cx.theme().foreground),
                    ),
            )
            .child(Icon::new(IconName::PanelBottom).small())
            .into_any_element()
    }

    fn render_empty_state(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .relative()
            .w(px(418.))
            .h(px(144.))
            .items_center()
            .justify_center()
            .gap(px(6.))
            .rounded(px(13.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                compact_icon_control(
                    SharedString::from(format!("{}-dismiss-empty", self.id)),
                    px(22.),
                    px(4.),
                    cx,
                )
                    .absolute()
                    .right(px(10.))
                    .top(px(16.5))
                    .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                        this.show_empty_state = false;
                        cx.emit(TimelineAction::EmptyStateDismissed);
                        cx.notify();
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.show_empty_state = false;
                        cx.emit(TimelineAction::EmptyStateDismissed);
                        cx.notify();
                    }))
                    .child(Icon::new(IconName::Close).small()),
            )
            .child(
                div()
                    .relative()
                    .top(px(11.))
                    .font_semibold()
                    .text_size(px(14.))
                    .child("No animations in timeline"),
            )
            .child(
                div()
                    .relative()
                    .top(px(4.5))
                    .max_w(px(330.))
                    .text_center()
                    .text_size(px(12.))
                    .line_height(px(16.))
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Select objects on the canvas to create an animation, or ask the Figma agent to create an idea from scratch.",
                    ),
            )
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-ask-agent", self.id)))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .mt(px(8.))
                    .relative()
                    .top(px(6.))
                    .h(px(24.))
                    .px(px(7.))
                    .gap(px(5.))
                    .rounded(px(5.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .cursor_pointer()
                    .text_size(px(12.))
                    .hover(|style| style.bg(cx.theme().accent))
                    .on_action(cx.listener(|_, _: &ActivateControl, _, cx| {
                        cx.emit(TimelineAction::AskAgentRequested);
                    }))
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(TimelineAction::AskAgentRequested);
                    }))
                    .child("✦")
                    .child("Ask agent"),
            )
            .into_any_element()
    }
}

impl Focusable for Timeline {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Timeline {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let segment_width = 280. * self.view_data.zoom.clamp(0.25, 2.);
        let playhead_left = 18.
            + if self.view_data.duration_ms == 0 {
                0.
            } else {
                self.view_data
                    .current_time_ms
                    .min(self.view_data.duration_ms) as f32
                    / self.view_data.duration_ms as f32
                    * segment_width
                    * 10.
            };
        v_flex()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .min_h(px(0.))
            .overflow_hidden()
            .rounded_b(px(13.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .h(px(50.))
                    .w_full()
                    .flex_none()
                    .child(self.render_transport(cx))
                    .child(self.render_ruler(cx))
                    .child(self.render_zoom(cx)),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_start()
                    .child(
                        div()
                            .w(px(296.))
                            .h_full()
                            .flex_none()
                            .border_r_1()
                            .border_color(cx.theme().border),
                    )
                    .child(
                        v_flex()
                            .relative()
                            .flex_1()
                            .h_full()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(playhead_left))
                                    .top_0()
                                    .bottom_0()
                                    .w(px(1.))
                                    .bg(cx.theme().selection),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .right(px(24.))
                                    .bottom(px(20.))
                                    .size(px(32.))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(16.))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .bg(cx.theme().background)
                                    .text_size(px(18.))
                                    .child("?"),
                            ),
                    ),
            )
            .when(self.show_empty_state, |root| {
                root.child(
                    h_flex()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(px(50.))
                        .bottom_0()
                        .items_center()
                        .justify_center()
                        .child(self.render_empty_state(cx)),
                )
            })
    }
}

#[cfg(test)]
mod interaction_tests;
