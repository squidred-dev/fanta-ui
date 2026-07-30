use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, MouseDownEvent, ParentElement as _, Render, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window, deferred,
    div, prelude::FluentBuilder as _, px, rgba,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex,
    input::{Input, InputEvent, InputState},
    tooltip::Tooltip,
    v_flex,
};

use super::{
    ActivateToolbarControl, AgentToolbarOptions, CloseToolbarOverlay, DevToolbarOptions,
    DrawToolbarOptions, EnterDevMode, MotionToolbarOptions, NextToolbarCommand, OpenToolbarActions,
    OpenToolbarAgent, PreviousToolbarCommand, SelectAnnotationTool, SelectArrowTool,
    SelectCommentTool, SelectEllipseTool, SelectFrameTool, SelectHandTool, SelectImageVideoTool,
    SelectLineTool, SelectMeasureTool, SelectMoveTool, SelectPenTool, SelectPencilTool,
    SelectRectangleTool, SelectScaleTool, SelectSectionTool, SelectSliceTool, SelectTextTool,
    ToolbarAction, ToolbarCommand, ToolbarControlValue, ToolbarItem, ToolbarMode,
    ToolbarSecondaryControl, ToolbarTool, ToolbarToolGroup,
    commands::{TOOLBAR_CONTROL_KEY_CONTEXT, TOOLBAR_KEY_CONTEXT, TOOLBAR_TEXT_ENTRY_KEY_CONTEXT},
};
use crate::color::parse_hex_rgba;

const TOOLBAR_HEIGHT: f32 = 48.;
const TOOL_SIZE: f32 = 32.;
const TOOLBAR_BOTTOM: f32 = 18.;
const OVERLAY_BOTTOM: f32 = TOOLBAR_BOTTOM + TOOLBAR_HEIGHT + 10.;
const DETACHED_CONTROL_BOTTOM: f32 = OVERLAY_BOTTOM;
const SECONDARY_TOOLBAR_BOTTOM: f32 = DETACHED_CONTROL_BOTTOM + 36. + 10.;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ToolbarOverlay {
    ToolGroup(ToolbarToolGroup),
    Actions,
    Agent,
    Zoom,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CommandScope {
    #[default]
    All,
    Assets,
    PluginsAndWidgets,
}

impl CommandScope {
    const ALL: [Self; 3] = [Self::All, Self::Assets, Self::PluginsAndWidgets];

    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Assets => "Assets",
            Self::PluginsAndWidgets => "Plugins & widgets",
        }
    }

    const fn includes(self, command: ToolbarCommand) -> bool {
        match self {
            Self::All => true,
            Self::Assets => matches!(
                command,
                ToolbarCommand::OpenResources | ToolbarCommand::OpenVariables
            ),
            Self::PluginsAndWidgets => matches!(
                command,
                ToolbarCommand::OpenPlugins | ToolbarCommand::OpenWidgets
            ),
        }
    }
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
    zoom_percent: u16,
    draw_options: DrawToolbarOptions,
    dev_options: DevToolbarOptions,
    motion_options: MotionToolbarOptions,
    agent_options: AgentToolbarOptions,
    commands: Vec<ToolbarCommand>,
    overlay: Option<ToolbarOverlay>,
    command_scope: CommandScope,
    command_cursor: usize,
    command_scroll_handle: ScrollHandle,
    command_input: Entity<InputState>,
    ai_input: Entity<InputState>,
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
        let command_input = cx
            .new(|cx| InputState::new(window, cx).placeholder("Search commands, plugins, and AI…"));
        let ai_input = cx.new(|cx| InputState::new(window, cx).placeholder("Describe your idea"));

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
                &ai_input,
                window,
                |this, _, event: &InputEvent, window, cx| match event {
                    InputEvent::PressEnter { .. } => this.submit_ai_prompt(window, cx),
                    InputEvent::Change => cx.notify(),
                    InputEvent::Focus | InputEvent::Blur => {}
                },
            ),
        ];

        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            mode,
            active_tool,
            zoom_percent: zoom_percent.clamp(1, 3_200),
            draw_options: DrawToolbarOptions::default(),
            dev_options: DevToolbarOptions::default(),
            motion_options: MotionToolbarOptions::default(),
            agent_options: AgentToolbarOptions::default(),
            commands: ToolbarCommand::ALL.to_vec(),
            overlay: None,
            command_scope: CommandScope::default(),
            command_cursor: 0,
            command_scroll_handle: ScrollHandle::new(),
            command_input,
            ai_input,
            _subscriptions: subscriptions,
        }
    }

    /// Returns the last mode supplied by the host.
    pub fn mode(&self) -> ToolbarMode {
        self.mode
    }

    /// Returns the last active tool supplied by the host.
    pub fn active_tool(&self) -> ToolbarTool {
        self.active_tool
    }

    /// Returns the clamped canvas zoom percentage.
    pub fn zoom_percent(&self) -> u16 {
        self.zoom_percent
    }

    /// Replaces the controlled mode and closes any transient overlay.
    pub fn set_mode(&mut self, mode: ToolbarMode, cx: &mut Context<Self>) {
        self.mode = mode;
        self.set_overlay(None, cx);
        cx.notify();
    }

    /// Replaces the controlled active tool and closes any transient overlay.
    pub fn set_active_tool(&mut self, tool: ToolbarTool, cx: &mut Context<Self>) {
        self.active_tool = tool;
        self.set_overlay(None, cx);
        cx.notify();
    }

    /// Replaces the controlled zoom, clamped to 1–3,200 percent.
    pub fn set_zoom_percent(&mut self, percent: u16, cx: &mut Context<Self>) {
        self.zoom_percent = percent.clamp(1, 3_200);
        cx.notify();
    }

    /// Replaces the controlled Draw-mode values.
    pub fn set_draw_options(&mut self, options: DrawToolbarOptions, cx: &mut Context<Self>) {
        self.draw_options = options;
        cx.notify();
    }

    /// Replaces the controlled Dev-mode values.
    pub fn set_dev_options(&mut self, options: DevToolbarOptions, cx: &mut Context<Self>) {
        self.dev_options = options;
        cx.notify();
    }

    /// Replaces the controlled Motion-mode values.
    pub fn set_motion_options(&mut self, options: MotionToolbarOptions, cx: &mut Context<Self>) {
        self.motion_options = options;
        cx.notify();
    }

    /// Replaces Agent context copy and suggestions.
    pub fn set_agent_options(&mut self, options: AgentToolbarOptions, cx: &mut Context<Self>) {
        self.agent_options = options;
        cx.notify();
    }

    /// Replaces the ordered command subset available in Actions.
    pub fn set_commands(
        &mut self,
        commands: impl IntoIterator<Item = ToolbarCommand>,
        cx: &mut Context<Self>,
    ) {
        self.commands.clear();
        for command in commands {
            if !self.commands.contains(&command) {
                self.commands.push(command);
            }
        }
        self.command_cursor = 0;
        self.command_scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn set_overlay(&mut self, overlay: Option<ToolbarOverlay>, cx: &mut Context<Self>) {
        let agent_was_visible = self.overlay == Some(ToolbarOverlay::Agent);
        let agent_is_visible = overlay == Some(ToolbarOverlay::Agent);
        self.overlay = overlay;
        if agent_was_visible != agent_is_visible {
            cx.emit(ToolbarAction::AgentVisibilityChanged {
                visible: agent_is_visible,
            });
        }
    }

    /// Opens the Actions command palette and transfers focus to its query.
    pub fn open_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_overlay(Some(ToolbarOverlay::Actions), cx);
        self.command_scope = CommandScope::All;
        self.command_cursor = 0;
        self.command_scroll_handle.scroll_to_item(0);
        self.command_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    /// Opens the contextual Figma Agent-style composer.
    pub fn open_agent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_overlay(Some(ToolbarOverlay::Agent), cx);
        self.ai_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    /// Closes the active overlay, restores toolbar focus, and reports whether
    /// an overlay was present.
    pub fn dismiss_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.overlay.is_none() {
            return false;
        }
        self.set_overlay(None, cx);
        let focus_handle = self.focus_handle.clone();
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
        cx.notify();
        true
    }

    fn dismiss_overlay_for_pointer(&mut self, cx: &mut Context<Self>) {
        if self.overlay.is_some() {
            self.set_overlay(None, cx);
            cx.notify();
        }
    }

    fn toggle_tool_group(&mut self, group: ToolbarToolGroup, cx: &mut Context<Self>) {
        let next = ToolbarOverlay::ToolGroup(group);
        let next = if self.overlay == Some(next) {
            None
        } else {
            Some(next)
        };
        self.set_overlay(next, cx);
        cx.notify();
    }

    fn toggle_zoom(&mut self, cx: &mut Context<Self>) {
        let next = if self.overlay == Some(ToolbarOverlay::Zoom) {
            None
        } else {
            Some(ToolbarOverlay::Zoom)
        };
        self.set_overlay(next, cx);
        cx.notify();
    }

    fn request_zoom(&mut self, percent: u16, cx: &mut Context<Self>) {
        cx.emit(ToolbarAction::ZoomChangeRequested {
            percent: percent.clamp(1, 3_200),
        });
    }

    fn toggle_agent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.overlay == Some(ToolbarOverlay::Agent) {
            self.dismiss_overlay(window, cx);
        } else {
            self.open_agent(window, cx);
        }
    }

    fn request_mode(&mut self, mode: ToolbarMode, cx: &mut Context<Self>) {
        self.set_overlay(None, cx);
        cx.emit(ToolbarAction::ModeChangeRequested { mode });
        cx.notify();
    }

    fn request_tool(&mut self, tool: ToolbarTool, cx: &mut Context<Self>) {
        self.set_overlay(None, cx);
        let requested_mode = if tool.is_available_in(self.mode) {
            self.mode
        } else {
            ToolbarMode::ALL
                .iter()
                .copied()
                .find(|mode| tool.is_available_in(*mode))
                .unwrap_or(self.mode)
        };
        if requested_mode != self.mode {
            cx.emit(ToolbarAction::ModeChangeRequested {
                mode: requested_mode,
            });
        }
        cx.emit(ToolbarAction::ToolChangeRequested {
            mode: requested_mode,
            tool,
        });
        cx.notify();
    }

    fn activate_tool_or_actions(
        &mut self,
        tool: ToolbarTool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if tool == ToolbarTool::Actions {
            self.open_actions(window, cx);
        } else {
            self.request_tool(tool, cx);
        }
    }

    fn filtered_commands(&self, cx: &App) -> Vec<ToolbarCommand> {
        let query = self.command_input.read(cx).value().to_lowercase();
        self.commands
            .iter()
            .copied()
            .filter(|command| {
                self.command_scope.includes(*command)
                    && (query.is_empty()
                        || command.label().to_lowercase().contains(&query)
                        || command.category().to_lowercase().contains(&query)
                        || command.description().to_lowercase().contains(&query))
            })
            .collect()
    }

    fn invoke_highlighted_command(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let commands = self.filtered_commands(cx);
        let Some(command) = commands
            .get(self.command_cursor.min(commands.len().saturating_sub(1)))
            .copied()
        else {
            return;
        };
        self.invoke_command(command, window, cx);
    }

    fn move_command_cursor(&mut self, direction: isize, cx: &mut Context<Self>) {
        if self.overlay != Some(ToolbarOverlay::Actions) {
            return;
        }
        let len = self.filtered_commands(cx).len();
        if len == 0 {
            self.command_cursor = 0;
            return;
        }
        self.command_cursor = if direction < 0 {
            self.command_cursor.checked_sub(1).unwrap_or(len - 1)
        } else {
            (self.command_cursor + 1) % len
        };
        self.command_scroll_handle
            .scroll_to_item(self.command_cursor);
        cx.notify();
    }

    fn set_command_scope(&mut self, scope: CommandScope, cx: &mut Context<Self>) {
        self.command_scope = scope;
        self.command_cursor = 0;
        self.command_scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn invoke_command(
        &mut self,
        command: ToolbarCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.emit(ToolbarAction::CommandInvoked { command });
        match command {
            ToolbarCommand::GenerateDesign
            | ToolbarCommand::ReplaceContent
            | ToolbarCommand::RewriteText
            | ToolbarCommand::TranslateText
            | ToolbarCommand::RenameLayers
            | ToolbarCommand::RemoveBackground
            | ToolbarCommand::GenerateImage
            | ToolbarCommand::MakePrototype => self.open_agent(window, cx),
            _ => {
                self.set_overlay(None, cx);
                let focus_handle = self.focus_handle.clone();
                window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
                cx.notify();
            }
        }
    }

    fn submit_ai_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.ai_input.read(cx).value();
        let prompt = value.trim();
        if prompt.is_empty() {
            return;
        }
        cx.emit(ToolbarAction::AiPromptSubmitted {
            prompt: SharedString::from(prompt.to_owned()),
        });
        self.ai_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    fn set_ai_suggestion(
        &mut self,
        suggestion: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.ai_input.update(cx, |input, cx| {
            input.set_value(suggestion, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    fn activate_agent_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.ai_input.read(cx).value().trim().is_empty() {
            cx.emit(ToolbarAction::AgentVoiceInputRequested);
        } else {
            self.submit_ai_prompt(window, cx);
        }
    }

    fn request_secondary(&mut self, control: ToolbarSecondaryControl, cx: &mut Context<Self>) {
        cx.emit(ToolbarAction::SecondaryControlInvoked {
            mode: self.mode,
            control,
        });
    }

    fn request_control_value(
        &mut self,
        control: ToolbarSecondaryControl,
        value: ToolbarControlValue,
        cx: &mut Context<Self>,
    ) {
        cx.emit(ToolbarAction::ControlChangeRequested {
            mode: self.mode,
            control,
            value,
        });
    }

    fn toggle_draw_color(&mut self, cx: &mut Context<Self>) {
        let next = if self.draw_options.stroke_color == "#1E1E1E" {
            "#0D99FF"
        } else {
            "#1E1E1E"
        };
        self.request_control_value(
            ToolbarSecondaryControl::DrawStrokeColor,
            ToolbarControlValue::Color(next.into()),
            cx,
        );
    }

    fn control_tooltip(label: &'static str, shortcut: Option<&'static str>) -> SharedString {
        match shortcut {
            Some(shortcut) => format!("{label}  {shortcut}").into(),
            None => label.into(),
        }
    }

    fn mode_accent(mode: ToolbarMode, cx: &App) -> gpui::Hsla {
        match mode {
            ToolbarMode::Draw => cx.theme().cyan,
            ToolbarMode::Design => cx.theme().blue,
            ToolbarMode::Motion => cx.theme().magenta,
            ToolbarMode::Dev => cx.theme().green,
        }
    }

    fn mode_accent_pale(mode: ToolbarMode, cx: &App) -> gpui::Hsla {
        match mode {
            ToolbarMode::Draw => cx.theme().cyan_light,
            ToolbarMode::Design => cx.theme().blue_light,
            ToolbarMode::Motion => cx.theme().magenta_light,
            ToolbarMode::Dev => cx.theme().green_light,
        }
    }

    fn mode_accent_foreground(accent: gpui::Hsla, cx: &App) -> gpui::Hsla {
        let foreground = cx.theme().foreground;
        let background = cx.theme().background;
        if (accent.l - foreground.l).abs() >= (accent.l - background.l).abs() {
            foreground
        } else {
            background
        }
    }

    fn parse_color(value: &str) -> Option<gpui::Rgba> {
        parse_hex_rgba(value).map(rgba)
    }

    fn render_tool_glyph(
        &self,
        tool: ToolbarTool,
        selected: bool,
        compact: bool,
        cx: &App,
    ) -> AnyElement {
        div()
            .w_full()
            .text_center()
            .text_size(px(if compact { 11. } else { 17. }))
            .font_semibold()
            .text_color(if selected {
                Self::mode_accent_foreground(Self::mode_accent(self.mode, cx), cx)
            } else {
                cx.theme().foreground
            })
            .child(tool.glyph())
            .into_any_element()
    }

    fn render_tool_button(&self, tool: ToolbarTool, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.active_tool == tool
            || (tool == ToolbarTool::Actions && self.overlay == Some(ToolbarOverlay::Actions));
        let accent = Self::mode_accent(self.mode, cx);
        let tooltip = Self::control_tooltip(tool.label(), tool.shortcut());
        h_flex()
            .id(SharedString::from(format!(
                "{}-tool-{}",
                self.id,
                tool.label().to_lowercase().replace([' ', '/'], "-")
            )))
            .debug_selector({
                let label = tool.label();
                move || format!("toolbar-tool-{}", label.to_lowercase().replace(' ', "-"))
            })
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .size(px(TOOL_SIZE))
            .justify_center()
            .rounded(px(8.))
            .cursor_pointer()
            .bg(if selected {
                accent
            } else {
                cx.theme().transparent
            })
            .hover(|style| style.bg(if selected { accent } else { cx.theme().accent }))
            .focus(|style| style.border_2().border_color(cx.theme().ring))
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .on_action(
                cx.listener(move |this, _: &ActivateToolbarControl, window, cx| {
                    this.activate_tool_or_actions(tool, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.activate_tool_or_actions(tool, window, cx);
            }))
            .child(self.render_tool_glyph(tool, selected, matches!(tool, ToolbarTool::Code), cx))
            .into_any_element()
    }

    fn render_tool_group(&self, group: ToolbarToolGroup, cx: &mut Context<Self>) -> AnyElement {
        let tool = group.display_tool(self.active_tool);
        let selected = group.tools().contains(&self.active_tool);
        let menu_open = self.overlay == Some(ToolbarOverlay::ToolGroup(group));
        let accent = Self::mode_accent(self.mode, cx);
        let tooltip = Self::control_tooltip(tool.label(), tool.shortcut());

        h_flex()
            .h(px(TOOL_SIZE))
            .flex_none()
            .child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-group-{}-main",
                        self.id,
                        group.label().to_lowercase().replace(' ', "-")
                    )))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .size(px(TOOL_SIZE))
                    .justify_center()
                    .rounded(px(8.))
                    .cursor_pointer()
                    .bg(if selected {
                        accent
                    } else {
                        cx.theme().transparent
                    })
                    .hover(|style| style.bg(if selected { accent } else { cx.theme().accent }))
                    .focus(|style| style.border_2().border_color(cx.theme().ring))
                    .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
                    .on_action(cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                        this.request_tool(tool, cx);
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
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
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .w(px(12.))
                    .h_full()
                    .justify_center()
                    .rounded(px(5.))
                    .cursor_pointer()
                    .text_color(cx.theme().muted_foreground)
                    .hover(|style| style.bg(cx.theme().accent))
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .on_action(cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                        this.toggle_tool_group(group, cx);
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_tool_group(group, cx);
                    }))
                    .child(
                        Icon::new(if menu_open {
                            IconName::ChevronUp
                        } else {
                            IconName::ChevronDown
                        })
                        .xsmall(),
                    ),
            )
            .into_any_element()
    }

    fn render_mode_button(&self, mode: ToolbarMode, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.mode == mode;
        let accent = Self::mode_accent(mode, cx);
        let tooltip = Self::control_tooltip(mode.label(), mode.shortcut());
        h_flex()
            .id(SharedString::from(format!(
                "{}-mode-{}",
                self.id,
                mode.label().to_lowercase()
            )))
            .debug_selector({
                let label = mode.label();
                move || format!("toolbar-mode-{}", label.to_lowercase())
            })
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .size(px(TOOL_SIZE))
            .justify_center()
            .rounded(px(8.))
            .cursor_pointer()
            .bg(if selected {
                cx.theme().background
            } else {
                cx.theme().transparent
            })
            .when(selected && cx.theme().shadow, |button| button.shadow_sm())
            .hover(|style| style.bg(cx.theme().background))
            .focus(|style| style.border_2().border_color(cx.theme().ring))
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .on_action(cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                this.request_mode(mode, cx);
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.request_mode(mode, cx);
            }))
            .child(
                div()
                    .w_full()
                    .text_center()
                    .font_semibold()
                    .text_size(px(if mode == ToolbarMode::Dev { 10. } else { 16. }))
                    .text_color(if selected {
                        accent
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(mode.glyph()),
            )
            .into_any_element()
    }

    fn render_main_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut tools = h_flex().h_full().gap(px(2.));
        for item in self.mode.layout() {
            tools = match item {
                ToolbarItem::Group(group) => tools.child(self.render_tool_group(*group, cx)),
                ToolbarItem::Tool(tool) => tools.child(self.render_tool_button(*tool, cx)),
                ToolbarItem::Separator => {
                    tools.child(div().w(px(1.)).h(px(24.)).mx_1().bg(cx.theme().border))
                }
            };
        }

        let mut modes = h_flex()
            .h(px(40.))
            .px_1()
            .gap(px(2.))
            .rounded(px(12.))
            .bg(cx.theme().secondary);
        for mode in ToolbarMode::ALL {
            modes = modes.child(self.render_mode_button(*mode, cx));
        }

        h_flex()
            .absolute()
            .left_0()
            .right_0()
            .bottom(px(TOOLBAR_BOTTOM))
            .justify_center()
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-surface", self.id)))
                    .debug_selector(|| "editor-toolbar-surface".to_owned())
                    .h(px(TOOLBAR_HEIGHT))
                    .px_2()
                    .gap_2()
                    .rounded(px(16.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().popover)
                    .text_color(cx.theme().popover_foreground)
                    .when(cx.theme().shadow, |surface| surface.shadow_lg())
                    .child(tools)
                    .child(div().w(px(1.)).h(px(28.)).flex_none().bg(cx.theme().border))
                    .child(modes),
            )
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
        let label = label.into();
        let accent = Self::mode_accent(self.mode, cx);
        let pale_accent = Self::mode_accent_pale(self.mode, cx);
        h_flex()
            .id(SharedString::from(format!(
                "{}-secondary-{suffix}",
                self.id
            )))
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(28.))
            .px_2()
            .gap_1()
            .rounded(px(7.))
            .cursor_pointer()
            .text_xs()
            .font_medium()
            .bg(if selected {
                pale_accent
            } else {
                cx.theme().transparent
            })
            .text_color(if selected {
                accent
            } else {
                cx.theme().popover_foreground
            })
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| style.border_1().border_color(cx.theme().ring))
            .on_action(cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                this.request_secondary(control, cx);
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.request_secondary(control, cx);
            }))
            .child(label)
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
        let label = label.into();
        let accent = Self::mode_accent(self.mode, cx);
        let pale_accent = Self::mode_accent_pale(self.mode, cx);
        h_flex()
            .id(SharedString::from(format!(
                "{}-secondary-{suffix}",
                self.id
            )))
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(28.))
            .px_2()
            .gap_1()
            .rounded(px(7.))
            .cursor_pointer()
            .text_xs()
            .font_medium()
            .bg(if selected {
                pale_accent
            } else {
                cx.theme().transparent
            })
            .text_color(if selected {
                accent
            } else {
                cx.theme().popover_foreground
            })
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| style.border_1().border_color(cx.theme().ring))
            .on_action({
                let value = value.clone();
                cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                    this.request_control_value(control, value.clone(), cx);
                })
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.request_control_value(control, value.clone(), cx);
            }))
            .child(label)
            .into_any_element()
    }

    fn render_draw_secondary(&self, cx: &mut Context<Self>) -> AnyElement {
        let next_weight = match self.draw_options.stroke_weight {
            1..=7 => 8,
            8..=15 => 16,
            _ => 2,
        };
        let next_smoothing = (self.draw_options.smoothing.saturating_add(16)) % 112;
        let next_style: SharedString = if self.draw_options.brush_style == "Solid" {
            "Charcoal".into()
        } else {
            "Solid".into()
        };
        let stroke_color = Self::parse_color(self.draw_options.stroke_color.as_ref())
            .unwrap_or_else(|| cx.theme().foreground.into());
        h_flex()
            .h(px(40.))
            .px_1()
            .gap_1()
            .rounded(px(12.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .text_color(cx.theme().popover_foreground)
            .child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-secondary-draw-color",
                        self.id
                    )))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .size(px(28.))
                    .justify_center()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .bg(cx.theme().secondary)
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .child(
                        div()
                            .size(px(16.))
                            .rounded_full()
                            .bg(stroke_color)
                            .border_2()
                            .border_color(cx.theme().popover),
                    )
                    .on_action(cx.listener(|this, _: &ActivateToolbarControl, _, cx| {
                        this.toggle_draw_color(cx);
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_draw_color(cx);
                    })),
            )
            .child(self.render_value_button(
                "draw-style",
                self.draw_options.brush_style.clone(),
                ToolbarSecondaryControl::DrawBrushStyle,
                ToolbarControlValue::Choice(next_style),
                false,
                cx,
            ))
            .child(self.render_value_button(
                "draw-weight",
                format!("{} px", self.draw_options.stroke_weight),
                ToolbarSecondaryControl::DrawStrokeWeight,
                ToolbarControlValue::Integer(next_weight),
                false,
                cx,
            ))
            .child(div().w(px(1.)).h(px(22.)).bg(cx.theme().border))
            .child(self.render_value_button(
                "draw-smoothing",
                format!("Smooth {}%", self.draw_options.smoothing),
                ToolbarSecondaryControl::DrawSmoothing,
                ToolbarControlValue::Integer(i32::from(next_smoothing.min(100))),
                false,
                cx,
            ))
            .child(self.render_value_button(
                "draw-pressure",
                "Pressure",
                ToolbarSecondaryControl::DrawPressure,
                ToolbarControlValue::Toggle(!self.draw_options.pressure),
                self.draw_options.pressure,
                cx,
            ))
            .into_any_element()
    }

    fn render_dev_secondary(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .h(px(40.))
            .px_1()
            .gap_1()
            .rounded(px(12.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .text_color(cx.theme().popover_foreground)
            .child(self.render_secondary_button(
                "dev-inspect",
                "⌖  Inspect",
                ToolbarSecondaryControl::DevInspect,
                self.active_tool == ToolbarTool::Inspect,
                cx,
            ))
            .child(self.render_secondary_button(
                "dev-annotate",
                "¶  Annotate",
                ToolbarSecondaryControl::DevAnnotate,
                self.active_tool == ToolbarTool::Annotation,
                cx,
            ))
            .child(self.render_secondary_button(
                "dev-measure",
                "↔  Measure",
                ToolbarSecondaryControl::DevMeasure,
                self.active_tool == ToolbarTool::Measure,
                cx,
            ))
            .child(div().w(px(1.)).h(px(22.)).bg(cx.theme().border))
            .child(self.render_value_button(
                "dev-ready",
                if self.dev_options.ready_for_development {
                    "✓  Ready for dev"
                } else {
                    "Mark ready for dev"
                },
                ToolbarSecondaryControl::DevReadyForDevelopment,
                ToolbarControlValue::Toggle(!self.dev_options.ready_for_development),
                self.dev_options.ready_for_development,
                cx,
            ))
            .into_any_element()
    }

    fn render_motion_secondary(&self, cx: &mut Context<Self>) -> AnyElement {
        let current_seconds = self.motion_options.current_time_ms as f32 / 1_000.;
        let duration_seconds = self.motion_options.duration_ms as f32 / 1_000.;
        let next_style: SharedString = if self.motion_options.animation_style == "Fade in" {
            "Spring".into()
        } else {
            "Fade in".into()
        };
        h_flex()
            .h(px(40.))
            .px_1()
            .gap_1()
            .rounded(px(12.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .text_color(cx.theme().popover_foreground)
            .child(self.render_value_button(
                "motion-play",
                if self.motion_options.playing {
                    "❚❚"
                } else {
                    "▶"
                },
                ToolbarSecondaryControl::MotionPlayPause,
                ToolbarControlValue::Toggle(!self.motion_options.playing),
                self.motion_options.playing,
                cx,
            ))
            .child(self.render_value_button(
                "motion-loop",
                "↻",
                ToolbarSecondaryControl::MotionLoop,
                ToolbarControlValue::Toggle(!self.motion_options.looping),
                self.motion_options.looping,
                cx,
            ))
            .child(
                div()
                    .h(px(28.))
                    .px_2()
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .bg(cx.theme().secondary)
                    .text_xs()
                    .child(format!("{current_seconds:.1}s / {duration_seconds:.1}s")),
            )
            .child(self.render_value_button(
                "motion-autokey",
                "●  Auto key",
                ToolbarSecondaryControl::MotionAutoKeyframe,
                ToolbarControlValue::Toggle(!self.motion_options.auto_keyframe),
                self.motion_options.auto_keyframe,
                cx,
            ))
            .child(self.render_secondary_button(
                "motion-keyframe",
                "◇+  Keyframe",
                ToolbarSecondaryControl::MotionAddKeyframe,
                false,
                cx,
            ))
            .child(self.render_value_button(
                "motion-style",
                self.motion_options.animation_style.clone(),
                ToolbarSecondaryControl::MotionAnimationStyle,
                ToolbarControlValue::Choice(next_style),
                false,
                cx,
            ))
            .child(self.render_secondary_button(
                "motion-timeline",
                "▤  Timeline",
                ToolbarSecondaryControl::MotionTimeline,
                true,
                cx,
            ))
            .child(self.render_secondary_button(
                "motion-time-comment",
                "◷  Comment",
                ToolbarSecondaryControl::MotionTimeComment,
                false,
                cx,
            ))
            .into_any_element()
    }

    fn render_secondary_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.overlay.is_some() {
            return div().into_any_element();
        }
        let content = match self.mode {
            ToolbarMode::Design => return div().into_any_element(),
            ToolbarMode::Draw
                if matches!(
                    self.active_tool,
                    ToolbarTool::Brush
                        | ToolbarTool::Pencil
                        | ToolbarTool::VariableWidth
                        | ToolbarTool::PaintBucket
                ) =>
            {
                self.render_draw_secondary(cx)
            }
            ToolbarMode::Draw => return div().into_any_element(),
            ToolbarMode::Dev => self.render_dev_secondary(cx),
            ToolbarMode::Motion => self.render_motion_secondary(cx),
        };
        h_flex()
            .absolute()
            .left_0()
            .right_0()
            .bottom(px(SECONDARY_TOOLBAR_BOTTOM))
            .justify_center()
            .child(content)
            .into_any_element()
    }

    fn render_tool_group_menu(
        &self,
        group: ToolbarToolGroup,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut menu = v_flex()
            .id(SharedString::from(format!(
                "{}-{}-flyout",
                self.id,
                group.label().to_lowercase().replace(' ', "-")
            )))
            .debug_selector(|| "toolbar-tool-flyout".to_owned())
            .block_mouse_except_scroll()
            .w(px(248.))
            .p_2()
            .rounded(px(14.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .text_color(cx.theme().popover_foreground)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                this.dismiss_overlay_for_pointer(cx);
            }));
        for tool in group.tools() {
            let tool = *tool;
            let selected = self.active_tool == tool;
            menu = menu.child(
                h_flex()
                    .id(SharedString::from(format!(
                        "{}-flyout-{}",
                        self.id,
                        tool.label().to_lowercase().replace([' ', '/'], "-")
                    )))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .h(px(38.))
                    .px_2()
                    .gap_2()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().accent))
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .on_action(cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                        this.request_tool(tool, cx);
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.request_tool(tool, cx);
                    }))
                    .child(
                        div()
                            .w(px(18.))
                            .text_center()
                            .font_semibold()
                            .text_color(if selected {
                                cx.theme().primary
                            } else {
                                cx.theme().popover_foreground
                            })
                            .child(tool.glyph()),
                    )
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
        deferred(
            h_flex()
                .absolute()
                .left_0()
                .right_0()
                .bottom(px(OVERLAY_BOTTOM))
                .justify_center()
                .child(menu),
        )
        .with_priority(10)
        .into_any_element()
    }

    fn render_command_row(
        &self,
        command: ToolbarCommand,
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.command_cursor == index;
        h_flex()
            .id(SharedString::from(format!(
                "{}-command-{}",
                self.id,
                command.label().to_lowercase().replace([' ', '/'], "-")
            )))
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(42.))
            .px_3()
            .gap_3()
            .rounded(px(8.))
            .cursor_pointer()
            .bg(if selected {
                cx.theme().list_active
            } else {
                cx.theme().transparent
            })
            .hover(|style| style.bg(cx.theme().list_hover))
            .focus(|style| style.border_1().border_color(cx.theme().ring))
            .on_hover(cx.listener(move |this, hovered, _, cx| {
                if *hovered {
                    this.command_cursor = index;
                    cx.notify();
                }
            }))
            .on_action(
                cx.listener(move |this, _: &ActivateToolbarControl, window, cx| {
                    this.invoke_command(command, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
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
                    .font_semibold()
                    .child(if command.category() == "AI" {
                        "✦"
                    } else {
                        "⌘"
                    }),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .gap(px(1.))
                    .child(div().truncate().text_sm().child(command.label()))
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

    fn render_actions_palette(&self, cx: &mut Context<Self>) -> AnyElement {
        let commands = self.filtered_commands(cx);
        let mut results = v_flex()
            .id(SharedString::from(format!("{}-actions-results", self.id)))
            .w_full()
            .h(px(378.))
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
            for (index, command) in commands.into_iter().enumerate() {
                results = results.child(self.render_command_row(command, index, cx));
            }
        }

        let palette =
            v_flex()
                .id(SharedString::from(format!("{}-actions-palette", self.id)))
                .debug_selector(|| "toolbar-actions-palette".to_owned())
                .block_mouse_except_scroll()
                .w(px(420.))
                .max_h(px(540.))
                .rounded(px(16.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .text_color(cx.theme().popover_foreground)
                .when(cx.theme().shadow, |surface| surface.shadow_lg())
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
                        .children(CommandScope::ALL.into_iter().enumerate().map(
                            |(index, scope)| {
                                h_flex()
                                    .id(SharedString::from(format!(
                                        "{}-actions-scope-{index}",
                                        self.id
                                    )))
                                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                                    .tab_index(0)
                                    .h(px(30.))
                                    .px_2()
                                    .justify_center()
                                    .rounded(px(7.))
                                    .cursor_pointer()
                                    .text_xs()
                                    .font_medium()
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
                                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                                    .on_action(cx.listener(
                                        move |this, _: &ActivateToolbarControl, _, cx| {
                                            this.set_command_scope(scope, cx);
                                        },
                                    ))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_command_scope(scope, cx);
                                    }))
                                    .child(scope.label())
                            },
                        )),
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
                        .child("↑↓ Navigate")
                        .child("↵ Run")
                        .child(div().flex_1())
                        .child("Commands · Plugins · Widgets · AI"),
                );

        deferred(
            h_flex()
                .absolute()
                .left_0()
                .right_0()
                .bottom(px(OVERLAY_BOTTOM))
                .justify_center()
                .child(palette),
        )
        .with_priority(12)
        .into_any_element()
    }

    fn render_agent_suggestion(
        &self,
        index: usize,
        label: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let action_label = label.clone();
        let click_label = label.clone();
        h_flex()
            .id(SharedString::from(format!(
                "{}-agent-suggestion-{index}",
                self.id
            )))
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(28.))
            .px_2()
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border)
            .cursor_pointer()
            .text_xs()
            .text_color(cx.theme().popover_foreground)
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| style.border_color(cx.theme().ring))
            .on_action(
                cx.listener(move |this, _: &ActivateToolbarControl, window, cx| {
                    this.set_ai_suggestion(action_label.clone(), window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.set_ai_suggestion(click_label.clone(), window, cx);
            }))
            .child(label)
            .into_any_element()
    }

    fn render_agent_composer(&self, cx: &mut Context<Self>) -> AnyElement {
        let has_prompt = !self.ai_input.read(cx).value().trim().is_empty();
        let mut suggestions = h_flex().gap_1();
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
        let composer = v_flex()
            .id(SharedString::from(format!("{}-agent-composer", self.id)))
            .debug_selector(|| "toolbar-agent-composer".to_owned())
            .block_mouse_except_scroll()
            .w(px(448.))
            .p_3()
            .gap_2()
            .rounded(px(20.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .text_color(cx.theme().popover_foreground)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .child(
                h_flex()
                    .h(px(28.))
                    .gap_2()
                    .child(
                        h_flex()
                            .size(px(24.))
                            .justify_center()
                            .rounded(px(7.))
                            .bg(cx.theme().magenta_light)
                            .text_color(cx.theme().magenta)
                            .font_semibold()
                            .child("✦"),
                    )
                    .child(div().font_semibold().text_sm().child("Figma Agent"))
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded(px(6.))
                            .bg(cx.theme().secondary)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.agent_options.context_label.clone()),
                    )
                    .child(div().flex_1())
                    .child(
                        h_flex()
                            .id(SharedString::from(format!("{}-agent-close", self.id)))
                            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .size(px(24.))
                            .justify_center()
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| style.border_1().border_color(cx.theme().ring))
                            .on_action(cx.listener(
                                |this, _: &ActivateToolbarControl, window, cx| {
                                    this.dismiss_overlay(window, cx);
                                },
                            ))
                            .on_click(cx.listener(|this, _, window, cx| {
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
                            .small(),
                    ),
            )
            .when(!has_prompt && has_suggestions, |composer| {
                composer.child(suggestions)
            })
            .child(
                h_flex()
                    .h(px(32.))
                    .gap_2()
                    .child(
                        h_flex()
                            .id(SharedString::from(format!("{}-agent-attachment", self.id)))
                            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .size(px(28.))
                            .justify_center()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| style.border_color(cx.theme().ring))
                            .on_action(cx.listener(|_, _: &ActivateToolbarControl, _, cx| {
                                cx.emit(ToolbarAction::AgentAttachmentRequested);
                            }))
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarAction::AgentAttachmentRequested);
                            }))
                            .child(Icon::new(IconName::Plus).xsmall()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.agent_options.mention_hint.clone()),
                    )
                    .child(div().flex_1())
                    .child(
                        h_flex()
                            .id(SharedString::from(format!("{}-agent-send", self.id)))
                            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .size(px(30.))
                            .justify_center()
                            .rounded_full()
                            .cursor_pointer()
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
                            .focus(|style| style.border_2().border_color(cx.theme().ring))
                            .on_action(cx.listener(
                                |this, _: &ActivateToolbarControl, window, cx| {
                                    this.activate_agent_send(window, cx);
                                },
                            ))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.activate_agent_send(window, cx);
                            }))
                            .child(if has_prompt { "↑" } else { "◉" }),
                    ),
            );

        deferred(
            h_flex()
                .absolute()
                .left_0()
                .right_0()
                .bottom(px(OVERLAY_BOTTOM))
                .justify_center()
                .child(composer),
        )
        .with_priority(14)
        .into_any_element()
    }

    fn render_zoom_control(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .absolute()
            .left(px(18.))
            .bottom(px(DETACHED_CONTROL_BOTTOM))
            .h(px(36.))
            .p_1()
            .gap(px(1.))
            .rounded(px(11.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .text_color(cx.theme().popover_foreground)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .debug_selector(|| "toolbar-zoom-control".to_owned())
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-zoom-out", self.id)))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .size(px(28.))
                    .justify_center()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().accent))
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .on_action(cx.listener(|this, _: &ActivateToolbarControl, _, cx| {
                        this.request_zoom(this.zoom_percent.saturating_sub(10), cx);
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.request_zoom(this.zoom_percent.saturating_sub(10), cx);
                    }))
                    .child("−"),
            )
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-zoom-menu", self.id)))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .h(px(28.))
                    .px_2()
                    .gap_1()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .text_xs()
                    .hover(|style| style.bg(cx.theme().accent))
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .on_action(cx.listener(|this, _: &ActivateToolbarControl, _, cx| {
                        this.toggle_zoom(cx);
                    }))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_zoom(cx)))
                    .child(format!("{}%", self.zoom_percent))
                    .child(Icon::new(IconName::ChevronUp).xsmall()),
            )
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-zoom-in", self.id)))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .size(px(28.))
                    .justify_center()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().accent))
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .on_action(cx.listener(|this, _: &ActivateToolbarControl, _, cx| {
                        this.request_zoom(this.zoom_percent.saturating_add(10), cx);
                    }))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.request_zoom(this.zoom_percent.saturating_add(10), cx);
                    }))
                    .child("+"),
            )
            .into_any_element()
    }

    fn render_zoom_menu(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut menu = v_flex()
            .id(SharedString::from(format!("{}-zoom-flyout", self.id)))
            .absolute()
            .left(px(18.))
            .bottom(px(SECONDARY_TOOLBAR_BOTTOM))
            .w(px(164.))
            .p_2()
            .rounded(px(12.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .text_color(cx.theme().popover_foreground)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                this.dismiss_overlay_for_pointer(cx);
            }));
        for percent in [25_u16, 50, 75, 100, 125, 150, 200, 400] {
            menu = menu.child(
                h_flex()
                    .id(SharedString::from(format!("{}-zoom-{percent}", self.id)))
                    .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .h(px(30.))
                    .px_2()
                    .rounded(px(6.))
                    .cursor_pointer()
                    .text_xs()
                    .hover(|style| style.bg(cx.theme().accent))
                    .focus(|style| style.border_1().border_color(cx.theme().ring))
                    .on_action(cx.listener(move |this, _: &ActivateToolbarControl, _, cx| {
                        this.set_overlay(None, cx);
                        this.request_zoom(percent, cx);
                        cx.notify();
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_overlay(None, cx);
                        this.request_zoom(percent, cx);
                        cx.notify();
                    }))
                    .child(div().flex_1().child(format!("{percent}%")))
                    .when(self.zoom_percent == percent, |row| {
                        row.child(
                            Icon::new(IconName::Check)
                                .xsmall()
                                .text_color(cx.theme().primary),
                        )
                    }),
            );
        }
        deferred(menu).with_priority(11).into_any_element()
    }

    fn render_agent_launcher(&self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.overlay == Some(ToolbarOverlay::Agent);
        h_flex()
            .id(SharedString::from(format!("{}-agent-launcher", self.id)))
            .debug_selector(|| "toolbar-agent-launcher".to_owned())
            .key_context(TOOLBAR_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .absolute()
            .right(px(18.))
            .bottom(px(DETACHED_CONTROL_BOTTOM))
            .h(px(36.))
            .px_3()
            .gap_2()
            .rounded(px(11.))
            .border_1()
            .border_color(if open {
                cx.theme().magenta
            } else {
                cx.theme().border
            })
            .bg(cx.theme().popover)
            .text_color(cx.theme().magenta)
            .when(cx.theme().shadow, |surface| surface.shadow_lg())
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| style.border_2().border_color(cx.theme().ring))
            .on_action(cx.listener(|this, _: &ActivateToolbarControl, window, cx| {
                this.toggle_agent(window, cx);
            }))
            .on_click(cx.listener(|this, _, window, cx| {
                this.toggle_agent(window, cx);
            }))
            .child(div().font_semibold().child("✦"))
            .child(div().text_xs().font_semibold().child("Agent"))
            .child(
                div()
                    .px_1()
                    .rounded(px(4.))
                    .bg(cx.theme().magenta_light)
                    .text_size(px(9.))
                    .child("⌘↵"),
            )
            .into_any_element()
    }

    fn render_overlay(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.overlay {
            Some(ToolbarOverlay::ToolGroup(group)) => self.render_tool_group_menu(group, cx),
            Some(ToolbarOverlay::Actions) => self.render_actions_palette(cx),
            Some(ToolbarOverlay::Agent) => self.render_agent_composer(cx),
            Some(ToolbarOverlay::Zoom) => self.render_zoom_menu(cx),
            None => div().into_any_element(),
        }
    }
}

impl Focusable for EditorToolbar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorToolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let key_context = if matches!(
            self.overlay,
            Some(ToolbarOverlay::Actions | ToolbarOverlay::Agent)
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
            .size_full()
            .absolute()
            .left_0()
            .right_0()
            .top_0()
            .bottom_0()
            .on_action(cx.listener(|this, _: &CloseToolbarOverlay, window, cx| {
                this.dismiss_overlay(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenToolbarActions, window, cx| {
                this.open_actions(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenToolbarAgent, window, cx| {
                this.open_agent(window, cx);
            }))
            .on_action(cx.listener(|this, _: &NextToolbarCommand, _, cx| {
                this.move_command_cursor(1, cx);
            }))
            .on_action(cx.listener(|this, _: &PreviousToolbarCommand, _, cx| {
                this.move_command_cursor(-1, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectMoveTool, _, cx| {
                this.request_tool(ToolbarTool::Move, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectHandTool, _, cx| {
                this.request_tool(ToolbarTool::Hand, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectScaleTool, _, cx| {
                this.request_tool(ToolbarTool::Scale, cx);
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
                this.request_tool(ToolbarTool::Line, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectArrowTool, _, cx| {
                this.request_tool(ToolbarTool::Arrow, cx);
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
                this.request_tool(ToolbarTool::Comment, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectAnnotationTool, _, cx| {
                this.request_tool(ToolbarTool::Annotation, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectMeasureTool, _, cx| {
                this.request_tool(ToolbarTool::Measure, cx);
            }))
            .on_action(cx.listener(|this, _: &EnterDevMode, _, cx| {
                this.request_mode(ToolbarMode::Dev, cx);
            }))
            .child(self.render_main_toolbar(cx))
            .child(self.render_zoom_control(cx))
            .child(self.render_agent_launcher(cx))
            .child(self.render_secondary_toolbar(cx))
            .child(self.render_overlay(cx))
    }
}

#[cfg(test)]
mod interaction_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_scopes_partition_resource_entries() {
        assert!(CommandScope::All.includes(ToolbarCommand::Undo));
        assert!(CommandScope::Assets.includes(ToolbarCommand::OpenResources));
        assert!(CommandScope::Assets.includes(ToolbarCommand::OpenVariables));
        assert!(!CommandScope::Assets.includes(ToolbarCommand::OpenPlugins));
        assert!(CommandScope::PluginsAndWidgets.includes(ToolbarCommand::OpenPlugins));
        assert!(CommandScope::PluginsAndWidgets.includes(ToolbarCommand::OpenWidgets));
    }

    #[test]
    fn controlled_draw_colors_accept_rgb_and_rgba_hex() {
        assert_eq!(
            EditorToolbar::parse_color("#0D99FF"),
            Some(rgba(0x0d99ffff))
        );
        assert_eq!(
            EditorToolbar::parse_color("0D99FF80"),
            Some(rgba(0x0d99ff80))
        );
        assert_eq!(EditorToolbar::parse_color("not-a-color"), None);
    }
}
