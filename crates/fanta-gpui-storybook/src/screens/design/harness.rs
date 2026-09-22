//! Design inspector preview and mock-host knobs.
//!
//! The preview contains only the inspector, its resize edge, and a neutral
//! canvas for left-opening popovers. Inspection scenarios, node fixtures,
//! width presets, and preferences use the Gallery's shared knobs section.

use super::*;
use crate::screens::knobs::{self, KnobOption};
use fanta_gpui::atoms::TypographyExt as _;

impl Storybook {
    fn render_design_context_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2().flex_wrap();
        for scenario in DesignInspectionScenario::ALL {
            buttons = buttons.child(knobs::knob_chip(
                format!("design-inspection-scenario-{}", scenario.id()).into(),
                scenario.label().into(),
                self.design_screen.harness.inspection_scenario == scenario,
                move |this, _, cx| {
                    this.design_screen
                        .activate_inspection_scenario(scenario, cx);
                },
                cx,
            ));
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("INSPECTION CONTEXT"),
            )
            .child(buttons)
            .when(
                self.design_screen.harness.inspection_scenario
                    == DesignInspectionScenario::TextEdit,
                |content| {
                    content.child(
                        fanta_gpui::atoms::ui_button("design-text-range-revision")
                            .label(format!(
                                "Select next text range · revision {}",
                                self.design_screen.harness.text_range_revision
                            ))
                            .xsmall()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.design_screen.advance_text_range(cx);
                            })),
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
        fanta_gpui::atoms::ui_button("design-panel-resize-handle")
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
                fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)
            } else {
                fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
            }))
            .into_any_element()
    }

    fn render_design_width_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2();
        for width in DESIGN_PANEL_WIDTH_PRESETS {
            let active = (self.design_screen.harness.panel_width - width).abs() < f32::EPSILON;
            buttons = buttons.child(
                fanta_gpui::atoms::ui_button(SharedString::from(format!(
                    "design-panel-width-{width:.0}"
                )))
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
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(format!(
                        "PANEL WIDTH · {:.0} PX",
                        self.design_screen.harness.panel_width
                    )),
            )
            .child(buttons)
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
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
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
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
            let label = node.name.clone();
            buttons = buttons.child(
                knobs::knob_chip(
                    format!("design-preset-{index}").into(),
                    label,
                    selected,
                    move |this, _, cx| {
                        this.design_screen.harness.selected_node = index;
                        this.design_screen.harness.inspection_scenario =
                            default_design_inspection_scenario_for_node(&node);
                        this.design_screen
                            .apply_inspection_context(&this.design_screen.panel, cx);
                        this.design_screen.harness.last_action = format!(
                            "Story selected {} preset in editable single context",
                            node.kind.label()
                        )
                        .into();
                        cx.notify();
                    },
                    cx,
                )
                .debug_selector(move || format!("design-preset-{index}"))
                .when(index == last_index, |button| {
                    button.debug_selector(|| "design-preset-last".to_owned())
                }),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .debug_selector(|| "design-node-variation-matrix-heading".to_owned())
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("NODE + VARIATION MATRIX"),
            )
            .child(buttons)
            .into_any_element()
    }

    /// All fixture controls live outside the preview in the Gallery's shared
    /// knobs section. The same rows can drive composed inspector stories.
    pub(crate) fn render_design_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        let controls = vec![
            self.render_design_context_selector(cx),
            self.render_design_selector(cx),
            self.render_design_workspace_toggle(cx),
            self.render_design_width_selector(cx),
            self.render_design_additional_labels_toggle(cx),
            self.render_design_nudge_preferences(cx),
        ];
        knobs::knobs_panel(
            "design-story-knobs",
            vec![
                v_flex()
                    .id("design-knobs-controls-scroll")
                    .debug_selector(|| "design-knobs-controls-scroll".to_owned())
                    .w_full()
                    .max_h(px(400.))
                    .overflow_y_scroll()
                    .track_scroll(&self.design_screen.harness.fixture_scroll_handle)
                    .gap_3()
                    .children(controls)
                    .into_any_element(),
            ],
            cx,
        )
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
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .child(self.properties_inspector_screen.design.clone())
            .into_any_element()
    }

    /// A neutral canvas and a right-docked inspector. Scenario and fixture
    /// controls belong to `render_design_knobs`, never to this surface.
    fn render_design_preview(&self, harness_width: f32, cx: &mut Context<Self>) -> gpui::Div {
        h_flex()
            .relative()
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundTertiary.resolve(cx))
            .child(div().flex_1().min_w(px(0.)).h_full())
            .child(self.render_design_resize_handle(cx))
            .child(self.render_design_panel_container(harness_width, cx))
            // The composed inspector intentionally occludes canvas gestures.
            // Capture an active resize above it so crossing into the inspector
            // cannot interrupt the drag or activate an underlying field.
            .when(
                self.design_screen.harness.panel_resize_drag.is_some(),
                |preview| {
                    preview.child(
                        div()
                            .id("design-panel-resize-shield")
                            .debug_selector(|| "design-panel-resize-shield".to_owned())
                            .absolute()
                            .inset_0()
                            .occlude()
                            .cursor_col_resize()
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
                            ),
                    )
                },
            )
    }

    pub(crate) fn render_design_story_harness(&self, cx: &mut Context<Self>) -> AnyElement {
        let (harness_width, _) = self.story_viewport.surface_size();
        self.render_design_preview(harness_width, cx)
            .size_full()
            .min_h(px(0.))
            .into_any_element()
    }

    pub(crate) fn render_design_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(self.render_reference_nav(cx))
            .child(
                self.render_design_preview(f32::MAX, cx)
                    .flex_1()
                    .min_h(px(0.)),
            )
            .into_any_element()
    }
}
