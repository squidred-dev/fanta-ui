use super::*;
use crate::molecules::{
    POPUP_SAFE_MARGIN, menu_item, popup_max_height, popup_surface, popup_width,
};
use gpui_component::{Sizable as _, input::Input};

impl Timeline {
    fn menu_action(
        &self,
        id: SharedString,
        label: SharedString,
        icon: LucideIcon,
        action: TimelineAction,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        menu_item(id.clone(), px(tokens::RowHeight::MENU), cx)
            .debug_selector(move || format!("timeline-menu-{id}"))
            .child(render_lucide_icon(
                icon,
                Color::TextSecondary.resolve(cx),
                tokens::IconSize::SM,
            ))
            .child(truncating_label(label))
            .on_activate(cx.listener(move |this, _, window, cx| {
                this.overlay = None;
                this.focus_handle.focus(window, cx);
                cx.emit(action.clone());
                cx.notify();
            }))
            .into_any_element()
    }
    fn apply_custom_easing(&mut self, spring: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Overlay::Easing(target)) = self.overlay.clone() else {
            return;
        };
        let value = self.easing_input.as_ref().unwrap().read(cx).value();
        let values: Option<Vec<f32>> = value
            .split(',')
            .map(|s| s.trim().parse::<f32>().ok())
            .collect();
        let easing = values
            .filter(|v| v.iter().all(|x| x.is_finite()))
            .and_then(|v| {
                if spring {
                    if v.len() == 1 && (0.0..=1.).contains(&v[0]) {
                        Some(TimelineEasing::Spring { bounce: v[0] })
                    } else {
                        None
                    }
                } else if v.len() == 4 && (0.0..=1.).contains(&v[0]) && (0.0..=1.).contains(&v[2]) {
                    Some(TimelineEasing::CubicBezier([v[0], v[1], v[2], v[3]]))
                } else {
                    None
                }
            });
        if let Some(easing) = easing {
            self.overlay = None;
            self.easing_error = None;
            self.focus_handle.focus(window, cx);
            cx.emit(TimelineAction::EasingChangeRequested { target, easing });
        } else {
            self.easing_error = Some(
                if spring {
                    "Enter a bounce value from 0 to 1."
                } else {
                    "Enter x1, y1, x2, y2. X values must be 0–1."
                }
                .into(),
            );
        }
        cx.notify();
    }
    pub(super) fn render_overlay(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let width = popup_width(window, tokens::MenuWidth::PICKER);
        let mut menu = popup_surface("timeline-menu", px(tokens::Radius::MENU), cx)
            .debug_selector(|| "timeline-menu".to_owned())
            .w(width)
            .max_h(popup_max_height(window))
            .p_2()
            .gap_1()
            .typography(TypographyToken::BodyMedium)
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.overlay = None;
                this.easing_error = None;
                cx.notify();
            }));
        match self.overlay.as_ref().unwrap() {
            Overlay::Playback => {
                menu = menu.child(
                    div()
                        .px_2()
                        .py_1()
                        .typography(TypographyToken::BodyMediumStrong)
                        .child("Playback"),
                );
                for playback in TimelinePlayback::ALL {
                    menu = menu.child(self.menu_action(
                        playback.label().into(),
                        playback.label().into(),
                        match playback {
                            TimelinePlayback::Once => LucideIcon::ArrowRight,
                            TimelinePlayback::Loop => LucideIcon::Repeat2,
                            TimelinePlayback::PingPong => LucideIcon::ArrowLeftRight,
                        },
                        TimelineAction::PlaybackChangeRequested { playback },
                        cx,
                    ));
                }
            }
            Overlay::Presets => {
                menu = menu.child(
                    div()
                        .px_2()
                        .py_1()
                        .typography(TypographyToken::BodyMediumStrong)
                        .child("Animation presets"),
                );
                for preset in &self.view_data.presets {
                    menu = menu.child(self.menu_action(
                        preset.id.clone(),
                        preset.name.clone(),
                        LucideIcon::Clapperboard,
                        TimelineAction::PresetApplyRequested {
                            track_ids: self.selected_tracks(),
                            preset_id: preset.id.clone(),
                            time_ms: self.view_data.current_time_ms,
                        },
                        cx,
                    ));
                }
            }
            Overlay::Easing(target) => {
                menu = menu.child(
                    div()
                        .px_2()
                        .py_1()
                        .typography(TypographyToken::BodyMediumStrong)
                        .child("Easing"),
                );
                for easing in TimelineEasing::presets() {
                    let label = easing.label();
                    menu = menu.child(self.menu_action(
                        label.into(),
                        label.into(),
                        LucideIcon::Spline,
                        TimelineAction::EasingChangeRequested {
                            target: target.clone(),
                            easing,
                        },
                        cx,
                    ));
                }
                menu = menu.child(
                    v_flex()
                        .border_t_1()
                        .border_color(Color::Border.resolve(cx))
                        .pt_2()
                        .gap_2()
                        .child(
                            div()
                                .typography(TypographyToken::BodyMediumStrong)
                                .child("Custom easing"),
                        )
                        .child(
                            Input::new(self.easing_input.as_ref().unwrap())
                                .small()
                                .typography(TypographyToken::BodyMedium),
                        )
                        .child(
                            div()
                                .text_color(Color::TextTertiary.resolve(cx))
                                .child("Bézier: x1, y1, x2, y2 · Spring: bounce 0–1"),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    self.button(
                                        "apply-bezier",
                                        "Apply custom Bézier",
                                        LucideIcon::Spline,
                                        false,
                                        true,
                                        cx,
                                    )
                                    .w_auto()
                                    .px_2()
                                    .gap_1()
                                    .child("Apply Bézier")
                                    .on_activate(cx.listener(|this, _, window, cx| {
                                        this.apply_custom_easing(false, window, cx)
                                    })),
                                )
                                .child(
                                    self.button(
                                        "apply-spring",
                                        "Apply custom spring",
                                        LucideIcon::Activity,
                                        false,
                                        true,
                                        cx,
                                    )
                                    .w_auto()
                                    .px_2()
                                    .gap_1()
                                    .child("Apply spring")
                                    .on_activate(cx.listener(|this, _, window, cx| {
                                        this.apply_custom_easing(true, window, cx)
                                    })),
                                ),
                        )
                        .when_some(self.easing_error.clone(), |panel, error| {
                            panel
                                .child(div().text_color(Color::TextDanger.resolve(cx)).child(error))
                        }),
                );
            }
        }
        deferred(
            anchored()
                .position(point(
                    (self.surface_bounds.center().x - width / 2.).max(px(POPUP_SAFE_MARGIN)),
                    self.surface_bounds.top() + px(HEADER),
                ))
                .snap_to_window_with_margin(px(POPUP_SAFE_MARGIN))
                .child(menu),
        )
        .with_priority(15)
        .into_any_element()
    }
}
