//! Controlled-state setters, the overlay state machine with focus
//! restoration, Actions command filtering/invocation, and value/styling
//! helpers for [`EditorToolbar`].

use gpui::{App, Context, Pixels, SharedString, Window, px, rgba};
use gpui_component::ActiveTheme as _;

use super::{CommandScope, EditorToolbar, ToolbarOverlay, ZoomMenuEntry};
use crate::color::parse_hex_rgba;
use crate::toolbar::{
    AgentToolbarOptions, DevToolbarOptions, DrawToolbarOptions, MotionToolbarOptions,
    ToolbarAction, ToolbarCommand, ToolbarControlValue, ToolbarMode, ToolbarSecondaryControl,
    ToolbarTool, ToolbarToolGroup,
};

/// Figma's multiplicative zoom ladder: the doubling 25→50→100→200→400 spine
/// extended to the 1–3,200 percent clamp. The +/- steppers and the zoom menu's
/// in/out entries snap to the nearest ladder step in their direction.
const ZOOM_LADDER: &[u16] = &[1, 2, 3, 6, 12, 25, 50, 100, 200, 400, 800, 1_600, 3_200];

/// One ranked Actions result: the command plus the label byte ranges its
/// match highlights.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CommandMatch {
    pub(super) command: ToolbarCommand,
    pub(super) highlights: Vec<std::ops::Range<usize>>,
}

/// Matches every query character (spaces skipped) against word-prefix runs of
/// an ASCII label, greedily taking the longest prefix per word.
fn word_boundary_match(label: &str, query: &str) -> Option<Vec<std::ops::Range<usize>>> {
    let bytes = label.as_bytes();
    let condensed: Vec<u8> = query.bytes().filter(|byte| *byte != b' ').collect();
    if condensed.is_empty() {
        return None;
    }
    let mut remaining = condensed.as_slice();
    let mut highlights = Vec::new();
    let mut index = 0;
    while index < bytes.len() && !remaining.is_empty() {
        let word_start = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
        if word_start && bytes[index].is_ascii_alphanumeric() {
            let mut len = 0;
            while index + len < bytes.len()
                && len < remaining.len()
                && bytes[index + len].is_ascii_alphanumeric()
                && bytes[index + len] == remaining[len]
            {
                len += 1;
            }
            if len > 0 {
                highlights.push(index..index + len);
                remaining = &remaining[len..];
                index += len;
                continue;
            }
        }
        index += 1;
    }
    remaining.is_empty().then_some(highlights)
}

/// Matches the query as a scattered in-order subsequence of an ASCII label
/// (spaces skipped), merging adjacent hits into ranges.
fn subsequence_match(label: &str, query: &str) -> Option<Vec<std::ops::Range<usize>>> {
    let bytes = label.as_bytes();
    let mut highlights: Vec<std::ops::Range<usize>> = Vec::new();
    let mut index = 0;
    let mut matched_any = false;
    for query_byte in query.bytes() {
        if query_byte == b' ' {
            continue;
        }
        loop {
            if index >= bytes.len() {
                return None;
            }
            if bytes[index] == query_byte {
                match highlights.last_mut() {
                    Some(last) if last.end == index => last.end = index + 1,
                    _ => highlights.push(index..index + 1),
                }
                matched_any = true;
                index += 1;
                break;
            }
            index += 1;
        }
    }
    matched_any.then_some(highlights)
}

impl EditorToolbar {
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
        self.restore_toolbar_focus(window, cx);
        cx.notify();
        true
    }

    fn restore_toolbar_focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        let focus_handle = self.focus_handle.clone();
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
    }

    // A popup's capture-phase outside handler may have already removed it by
    // the time its own trigger handles the same pointer event. Closing from a
    // trigger must still restore focus even when the overlay is already gone.
    fn close_overlay_from_trigger(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_overlay(None, cx);
        self.restore_toolbar_focus(window, cx);
        cx.notify();
    }

    pub(super) fn dismiss_overlay_for_pointer(&mut self, cx: &mut Context<Self>) {
        if self.overlay.is_some() {
            self.set_overlay(None, cx);
            cx.notify();
        }
    }

    pub(super) fn toggle_tool_group(
        &mut self,
        group: ToolbarToolGroup,
        was_open: bool,
        cx: &mut Context<Self>,
    ) {
        let next = (!was_open).then_some(ToolbarOverlay::ToolGroup(group));
        if !was_open {
            self.reset_menu_cursor(
                group
                    .tools()
                    .iter()
                    .position(|tool| *tool == self.active_tool)
                    .unwrap_or(0),
            );
        }
        self.set_overlay(next, cx);
        cx.notify();
    }

    pub(super) fn toggle_zoom(&mut self, was_open: bool, cx: &mut Context<Self>) {
        let next = (!was_open).then_some(ToolbarOverlay::Zoom);
        if !was_open {
            self.reset_menu_cursor(0);
        }
        self.set_overlay(next, cx);
        cx.notify();
    }

    /// Opens or closes the anchored editor for one secondary chip. Choice
    /// editors start highlighted on the accepted current value.
    pub(super) fn toggle_option_editor(
        &mut self,
        control: ToolbarSecondaryControl,
        was_open: bool,
        cx: &mut Context<Self>,
    ) {
        let next = (!was_open).then_some(ToolbarOverlay::OptionEditor(control));
        if !was_open {
            let cursor = self
                .current_choice(control)
                .and_then(|current| {
                    self.choice_candidates(control)
                        .iter()
                        .position(|candidate| *candidate == current)
                })
                .unwrap_or(0);
            self.reset_menu_cursor(cursor);
        }
        self.set_overlay(next, cx);
        cx.notify();
    }

    fn reset_menu_cursor(&mut self, cursor: usize) {
        self.menu_cursor = cursor;
        self.menu_scroll_handle.scroll_to_item(cursor);
    }

    pub(super) fn request_zoom(&mut self, percent: u16, cx: &mut Context<Self>) {
        cx.emit(ToolbarAction::ZoomChangeRequested {
            percent: percent.clamp(1, 3_200),
        });
    }

    pub(super) fn request_zoom_command(&mut self, command: ToolbarCommand, cx: &mut Context<Self>) {
        cx.emit(ToolbarAction::CommandInvoked { command });
    }

    /// Nearest ladder step in the requested direction; clamps at the ladder
    /// ends so repeated stepping settles on 1 and 3,200 percent.
    pub(super) fn zoom_ladder_step(current: u16, zoom_in: bool) -> u16 {
        if zoom_in {
            ZOOM_LADDER
                .iter()
                .copied()
                .find(|step| *step > current)
                .unwrap_or(3_200)
        } else {
            ZOOM_LADDER
                .iter()
                .rev()
                .copied()
                .find(|step| *step < current)
                .unwrap_or(1)
        }
    }

    pub(super) fn toggle_agent(
        &mut self,
        was_open: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if was_open {
            self.close_overlay_from_trigger(window, cx);
        } else {
            self.open_agent(window, cx);
        }
    }

    pub(super) fn request_mode(&mut self, mode: ToolbarMode, cx: &mut Context<Self>) {
        self.set_overlay(None, cx);
        cx.emit(ToolbarAction::ModeChangeRequested { mode });
        cx.notify();
    }

    pub(super) fn request_tool(&mut self, tool: ToolbarTool, cx: &mut Context<Self>) {
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

    pub(super) fn choose_tool_from_menu(
        &mut self,
        tool: ToolbarTool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_tool(tool, cx);
        let focus_handle = self.focus_handle.clone();
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
    }

    /// Commits one zoom-menu entry: the menu closes, toolbar focus returns,
    /// and the entry's zoom command or typed percent intent is emitted.
    pub(super) fn choose_zoom_entry(
        &mut self,
        entry: ZoomMenuEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_overlay(None, cx);
        let focus_handle = self.focus_handle.clone();
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
        match entry {
            ZoomMenuEntry::ZoomIn => {
                self.request_zoom(Self::zoom_ladder_step(self.zoom_percent, true), cx)
            }
            ZoomMenuEntry::ZoomOut => {
                self.request_zoom(Self::zoom_ladder_step(self.zoom_percent, false), cx)
            }
            ZoomMenuEntry::ZoomToFit => self.request_zoom_command(ToolbarCommand::ZoomToFit, cx),
            ZoomMenuEntry::ZoomToSelection => {
                self.request_zoom_command(ToolbarCommand::ZoomToSelection, cx)
            }
            ZoomMenuEntry::ZoomTo100 => self.request_zoom(100, cx),
            ZoomMenuEntry::ZoomTo50 => self.request_zoom(50, cx),
        }
        cx.notify();
    }

    /// Commits one host-supplied candidate from a chip's choice editor.
    pub(super) fn choose_option_candidate(
        &mut self,
        control: ToolbarSecondaryControl,
        candidate: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = match control {
            ToolbarSecondaryControl::DrawStrokeColor => ToolbarControlValue::Color(candidate),
            _ => ToolbarControlValue::Choice(candidate),
        };
        self.set_overlay(None, cx);
        self.request_control_value(control, value, cx);
        let focus_handle = self.focus_handle.clone();
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
        cx.notify();
    }

    /// Host-supplied candidates behind a chip's choice editor; empty for the
    /// slider-driven controls.
    pub(super) fn choice_candidates(&self, control: ToolbarSecondaryControl) -> &[SharedString] {
        match control {
            ToolbarSecondaryControl::DrawStrokeColor => &self.draw_options.available_colors,
            ToolbarSecondaryControl::DrawBrushStyle => &self.draw_options.available_styles,
            ToolbarSecondaryControl::MotionAnimationStyle => {
                &self.motion_options.available_animation_styles
            }
            _ => &[],
        }
    }

    fn current_choice(&self, control: ToolbarSecondaryControl) -> Option<SharedString> {
        match control {
            ToolbarSecondaryControl::DrawStrokeColor => {
                Some(self.draw_options.stroke_color.clone())
            }
            ToolbarSecondaryControl::DrawBrushStyle => Some(self.draw_options.brush_style.clone()),
            ToolbarSecondaryControl::MotionAnimationStyle => {
                Some(self.motion_options.animation_style.clone())
            }
            _ => None,
        }
    }

    /// `(value, min, max, step)` for a slider-driven control, normalized so an
    /// inverted host range cannot panic the clamp.
    pub(super) fn slider_params(
        &self,
        control: ToolbarSecondaryControl,
    ) -> Option<(i32, i32, i32, i32)> {
        let (value, min, max, step) = match control {
            ToolbarSecondaryControl::DrawStrokeWeight => (
                i32::from(self.draw_options.stroke_weight),
                i32::from(self.draw_options.weight_min),
                i32::from(self.draw_options.weight_max),
                i32::from(self.draw_options.weight_step),
            ),
            ToolbarSecondaryControl::DrawSmoothing => (
                i32::from(self.draw_options.smoothing),
                i32::from(self.draw_options.smoothing_min),
                i32::from(self.draw_options.smoothing_max),
                i32::from(self.draw_options.smoothing_step),
            ),
            _ => return None,
        };
        let (min, max) = (min.min(max), min.max(max));
        Some((value.clamp(min, max), min, max, step.max(1)))
    }

    /// Steps an open slider editor by one host-supplied increment, or moves
    /// the highlight of an open choice editor.
    pub(super) fn adjust_option_control(&mut self, direction: i32, cx: &mut Context<Self>) {
        let Some(ToolbarOverlay::OptionEditor(control)) = self.overlay else {
            return;
        };
        if let Some((value, min, max, step)) = self.slider_params(control) {
            let next = (value + direction * step).clamp(min, max);
            if next != value {
                self.request_control_value(control, ToolbarControlValue::Integer(next), cx);
            }
        } else {
            self.move_command_cursor(direction as isize, cx);
        }
    }

    /// Maps a pointer position on a slider's tracked bounds to the nearest
    /// host-supplied step and requests that value.
    pub(super) fn set_slider_from_pointer(
        &mut self,
        control: ToolbarSecondaryControl,
        position_x: Pixels,
        cx: &mut Context<Self>,
    ) {
        let Some((_, min, max, step)) = self.slider_params(control) else {
            return;
        };
        let bounds = self.slider_track_bounds;
        if bounds.size.width <= px(0.) {
            return;
        }
        let fraction = ((position_x - bounds.left()) / bounds.size.width).clamp(0., 1.);
        let raw = min as f32 + fraction * (max - min) as f32;
        let stepped = min + ((raw - min as f32) / step as f32).round() as i32 * step;
        self.request_control_value(
            control,
            ToolbarControlValue::Integer(stepped.clamp(min, max)),
            cx,
        );
    }

    /// Number of highlightable rows in the open non-Actions menu overlay.
    fn open_menu_len(&self) -> Option<usize> {
        match self.overlay? {
            ToolbarOverlay::ToolGroup(group) => Some(group.tools().len()),
            ToolbarOverlay::Zoom => Some(ZoomMenuEntry::ALL.len()),
            ToolbarOverlay::OptionEditor(control) => {
                let len = self.choice_candidates(control).len();
                (len > 0).then_some(len)
            }
            ToolbarOverlay::Actions | ToolbarOverlay::Agent => None,
        }
    }

    /// Activates the row highlighted in the open menu overlay. Returns false
    /// when no highlightable menu is open so trigger activation can fall back
    /// to its toggle behavior.
    pub(super) fn commit_open_menu_entry(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(len) = self.open_menu_len() else {
            return false;
        };
        let cursor = self.menu_cursor.min(len - 1);
        match self.overlay {
            Some(ToolbarOverlay::ToolGroup(group)) => {
                self.choose_tool_from_menu(group.tools()[cursor], window, cx);
            }
            Some(ToolbarOverlay::Zoom) => {
                self.choose_zoom_entry(ZoomMenuEntry::ALL[cursor], window, cx);
            }
            Some(ToolbarOverlay::OptionEditor(control)) => {
                let candidate = self.choice_candidates(control)[cursor].clone();
                self.choose_option_candidate(control, candidate, window, cx);
            }
            _ => return false,
        }
        true
    }

    pub(super) fn activate_tool_or_actions(
        &mut self,
        tool: ToolbarTool,
        actions_was_open: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if tool == ToolbarTool::Actions {
            if actions_was_open {
                self.close_overlay_from_trigger(window, cx);
            } else {
                self.open_actions(window, cx);
            }
        } else {
            self.request_tool(tool, cx);
        }
    }

    /// Filtered, ranked Actions results: matches are ordered label prefix >
    /// word-boundary subsequence > contiguous substring > scattered
    /// subsequence, with the legacy category/description substring as the
    /// final fallback tier. Host command order is preserved within a tier.
    pub(super) fn filtered_commands(&self, cx: &App) -> Vec<CommandMatch> {
        let query = self.command_input.read(cx).value().to_lowercase();
        let mut ranked: Vec<(u8, CommandMatch)> = self
            .commands
            .iter()
            .copied()
            .filter(|command| self.command_scope.includes(*command))
            .filter_map(|command| {
                if query.is_empty() {
                    return Some((
                        0,
                        CommandMatch {
                            command,
                            highlights: Vec::new(),
                        },
                    ));
                }
                Self::match_command(command, &query).map(|(tier, highlights)| {
                    (
                        tier,
                        CommandMatch {
                            command,
                            highlights,
                        },
                    )
                })
            })
            .collect();
        ranked.sort_by_key(|(tier, _)| *tier);
        ranked.into_iter().map(|(_, entry)| entry).collect()
    }

    /// Ranks one command against a lowercase query. Returns the match tier
    /// plus the label byte ranges to highlight; `None` when nothing matches.
    fn match_command(
        command: ToolbarCommand,
        query: &str,
    ) -> Option<(u8, Vec<std::ops::Range<usize>>)> {
        let label_lower = command.label().to_lowercase();
        if label_lower.is_ascii() && query.is_ascii() {
            if label_lower.starts_with(query) {
                return Some((0, std::iter::once(0..query.len()).collect()));
            }
            if let Some(highlights) = word_boundary_match(&label_lower, query) {
                return Some((1, highlights));
            }
            if let Some(position) = label_lower.find(query) {
                return Some((
                    2,
                    std::iter::once(position..position + query.len()).collect(),
                ));
            }
            if let Some(highlights) = subsequence_match(&label_lower, query) {
                return Some((3, highlights));
            }
        } else if label_lower.contains(query) {
            return Some((2, Vec::new()));
        }
        (command.category().to_lowercase().contains(query)
            || command.description().to_lowercase().contains(query))
        .then(|| (4, Vec::new()))
    }

    pub(super) fn invoke_highlighted_command(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let commands = self.filtered_commands(cx);
        let Some(command) = commands
            .get(self.command_cursor.min(commands.len().saturating_sub(1)))
            .map(|entry| entry.command)
        else {
            return;
        };
        self.invoke_command(command, window, cx);
    }

    pub(super) fn move_command_cursor(&mut self, direction: isize, cx: &mut Context<Self>) {
        if self.overlay == Some(ToolbarOverlay::Actions) {
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
            return;
        }
        let Some(len) = self.open_menu_len() else {
            return;
        };
        let cursor = self.menu_cursor.min(len - 1);
        self.menu_cursor = if direction < 0 {
            cursor.checked_sub(1).unwrap_or(len - 1)
        } else {
            (cursor + 1) % len
        };
        self.menu_scroll_handle.scroll_to_item(self.menu_cursor);
        cx.notify();
    }

    /// Home/End inside a menu overlay: jump the highlight to the first or
    /// last row and keep it scrolled into view.
    pub(super) fn move_cursor_to_boundary(&mut self, end: bool, cx: &mut Context<Self>) {
        let Some(len) = self.open_menu_len() else {
            return;
        };
        self.reset_menu_cursor(if end { len - 1 } else { 0 });
        cx.notify();
    }

    pub(super) fn set_command_scope(
        &mut self,
        scope: CommandScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_scope = scope;
        self.command_cursor = 0;
        self.command_scroll_handle.scroll_to_item(0);
        let command_input = self.command_input.clone();
        window.defer(cx, move |window, cx| {
            command_input.update(cx, |input, cx| input.focus(window, cx));
        });
        cx.notify();
    }

    pub(super) fn invoke_command(
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

    pub(super) fn submit_ai_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub(super) fn set_ai_suggestion(
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

    pub(super) fn activate_agent_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.ai_input.read(cx).value().trim().is_empty() {
            cx.emit(ToolbarAction::AgentVoiceInputRequested);
        } else {
            self.submit_ai_prompt(window, cx);
        }
    }

    pub(super) fn request_secondary(
        &mut self,
        control: ToolbarSecondaryControl,
        cx: &mut Context<Self>,
    ) {
        cx.emit(ToolbarAction::SecondaryControlInvoked {
            mode: self.mode,
            control,
        });
    }

    pub(super) fn request_control_value(
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

    pub(super) fn control_tooltip(
        label: &'static str,
        shortcut: Option<&'static str>,
    ) -> SharedString {
        match shortcut {
            Some(shortcut) => format!("{label}  {shortcut}").into(),
            None => label.into(),
        }
    }

    pub(super) fn mode_accent(mode: ToolbarMode, cx: &App) -> gpui::Hsla {
        match mode {
            ToolbarMode::Draw => cx.theme().cyan,
            ToolbarMode::Design => cx.theme().blue,
            ToolbarMode::Motion => cx.theme().magenta,
            ToolbarMode::Dev => cx.theme().green,
        }
    }

    pub(super) fn mode_accent_pale(mode: ToolbarMode, cx: &App) -> gpui::Hsla {
        match mode {
            ToolbarMode::Draw => cx.theme().cyan_light,
            ToolbarMode::Design => cx.theme().blue_light,
            ToolbarMode::Motion => cx.theme().magenta_light,
            ToolbarMode::Dev => cx.theme().green_light,
        }
    }

    pub(super) fn mode_accent_foreground(accent: gpui::Hsla, cx: &App) -> gpui::Hsla {
        let foreground = cx.theme().foreground;
        let background = cx.theme().background;
        if (accent.l - foreground.l).abs() >= (accent.l - background.l).abs() {
            foreground
        } else {
            background
        }
    }

    pub(super) fn parse_color(value: &str) -> Option<gpui::Rgba> {
        parse_hex_rgba(value).map(rgba)
    }
}

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
    fn actions_matching_ranks_prefix_word_boundary_substring_then_scattered() {
        assert_eq!(
            EditorToolbar::match_command(ToolbarCommand::ZoomToFit, "zoom"),
            Some((0, std::iter::once(0..4).collect()))
        );
        assert_eq!(
            EditorToolbar::match_command(ToolbarCommand::ZoomToFit, "zf"),
            Some((1, vec![0..1, 8..9]))
        );
        assert_eq!(
            EditorToolbar::match_command(ToolbarCommand::ZoomToFit, "om to"),
            Some((2, std::iter::once(2..7).collect()))
        );
        assert_eq!(
            EditorToolbar::match_command(ToolbarCommand::ZoomToFit, "zmf"),
            Some((3, vec![0..1, 3..4, 8..9]))
        );
        // The legacy category/description substring survives as the fallback
        // tier without label highlights.
        assert_eq!(
            EditorToolbar::match_command(ToolbarCommand::GenerateDesign, "figma agent"),
            Some((4, Vec::new()))
        );
        assert_eq!(
            EditorToolbar::match_command(ToolbarCommand::Undo, "xyz"),
            None
        );
    }

    #[test]
    fn zoom_ladder_steps_multiplicatively_and_clamps_at_the_ends() {
        assert_eq!(EditorToolbar::zoom_ladder_step(100, true), 200);
        assert_eq!(EditorToolbar::zoom_ladder_step(100, false), 50);
        assert_eq!(EditorToolbar::zoom_ladder_step(110, true), 200);
        assert_eq!(EditorToolbar::zoom_ladder_step(110, false), 100);
        assert_eq!(EditorToolbar::zoom_ladder_step(3_200, true), 3_200);
        assert_eq!(EditorToolbar::zoom_ladder_step(1, false), 1);
        assert_eq!(EditorToolbar::zoom_ladder_step(26, true), 50);
        assert_eq!(EditorToolbar::zoom_ladder_step(26, false), 25);
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
