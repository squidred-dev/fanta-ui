use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, ScrollHandle, SharedString,
    Styled as _, Subscription, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _,
    input::{InputEvent, InputState},
    v_flex,
};

use super::{
    CloseToolbarOverlay, ConfirmToolbarTextEntry, DecrementToolbarControl, DevToolbarOptions,
    EnterDevMode, FirstToolbarCommand, IncrementToolbarControl, LastToolbarCommand,
    MotionToolbarOptions, NextToolbarCommand, OpenToolbarActions, PreviousToolbarCommand,
    SelectAnnotationTool, SelectArrowTool, SelectBrushTool, SelectCommentTool, SelectEllipseTool,
    SelectEraserTool, SelectFrameTool, SelectHandTool, SelectImageVideoTool, SelectLineTool,
    SelectMarqueeTool, SelectMeasureTool, SelectMoveTool, SelectPenTool, SelectPencilTool,
    SelectRectangleTool, SelectSectionTool, SelectSliceTool, SelectTextTool, SelectWandTool,
    ToolbarAction, ToolbarChromeControl, ToolbarCommand, ToolbarMode, ToolbarSecondaryControl,
    ToolbarTool, ToolbarToolGroup, ZoomCanvasTo100, ZoomCanvasToFit, ZoomCanvasToSelection,
    commands::{TOOLBAR_KEY_CONTEXT, TOOLBAR_TEXT_ENTRY_KEY_CONTEXT},
};
use crate::atoms::tokens;
use crate::molecules::EdgeFades;

mod draw;
mod overlays;
mod rows;
mod state;

const TOOL_SIZE: f32 = tokens::ControlSize::TOOL;
const POPOVER_GAP: f32 = tokens::Space::XS;
/// Square size of one host chrome control in the main row's trailing
/// capsule; the capsule adds `px_1` around them and 2 px between them.
const CHROME_CONTROL_SIZE: f32 = tokens::ControlSize::CHROME;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ToolbarOverlay {
    ToolGroup(ToolbarToolGroup),
    Actions,
    Mode,
    DrawNumber(draw::DrawNumber),
    DrawChoice(draw::DrawChoice),
    /// Anchored candidate menu for one secondary chip, always fed by
    /// host-supplied candidates.
    OptionEditor(ToolbarSecondaryControl),
}

/// Stateful presentation for a controlled, Figma-like editor toolbar.
///
/// The host owns the active mode, selected tool, zoom, and mode-specific
/// values. The component owns only input drafts, open flyouts, and focus.
pub struct EditorToolbar {
    id: SharedString,
    focus_handle: FocusHandle,
    mode: ToolbarMode,
    active_tool: ToolbarTool,
    supported_tools: Option<Vec<ToolbarTool>>,
    supported_secondary_controls: Option<Vec<ToolbarSecondaryControl>>,
    zoom_percent: u16,
    dev_options: DevToolbarOptions,
    motion_options: MotionToolbarOptions,
    draw_options: super::DrawToolbarOptions,
    draw_brush_capabilities: super::DrawBrushCapabilities,
    draw_slider: Entity<crate::molecules::Slider>,
    draw_input: Entity<InputState>,
    /// Host chrome rendered in the main row's trailing capsule (§12: the
    /// host owns chrome, the toolbar owns the dock).
    chrome_controls: Vec<ToolbarChromeControl>,
    commands: Vec<ToolbarCommand>,
    overlay: Option<ToolbarOverlay>,
    command_cursor: usize,
    command_scroll_handle: ScrollHandle,
    command_input: Entity<InputState>,
    menu_cursor: usize,
    menu_scroll_handle: ScrollHandle,
    /// Last dock bounds, used to align transient menus to its edges.
    dock_bounds: gpui::Bounds<gpui::Pixels>,
    primary_scroll_handle: ScrollHandle,
    secondary_scroll_handle: ScrollHandle,
    primary_fades: EdgeFades,
    secondary_fades: EdgeFades,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ToolbarAction> for EditorToolbar {}

impl EditorToolbar {
    /// Creates a toolbar from the host's current mode, tool, and canvas zoom.
    pub fn new(
        id: impl Into<SharedString>,
        mode: ToolbarMode,
        active_tool: ToolbarTool,
        zoom_percent: u16,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let command_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search actions…"));
        let draw_slider = cx.new(|cx| crate::molecules::Slider::new("toolbar-draw-slider", 0., cx));
        let draw_input = cx.new(|cx| InputState::new(window, cx));
        let subscriptions = vec![
            cx.subscribe_in(
                &command_input,
                window,
                |this, _, event: &InputEvent, window, cx| match event {
                    InputEvent::Change => {
                        this.command_cursor = 0;
                        this.command_scroll_handle.scroll_to_item(0);
                        cx.emit(ToolbarAction::CommandQueryChanged {
                            query: this.command_input.read(cx).value(),
                        });
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        this.invoke_highlighted_command(window, cx);
                    }
                    InputEvent::Focus | InputEvent::Blur => {}
                },
            ),
            cx.subscribe_in(
                &draw_slider,
                window,
                |this, _, event: &crate::molecules::SliderAction, window, cx| {
                    if let Some(ToolbarOverlay::DrawNumber(field)) = this.overlay
                        && matches!(event.phase, crate::molecules::SliderPhase::Commit)
                    {
                        let value = field.value_at(event.value);
                        this.draw_input.update(cx, |input, cx| {
                            input.set_value(value.to_string(), window, cx)
                        });
                        this.request_draw_number(field, value, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &draw_input,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.commit_draw_input(window, cx);
                    }
                },
            ),
        ];

        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            mode,
            active_tool,
            supported_tools: None,
            supported_secondary_controls: None,
            zoom_percent: zoom_percent.clamp(1, 3_200),
            dev_options: DevToolbarOptions::default(),
            motion_options: MotionToolbarOptions::default(),
            draw_options: super::DrawToolbarOptions::default(),
            draw_brush_capabilities: super::DrawBrushCapabilities::default(),
            draw_slider,
            draw_input,
            chrome_controls: Vec::new(),
            commands: ToolbarCommand::ALL.to_vec(),
            overlay: None,
            command_cursor: 0,
            command_scroll_handle: ScrollHandle::new(),
            command_input,
            menu_cursor: 0,
            menu_scroll_handle: ScrollHandle::new(),
            dock_bounds: gpui::Bounds::default(),
            primary_scroll_handle: ScrollHandle::new(),
            secondary_scroll_handle: ScrollHandle::new(),
            primary_fades: EdgeFades::default(),
            secondary_fades: EdgeFades::default(),
            _subscriptions: subscriptions,
        }
    }
}

impl Focusable for EditorToolbar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorToolbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let key_context = if matches!(
            self.overlay,
            Some(ToolbarOverlay::Actions | ToolbarOverlay::DrawNumber(_))
        ) {
            TOOLBAR_TEXT_ENTRY_KEY_CONTEXT
        } else {
            TOOLBAR_KEY_CONTEXT
        };
        div()
            .id(self.id.clone())
            .debug_selector(|| "editor-toolbar".to_owned())
            .key_context(key_context)
            .track_focus(&self.focus_handle)
            .relative()
            .flex_shrink_1()
            .min_w(px(0.))
            .max_w_full()
            .on_action(cx.listener(|this, _: &CloseToolbarOverlay, window, cx| {
                this.dismiss_overlay(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenToolbarActions, window, cx| {
                this.open_actions(window, cx);
            }))
            // The focused Input already emitted PressEnter (which invokes the
            // highlighted command or commits a numeric value) before
            // propagating Enter; consuming the propagated action here keeps
            // the keystroke's "\n" key_char out of the single-line field.
            .on_action(cx.listener(|_, _: &ConfirmToolbarTextEntry, _, _| {}))
            .on_action(cx.listener(|this, _: &NextToolbarCommand, _, cx| {
                this.move_command_cursor(1, cx);
            }))
            .on_action(cx.listener(|this, _: &PreviousToolbarCommand, _, cx| {
                this.move_command_cursor(-1, cx);
            }))
            .on_action(cx.listener(|this, _: &FirstToolbarCommand, _, cx| {
                this.move_cursor_to_boundary(false, cx);
            }))
            .on_action(cx.listener(|this, _: &LastToolbarCommand, _, cx| {
                this.move_cursor_to_boundary(true, cx);
            }))
            .on_action(cx.listener(|this, _: &IncrementToolbarControl, _, cx| {
                this.adjust_option_control(1, cx);
            }))
            .on_action(cx.listener(|this, _: &DecrementToolbarControl, _, cx| {
                this.adjust_option_control(-1, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectBrushTool, _, cx| {
                this.request_tool(ToolbarTool::Brush, cx)
            }))
            .on_action(cx.listener(|this, _: &SelectEraserTool, _, cx| {
                this.request_tool(ToolbarTool::Eraser, cx)
            }))
            .on_action(cx.listener(|this, _: &SelectMarqueeTool, _, cx| {
                this.request_tool(ToolbarTool::RectangleSelect, cx)
            }))
            .on_action(cx.listener(|this, _: &SelectWandTool, _, cx| {
                this.request_tool(ToolbarTool::MagicWand, cx)
            }))
            .on_action(cx.listener(|this, _: &SelectMoveTool, _, cx| {
                this.request_tool(
                    if this.mode == ToolbarMode::Motion {
                        ToolbarTool::MotionSelect
                    } else {
                        ToolbarTool::Move
                    },
                    cx,
                );
            }))
            .on_action(cx.listener(|this, _: &SelectHandTool, _, cx| {
                this.request_tool(ToolbarTool::Hand, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectFrameTool, _, cx| {
                this.request_tool(ToolbarTool::Frame, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectSectionTool, _, cx| {
                this.request_tool(ToolbarTool::Section, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectSliceTool, _, cx| {
                this.request_tool(ToolbarTool::Slice, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectRectangleTool, _, cx| {
                this.request_tool(ToolbarTool::Rectangle, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectLineTool, _, cx| {
                this.request_tool(
                    if this.mode == ToolbarMode::Draw {
                        ToolbarTool::Lasso
                    } else {
                        ToolbarTool::Line
                    },
                    cx,
                );
            }))
            .on_action(cx.listener(|this, _: &SelectArrowTool, _, cx| {
                this.request_tool(
                    if this.mode == ToolbarMode::Draw {
                        ToolbarTool::PolygonalLasso
                    } else {
                        ToolbarTool::Arrow
                    },
                    cx,
                );
            }))
            .on_action(cx.listener(|this, _: &SelectEllipseTool, _, cx| {
                this.request_tool(ToolbarTool::Ellipse, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectImageVideoTool, _, cx| {
                this.request_tool(ToolbarTool::ImageVideo, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectPenTool, _, cx| {
                this.request_tool(ToolbarTool::Pen, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectPencilTool, _, cx| {
                this.request_tool(ToolbarTool::Pencil, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectTextTool, _, cx| {
                this.request_tool(ToolbarTool::Text, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectCommentTool, _, cx| {
                this.request_tool(
                    if this.mode == ToolbarMode::Draw {
                        ToolbarTool::Crop
                    } else {
                        ToolbarTool::Comment
                    },
                    cx,
                );
            }))
            .on_action(cx.listener(|this, _: &SelectAnnotationTool, _, cx| {
                this.request_tool(ToolbarTool::Annotation, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectMeasureTool, _, cx| {
                this.request_tool(
                    if this.mode == ToolbarMode::Draw {
                        ToolbarTool::EllipseSelect
                    } else {
                        ToolbarTool::Measure
                    },
                    cx,
                );
            }))
            .on_action(cx.listener(|this, _: &ZoomCanvasToFit, _, cx| {
                this.request_zoom_command(ToolbarCommand::ZoomToFit, cx);
            }))
            .on_action(cx.listener(|this, _: &ZoomCanvasToSelection, _, cx| {
                this.request_zoom_command(ToolbarCommand::ZoomToSelection, cx);
            }))
            .on_action(cx.listener(|this, _: &ZoomCanvasTo100, _, cx| {
                this.request_zoom(100, cx);
            }))
            .on_action(cx.listener(|this, _: &EnterDevMode, _, cx| {
                this.request_mode(ToolbarMode::Dev, cx);
            }))
            .child(
                v_flex()
                    .id(SharedString::from(format!("{}-surface", self.id)))
                    .debug_selector(|| "editor-toolbar-surface".to_owned())
                    // The dock floats over the host canvas: every pointer
                    // event on it — clicks, presses, hovers, and wheel — stops
                    // here so a tool press never also reaches the canvas
                    // underneath. The dock's own overflow rows sit above this
                    // hitbox and keep scrolling; the transient popups block
                    // on their own surfaces.
                    .occlude()
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .relative()
                    .min_w(px(0.))
                    .max_w_full()
                    .p_1()
                    .gap_1()
                    .rounded(px(16.))
                    .border_1()
                    .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
                    .bg(crate::atoms::SemanticColor::BackgroundMenu.resolve(cx))
                    .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
                    .when(cx.theme().shadow, |surface| surface.shadow_lg())
                    .when(self.mode != ToolbarMode::Design, |surface| {
                        surface.child(self.render_secondary_toolbar(window, cx))
                    })
                    .child(self.render_main_toolbar(window, cx))
                    .child(crate::atoms::track_bounds(cx.entity(), |this, bounds| {
                        this.dock_bounds = bounds.dilate(px(1.))
                    })),
            )
            .when(self.overlay == Some(ToolbarOverlay::Actions), |root| {
                root.child(self.render_actions_palette(window, cx))
            })
    }
}

#[cfg(test)]
mod interaction_tests;
