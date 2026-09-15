//! The Design story's development-host chrome: the scenario selector, the
//! node-and-variation preset matrix, the bespoke width resizer, knob rows,
//! and the gallery/reference shells around the inspector.
//!
//! The gallery harness is responsive: at or above
//! [`DESIGN_HARNESS_STACK_BREAKPOINT`] the fixture rail sits beside the
//! mock canvas and the inspector; below it the chrome stacks, folding the
//! rail into a collapsible controls section beneath the inspector, which
//! stays the priority element down to [`DESIGN_STORY_MIN_WIDTH`].

use super::*;
use crate::screens::knobs::{self, KnobOption};

impl Storybook {
    fn render_design_context_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2().flex_wrap();
        for scenario in DesignInspectionScenario::ALL {
            let active = self.design_screen.harness.inspection_scenario == scenario;
            buttons = buttons.child(
                Button::new(SharedString::from(format!(
                    "design-inspection-scenario-{}",
                    scenario.id()
                )))
                .label(scenario.label())
                .custom(
                    ButtonCustomVariant::new(cx)
                        .color(cx.theme().secondary)
                        .foreground(cx.theme().foreground)
                        .border(if active {
                            cx.theme().selection
                        } else {
                            cx.theme().border
                        })
                        .hover(cx.theme().accent)
                        .active(cx.theme().selection.opacity(0.22)),
                )
                .xsmall()
                .selected(active)
                .h(px(30.))
                .px_2()
                .rounded(px(5.))
                .border_1()
                .when(active, |button| {
                    button.hover(|style| style.bg(cx.theme().accent))
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.design_screen
                        .activate_inspection_scenario(scenario, cx);
                    this.design_screen.panel.focus_handle(cx).focus(window, cx);
                    cx.notify();
                })),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("INSPECTION CONTEXT"),
            )
            .child(buttons)
            .when(
                self.design_screen.harness.inspection_scenario
                    == DesignInspectionScenario::TextEdit,
                |content| {
                    content.child(
                        Button::new("design-text-range-revision")
                            .custom(
                                ButtonCustomVariant::new(cx)
                                    .color(cx.theme().selection.opacity(0.14))
                                    .foreground(cx.theme().foreground)
                                    .border(cx.theme().selection)
                                    .hover(cx.theme().selection.opacity(0.24))
                                    .active(cx.theme().selection.opacity(0.14)),
                            )
                            .xsmall()
                            .selected(true)
                            .w_full()
                            .h(px(30.))
                            .px_2()
                            .rounded(px(5.))
                            .border_1()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().selection.opacity(0.24)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.design_screen.advance_text_range(cx);
                                this.design_screen.panel.focus_handle(cx).focus(window, cx);
                            }))
                            .child(
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .child(div().text_xs().child(format!(
                                        "Selected text · revision {}",
                                        self.design_screen.harness.text_range_revision
                                    )))
                                    .child(div().text_xs().child("Select next range")),
                            ),
                    )
                },
            )
            .into_any_element()
    }

    fn render_design_resize_handle(&self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.design_screen.harness.panel_resize_drag.is_some();
        let tooltip = format!(
            "Resize Design inspector · {:.0} px · drag or use arrows and +/− (Shift for 32 px)",
            self.design_screen.harness.panel_width
        );
        Button::new("design-panel-resize-handle")
            .debug_selector(|| "design-panel-resize-handle".to_owned())
            .tooltip(tooltip)
            .xsmall()
            .compact()
            .selected(active)
            .w(px(DESIGN_HARNESS_RESIZE_HANDLE_WIDTH))
            .h_full()
            .p_0()
            .cursor_col_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    this.design_screen
                        .begin_panel_resize(f32::from(event.position.x), cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.design_screen
                    .handle_panel_resize_key(event, window, cx);
            }))
            .child(div().w(px(2.)).h(px(36.)).rounded_full().bg(if active {
                cx.theme().selection
            } else {
                cx.theme().border
            }))
            .into_any_element()
    }

    fn render_design_width_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2();
        for width in DESIGN_PANEL_WIDTH_PRESETS {
            let active = (self.design_screen.harness.panel_width - width).abs() < f32::EPSILON;
            buttons = buttons.child(
                Button::new(SharedString::from(format!("design-panel-width-{width:.0}")))
                    .label(format!("{width:.0}"))
                    .tooltip(format!("Set inspector width to {width:.0} px"))
                    .xsmall()
                    .compact()
                    .flex_1()
                    .selected(active)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.design_screen.set_panel_width(width, "set", cx);
                    })),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "PANEL WIDTH · {:.0} PX",
                        self.design_screen.harness.panel_width
                    )),
            )
            .child(buttons)
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Drag the inspector divider, or use arrows and +/− (Shift: 32 px)."),
            )
            .into_any_element()
    }

    fn render_design_workspace_toggle(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::enum_knob_row(
            "design-workspace",
            "INSPECTOR WORKSPACE",
            [
                DesignPanelWorkspaceMode::Design,
                DesignPanelWorkspaceMode::Draw,
            ]
            .map(|workspace_mode| KnobOption::new(workspace_mode, workspace_mode.label())),
            self.design_screen.harness.workspace_mode,
            |this, workspace_mode, _, cx| {
                this.design_screen
                    .set_workspace_mode(workspace_mode, "Story", cx);
            },
            cx,
        )
    }

    fn render_design_additional_labels_toggle(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::bool_knob_row(
            "design-additional-labels",
            "VIEW PREFERENCE",
            "Additional labels",
            self.design_screen.harness.additional_labels,
            |this, enabled, _, cx| {
                this.design_screen.harness.additional_labels = enabled;
                let panel = this.design_screen.panel.clone();
                this.design_screen.apply_inspection_context(&panel, cx);
                this.design_screen.harness.last_action = format!(
                    "Story set Additional labels {} — no document intent",
                    design_additional_labels_status(enabled).to_ascii_lowercase()
                )
                .into();
                cx.notify();
            },
            cx,
        )
    }

    fn render_design_nudge_preferences(&self, cx: &mut Context<Self>) -> AnyElement {
        let default = DesignNudgeSettings::default();
        let precise = DesignNudgeSettings::new(0.5, 8.).expect("valid Storybook nudge preset");
        v_flex()
            .w_full()
            .gap_2()
            .child(knobs::enum_knob_row(
                "design-nudge",
                "KEYBOARD NUDGE · SMALL / BIG",
                [
                    KnobOption::new(default, "Default · 1 / 10"),
                    KnobOption::new(precise, "Custom · 0.5 / 8"),
                ],
                self.design_screen.harness.nudge_settings,
                |this, nudge, _, cx| {
                    this.design_screen.harness.nudge_settings = nudge;
                    let panel = this.design_screen.panel.clone();
                    this.design_screen.apply_inspection_context(&panel, cx);
                    this.design_screen.harness.last_action = format!(
                        "Story set keyboard nudge to {} / {} — no document intent",
                        nudge.small(),
                        nudge.big()
                    )
                    .into();
                    cx.notify();
                },
                cx,
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Arrow uses Small; Shift+Arrow uses Big across numeric inspector fields.",
                    ),
            )
            .into_any_element()
    }

    fn render_design_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2().flex_wrap();
        let last_index = self.design_screen.host.nodes.len().saturating_sub(1);
        for (index, node) in self.design_screen.host.nodes.iter().cloned().enumerate() {
            let selected = self.design_screen.harness.selected_node == index;
            let kind = node.kind;
            let label = node.name.clone();
            buttons = buttons.child(
                Button::new(SharedString::from(format!("design-preset-{index}")))
                    .debug_selector(move || format!("design-preset-{index}"))
                    .when(index == last_index, |button| {
                        button.debug_selector(|| "design-preset-last".to_owned())
                    })
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(cx.theme().secondary)
                            .foreground(cx.theme().foreground)
                            .border(if selected {
                                cx.theme().selection
                            } else {
                                cx.theme().border
                            })
                            .hover(cx.theme().accent)
                            .active(cx.theme().selection.opacity(0.22)),
                    )
                    .xsmall()
                    .selected(selected)
                    .h(px(30.))
                    .px_2()
                    .rounded(px(5.))
                    .border_1()
                    .when(selected, |button| {
                        button.hover(|style| style.bg(cx.theme().accent))
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.design_screen.harness.selected_node = index;
                        this.design_screen.harness.inspection_scenario =
                            default_design_inspection_scenario_for_node(&node);
                        this.design_screen
                            .apply_inspection_context(&this.design_screen.panel, cx);
                        this.design_screen.panel.focus_handle(cx).focus(window, cx);
                        this.design_screen.harness.last_action = format!(
                            "Story selected {} preset in editable single context",
                            kind.label()
                        )
                        .into();
                        cx.notify();
                    }))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(16.))
                                    .text_center()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(render_lucide_icon(
                                        kind.lucide_icon(),
                                        cx.theme().muted_foreground,
                                        16.,
                                    )),
                            )
                            .child(div().text_xs().child(label)),
                    ),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .debug_selector(|| "design-node-variation-matrix-heading".to_owned())
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("NODE + VARIATION MATRIX"),
            )
            .child(div().id("design-selector-scroll").pb_6().child(buttons))
            .into_any_element()
    }

    fn render_design_last_intent(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("LAST TYPED INTENT"),
            )
            .child(
                div()
                    .max_h(px(52.))
                    .overflow_hidden()
                    .text_xs()
                    .child(self.design_screen.harness.last_action.clone()),
            )
            .into_any_element()
    }

    /// Every fixture control the story offers, in rail order. The wide
    /// harness scrolls them in the fixture rail; the stacked harness folds
    /// the same controls into its collapsible section.
    fn design_fixture_controls(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        vec![
            self.render_design_context_selector(cx),
            self.render_design_workspace_toggle(cx),
            self.render_design_width_selector(cx),
            self.render_design_additional_labels_toggle(cx),
            self.render_design_nudge_preferences(cx),
            self.render_design_last_intent(cx),
            self.render_design_selector(cx),
        ]
    }

    fn render_design_fixture_rail(&self, bg: gpui::Hsla, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .id("design-fixture-rail-scroll")
            .debug_selector(|| "design-fixture-rail-scroll".to_owned())
            .w(px(DESIGN_HARNESS_RAIL_WIDTH))
            .h_full()
            .min_h(px(0.))
            .overflow_y_scroll()
            .track_scroll(&self.design_screen.harness.fixture_scroll_handle)
            .p_3()
            .gap_3()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(bg)
            .children(self.design_fixture_controls(cx))
            .into_any_element()
    }

    /// The flexible mock-canvas column between the rail and the inspector.
    /// It compresses to any leftover width, clipping the selection card
    /// instead of forcing the harness wider.
    fn render_design_mock_canvas(
        &self,
        caption: Option<&'static str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = &self.design_screen.host.nodes[self.design_screen.harness.selected_node];
        let selected_name = selected.name.clone();
        let selected_kind = selected.kind;
        let selected_size = format!(
            "{} × {}",
            selected.width.round() as i64,
            selected.height.round() as i64
        );
        v_flex()
            .flex_1()
            .h_full()
            .min_w(px(0.))
            .overflow_hidden()
            .items_center()
            .justify_center()
            .gap_3()
            .bg(cx.theme().muted.opacity(0.45))
            .child(
                div()
                    .w(px(264.))
                    .max_w_full()
                    .h(px(176.))
                    .overflow_hidden()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(cx.theme().selection)
                    .bg(cx.theme().secondary)
                    .shadow_lg()
                    .child(
                        v_flex()
                            .size_full()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(24.))
                                    .text_color(cx.theme().selection)
                                    .child(render_lucide_icon(
                                        selected_kind.lucide_icon(),
                                        cx.theme().selection,
                                        24.,
                                    )),
                            )
                            .child(
                                div()
                                    .max_w(px(220.))
                                    .truncate()
                                    .text_sm()
                                    .font_semibold()
                                    .child(selected_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(selected_size),
                            ),
                    ),
            )
            .when_some(caption, |column, caption| {
                column.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(caption),
                )
            })
            .into_any_element()
    }

    /// The inspector column, rendered at the user's chosen width capped so
    /// the surrounding harness chrome always fits (see
    /// [`design_harness_panel_width`]).
    fn render_design_panel_container(
        &self,
        harness_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .debug_selector(|| "design-harness-panel".to_owned())
            .w(px(design_harness_panel_width(
                self.design_screen.harness.panel_width,
                harness_width,
            )))
            .flex_none()
            .h_full()
            .min_h(px(0.))
            .overflow_hidden()
            .border_l_1()
            .border_color(cx.theme().border)
            .child(self.design_screen.panel.clone())
            .into_any_element()
    }

    /// The stacked harness's folded story controls: a full-width toggle
    /// bar plus, when expanded, the fixture controls in a height-capped
    /// scroll region so the inspector keeps most of the story height.
    fn render_design_stacked_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        let expanded = self.design_screen.harness.harness_controls_expanded;
        v_flex()
            .flex_none()
            .w_full()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(
                Button::new("design-harness-controls-toggle")
                    .debug_selector(|| "design-harness-controls-toggle".to_owned())
                    .tooltip(
                        "Scenario, node, and preference controls for the Design story fold \
                         down here at narrow widths",
                    )
                    .xsmall()
                    .compact()
                    .w_full()
                    .h(px(28.))
                    .label(if expanded {
                        "Hide story controls"
                    } else {
                        "Show story controls"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.design_screen.toggle_harness_controls(cx);
                    })),
            )
            .when(expanded, |section| {
                section.child(
                    v_flex()
                        .id("design-harness-controls-scroll")
                        .debug_selector(|| "design-harness-controls-scroll".to_owned())
                        .w_full()
                        .max_h(px(DESIGN_HARNESS_CONTROLS_MAX_HEIGHT))
                        .overflow_y_scroll()
                        .p_3()
                        .gap_3()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .children(self.design_fixture_controls(cx)),
                )
            })
            .into_any_element()
    }

    /// The wide harness: fixture rail, mock canvas, resize handle, and the
    /// inspector, side by side. Callers add the outer sizing.
    fn render_design_wide_harness(
        &self,
        harness_width: f32,
        rail_bg: gpui::Hsla,
        caption: &'static str,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        h_flex()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() {
                    this.design_screen
                        .update_panel_resize(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.design_screen.finish_panel_resize(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.design_screen.finish_panel_resize(cx);
                }),
            )
            .child(self.render_design_fixture_rail(rail_bg, cx))
            .child(self.render_design_mock_canvas(Some(caption), cx))
            .child(self.render_design_resize_handle(cx))
            .child(self.render_design_panel_container(harness_width, cx))
    }

    /// The stacked harness for narrow story viewports: the inspector (and
    /// its still-working width resizer) on top, the folded story controls
    /// below. Callers add the outer sizing.
    fn render_design_stacked_harness(
        &self,
        harness_width: f32,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        v_flex()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() {
                    this.design_screen
                        .update_panel_resize(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.design_screen.finish_panel_resize(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.design_screen.finish_panel_resize(cx);
                }),
            )
            .child(
                h_flex()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .child(self.render_design_mock_canvas(None, cx))
                    .child(self.render_design_resize_handle(cx))
                    .child(self.render_design_panel_container(harness_width, cx)),
            )
            .child(self.render_design_stacked_controls(cx))
    }

    pub(crate) fn render_design_story_harness(&self, cx: &mut Context<Self>) -> AnyElement {
        let (harness_width, _) = self.story_viewport.surface_size();
        if design_harness_stacked(harness_width) {
            self.render_design_stacked_harness(harness_width, cx)
                .size_full()
                .min_h(px(0.))
                .into_any_element()
        } else {
            self.render_design_wide_harness(
                harness_width,
                cx.theme().sidebar,
                "The mock canvas keeps left-opening inspectors visible",
                cx,
            )
            .size_full()
            .min_h(px(0.))
            .into_any_element()
        }
    }

    pub(crate) fn render_design_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_reference_nav(cx))
            .child(
                self.render_design_wide_harness(
                    f32::MAX,
                    cx.theme().secondary.opacity(0.32),
                    "Neutral canvas keeps left-opening inspectors visible",
                    cx,
                )
                .flex_1()
                .min_h(px(0.)),
            )
            .into_any_element()
    }
}
