//! Transient trigger-anchored surfaces: the split-tool flyout, the Actions
//! command palette, the Agent composer, and the zoom menu. All four ride the
//! shared anchored-popup molecule (`crate::molecules::popup`).

use gpui::{
    Anchor, AnyElement, Context, FontWeight, HighlightStyle, InteractiveElement as _, IntoElement,
    MouseButton, MouseDownEvent, ParentElement as _, SharedString, StatefulInteractiveElement as _,
    Styled as _, StyledText, Window, div, point, prelude::FluentBuilder as _, px, relative,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, input::Input, v_flex,
};

use super::state::CommandMatch;
use super::{CommandScope, EditorToolbar, POPOVER_GAP, TOOL_SIZE, ZoomMenuEntry};
use crate::atoms::{CONTROL_KEY_CONTEXT, ControlExt as _, icon_button, track_bounds};
use crate::molecules::{
    POPUP_SAFE_MARGIN, anchored_popup, popup_height, popup_max_height, popup_surface, popup_width,
};
use crate::toolbar::icons::render_tool_icon;
use crate::toolbar::{ToolbarAction, ToolbarSecondaryControl, ToolbarToolGroup};

/// Tool flyout rows are 38 px tall; the preferred flyout height mirrors the
/// row stack exactly.
const TOOL_FLYOUT_ROW_HEIGHT: f32 = 38.;
/// The flyout surface wraps its rows in `p_2` (8 px above plus 8 px below).
const TOOL_FLYOUT_VERTICAL_PADDING: f32 = 16.;
/// Zoom-menu and option-editor rows are 30 px tall inside the same `p_2`
/// surface padding as the tool flyout.
const MENU_ROW_HEIGHT: f32 = 30.;
const MENU_VERTICAL_PADDING: f32 = 16.;
/// Palette chrome around the results viewport: 54 px header + 38 px scope
/// switcher + 34 px footer + the surface's top and bottom 1 px borders.
const ACTIONS_PALETTE_CHROME_HEIGHT: f32 = 128.;
/// Tallest the Actions results viewport grows inside the 540 px palette cap.
const ACTIONS_RESULTS_MAX_HEIGHT: f32 = 378.;
/// Anchor inset mirroring the 36 px Agent launcher, so the composer's right
/// edge aligns with its trigger's right edge.
const AGENT_COMPOSER_ANCHOR_INSET: f32 = 36.;
/// Anchor inset mirroring the 58 px zoom trigger, so the zoom flyout's right
/// edge aligns with its trigger's right edge.
const ZOOM_MENU_ANCHOR_INSET: f32 = 58.;

impl EditorToolbar {
    pub(super) fn render_tool_group_menu(
        &self,
        group: ToolbarToolGroup,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let menu_width = popup_width(window, 248.);
        let menu_height = popup_height(
            window,
            group.tools().len() as f32 * TOOL_FLYOUT_ROW_HEIGHT + TOOL_FLYOUT_VERTICAL_PADDING,
        );
        let mut menu = popup_surface(
            SharedString::from(format!(
                "{}-{}-flyout",
                self.id,
                group.label().to_lowercase().replace(' ', "-")
            )),
            px(14.),
            cx,
        )
        .debug_selector(|| "toolbar-tool-flyout".to_owned())
        .w(menu_width)
        .h(menu_height)
        .p_2()
        .track_scroll(&self.menu_scroll_handle)
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }));
        for (index, tool) in group.tools().iter().enumerate() {
            let tool = *tool;
            let selected = self.active_tool == tool;
            let highlighted = self.menu_cursor == index;
            menu = menu.child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-flyout-{}",
                        self.id,
                        tool.label().to_lowercase().replace([' ', '/'], "-")
                    )))
                    .debug_selector(move || {
                        format!(
                            "toolbar-flyout-{}",
                            tool.label().to_lowercase().replace([' ', '/'], "-")
                        )
                    })
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .flex_none()
                    .h(px(TOOL_FLYOUT_ROW_HEIGHT))
                    .px_2()
                    .gap_2()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .bg(if highlighted {
                        cx.theme().list_active
                    } else {
                        cx.theme().transparent
                    })
                    .hover(|style| style.bg(cx.theme().list_hover))
                    .focus(|style| style.border_color(cx.theme().selection))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered && this.menu_cursor != index {
                            this.menu_cursor = index;
                            cx.notify();
                        }
                    }))
                    .on_activate(cx.listener(move |this, _, window, cx| {
                        this.choose_tool_from_menu(tool, window, cx);
                    }))
                    .child(render_tool_icon(
                        tool,
                        if selected {
                            cx.theme().primary
                        } else {
                            cx.theme().popover_foreground
                        },
                        16.,
                    ))
                    .child(div().flex_1().text_sm().child(tool.label()))
                    .when_some(tool.shortcut(), |row, shortcut| {
                        row.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(shortcut),
                        )
                    })
                    .when(selected, |row| {
                        row.child(
                            Icon::new(IconName::Check)
                                .xsmall()
                                .text_color(cx.theme().primary),
                        )
                    }),
            );
        }
        anchored_popup(
            Anchor::BottomLeft,
            point(px(0.), px(-POPOVER_GAP)),
            10,
            menu,
        )
    }

    fn render_command_row(
        &self,
        entry: CommandMatch,
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let CommandMatch {
            command,
            highlights,
        } = entry;
        let selected = self.command_cursor == index;
        let highlight_style = HighlightStyle {
            color: Some(cx.theme().primary),
            font_weight: Some(FontWeight::SEMIBOLD),
            ..HighlightStyle::default()
        };
        h_flex()
            .id(SharedString::from(format!(
                "{}-command-{}",
                self.id,
                command.label().to_lowercase().replace([' ', '/'], "-")
            )))
            .debug_selector(move || {
                format!(
                    "toolbar-command-{}",
                    command.label().to_lowercase().replace([' ', '/'], "-")
                )
            })
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .flex_none()
            .h(px(42.))
            .px_3()
            .gap_3()
            .rounded(px(8.))
            .cursor_pointer()
            .border_1()
            .border_color(cx.theme().transparent)
            .bg(if selected {
                cx.theme().list_active
            } else {
                cx.theme().transparent
            })
            .hover(|style| style.bg(cx.theme().list_hover))
            .focus(|style| style.border_color(cx.theme().selection))
            .on_hover(cx.listener(move |this, hovered, _, cx| {
                if *hovered {
                    this.command_cursor = index;
                    cx.notify();
                }
            }))
            .on_activate(cx.listener(move |this, _, window, cx| {
                this.invoke_command(command, window, cx);
            }))
            .child(
                h_flex()
                    .size(px(26.))
                    .justify_center()
                    .rounded(px(7.))
                    .bg(if command.category() == "AI" {
                        cx.theme().magenta_light
                    } else {
                        cx.theme().secondary
                    })
                    .text_color(if command.category() == "AI" {
                        cx.theme().magenta
                    } else {
                        cx.theme().secondary_foreground
                    })
                    .child(
                        Icon::new(if command.category() == "AI" {
                            IconName::Asterisk
                        } else {
                            IconName::SquareTerminal
                        })
                        .xsmall(),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .gap(px(1.))
                    .child(div().truncate().text_sm().child(
                        StyledText::new(command.label()).with_highlights(
                            highlights.into_iter().map(|range| (range, highlight_style)),
                        ),
                    ))
                    .child(
                        div()
                            .truncate()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(command.category()),
                    ),
            )
            .when_some(command.shortcut(), |row, shortcut| {
                row.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(shortcut),
                )
            })
            .into_any_element()
    }

    pub(super) fn render_actions_palette(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let commands = self.filtered_commands(cx);
        let palette_width = popup_width(window, 420.);
        // Let the results viewport absorb short-window pressure so the surface
        // itself always fits inside the anchored layer's safe margin.
        let results_height = px((window.viewport_size().height.as_f32()
            - 2. * POPUP_SAFE_MARGIN
            - ACTIONS_PALETTE_CHROME_HEIGHT)
            .clamp(1., ACTIONS_RESULTS_MAX_HEIGHT));
        let mut results = v_flex()
            .id(SharedString::from(format!("{}-actions-results", self.id)))
            .w_full()
            .h(results_height)
            .px_2()
            .pb_2()
            .gap(px(1.))
            .overflow_scroll()
            .track_scroll(&self.command_scroll_handle);
        if commands.is_empty() {
            results = results.child(
                v_flex()
                    .h(px(112.))
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .text_color(cx.theme().muted_foreground)
                    .child(div().text_sm().child("No matching actions"))
                    .child(div().text_xs().child("Try a tool, plugin, or AI task")),
            );
        } else {
            for (index, entry) in commands.into_iter().enumerate() {
                results = results.child(self.render_command_row(entry, index, cx));
            }
        }

        let palette = popup_surface(
            SharedString::from(format!("{}-actions-palette", self.id)),
            px(16.),
            cx,
        )
        .debug_selector(|| "toolbar-actions-palette".to_owned())
        .w(palette_width)
        .max_h(px(540.))
        // The surface clips; the results viewport above owns scrolling.
        .overflow_hidden()
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }))
        .child(
            h_flex()
                .h(px(54.))
                .px_3()
                .gap_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(Icon::new(IconName::Search).small())
                .child(
                    div().flex_1().min_w(px(0.)).child(
                        Input::new(&self.command_input)
                            .appearance(false)
                            .bordered(false)
                            .focus_bordered(false)
                            .cleanable(true)
                            .small(),
                    ),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded(px(5.))
                        .bg(cx.theme().secondary)
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("esc"),
                ),
        )
        .child(
            h_flex()
                .h(px(38.))
                .px_2()
                .gap_1()
                .border_b_1()
                .border_color(cx.theme().border)
                .children(
                    CommandScope::ALL
                        .into_iter()
                        .enumerate()
                        .map(|(index, scope)| {
                            h_flex()
                                .id(SharedString::from(format!(
                                    "{}-actions-scope-{index}",
                                    self.id
                                )))
                                .debug_selector(move || {
                                    format!(
                                        "toolbar-actions-scope-{}",
                                        scope.label().to_lowercase().replace([' ', '&'], "-")
                                    )
                                })
                                .key_context(CONTROL_KEY_CONTEXT)
                                .tab_index(0)
                                .h(px(30.))
                                .px_2()
                                .justify_center()
                                .rounded(px(7.))
                                .cursor_pointer()
                                .text_xs()
                                .font_medium()
                                .border_1()
                                .border_color(cx.theme().transparent)
                                .bg(if self.command_scope == scope {
                                    cx.theme().selection
                                } else {
                                    cx.theme().transparent
                                })
                                .text_color(if self.command_scope == scope {
                                    cx.theme().primary
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .hover(|style| style.bg(cx.theme().accent))
                                .focus(|style| style.border_color(cx.theme().selection))
                                .on_activate(cx.listener(move |this, _, window, cx| {
                                    this.set_command_scope(scope, window, cx);
                                }))
                                .child(scope.label())
                        }),
                ),
        )
        .child(results)
        .child(
            h_flex()
                .h(px(34.))
                .px_3()
                .gap_3()
                .border_t_1()
                .border_color(cx.theme().border)
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Arrow keys · Navigate")
                .child("Enter · Run")
                .child(div().flex_1())
                .child("Commands · Plugins · Widgets · AI"),
        );

        anchored_popup(
            Anchor::BottomRight,
            point(px(TOOL_SIZE), px(-POPOVER_GAP)),
            12,
            palette,
        )
    }

    fn render_agent_suggestion(
        &self,
        index: usize,
        label: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let activate_label = label.clone();
        h_flex()
            .id(SharedString::from(format!(
                "{}-agent-suggestion-{index}",
                self.id
            )))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .flex_none()
            .h(px(28.))
            .max_w(px(168.))
            .px_2()
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border)
            .cursor_pointer()
            .text_xs()
            .text_color(cx.theme().popover_foreground)
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| style.border_color(cx.theme().selection))
            .on_activate(cx.listener(move |this, _, window, cx| {
                this.set_ai_suggestion(activate_label.clone(), window, cx);
            }))
            .child(div().min_w(px(0.)).truncate().child(label))
            .into_any_element()
    }

    pub(super) fn render_agent_composer(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_prompt = !self.ai_input.read(cx).value().trim().is_empty();
        let composer_width = popup_width(window, 448.);
        let mut suggestions = h_flex().flex_none().gap_1();
        for (index, suggestion) in self
            .agent_options
            .suggestions
            .iter()
            .take(3)
            .cloned()
            .enumerate()
        {
            suggestions = suggestions.child(self.render_agent_suggestion(index, suggestion, cx));
        }
        let has_suggestions = !self.agent_options.suggestions.is_empty();
        let composer = popup_surface(
            SharedString::from(format!("{}-agent-composer", self.id)),
            px(20.),
            cx,
        )
        .debug_selector(|| "toolbar-agent-composer".to_owned())
        .w(composer_width)
        .max_h(popup_max_height(window))
        .p_3()
        .gap_2()
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }))
        .child(
            h_flex()
                .h(px(28.))
                .min_w(px(0.))
                .gap_2()
                .child(
                    h_flex()
                        .flex_none()
                        .size(px(24.))
                        .justify_center()
                        .rounded(px(7.))
                        .bg(cx.theme().magenta_light)
                        .text_color(cx.theme().magenta)
                        .child(Icon::new(IconName::Bot).xsmall()),
                )
                .child(
                    div()
                        .flex_none()
                        .font_semibold()
                        .text_sm()
                        .child("Figma Agent"),
                )
                .child(
                    div()
                        .debug_selector(|| "toolbar-agent-context-label".to_owned())
                        .flex_1()
                        .min_w(px(0.))
                        .px_2()
                        .py_1()
                        .rounded(px(6.))
                        .bg(cx.theme().secondary)
                        .truncate()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.agent_options.context_label.clone()),
                )
                .child(
                    icon_button(
                        SharedString::from(format!("{}-agent-close", self.id)),
                        px(24.),
                        px(6.),
                        cx,
                    )
                    .flex_none()
                    .on_activate(cx.listener(|this, _, window, cx| {
                        this.dismiss_overlay(window, cx);
                    }))
                    .child(Icon::new(IconName::Close).xsmall()),
                ),
        )
        .child(
            div()
                .min_h(px(44.))
                .rounded(px(10.))
                .bg(cx.theme().secondary)
                .child(
                    Input::new(&self.ai_input)
                        .appearance(false)
                        .bordered(false)
                        .focus_bordered(false)
                        .cleanable(true)
                        .small(),
                ),
        )
        .when(!has_prompt && has_suggestions, |composer| {
            composer.child(
                div()
                    .id(SharedString::from(format!(
                        "{}-agent-suggestions-viewport",
                        self.id
                    )))
                    .debug_selector(|| "toolbar-agent-suggestions-viewport".to_owned())
                    .w_full()
                    .min_w(px(0.))
                    .overflow_x_scroll()
                    .child(suggestions),
            )
        })
        .child(
            h_flex()
                .h(px(32.))
                .gap_2()
                .child(
                    icon_button(
                        SharedString::from(format!("{}-agent-attachment", self.id)),
                        px(28.),
                        px(8.),
                        cx,
                    )
                    .flex_none()
                    .border_color(cx.theme().border)
                    .on_activate(cx.listener(|_, _, _, cx| {
                        cx.emit(ToolbarAction::AgentAttachmentRequested);
                    }))
                    .child(Icon::new(IconName::Plus).xsmall()),
                )
                .child(
                    div()
                        .debug_selector(|| "toolbar-agent-mention-hint".to_owned())
                        .flex_1()
                        .min_w(px(0.))
                        .truncate()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.agent_options.mention_hint.clone()),
                )
                .child(
                    h_flex()
                        .id(SharedString::from(format!("{}-agent-send", self.id)))
                        .key_context(CONTROL_KEY_CONTEXT)
                        .tab_index(0)
                        .flex_none()
                        .size(px(30.))
                        .justify_center()
                        .rounded_full()
                        .cursor_pointer()
                        .border_1()
                        .border_color(cx.theme().transparent)
                        .bg(if has_prompt {
                            cx.theme().primary
                        } else {
                            cx.theme().selection
                        })
                        .text_color(if has_prompt {
                            cx.theme().primary_foreground
                        } else {
                            cx.theme().primary
                        })
                        .hover(|style| style.bg(cx.theme().primary_hover))
                        .focus(|style| style.border_color(cx.theme().selection))
                        .on_activate(cx.listener(|this, _, window, cx| {
                            this.activate_agent_send(window, cx);
                        }))
                        .child(
                            Icon::new(if has_prompt {
                                IconName::ArrowUp
                            } else {
                                IconName::Bot
                            })
                            .xsmall(),
                        ),
                ),
        );

        anchored_popup(
            Anchor::BottomRight,
            point(px(AGENT_COMPOSER_ANCHOR_INSET), px(-POPOVER_GAP)),
            14,
            composer,
        )
    }

    pub(super) fn render_zoom_menu(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let menu_width = popup_width(window, 208.);
        let menu_height = popup_height(
            window,
            ZoomMenuEntry::ALL.len() as f32 * MENU_ROW_HEIGHT + MENU_VERTICAL_PADDING,
        );
        let mut menu = popup_surface(
            SharedString::from(format!("{}-zoom-flyout", self.id)),
            px(12.),
            cx,
        )
        .debug_selector(|| "toolbar-zoom-flyout".to_owned())
        .w(menu_width)
        .h(menu_height)
        .p_2()
        .track_scroll(&self.menu_scroll_handle)
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }));
        for (index, entry) in ZoomMenuEntry::ALL.into_iter().enumerate() {
            let highlighted = self.menu_cursor == index;
            let checked = matches!(
                (entry, self.zoom_percent),
                (ZoomMenuEntry::ZoomTo100, 100) | (ZoomMenuEntry::ZoomTo50, 50)
            );
            menu = menu.child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-zoom-entry-{index}",
                        self.id
                    )))
                    .debug_selector(move || {
                        format!(
                            "toolbar-zoom-entry-{}",
                            entry
                                .label()
                                .to_lowercase()
                                .replace(' ', "-")
                                .replace('%', "")
                        )
                    })
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .flex_none()
                    .h(px(MENU_ROW_HEIGHT))
                    .px_2()
                    .gap_2()
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_xs()
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .bg(if highlighted {
                        cx.theme().list_active
                    } else {
                        cx.theme().transparent
                    })
                    .hover(|style| style.bg(cx.theme().list_hover))
                    .focus(|style| style.border_color(cx.theme().selection))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered && this.menu_cursor != index {
                            this.menu_cursor = index;
                            cx.notify();
                        }
                    }))
                    .on_activate(cx.listener(move |this, _, window, cx| {
                        this.choose_zoom_entry(entry, window, cx);
                    }))
                    .child(div().flex_1().child(entry.label()))
                    .when_some(entry.shortcut(), |row, shortcut| {
                        row.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(shortcut),
                        )
                    })
                    .when(checked, |row| {
                        row.child(
                            Icon::new(IconName::Check)
                                .xsmall()
                                .text_color(cx.theme().primary),
                        )
                    }),
            );
        }
        anchored_popup(
            Anchor::BottomRight,
            point(px(ZOOM_MENU_ANCHOR_INSET), px(-POPOVER_GAP)),
            11,
            menu,
        )
    }

    /// Debug-selector prefix for one secondary chip's anchored editor.
    fn editor_selector_prefix(control: ToolbarSecondaryControl) -> &'static str {
        match control {
            ToolbarSecondaryControl::DrawStrokeColor => "toolbar-draw-color",
            ToolbarSecondaryControl::DrawBrushStyle => "toolbar-draw-style",
            ToolbarSecondaryControl::DrawStrokeWeight => "toolbar-draw-weight",
            ToolbarSecondaryControl::DrawSmoothing => "toolbar-draw-smoothing",
            ToolbarSecondaryControl::MotionAnimationStyle => "toolbar-motion-style",
            _ => "toolbar-option",
        }
    }

    /// Anchored editor for one secondary chip: a swatch grid for the stroke
    /// color, a slider for the ranged values, and a candidate menu otherwise.
    /// Every candidate it offers comes from the host's options (§12).
    pub(super) fn render_option_editor(
        &self,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match control {
            ToolbarSecondaryControl::DrawStrokeColor => {
                self.render_color_editor(control, window, cx)
            }
            ToolbarSecondaryControl::DrawStrokeWeight | ToolbarSecondaryControl::DrawSmoothing => {
                self.render_slider_editor(control, window, cx)
            }
            _ => self.render_choice_editor(control, window, cx),
        }
    }

    fn render_color_editor(
        &self,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let prefix = Self::editor_selector_prefix(control);
        let mut grid = h_flex().flex_wrap().gap_1();
        for (index, color) in self.choice_candidates(control).iter().enumerate() {
            let candidate = color.clone();
            let selected = self.draw_options.stroke_color == *color;
            let highlighted = self.menu_cursor == index;
            let swatch_color = Self::parse_color(color.as_ref())
                .map(gpui::Hsla::from)
                .unwrap_or(cx.theme().foreground);
            let selector = format!(
                "{prefix}-option-{}",
                color.trim_start_matches('#').to_ascii_lowercase()
            );
            grid = grid.child(
                icon_button(
                    SharedString::from(format!("{}-{prefix}-option-{index}", self.id)),
                    px(24.),
                    px(6.),
                    cx,
                )
                .debug_selector(move || selector.clone())
                .flex_none()
                .when(highlighted, |swatch| swatch.bg(cx.theme().list_active))
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered && this.menu_cursor != index {
                        this.menu_cursor = index;
                        cx.notify();
                    }
                }))
                .on_activate(cx.listener(move |this, _, window, cx| {
                    this.choose_option_candidate(control, candidate.clone(), window, cx);
                }))
                .child(
                    div()
                        .size(px(16.))
                        .rounded_full()
                        .bg(swatch_color)
                        .border_1()
                        .border_color(if selected {
                            cx.theme().primary
                        } else {
                            cx.theme().border
                        }),
                ),
            );
        }
        let editor = popup_surface(
            SharedString::from(format!("{}-{prefix}-editor", self.id)),
            px(10.),
            cx,
        )
        .debug_selector(move || format!("{prefix}-editor"))
        .w(popup_width(window, 240.))
        .max_h(popup_max_height(window))
        .p_2()
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }))
        .child(grid);
        anchored_popup(
            Anchor::BottomLeft,
            point(px(0.), px(-POPOVER_GAP)),
            10,
            editor,
        )
    }

    fn render_choice_editor(
        &self,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let prefix = Self::editor_selector_prefix(control);
        let candidates = self.choice_candidates(control).to_vec();
        let current = match control {
            ToolbarSecondaryControl::DrawBrushStyle => self.draw_options.brush_style.clone(),
            _ => self.motion_options.animation_style.clone(),
        };
        let mut menu = popup_surface(
            SharedString::from(format!("{}-{prefix}-editor", self.id)),
            px(10.),
            cx,
        )
        .debug_selector(move || format!("{prefix}-editor"))
        .w(popup_width(window, 180.))
        .h(popup_height(
            window,
            candidates.len().max(1) as f32 * MENU_ROW_HEIGHT + MENU_VERTICAL_PADDING,
        ))
        .p_2()
        .track_scroll(&self.menu_scroll_handle)
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }));
        for (index, candidate) in candidates.into_iter().enumerate() {
            let highlighted = self.menu_cursor == index;
            let selected = candidate == current;
            let selector = format!(
                "{prefix}-option-{}",
                candidate.to_lowercase().replace(' ', "-")
            );
            let chosen = candidate.clone();
            menu = menu.child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-{prefix}-option-{index}",
                        self.id
                    )))
                    .debug_selector(move || selector.clone())
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .flex_none()
                    .h(px(MENU_ROW_HEIGHT))
                    .px_2()
                    .gap_2()
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_xs()
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .bg(if highlighted {
                        cx.theme().list_active
                    } else {
                        cx.theme().transparent
                    })
                    .hover(|style| style.bg(cx.theme().list_hover))
                    .focus(|style| style.border_color(cx.theme().selection))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered && this.menu_cursor != index {
                            this.menu_cursor = index;
                            cx.notify();
                        }
                    }))
                    .on_activate(cx.listener(move |this, _, window, cx| {
                        this.choose_option_candidate(control, chosen.clone(), window, cx);
                    }))
                    .child(div().flex_1().child(candidate))
                    .when(selected, |row| {
                        row.child(
                            Icon::new(IconName::Check)
                                .xsmall()
                                .text_color(cx.theme().primary),
                        )
                    }),
            );
        }
        anchored_popup(
            Anchor::BottomLeft,
            point(px(0.), px(-POPOVER_GAP)),
            10,
            menu,
        )
    }

    fn render_slider_editor(
        &self,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let prefix = Self::editor_selector_prefix(control);
        let Some((value, min, max, _)) = self.slider_params(control) else {
            return div().into_any_element();
        };
        let fraction = if max > min {
            (value - min) as f32 / (max - min) as f32
        } else {
            0.
        };
        let value_label = match control {
            ToolbarSecondaryControl::DrawSmoothing => format!("{value}%"),
            _ => format!("{value} px"),
        };
        let editor = popup_surface(
            SharedString::from(format!("{}-{prefix}-editor", self.id)),
            px(10.),
            cx,
        )
        .debug_selector(move || format!("{prefix}-editor"))
        .w(popup_width(window, 220.))
        .max_h(popup_max_height(window))
        .p_3()
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }))
        .child(
            h_flex()
                .gap_2()
                .child(
                    div()
                        .id(SharedString::from(format!("{}-{prefix}-slider", self.id)))
                        .debug_selector(move || format!("{prefix}-slider"))
                        .key_context(CONTROL_KEY_CONTEXT)
                        .tab_index(0)
                        .relative()
                        .flex_1()
                        .h(px(20.))
                        .rounded(px(5.))
                        .cursor_pointer()
                        .border_1()
                        .border_color(cx.theme().transparent)
                        .focus(|style| style.border_color(cx.theme().selection))
                        // Contract (§16): the pointer path needs the event
                        // position to map the press onto the host-supplied
                        // range; Left/Right keyboard stepping converges on the
                        // same `ControlChangeRequested` intent through
                        // `Increment`/`DecrementToolbarControl`.
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                                this.set_slider_from_pointer(control, event.position.x, cx);
                            }),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(8.))
                                .left_0()
                                .right_0()
                                .h(px(3.))
                                .rounded_full()
                                .bg(cx.theme().border),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(8.))
                                .left_0()
                                .w(relative(fraction))
                                .h(px(3.))
                                .rounded_full()
                                .bg(cx.theme().primary),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(4.))
                                .left(relative(fraction))
                                .ml(px(-5.))
                                .size(px(11.))
                                .rounded_full()
                                .bg(cx.theme().primary),
                        )
                        .child(track_bounds(cx.entity(), |this, bounds| {
                            this.slider_track_bounds = bounds;
                        })),
                )
                .child(
                    div()
                        .flex_none()
                        .w(px(44.))
                        .text_right()
                        .text_xs()
                        .font_medium()
                        .child(value_label),
                ),
        );
        anchored_popup(
            Anchor::BottomLeft,
            point(px(0.), px(-POPOVER_GAP)),
            10,
            editor,
        )
    }
}
