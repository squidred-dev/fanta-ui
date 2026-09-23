//! Compact main row and the optional contextual strip above it.

use crate::atoms::{TypographyExt as _, tokens};
use gpui::{
    AnyElement, Context, InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, tooltip::Tooltip,
};

use super::{CHROME_CONTROL_SIZE, EditorToolbar, TOOL_SIZE, ToolbarOverlay};
use crate::atoms::{ActivateControl, CONTROL_KEY_CONTEXT, ControlExt as _, icon_button};
use crate::molecules::{horizontal_fade_overlays, track_horizontal_edge_fades};
use crate::toolbar::icons::{render_icon_asset, render_mode_icon, render_tool_icon};
use crate::toolbar::{
    ToolbarChromeControl, ToolbarControlValue, ToolbarItem, ToolbarMode, ToolbarSecondaryControl,
    ToolbarTool, ToolbarToolGroup,
};

/// Width of the pointer-transparent edge fades that mark clipped row content.
const ROW_FADE_WIDTH: f32 = 24.;

impl EditorToolbar {
    fn render_tool_glyph(
        &self,
        tool: ToolbarTool,
        selected: bool,
        compact: bool,
        cx: &gpui::App,
    ) -> AnyElement {
        let color = if selected {
            Self::mode_accent_foreground(Self::mode_accent(self.mode, cx), cx)
        } else {
            crate::atoms::SemanticColor::Text.resolve(cx)
        };
        render_tool_icon(tool, color, if compact { 15. } else { 17. })
    }

    fn render_tool_button(
        &self,
        tool: ToolbarTool,
        _window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let actions_open =
            tool == ToolbarTool::Actions && self.overlay == Some(ToolbarOverlay::Actions);
        let selected = self.active_tool == tool || actions_open;
        let accent = Self::mode_accent(self.mode, cx);
        let tooltip = Self::control_tooltip(tool.label(), tool.shortcut());
        let button = h_flex()
            .id(SharedString::from(format!(
                "{}-tool-{}",
                self.id,
                tool.label().to_lowercase().replace([' ', '/'], "-")
            )))
            .debug_selector({
                let label = tool.label();
                move || format!("toolbar-tool-{}", label.to_lowercase().replace(' ', "-"))
            })
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .size(px(TOOL_SIZE))
            .justify_center()
            .rounded(px(8.))
            .cursor_pointer()
            .border_1()
            .border_color(cx.theme().transparent)
            .bg(if selected {
                accent
            } else {
                cx.theme().transparent
            })
            .hover(|style| {
                style.bg(if selected {
                    accent
                } else {
                    crate::atoms::SemanticColor::BackgroundHover.resolve(cx)
                })
            })
            .focus(|style| {
                style.border_color(crate::atoms::SemanticColor::BackgroundSelected.resolve(cx))
            })
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .when(tool == ToolbarTool::Actions, |button| {
                // Contract (§16): the Actions trigger activates on mouse-down
                // so it wins the race against the palette's capture-phase
                // outside-dismiss; `on_activate`'s click path would reopen the
                // palette that dismissal just closed. Keyboard activation
                // converges on the same method through `ActivateControl`.
                button
                    .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
                        this.activate_tool_or_actions(tool, actions_open, window, cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                            // The trigger's default focus would otherwise replace
                            // the query focus set while opening the palette.
                            window.prevent_default();
                            this.activate_tool_or_actions(tool, actions_open, window, cx);
                        }),
                    )
            })
            .when(tool != ToolbarTool::Actions, |button| {
                button.on_activate(cx.listener(move |this, _, window, cx| {
                    this.activate_tool_or_actions(tool, actions_open, window, cx);
                }))
            })
            .child(self.render_tool_glyph(tool, selected, matches!(tool, ToolbarTool::Code), cx));

        h_flex()
            .relative()
            .flex_none()
            .items_start()
            .child(button)
            .into_any_element()
    }

    fn render_tool_group(
        &self,
        group: ToolbarToolGroup,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tool = self
            .group_tools(group)
            .find(|tool| *tool == self.active_tool)
            .or_else(|| self.group_tools(group).next())
            .unwrap_or_else(|| group.default_tool());
        let selected = group.tools().contains(&self.active_tool);
        let menu_open = self.overlay == Some(ToolbarOverlay::ToolGroup(group));
        let accent = Self::mode_accent(self.mode, cx);
        let tooltip = Self::control_tooltip(tool.label(), tool.shortcut());

        h_flex()
            .relative()
            .h(px(TOOL_SIZE))
            .flex_none()
            .items_start()
            .child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-group-{}-main",
                        self.id,
                        group.label().to_lowercase().replace(' ', "-")
                    )))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .size(px(TOOL_SIZE))
                    .justify_center()
                    .rounded(px(8.))
                    .cursor_pointer()
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .bg(if selected {
                        accent
                    } else {
                        cx.theme().transparent
                    })
                    .hover(|style| {
                        style.bg(if selected {
                            accent
                        } else {
                            crate::atoms::SemanticColor::BackgroundHover.resolve(cx)
                        })
                    })
                    .focus(|style| {
                        style.border_color(
                            crate::atoms::SemanticColor::BackgroundSelected.resolve(cx),
                        )
                    })
                    .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.request_tool(tool, cx);
                    }))
                    .child(self.render_tool_glyph(tool, selected, false, cx)),
            )
            .child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-group-{}-menu",
                        self.id,
                        group.label().to_lowercase().replace(' ', "-")
                    )))
                    .debug_selector({
                        let label = group.label();
                        move || {
                            format!(
                                "toolbar-group-{}-trigger",
                                label.to_lowercase().replace(' ', "-")
                            )
                        }
                    })
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .w(px(12.))
                    .h_full()
                    .justify_center()
                    .rounded(px(5.))
                    .cursor_pointer()
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .hover(|style| {
                        style.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx))
                    })
                    .focus(|style| {
                        style.border_color(
                            crate::atoms::SemanticColor::BackgroundSelected.resolve(cx),
                        )
                    })
                    // Contract (§16): press-activation so the caret wins the
                    // race against its flyout's capture-phase outside-dismiss;
                    // a click handler would reopen the flyout it just closed.
                    // Enter/Space on an open flyout commit its highlighted row.
                    .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
                        if !(menu_open && this.commit_open_menu_entry(window, cx)) {
                            this.toggle_tool_group(group, menu_open, cx);
                        }
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                            this.toggle_tool_group(group, menu_open, cx);
                        }),
                    )
                    .child(
                        Icon::new(if menu_open {
                            IconName::ChevronUp
                        } else {
                            IconName::ChevronDown
                        })
                        .xsmall(),
                    ),
            )
            .when(menu_open, |control| {
                control.child(self.render_tool_group_menu(group, window, cx))
            })
            .into_any_element()
    }

    pub(super) fn render_main_toolbar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut tools = h_flex().h_full().gap(px(2.));
        for item in self.mode.layout() {
            tools = match item {
                ToolbarItem::Group(group) if self.group_tools(*group).next().is_some() => {
                    tools.child(self.render_tool_group(*group, window, cx))
                }
                ToolbarItem::Tool(tool)
                    if *tool != ToolbarTool::Actions && self.tool_supported(*tool) =>
                {
                    tools.child(self.render_tool_button(*tool, window, cx))
                }
                ToolbarItem::Separator => tools.child(
                    div()
                        .w(px(1.))
                        .h(px(24.))
                        .mx_1()
                        .bg(crate::atoms::SemanticColor::Border.resolve(cx)),
                ),
                _ => tools,
            };
        }

        h_flex()
            .debug_selector(|| "toolbar-main-row".to_owned())
            .relative()
            .h(px(tokens::RowHeight::SECTION_HEADER))
            .gap_1()
            .max_w_full()
            .min_w(px(0.))
            .child(self.render_mode_selector(window, cx))
            .child(
                div()
                    .w(px(1.))
                    .h(px(24.))
                    .flex_none()
                    .bg(crate::atoms::SemanticColor::Border.resolve(cx)),
            )
            .child(
                // A flex wrapper so the scroll viewport keeps its flex-item
                // sizing; a block wrapper collapses a scroll child to zero
                // width and culls the row's hitboxes.
                h_flex()
                    .relative()
                    .min_w(px(0.))
                    .flex_shrink_1()
                    .child(
                        // The viewport itself is a flex row: a block scroll
                        // container clamps its row to the viewport width, so
                        // the content never overflows and cannot scroll.
                        div()
                            .id(SharedString::from(format!("{}-primary-viewport", self.id)))
                            .debug_selector(|| "toolbar-primary-viewport".to_owned())
                            .flex()
                            .min_w(px(0.))
                            .flex_shrink_1()
                            .overflow_x_scroll()
                            .track_scroll(&self.primary_scroll_handle)
                            .child(
                                h_flex()
                                    .id(SharedString::from(format!("{}-primary", self.id)))
                                    .debug_selector(|| "toolbar-primary-row".to_owned())
                                    .h(px(40.))
                                    .px_1()
                                    .flex_none()
                                    .child(tools),
                            ),
                    )
                    .children(horizontal_fade_overlays(
                        self.primary_fades,
                        px(ROW_FADE_WIDTH),
                        crate::atoms::SemanticColor::BackgroundMenu.resolve(cx),
                        "toolbar-primary",
                    ))
                    .child(track_horizontal_edge_fades(
                        cx.entity(),
                        self.primary_scroll_handle.clone(),
                        |this| this.primary_fades,
                        |this, fades| this.primary_fades = fades,
                    )),
            )
            .child(
                div()
                    .w(px(1.))
                    .h(px(24.))
                    .flex_none()
                    .bg(crate::atoms::SemanticColor::Border.resolve(cx)),
            )
            // Keep Actions visible while mode-specific tools scroll, so a
            // keyboard invocation always has a real, nearby anchor.
            .when(self.tool_supported(ToolbarTool::Actions), |row| {
                row.child(self.render_tool_button(ToolbarTool::Actions, window, cx))
            })
            .when(!self.chrome_controls.is_empty(), |row| {
                row.child(self.render_chrome_cluster(cx))
            })
            .into_any_element()
    }

    fn render_mode_selector(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let open = self.overlay == Some(ToolbarOverlay::Mode);
        let mode = self.mode;
        let tooltip = format!("{} mode · Switch workspace", mode.label());
        icon_button(
            SharedString::from(format!("{}-mode-selector", self.id)),
            px(TOOL_SIZE),
            px(tokens::Radius::CONTROL),
            cx,
        )
        .debug_selector(|| "toolbar-mode-selector".to_owned())
        .relative()
        .w(px(48.))
        .flex_none()
        .gap_1()
        .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
        // Press activation pairs with outside-dismiss, as on the tool flyouts.
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| this.toggle_mode_menu(open, cx)),
        )
        .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
            if !(open && this.commit_open_menu_entry(window, cx)) {
                this.toggle_mode_menu(open, cx);
            }
        }))
        .child(render_mode_icon(
            mode,
            crate::atoms::SemanticColor::Text.resolve(cx),
            17.,
        ))
        .child(Icon::new(IconName::ChevronDown).xsmall())
        .when(open, |trigger| {
            trigger.child(self.render_mode_menu(window, cx))
        })
        .into_any_element()
    }

    fn render_secondary_button(
        &self,
        suffix: &'static str,
        label: impl Into<SharedString>,
        control: ToolbarSecondaryControl,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !self.secondary_control_supported(control) {
            return div().into_any_element();
        }
        let label = label.into();
        let tooltip = label.clone();
        let accent = crate::atoms::SemanticColor::TextOnBrand.resolve(cx);
        let pale_accent = Self::mode_accent_pale(self.mode, cx);
        h_flex()
            .id(SharedString::from(format!(
                "{}-secondary-{suffix}",
                self.id
            )))
            .debug_selector(move || format!("toolbar-secondary-{suffix}"))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(28.))
            .px_2()
            .gap_1()
            .rounded(px(7.))
            .cursor_pointer()
            .typography(crate::atoms::TypographyToken::BodyMedium)
            .font_medium()
            .border_1()
            .border_color(cx.theme().transparent)
            .bg(if selected {
                pale_accent
            } else {
                cx.theme().transparent
            })
            .text_color(if selected {
                accent
            } else {
                crate::atoms::SemanticColor::Text.resolve(cx)
            })
            .hover(|style| {
                style.bg(if selected {
                    pale_accent
                } else {
                    crate::atoms::SemanticColor::BackgroundHover.resolve(cx)
                })
            })
            .focus(|style| {
                style.border_color(crate::atoms::SemanticColor::BackgroundSelected.resolve(cx))
            })
            .on_activate(cx.listener(move |this, _, _, cx| {
                this.request_secondary(control, cx);
            }))
            .child(crate::atoms::render_lucide_icon(
                if control == ToolbarSecondaryControl::MotionPlayPause
                    && self.motion_options.playing
                {
                    crate::atoms::LucideIcon::Pause
                } else {
                    crate::toolbar::icons::secondary_icon(control)
                },
                if selected {
                    accent
                } else {
                    crate::atoms::SemanticColor::Text.resolve(cx)
                },
                tokens::IconSize::SM,
            ))
            .when(self.mode != ToolbarMode::Motion, |button| {
                button.child(label)
            })
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .into_any_element()
    }

    fn render_value_button(
        &self,
        suffix: &'static str,
        label: impl Into<SharedString>,
        control: ToolbarSecondaryControl,
        value: ToolbarControlValue,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !self.secondary_control_supported(control) {
            return div().into_any_element();
        }
        let label = label.into();
        let tooltip = label.clone();
        let accent = crate::atoms::SemanticColor::TextOnBrand.resolve(cx);
        let pale_accent = Self::mode_accent_pale(self.mode, cx);
        h_flex()
            .id(SharedString::from(format!(
                "{}-secondary-{suffix}",
                self.id
            )))
            .debug_selector(move || format!("toolbar-secondary-{suffix}"))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(28.))
            .px_2()
            .gap_1()
            .rounded(px(7.))
            .cursor_pointer()
            .typography(crate::atoms::TypographyToken::BodyMedium)
            .font_medium()
            .border_1()
            .border_color(cx.theme().transparent)
            .bg(if selected {
                pale_accent
            } else {
                cx.theme().transparent
            })
            .text_color(if selected {
                accent
            } else {
                crate::atoms::SemanticColor::Text.resolve(cx)
            })
            .hover(|style| {
                style.bg(if selected {
                    pale_accent
                } else {
                    crate::atoms::SemanticColor::BackgroundHover.resolve(cx)
                })
            })
            .focus(|style| {
                style.border_color(crate::atoms::SemanticColor::BackgroundSelected.resolve(cx))
            })
            .on_activate(cx.listener(move |this, _, _, cx| {
                this.request_control_value(control, value.clone(), cx);
            }))
            .child(crate::atoms::render_lucide_icon(
                if control == ToolbarSecondaryControl::MotionPlayPause
                    && self.motion_options.playing
                {
                    crate::atoms::LucideIcon::Pause
                } else {
                    crate::toolbar::icons::secondary_icon(control)
                },
                if selected {
                    accent
                } else {
                    crate::atoms::SemanticColor::Text.resolve(cx)
                },
                tokens::IconSize::SM,
            ))
            .when(self.mode != ToolbarMode::Motion, |button| {
                button.child(label)
            })
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .into_any_element()
    }

    /// A secondary chip that opens an anchored editor of host-supplied
    /// candidates instead of cycling a hardcoded next value.
    fn render_editor_chip(
        &self,
        suffix: &'static str,
        label: impl Into<SharedString>,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !self.secondary_control_supported(control) {
            return div().into_any_element();
        }
        let label = label.into();
        let open = self.overlay == Some(ToolbarOverlay::OptionEditor(control));
        h_flex()
            .relative()
            .h(px(28.))
            .flex_none()
            .items_start()
            .child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-secondary-{suffix}",
                        self.id
                    )))
                    .debug_selector(move || format!("toolbar-secondary-{suffix}"))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .h(px(28.))
                    .w(px(tokens::DropdownGeometry::WIDTH))
                    .flex_none()
                    .px_2()
                    .gap_1()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .typography(crate::atoms::TypographyToken::BodyMedium)
                    .font_medium()
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
                    .when(open, |chip| {
                        chip.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx))
                    })
                    .hover(|style| {
                        style.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx))
                    })
                    .focus(|style| {
                        style.border_color(
                            crate::atoms::SemanticColor::BackgroundSelected.resolve(cx),
                        )
                    })
                    // Contract (§16): press-activation so the chip wins the
                    // race against its editor's capture-phase outside-dismiss;
                    // a click handler would reopen the editor it just closed.
                    // Enter/Space on an open choice editor commit its
                    // highlighted candidate.
                    .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
                        if !(open && this.commit_open_menu_entry(window, cx)) {
                            this.toggle_option_editor(control, open, cx);
                        }
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                            this.toggle_option_editor(control, open, cx);
                        }),
                    )
                    .child(crate::atoms::truncating_label(label.clone()))
                    .tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
                    .child(
                        Icon::new(if open {
                            IconName::ChevronUp
                        } else {
                            IconName::ChevronDown
                        })
                        .xsmall()
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx)),
                    ),
            )
            .when(open, |chip| {
                chip.child(self.render_option_editor(control, window, cx))
            })
            .into_any_element()
    }

    fn render_dev_secondary(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .flex_none()
            .h(px(40.))
            .px_1()
            .gap_1()
            .rounded(px(10.))
            .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
            .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
            .child(self.render_secondary_button(
                "dev-inspect",
                "Inspect",
                ToolbarSecondaryControl::DevInspect,
                self.active_tool == ToolbarTool::Inspect,
                cx,
            ))
            .child(self.render_secondary_button(
                "dev-annotate",
                "Annotate",
                ToolbarSecondaryControl::DevAnnotate,
                self.active_tool == ToolbarTool::Annotation,
                cx,
            ))
            .child(self.render_secondary_button(
                "dev-measure",
                "Measure",
                ToolbarSecondaryControl::DevMeasure,
                self.active_tool == ToolbarTool::Measure,
                cx,
            ))
            .child(
                div()
                    .w(px(1.))
                    .h(px(22.))
                    .bg(crate::atoms::SemanticColor::Border.resolve(cx)),
            )
            .child(self.render_value_button(
                "dev-ready",
                "Ready for dev",
                ToolbarSecondaryControl::DevReadyForDevelopment,
                ToolbarControlValue::Toggle(!self.dev_options.ready_for_development),
                self.dev_options.ready_for_development,
                cx,
            ))
            .into_any_element()
    }

    fn render_motion_secondary(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let current_seconds = self.motion_options.current_time_ms as f32 / 1_000.;
        let duration_seconds = self.motion_options.duration_ms as f32 / 1_000.;
        h_flex()
            .flex_none()
            .h(px(40.))
            .px_1()
            .gap_1()
            .rounded(px(10.))
            .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
            .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
            .child(self.render_value_button(
                "motion-play",
                if self.motion_options.playing {
                    "Pause"
                } else {
                    "Play"
                },
                ToolbarSecondaryControl::MotionPlayPause,
                ToolbarControlValue::Toggle(!self.motion_options.playing),
                self.motion_options.playing,
                cx,
            ))
            .child(self.render_value_button(
                "motion-loop",
                "Loop",
                ToolbarSecondaryControl::MotionLoop,
                ToolbarControlValue::Toggle(!self.motion_options.looping),
                self.motion_options.looping,
                cx,
            ))
            .child(
                div()
                    .id(SharedString::from(format!("{}-motion-time", self.id)))
                    .debug_selector(|| "toolbar-motion-time".to_owned())
                    .h(px(28.))
                    .w(px(tokens::InputGeometry::COMBO_WIDTH))
                    .flex_none()
                    .px_2()
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
                    .typography(crate::atoms::TypographyToken::BodyMedium)
                    .child(crate::atoms::truncating_label(format!(
                        "{current_seconds:.1}s / {duration_seconds:.1}s"
                    )))
                    .tooltip(move |window, cx| {
                        Tooltip::new(format!("{current_seconds:.1}s / {duration_seconds:.1}s"))
                            .build(window, cx)
                    }),
            )
            .child(self.render_value_button(
                "motion-autokey",
                "Auto key",
                ToolbarSecondaryControl::MotionAutoKeyframe,
                ToolbarControlValue::Toggle(!self.motion_options.auto_keyframe),
                self.motion_options.auto_keyframe,
                cx,
            ))
            .child(self.render_secondary_button(
                "motion-keyframe",
                "Keyframe",
                ToolbarSecondaryControl::MotionAddKeyframe,
                false,
                cx,
            ))
            .child(self.render_editor_chip(
                "motion-style",
                self.motion_options.animation_style.clone(),
                ToolbarSecondaryControl::MotionAnimationStyle,
                window,
                cx,
            ))
            .child(self.render_secondary_button(
                "motion-timeline",
                "Timeline",
                ToolbarSecondaryControl::MotionTimeline,
                true,
                cx,
            ))
            .into_any_element()
    }

    pub(super) fn render_secondary_toolbar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content = match self.mode {
            ToolbarMode::Design => return div().into_any_element(),
            ToolbarMode::Dev => self.render_dev_secondary(cx),
            ToolbarMode::Draw => self.render_draw_secondary(window, cx),
            ToolbarMode::Motion => self.render_motion_secondary(window, cx),
        };
        // A flex wrapper so the scroll viewport keeps its flex-item sizing; a
        // block wrapper collapses a scroll child to zero width and culls the
        // row's hitboxes.
        h_flex()
            .debug_selector(|| "toolbar-context-row".to_owned())
            .relative()
            .max_w_full()
            .min_w(px(0.))
            .pb_1()
            .border_b_1()
            .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
            .child(
                // The viewport itself is a flex row: a block scroll container
                // clamps its row to the viewport width, so the content never
                // overflows and cannot scroll.
                div()
                    .id(SharedString::from(format!(
                        "{}-secondary-viewport",
                        self.id
                    )))
                    .debug_selector(|| "toolbar-secondary-viewport".to_owned())
                    .flex()
                    .w_full()
                    .min_w(px(0.))
                    .overflow_x_scroll()
                    .track_scroll(&self.secondary_scroll_handle)
                    .child(content),
            )
            .children(horizontal_fade_overlays(
                self.secondary_fades,
                px(ROW_FADE_WIDTH),
                crate::atoms::SemanticColor::BackgroundMenu.resolve(cx),
                "toolbar-secondary",
            ))
            .child(track_horizontal_edge_fades(
                cx.entity(),
                self.secondary_scroll_handle.clone(),
                |this| this.secondary_fades,
                |this, fades| this.secondary_fades = fades,
            ))
            .into_any_element()
    }

    /// One host chrome control: a compact icon tile whose active state uses
    /// the same raised-tile treatment as the selected mode.
    fn render_chrome_control(
        &self,
        control: &ToolbarChromeControl,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = control.id.clone();
        let selector_id = control.id.to_string();
        let state_selector = format!(
            "toolbar-chrome-{}-{}",
            control.id,
            if control.active { "active" } else { "inactive" }
        );
        let tooltip: SharedString = match &control.shortcut {
            Some(shortcut) => format!("{}  {shortcut}", control.label).into(),
            None => control.label.clone(),
        };
        let active = control.active;
        icon_button(
            SharedString::from(format!("{}-chrome-{}", self.id, control.id)),
            px(CHROME_CONTROL_SIZE),
            px(8.),
            cx,
        )
        .debug_selector(move || format!("toolbar-chrome-{selector_id}"))
        .flex_none()
        .when(active, |button| {
            button
                .bg(crate::atoms::SemanticColor::Background.resolve(cx))
                .when(cx.theme().shadow, |button| button.shadow_sm())
        })
        .text_color(if active {
            crate::atoms::SemanticColor::Text.resolve(cx)
        } else {
            crate::atoms::SemanticColor::TextTertiary.resolve(cx)
        })
        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
        .on_activate(cx.listener(move |this, _, _, cx| {
            this.request_chrome_control(id.clone(), cx);
        }))
        .child(
            div()
                .debug_selector(move || state_selector.clone())
                .flex()
                .child(render_icon_asset(
                    control.icon.clone(),
                    if active {
                        crate::atoms::SemanticColor::Text.resolve(cx)
                    } else {
                        crate::atoms::SemanticColor::TextTertiary.resolve(cx)
                    },
                    15.,
                )),
        )
        .into_any_element()
    }

    /// The host chrome capsule at the end of the utility row (§12): a
    /// secondary-filled pill matching the mode tray, one tile per control.
    fn render_chrome_cluster(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut cluster = h_flex()
            .debug_selector(|| "toolbar-chrome-cluster".to_owned())
            .h(px(36.))
            .flex_none()
            .px_1()
            .gap(px(2.))
            .rounded(px(10.))
            .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx));
        for control in &self.chrome_controls {
            cluster = cluster.child(self.render_chrome_control(control, cx));
        }
        cluster.into_any_element()
    }
}
