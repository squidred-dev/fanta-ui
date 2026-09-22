//! Dock-aligned tool, mode and option menus, and the centered Actions palette.
//! Surfaces use the shared popup and menu primitives.

use crate::atoms::TypographyExt as _;
use gpui::{
    Anchor, AnyElement, Context, FontWeight, HighlightStyle, InteractiveElement as _, IntoElement,
    MouseDownEvent, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    StyledText, Window, div, point, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, h_flex, input::Input, v_flex,
};

use super::state::CommandMatch;
use super::{EditorToolbar, POPOVER_GAP};
use crate::atoms::{CONTROL_KEY_CONTEXT, ControlExt as _, tokens};
use crate::molecules::{POPUP_SAFE_MARGIN, popup_height, popup_surface, popup_width};
use crate::toolbar::icons::render_tool_icon;
use crate::toolbar::{ToolbarSecondaryControl, ToolbarToolGroup};

/// Tool flyout rows are 38 px tall; the preferred flyout height mirrors the
/// row stack exactly.
const TOOL_FLYOUT_ROW_HEIGHT: f32 = tokens::RowHeight::FLYOUT;
/// The flyout surface wraps its rows in `p_2`: one `Space::SM` step above
/// plus one below, so the height tracks that padding instead of merely
/// coinciding with it.
const TOOL_FLYOUT_VERTICAL_PADDING: f32 = 2. * tokens::Space::SM;
/// Zoom-menu and option-editor rows are 30 px tall inside the same `p_2`
/// surface padding as the tool flyout.
const MENU_ROW_HEIGHT: f32 = tokens::RowHeight::MENU;
const MENU_VERTICAL_PADDING: f32 = 2. * tokens::Space::SM;
/// Palette chrome around the results viewport: 54 px header + 34 px footer
/// + the surface's top and bottom 1 px borders.
const ACTIONS_PALETTE_CHROME_HEIGHT: f32 = 90.;
/// Tallest the Actions results viewport grows inside the 540 px palette cap.
const ACTIONS_RESULTS_MAX_HEIGHT: f32 = 378.;
impl EditorToolbar {
    /// Align flyouts to the dock edges with a small, stable gap above it.
    pub(super) fn dock_popup(
        &self,
        surface: impl IntoElement,
        width: gpui::Pixels,
        right: bool,
        priority: usize,
    ) -> AnyElement {
        let x = if right {
            self.dock_bounds.right() - width
        } else {
            self.dock_bounds.left()
        };
        gpui::deferred(
            gpui::anchored()
                .position(point(x, self.dock_bounds.top() - px(POPOVER_GAP)))
                .anchor(Anchor::BottomLeft)
                .snap_to_window_with_margin(px(POPUP_SAFE_MARGIN))
                .child(surface),
        )
        .with_priority(priority)
        .into_any_element()
    }

    pub(super) fn render_mode_menu(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let mut menu = popup_surface(
            SharedString::from(format!("{}-mode-menu", self.id)),
            px(tokens::Radius::MENU),
            cx,
        )
        .debug_selector(|| "toolbar-mode-menu".to_owned())
        .w(popup_width(window, 200.))
        .p_1()
        .occlude()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
        .on_mouse_down_out(cx.listener(|this, _, _, cx| this.dismiss_overlay_for_pointer(cx)));
        for (index, &mode) in crate::toolbar::ToolbarMode::ALL.iter().enumerate() {
            menu = menu.child(
                crate::molecules::menu_item(
                    SharedString::from(format!("{}-mode-{}", self.id, mode.label())),
                    px(tokens::RowHeight::FLYOUT),
                    cx,
                )
                .debug_selector(move || format!("toolbar-mode-{}", mode.label().to_lowercase()))
                .when(self.menu_cursor == index, |row| {
                    row.bg(crate::atoms::SemanticColor::BackgroundSelected.resolve(cx))
                })
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        this.menu_cursor = index;
                        cx.notify();
                    }
                }))
                .on_activate(cx.listener(move |this, _, window, cx| {
                    this.focus_handle.focus(window, cx);
                    this.request_mode(mode, cx);
                }))
                .child(crate::toolbar::icons::render_mode_icon(
                    mode,
                    crate::atoms::SemanticColor::Text.resolve(cx),
                    16.,
                ))
                .child(div().flex_1().child(mode.label()))
                .when(mode == self.mode, |row| {
                    row.child(Icon::new(IconName::Check).xsmall())
                }),
            );
        }
        self.dock_popup(menu, popup_width(window, 200.), false, 11)
    }

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
        .occlude()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
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
                        crate::atoms::SemanticColor::BackgroundSelected.resolve(cx)
                    } else {
                        cx.theme().transparent
                    })
                    .hover(|style| {
                        style.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx))
                    })
                    .focus(|style| {
                        style.border_color(
                            crate::atoms::SemanticColor::BackgroundSelected.resolve(cx),
                        )
                    })
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
                            crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)
                        } else {
                            crate::atoms::SemanticColor::Text.resolve(cx)
                        },
                        16.,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .typography(if self.mode == crate::toolbar::ToolbarMode::Draw {
                                crate::atoms::TypographyToken::BodyMedium
                            } else {
                                crate::atoms::TypographyToken::BodyLarge
                            })
                            .child(tool.label()),
                    )
                    .when_some(tool.shortcut(), |row, shortcut| {
                        row.child(
                            div()
                                .typography(crate::atoms::TypographyToken::BodyMedium)
                                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                                .child(shortcut),
                        )
                    })
                    .when(selected, |row| {
                        row.child(
                            Icon::new(IconName::Check).xsmall().text_color(
                                crate::atoms::SemanticColor::BackgroundBrand.resolve(cx),
                            ),
                        )
                    }),
            );
        }
        self.dock_popup(
            menu,
            menu_width,
            matches!(
                group,
                ToolbarToolGroup::Creation | ToolbarToolGroup::Feedback | ToolbarToolGroup::Paths
            ),
            10,
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
            color: Some(crate::atoms::SemanticColor::BackgroundBrand.resolve(cx)),
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
                crate::atoms::SemanticColor::BackgroundSelected.resolve(cx)
            } else {
                cx.theme().transparent
            })
            .hover(|style| style.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx)))
            .focus(|style| {
                style.border_color(crate::atoms::SemanticColor::BackgroundSelected.resolve(cx))
            })
            .on_hover(cx.listener(move |this, hovered, _, cx| {
                if *hovered {
                    this.command_cursor = index;
                    cx.notify();
                }
            }))
            .on_activate(cx.listener(move |this, _, window, cx| {
                this.invoke_command(command, window, cx);
            }))
            .child(crate::atoms::render_lucide_icon(
                crate::toolbar::icons::command_icon(command),
                crate::atoms::SemanticColor::Text.resolve(cx),
                tokens::IconSize::SM,
            ))
            .child(
                v_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .gap(px(1.))
                    .child(
                        div()
                            .truncate()
                            .typography(crate::atoms::TypographyToken::BodyLarge)
                            .child(StyledText::new(command.label()).with_highlights(
                                highlights.into_iter().map(|range| (range, highlight_style)),
                            )),
                    )
                    .child(
                        div()
                            .truncate()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(command.category()),
                    ),
            )
            .when_some(command.shortcut(), |row, shortcut| {
                row.child(
                    div()
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
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
        let palette_width = popup_width(window, 480.);
        // Keep the palette attached to the dock, including when the host
        // offsets it from the window center. Short windows shrink the results
        // viewport so the footer stays above the toolbar instead of overlapping it.
        let results_height = px((self.dock_bounds.top().as_f32()
            - POPOVER_GAP
            - POPUP_SAFE_MARGIN
            - ACTIONS_PALETTE_CHROME_HEIGHT)
            .clamp(1., ACTIONS_RESULTS_MAX_HEIGHT));
        let mut results = v_flex()
            .id(SharedString::from(format!("{}-actions-results", self.id)))
            .debug_selector(|| "toolbar-actions-results".to_owned())
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
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::BodyLarge)
                            .child("No matching actions"),
                    )
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .child("Try an action name or shortcut"),
                    ),
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
        .occlude()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
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
                .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
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
                        .bg(crate::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child("esc"),
                ),
        )
        .child(results)
        .child(
            h_flex()
                .h(px(34.))
                .px_3()
                .gap_3()
                .border_t_1()
                .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
                .typography(crate::atoms::TypographyToken::BodyMedium)
                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child("Arrow keys · Navigate")
                .child("Enter · Run")
                .child(div().flex_1())
                .child("Actions"),
        );

        gpui::deferred(
            gpui::anchored()
                .position(point(
                    self.dock_bounds.center().x - palette_width / 2.,
                    self.dock_bounds.top() - px(POPOVER_GAP),
                ))
                .anchor(Anchor::BottomLeft)
                .snap_to_window_with_margin(px(POPUP_SAFE_MARGIN))
                .child(palette),
        )
        .with_priority(12)
        .into_any_element()
    }

    /// Debug-selector prefix for one secondary chip's anchored editor.
    fn editor_selector_prefix(control: ToolbarSecondaryControl) -> &'static str {
        match control {
            ToolbarSecondaryControl::MotionAnimationStyle => "toolbar-motion-style",
            _ => "toolbar-option",
        }
    }

    /// Anchored candidate menu for one secondary chip. Every candidate it
    /// offers comes from the host's options (§12).
    pub(super) fn render_option_editor(
        &self,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_choice_editor(control, window, cx)
    }

    fn render_choice_editor(
        &self,
        control: ToolbarSecondaryControl,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let prefix = Self::editor_selector_prefix(control);
        let candidates = self.choice_candidates(control).to_vec();
        let current = self.current_choice(control);
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
        .occlude()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.dismiss_overlay_for_pointer(cx);
        }));
        for (index, candidate) in candidates.into_iter().enumerate() {
            let highlighted = self.menu_cursor == index;
            let selected = current.as_ref() == Some(&candidate);
            let selector = format!(
                "{prefix}-option-{}",
                candidate.to_lowercase().replace(' ', "-")
            );
            let chosen = candidate.clone();
            menu =
                menu.child(
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
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .border_1()
                        .border_color(cx.theme().transparent)
                        .bg(if highlighted {
                            crate::atoms::SemanticColor::BackgroundSelected.resolve(cx)
                        } else {
                            cx.theme().transparent
                        })
                        .hover(|style| {
                            style.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx))
                        })
                        .focus(|style| {
                            style.border_color(
                                crate::atoms::SemanticColor::BackgroundSelected.resolve(cx),
                            )
                        })
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
                            row.child(Icon::new(IconName::Check).xsmall().text_color(
                                crate::atoms::SemanticColor::BackgroundBrand.resolve(cx),
                            ))
                        }),
                );
        }
        self.dock_popup(menu, popup_width(window, 180.), true, 10)
    }
}
