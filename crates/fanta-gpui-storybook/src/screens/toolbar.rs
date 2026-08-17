//! The Toolbar story: mock host options, env-seeded knob defaults,
//! reducer, demo canvas, and the mode/overlay/zoom knobs.
//!
//! `FANTA_TOOLBAR_MODE` and `FANTA_TOOLBAR_OVERLAY` stay supported as the
//! initial values of the runtime knobs, preserving the screenshot-CI
//! launch interface.

use crate::*;

use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolbarOverlay {
    None,
    Actions,
    Agent,
}

impl ToolbarOverlay {
    pub(crate) const ALL: [Self; 3] = [Self::None, Self::Actions, Self::Agent];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Actions => "Actions",
            Self::Agent => "Agent",
        }
    }
}

/// Story width below which the mock inspector column hides so the canvas
/// keeps hosting the dock on narrow fluid viewports.
const STORY_INSPECTOR_MIN_WIDTH: f32 = 1020.;
/// Story width below which the mock file sidebar hides as well, leaving the
/// canvas as the whole story.
const STORY_SIDEBAR_MIN_WIDTH: f32 = 760.;

/// The launch-mode order is also the knob's chip order.
pub(crate) const TOOLBAR_MODE_KNOB_ORDER: [ToolbarMode; 3] =
    [ToolbarMode::Design, ToolbarMode::Motion, ToolbarMode::Dev];

pub(crate) fn parse_toolbar_mode(value: Option<&str>) -> ToolbarMode {
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "motion" => ToolbarMode::Motion,
        "dev" => ToolbarMode::Dev,
        _ => ToolbarMode::Design,
    }
}

/// Ids of the three example host chrome controls the story seeds into the
/// dock's trailing capsule (§12: the host owns chrome, the toolbar owns the
/// dock).
pub(crate) const TOOLBAR_CHROME_FIT: &str = "fit-to-view";
pub(crate) const TOOLBAR_CHROME_LEFT: &str = "toggle-left-sidebar";
pub(crate) const TOOLBAR_CHROME_RIGHT: &str = "toggle-right-sidebar";

/// The mock host's chrome read model: fit-to-view plus the two sidebar
/// toggles, whose active state mirrors the story's mock columns.
pub(crate) fn toolbar_chrome_controls(
    left_visible: bool,
    right_visible: bool,
) -> Vec<ToolbarChromeControl> {
    vec![
        ToolbarChromeControl::new(TOOLBAR_CHROME_FIT, IconName::Maximize, "Fit to view")
            .shortcut("⇧ 1"),
        ToolbarChromeControl::new(
            TOOLBAR_CHROME_LEFT,
            if left_visible {
                IconName::PanelLeftClose
            } else {
                IconName::PanelLeftOpen
            },
            if left_visible {
                "Hide layers sidebar"
            } else {
                "Show layers sidebar"
            },
        )
        .active(left_visible),
        ToolbarChromeControl::new(
            TOOLBAR_CHROME_RIGHT,
            if right_visible {
                IconName::PanelRightClose
            } else {
                IconName::PanelRightOpen
            },
            if right_visible {
                "Hide inspector sidebar"
            } else {
                "Show inspector sidebar"
            },
        )
        .active(right_visible),
    ]
}

pub(crate) fn parse_toolbar_overlay(value: Option<&str>) -> ToolbarOverlay {
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "actions" => ToolbarOverlay::Actions,
        "agent" => ToolbarOverlay::Agent,
        _ => ToolbarOverlay::None,
    }
}

pub(crate) struct ToolbarScreen {
    pub(crate) toolbar: Entity<EditorToolbar>,
    pub(crate) mode: ToolbarMode,
    pub(crate) tool: ToolbarTool,
    pub(crate) zoom: u16,
    pub(crate) dev_options: DevToolbarOptions,
    pub(crate) motion_options: MotionToolbarOptions,
    pub(crate) overlay: ToolbarOverlay,
    /// Mock host chrome state echoed into the dock's trailing capsule and
    /// mirrored by the story's mock sidebar / inspector columns.
    pub(crate) left_sidebar_visible: bool,
    pub(crate) right_sidebar_visible: bool,
    pub(crate) last_action: SharedString,
    /// Measured width of the story surface; the mock chrome columns hide
    /// below the named story breakpoints so the dock keeps its canvas host.
    pub(crate) story_width: Option<f32>,
}

impl ToolbarScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let mode = parse_toolbar_mode(std::env::var("FANTA_TOOLBAR_MODE").ok().as_deref());
        let tool = match mode {
            ToolbarMode::Design => ToolbarTool::Move,
            ToolbarMode::Motion => ToolbarTool::MotionSelect,
            ToolbarMode::Dev => ToolbarTool::Inspect,
        };
        let zoom = 100;
        // The mock host seeds every candidate the Motion option editor may
        // offer; the toolbar only presents these host values.
        let dev_options = DevToolbarOptions::default();
        let motion_options = MotionToolbarOptions {
            available_animation_styles: ["Fade in", "Spring", "Slide up", "Pop"]
                .into_iter()
                .map(Into::into)
                .collect(),
            ..MotionToolbarOptions::default()
        };
        let toolbar =
            cx.new(|cx| EditorToolbar::new("storybook-toolbar", mode, tool, zoom, window, cx));
        toolbar.update(cx, |toolbar, cx| {
            toolbar.set_agent_options(
                AgentToolbarOptions::new("Hero frame").suggestions([
                    "Explore 3 directions",
                    "Polish this screen",
                    "Animate the hero",
                ]),
                cx,
            );
            toolbar.set_motion_options(motion_options.clone(), cx);
            toolbar.set_dev_options(dev_options, cx);
            toolbar.set_chrome_controls(toolbar_chrome_controls(true, true), cx);
        });
        toolbar.focus_handle(cx).focus(window, cx);
        let overlay = parse_toolbar_overlay(std::env::var("FANTA_TOOLBAR_OVERLAY").ok().as_deref());
        match overlay {
            ToolbarOverlay::Actions => toolbar.update(cx, |toolbar, cx| {
                toolbar.open_actions(window, cx);
            }),
            ToolbarOverlay::Agent => toolbar.update(cx, |toolbar, cx| {
                toolbar.open_agent(window, cx);
            }),
            ToolbarOverlay::None => {}
        }
        Self {
            toolbar,
            mode,
            tool,
            zoom,
            dev_options,
            motion_options,
            overlay,
            left_sidebar_visible: true,
            right_sidebar_visible: true,
            last_action: "Ready — open every split tool, Actions, Agent, zoom, the chrome capsule, and all three modes"
                .into(),
            story_width: None,
        }
    }
}

impl Storybook {
    pub(crate) fn handle_toolbar_action(
        &mut self,
        toolbar: Entity<EditorToolbar>,
        action: &ToolbarAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            ToolbarAction::ModeChangeRequested { mode } => {
                self.toolbar_screen.mode = *mode;
                self.toolbar_screen.tool = match mode {
                    ToolbarMode::Design => ToolbarTool::Move,
                    ToolbarMode::Motion => ToolbarTool::MotionSelect,
                    ToolbarMode::Dev => ToolbarTool::Inspect,
                };
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_mode(*mode, cx);
                    toolbar.set_active_tool(self.toolbar_screen.tool, cx);
                });
                self.toolbar_screen.last_action =
                    format!("Host switched to {} mode", mode.label()).into();
            }
            ToolbarAction::ToolChangeRequested { mode, tool } => {
                self.toolbar_screen.mode = *mode;
                self.toolbar_screen.tool = *tool;
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_mode(*mode, cx);
                    toolbar.set_active_tool(*tool, cx);
                });
                self.toolbar_screen.last_action =
                    format!("Host selected {} in {}", tool.label(), mode.label()).into();
            }
            ToolbarAction::SecondaryControlInvoked { mode, control } => {
                let tool = match control {
                    ToolbarSecondaryControl::DevInspect => Some(ToolbarTool::Inspect),
                    ToolbarSecondaryControl::DevAnnotate => Some(ToolbarTool::Annotation),
                    ToolbarSecondaryControl::DevMeasure => Some(ToolbarTool::Measure),
                    ToolbarSecondaryControl::MotionAddKeyframe => {
                        self.toolbar_screen.last_action = "Host inserted a keyframe at 0.0s".into();
                        None
                    }
                    _ => None,
                };
                if let Some(tool) = tool {
                    self.toolbar_screen.tool = tool;
                    toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_active_tool(tool, cx);
                    });
                }
                if !matches!(control, ToolbarSecondaryControl::MotionAddKeyframe) {
                    self.toolbar_screen.last_action =
                        format!("Host invoked {} in {}", control.label(), mode.label()).into();
                }
            }
            ToolbarAction::ControlChangeRequested {
                mode,
                control,
                value,
            } => {
                match (control, value) {
                    (
                        ToolbarSecondaryControl::DevReadyForDevelopment,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_screen.dev_options.ready_for_development = *value,
                    (
                        ToolbarSecondaryControl::MotionPlayPause,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_screen.motion_options.playing = *value,
                    (ToolbarSecondaryControl::MotionLoop, ToolbarControlValue::Toggle(value)) => {
                        self.toolbar_screen.motion_options.looping = *value
                    }
                    (
                        ToolbarSecondaryControl::MotionAutoKeyframe,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_screen.motion_options.auto_keyframe = *value,
                    (
                        ToolbarSecondaryControl::MotionAnimationStyle,
                        ToolbarControlValue::Choice(value),
                    ) => self.toolbar_screen.motion_options.animation_style = value.clone(),
                    _ => {}
                }
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_dev_options(self.toolbar_screen.dev_options, cx);
                    toolbar.set_motion_options(self.toolbar_screen.motion_options.clone(), cx);
                });
                self.toolbar_screen.last_action = format!(
                    "Host applied {} = {value:?} in {}",
                    control.label(),
                    mode.label()
                )
                .into();
            }
            ToolbarAction::CommandQueryChanged { query } => {
                self.toolbar_screen.last_action = if query.is_empty() {
                    "Actions search cleared".into()
                } else {
                    format!("Actions query: “{query}”").into()
                };
            }
            ToolbarAction::CommandInvoked { command } => {
                let requested_mode = match command {
                    ToolbarCommand::OpenDesignMode => Some(ToolbarMode::Design),
                    ToolbarCommand::OpenMotionMode => Some(ToolbarMode::Motion),
                    ToolbarCommand::OpenDevMode => Some(ToolbarMode::Dev),
                    _ => None,
                };
                if let Some(mode) = requested_mode {
                    self.toolbar_screen.mode = mode;
                    self.toolbar_screen.tool = match mode {
                        ToolbarMode::Design => ToolbarTool::Move,
                        ToolbarMode::Motion => ToolbarTool::MotionSelect,
                        ToolbarMode::Dev => ToolbarTool::Inspect,
                    };
                    toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_mode(mode, cx);
                        toolbar.set_active_tool(self.toolbar_screen.tool, cx);
                    });
                }
                self.toolbar_screen.last_action =
                    format!("Host ran {} · {}", command.category(), command.label()).into();
            }
            ToolbarAction::AiPromptSubmitted { prompt } => {
                self.toolbar_screen.last_action =
                    format!("Agent task started: “{prompt}” · working in parallel").into();
            }
            ToolbarAction::AgentVisibilityChanged { visible } => {
                // Keep the overlay knob in sync when the toolbar itself
                // opens or dismisses the Agent composer.
                self.toolbar_screen.overlay = if *visible {
                    ToolbarOverlay::Agent
                } else {
                    ToolbarOverlay::None
                };
                self.toolbar_screen.last_action = if *visible {
                    "Agent composer opened with Hero frame context".into()
                } else {
                    "Agent composer closed".into()
                };
            }
            ToolbarAction::AgentAttachmentRequested => {
                self.toolbar_screen.last_action =
                    "Host opened the Agent context and attachment picker".into();
            }
            ToolbarAction::AgentVoiceInputRequested => {
                self.toolbar_screen.last_action = "Host started Agent voice input".into();
            }
            ToolbarAction::ZoomChangeRequested { percent } => {
                self.toolbar_screen.zoom = *percent;
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_zoom_percent(*percent, cx);
                });
                self.toolbar_screen.last_action =
                    format!("Host set canvas zoom to {percent}%").into();
            }
            // Host chrome: the toolbar reports the press; this mock host owns
            // the effect and echoes the new active state back into the dock.
            ToolbarAction::ChromeControlInvoked { id } => {
                match id.as_ref() {
                    TOOLBAR_CHROME_FIT => {
                        self.toolbar_screen.zoom = 100;
                        toolbar.update(cx, |toolbar, cx| toolbar.set_zoom_percent(100, cx));
                        self.toolbar_screen.last_action =
                            "Host fit the canvas to the Hero frame".into();
                    }
                    TOOLBAR_CHROME_LEFT => {
                        self.toolbar_screen.left_sidebar_visible =
                            !self.toolbar_screen.left_sidebar_visible;
                        self.toolbar_screen.last_action =
                            if self.toolbar_screen.left_sidebar_visible {
                                "Host showed the layers sidebar".into()
                            } else {
                                "Host hid the layers sidebar".into()
                            };
                    }
                    TOOLBAR_CHROME_RIGHT => {
                        self.toolbar_screen.right_sidebar_visible =
                            !self.toolbar_screen.right_sidebar_visible;
                        self.toolbar_screen.last_action =
                            if self.toolbar_screen.right_sidebar_visible {
                                "Host showed the inspector sidebar".into()
                            } else {
                                "Host hid the inspector sidebar".into()
                            };
                    }
                    other => {
                        self.toolbar_screen.last_action =
                            format!("Host received unknown chrome control “{other}”").into();
                    }
                }
                let controls = toolbar_chrome_controls(
                    self.toolbar_screen.left_sidebar_visible,
                    self.toolbar_screen.right_sidebar_visible,
                );
                toolbar.update(cx, |toolbar, cx| toolbar.set_chrome_controls(controls, cx));
            }
        }
        cx.notify();
    }

    /// The overlay knob has no component intent to synthesize, so it
    /// drives the toolbar entity's overlay surface directly.
    pub(crate) fn apply_toolbar_overlay(
        &mut self,
        overlay: ToolbarOverlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toolbar_screen.overlay = overlay;
        self.toolbar_screen.toolbar.update(cx, |toolbar, cx| {
            toolbar.dismiss_overlay(window, cx);
            match overlay {
                ToolbarOverlay::Actions => toolbar.open_actions(window, cx),
                ToolbarOverlay::Agent => toolbar.open_agent(window, cx),
                ToolbarOverlay::None => {}
            }
        });
        self.toolbar_screen.last_action =
            format!("Story set the {} toolbar overlay", overlay.label()).into();
        cx.notify();
    }

    fn render_toolbar_demo_frame(&self, cx: &mut Context<Self>) -> AnyElement {
        let agent = self.toolbar_screen.toolbar.clone();
        let accent = cx.theme().selection;
        let accent_foreground = if (accent.l - cx.theme().foreground.l).abs()
            >= (accent.l - cx.theme().background.l).abs()
        {
            cx.theme().foreground
        } else {
            cx.theme().background
        };

        // These fixed colors belong to the mock document artwork inside the selected frame.
        // Editor chrome around the artwork uses active theme tokens below.
        let artwork = h_flex()
            .size_full()
            .bg(rgba(0xf7f8faff))
            .child(
                v_flex()
                    .w(px(118.))
                    .h_full()
                    .p_3()
                    .gap_2()
                    .bg(rgba(0x111827ff))
                    .child(div().size(px(24.)).rounded(px(7.)).bg(rgba(0x6366f1ff)))
                    .child(
                        div()
                            .h(px(8.))
                            .w(px(68.))
                            .rounded_full()
                            .bg(rgba(0xffffff66)),
                    )
                    .child(
                        div()
                            .h(px(8.))
                            .w(px(82.))
                            .rounded_full()
                            .bg(rgba(0xffffff33)),
                    )
                    .child(
                        div()
                            .h(px(8.))
                            .w(px(56.))
                            .rounded_full()
                            .bg(rgba(0xffffff33)),
                    )
                    .child(div().flex_1())
                    .child(
                        h_flex()
                            .h(px(30.))
                            .px_2()
                            .rounded(px(8.))
                            .bg(rgba(0x6366f1ff))
                            .text_xs()
                            .text_color(rgba(0xffffffff))
                            .child("Upgrade"),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .h_full()
                    .p_5()
                    .gap_3()
                    .child(
                        h_flex()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgba(0x6b7280ff))
                                            .child("OVERVIEW"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(22.))
                                            .font_semibold()
                                            .text_color(rgba(0x111827ff))
                                            .child("Good morning, Maya"),
                                    ),
                            )
                            .child(div().flex_1())
                            .child(div().size(px(32.)).rounded_full().bg(rgba(0xf59e0bff))),
                    )
                    .child(
                        h_flex()
                            .h(px(92.))
                            .gap_3()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .h_full()
                                    .p_3()
                                    .rounded(px(12.))
                                    .bg(rgba(0x6366f1ff))
                                    .text_color(rgba(0xffffffff))
                                    .child(div().text_xs().child("Revenue"))
                                    .child(
                                        div()
                                            .mt_2()
                                            .text_size(px(20.))
                                            .font_semibold()
                                            .child("$48.2k"),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .h_full()
                                    .p_3()
                                    .rounded(px(12.))
                                    .bg(rgba(0xffffffff))
                                    .border_1()
                                    .border_color(rgba(0x00000010))
                                    .text_color(rgba(0x111827ff))
                                    .child(div().text_xs().child("Customers"))
                                    .child(
                                        div()
                                            .mt_2()
                                            .text_size(px(20.))
                                            .font_semibold()
                                            .child("1,429"),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .h_full()
                                    .p_3()
                                    .rounded(px(12.))
                                    .bg(rgba(0xffffffff))
                                    .border_1()
                                    .border_color(rgba(0x00000010))
                                    .text_color(rgba(0x111827ff))
                                    .child(div().text_xs().child("Conversion"))
                                    .child(
                                        div()
                                            .mt_2()
                                            .text_size(px(20.))
                                            .font_semibold()
                                            .child("12.8%"),
                                    ),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_h(px(0.))
                            .p_3()
                            .rounded(px(12.))
                            .bg(rgba(0xffffffff))
                            .border_1()
                            .border_color(rgba(0x00000010))
                            .child(
                                h_flex()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_semibold()
                                            .text_color(rgba(0x111827ff))
                                            .child("Activity"),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded(px(6.))
                                            .bg(rgba(0xf3f4f6ff))
                                            .text_xs()
                                            .text_color(rgba(0x6b7280ff))
                                            .child("Last 30 days"),
                                    ),
                            )
                            .child(h_flex().flex_1().items_end().gap_2().pt_3().children(
                                [44., 68., 38., 82., 58., 94., 74.].map(|height| {
                                    div()
                                        .flex_1()
                                        .h(px(height))
                                        .rounded_t(px(4.))
                                        .bg(rgba(0x6366f1cc))
                                }),
                            )),
                    ),
            )
            .into_any_element();

        div()
            .relative()
            // The mock document keeps its design size like real canvas
            // artwork: a narrow story clips it instead of squashing it.
            .flex_none()
            .w(px(596.))
            .h(px(360.))
            .rounded(px(3.))
            .border_2()
            .border_color(accent)
            .bg(cx.theme().background)
            .shadow_lg()
            .child(
                div()
                    .absolute()
                    .left(px(-2.))
                    .top(px(-24.))
                    .px_2()
                    .h(px(22.))
                    .flex()
                    .items_center()
                    .rounded_t(px(5.))
                    .bg(accent)
                    .text_xs()
                    .font_semibold()
                    .text_color(accent_foreground)
                    .child("Hero frame"),
            )
            .child(
                h_flex()
                    .id("toolbar-story-agent-context")
                    .absolute()
                    .right(px(-17.))
                    .top(px(-17.))
                    .size(px(34.))
                    .justify_center()
                    .rounded_full()
                    .border_2()
                    .border_color(cx.theme().background)
                    .bg(cx.theme().primary)
                    .text_color(cx.theme().primary_foreground)
                    .shadow_lg()
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().primary_hover))
                    .on_click(cx.listener(move |_this, _, window, cx| {
                        agent.update(cx, |toolbar, cx| {
                            toolbar.open_agent(window, cx);
                        });
                    }))
                    .child("✦"),
            )
            .child(artwork)
            .when(self.toolbar_screen.mode == ToolbarMode::Dev, |frame| {
                frame
                    .child(
                        div()
                            .absolute()
                            .left(px(116.))
                            .top(px(118.))
                            .w(px(176.))
                            .h(px(1.))
                            .bg(cx.theme().danger),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(174.))
                            .top(px(99.))
                            .px_2()
                            .py_1()
                            .rounded(px(5.))
                            .bg(cx.theme().danger)
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().danger_foreground)
                            .child("176"),
                    )
            })
            .when(self.toolbar_screen.mode == ToolbarMode::Motion, |frame| {
                frame.child(
                    h_flex()
                        .absolute()
                        .right(px(16.))
                        .top(px(16.))
                        .h(px(28.))
                        .px_2()
                        .gap_1()
                        .rounded(px(8.))
                        .bg(cx.theme().primary)
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().primary_foreground)
                        .child("◆  3 keyframes"),
                )
            })
            .into_any_element()
    }

    fn render_toolbar_timeline(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.toolbar_screen.mode != ToolbarMode::Motion {
            return div().into_any_element();
        }
        let mut tracks = v_flex().flex_1().min_h(px(0.)).py_2();
        for (index, label) in ["Position", "Scale", "Opacity"].into_iter().enumerate() {
            tracks = tracks.child(
                h_flex()
                    .h(px(26.))
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .w(px(138.))
                            .px_3()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(label),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .h_full()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(68. + index as f32 * 26.))
                                    .top(px(4.))
                                    .text_color(cx.theme().selection)
                                    .child("◆"),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .left(px(218. + index as f32 * 34.))
                                    .top(px(4.))
                                    .text_color(cx.theme().selection)
                                    .child("◆"),
                            ),
                    ),
            );
        }
        v_flex()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(142.))
            .bg(cx.theme().popover)
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .h(px(34.))
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().popover_foreground)
                    .child("Timeline")
                    .child(div().flex_1())
                    .child("0 ms       500       1000       1500       2000 ms"),
            )
            .child(tracks)
            .into_any_element()
    }

    pub(crate) fn render_toolbar_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let agent = self.toolbar_screen.toolbar.clone();
        let mode_accent = cx.theme().selection;
        let story = cx.entity();
        let story_width = self.toolbar_screen.story_width;
        let show_sidebar = self.toolbar_screen.left_sidebar_visible
            && story_width.is_none_or(|width| width >= STORY_SIDEBAR_MIN_WIDTH);
        let show_inspector = self.toolbar_screen.right_sidebar_visible
            && story_width.is_none_or(|width| width >= STORY_INSPECTOR_MIN_WIDTH);

        h_flex()
            .relative()
            .flex_1()
            .min_h(px(0.))
            .child(
                // Measures the story surface so the mock chrome columns can
                // yield to the canvas on narrow fluid viewports. The write
                // is deferred past the draw so a changed width schedules a
                // re-render (notifying mid-draw is a no-op).
                canvas(
                    move |bounds, _, app| {
                        let width = f32::from(bounds.size.width);
                        let known = story.read(app).toolbar_screen.story_width;
                        if known.is_none_or(|known| (known - width).abs() > 0.5) {
                            let story = story.clone();
                            app.defer(move |app| {
                                story.update(app, |this, cx| {
                                    this.toolbar_screen.story_width = Some(width);
                                    cx.notify();
                                });
                            });
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .left_0()
                .top_0()
                .size_full(),
            )
            .when(show_sidebar, |story| {
                story.child(
                    v_flex()
                        .w(px(226.))
                        .h_full()
                        .bg(cx.theme().sidebar)
                        .text_color(cx.theme().sidebar_foreground)
                        .border_r_1()
                        .border_color(cx.theme().sidebar_border)
                        .child(
                            h_flex()
                                .h(px(48.))
                                .px_3()
                                .gap_2()
                                .border_b_1()
                                .border_color(cx.theme().sidebar_border)
                                .child(div().size(px(24.)).rounded(px(7.)).bg(cx.theme().primary))
                                .child(
                                    v_flex()
                                        .gap(px(1.))
                                        .child(div().text_sm().font_semibold().child("Commerce OS"))
                                        .child(
                                            div()
                                                .text_size(px(10.))
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Drafts / Toolbar clone"),
                                        ),
                                ),
                        )
                        .child(
                            h_flex()
                                .h(px(38.))
                                .px_3()
                                .gap_2()
                                .text_xs()
                                .font_semibold()
                                .child("File"),
                        )
                        .child(
                            h_flex()
                                .id("toolbar-story-agents")
                                .h(px(38.))
                                .mx_2()
                                .px_2()
                                .gap_2()
                                .rounded(px(7.))
                                .cursor_pointer()
                                .hover(|style| style.bg(cx.theme().sidebar_accent))
                                .on_click(cx.listener(move |_this, _, window, cx| {
                                    agent.update(cx, |toolbar, cx| {
                                        toolbar.open_agent(window, cx);
                                    });
                                }))
                                .child(
                                    div()
                                        .w(px(20.))
                                        .text_center()
                                        .font_semibold()
                                        .text_color(cx.theme().primary)
                                        .child("✦"),
                                )
                                .child(div().flex_1().text_sm().child("Agents"))
                                .child(div().size(px(6.)).rounded_full().bg(cx.theme().primary)),
                        )
                        .child(
                            h_flex()
                                .h(px(38.))
                                .mx_2()
                                .px_2()
                                .gap_2()
                                .rounded(px(7.))
                                .bg(cx.theme().sidebar_accent)
                                .child(
                                    div()
                                        .w(px(20.))
                                        .text_center()
                                        .text_color(cx.theme().sidebar_accent_foreground)
                                        .child("▤"),
                                )
                                .child(div().text_sm().font_semibold().child("Layers")),
                        )
                        .child(
                            h_flex()
                                .h(px(38.))
                                .mx_2()
                                .px_2()
                                .gap_2()
                                .rounded(px(7.))
                                .child(
                                    div()
                                        .w(px(20.))
                                        .text_center()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("◇"),
                                )
                                .child(div().text_sm().child("Assets")),
                        )
                        .child(
                            v_flex()
                                .mt_3()
                                .mx_2()
                                .pt_2()
                                .gap_1()
                                .border_t_1()
                                .border_color(cx.theme().sidebar_border)
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .text_size(px(10.))
                                        .font_semibold()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("PAGES"),
                                )
                                .child(
                                    h_flex()
                                        .h(px(32.))
                                        .px_2()
                                        .rounded(px(6.))
                                        .bg(cx.theme().sidebar_accent)
                                        .text_sm()
                                        .child("Dashboard"),
                                )
                                .child(h_flex().h(px(32.)).px_2().text_sm().child("Components")),
                        )
                        .child(div().flex_1())
                        .child(
                            v_flex()
                                .m_3()
                                .p_3()
                                .gap_1()
                                .rounded(px(10.))
                                .bg(cx.theme().secondary)
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .font_semibold()
                                        .text_color(mode_accent)
                                        .child(format!(
                                            "{} MODE",
                                            self.toolbar_screen.mode.label().to_uppercase()
                                        )),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(self.toolbar_screen.last_action.clone()),
                                ),
                        ),
                )
            })
            .child(
                div()
                    .debug_selector(|| "toolbar-story-canvas".to_owned())
                    .relative()
                    .flex_1()
                    .min_w(px(0.))
                    .h_full()
                    .overflow_hidden()
                    .bg(cx.theme().muted)
                    .child(
                        h_flex()
                            .size_full()
                            .justify_center()
                            .items_center()
                            .pb(if self.toolbar_screen.mode == ToolbarMode::Motion {
                                px(190.)
                            } else {
                                px(92.)
                            })
                            .child(self.render_toolbar_demo_frame(cx)),
                    )
                    .child(self.render_toolbar_timeline(cx))
                    .child(
                        h_flex()
                            .absolute()
                            .left_0()
                            .right_0()
                            .bottom(if self.toolbar_screen.mode == ToolbarMode::Motion {
                                px(160.)
                            } else {
                                px(18.)
                            })
                            .px_4()
                            .justify_center()
                            .child(self.toolbar_screen.toolbar.clone()),
                    ),
            )
            .when(show_inspector, |story| {
                story.child(
                    v_flex()
                        .w(px(258.))
                        .h_full()
                        .bg(cx.theme().background)
                        .text_color(cx.theme().foreground)
                        .border_l_1()
                        .border_color(cx.theme().border)
                        .child(
                            h_flex()
                                .h(px(48.))
                                .px_4()
                                .gap_4()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .child(
                                    div()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .border_b_2()
                                        .border_color(mode_accent)
                                        .text_sm()
                                        .font_semibold()
                                        .child(self.toolbar_screen.mode.label()),
                                )
                                .child(
                                    div()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(if self.toolbar_screen.mode == ToolbarMode::Dev {
                                            "Plugins"
                                        } else {
                                            "Prototype"
                                        }),
                                ),
                        )
                        .child(
                            v_flex()
                                .p_4()
                                .gap_4()
                                .child(
                                    v_flex()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_semibold()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("SELECTION"),
                                        )
                                        .child(
                                            h_flex()
                                                .child(div().text_sm().child("Hero frame"))
                                                .child(div().flex_1())
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(mode_accent)
                                                        .child(self.toolbar_screen.tool.label()),
                                                ),
                                        ),
                                )
                                .child(div().h(px(1.)).bg(cx.theme().border))
                                .child(
                                    v_flex()
                                        .gap_2()
                                        .child(div().text_sm().font_semibold().child(
                                            match self.toolbar_screen.mode {
                                                ToolbarMode::Design => "Layout",
                                                ToolbarMode::Motion => "Animation",
                                                ToolbarMode::Dev => "Inspect",
                                            },
                                        ))
                                        .child(
                                            h_flex()
                                                .h(px(34.))
                                                .px_2()
                                                .rounded(px(7.))
                                                .bg(cx.theme().secondary)
                                                .text_xs()
                                                .child(match self.toolbar_screen.mode {
                                                    ToolbarMode::Design => {
                                                        "Auto layout · Vertical".to_owned()
                                                    }
                                                    ToolbarMode::Motion => format!(
                                                        "{} · {} ms",
                                                        self.toolbar_screen
                                                            .motion_options
                                                            .animation_style,
                                                        self.toolbar_screen
                                                            .motion_options
                                                            .duration_ms
                                                    ),
                                                    ToolbarMode::Dev => "CSS · Web · px".to_owned(),
                                                }),
                                        ),
                                ),
                        ),
                )
            })
            .into_any_element()
    }

    pub(crate) fn render_toolbar_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .id("storybook-reference-toolbar")
            .debug_selector(|| "storybook-reference-toolbar".to_owned())
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_reference_nav(cx))
            .child(self.render_toolbar_story(cx))
            .into_any_element()
    }

    pub(crate) fn render_toolbar_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "toolbar-story-knobs",
            vec![
                knobs::enum_knob_row(
                    "toolbar-knob-mode",
                    "MODE",
                    TOOLBAR_MODE_KNOB_ORDER.map(|mode| KnobOption::new(mode, mode.label())),
                    self.toolbar_screen.mode,
                    |this, mode, _, cx| {
                        let toolbar = this.toolbar_screen.toolbar.clone();
                        this.handle_toolbar_action(
                            toolbar,
                            &ToolbarAction::ModeChangeRequested { mode },
                            cx,
                        );
                    },
                    cx,
                ),
                knobs::enum_knob_row(
                    "toolbar-knob-overlay",
                    "OVERLAY",
                    ToolbarOverlay::ALL.map(|overlay| KnobOption::new(overlay, overlay.label())),
                    self.toolbar_screen.overlay,
                    |this, overlay, window, cx| {
                        this.apply_toolbar_overlay(overlay, window, cx);
                    },
                    cx,
                ),
                knobs::numeric_knob_row(
                    "toolbar-knob-zoom",
                    "ZOOM",
                    [50u16, 100, 200]
                        .map(|percent| KnobOption::new(percent, format!("{percent}%"))),
                    self.toolbar_screen.zoom,
                    |this, percent, _, cx| {
                        let toolbar = this.toolbar_screen.toolbar.clone();
                        this.handle_toolbar_action(
                            toolbar,
                            &ToolbarAction::ZoomChangeRequested { percent },
                            cx,
                        );
                    },
                    cx,
                ),
            ],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_env_vars_seed_the_runtime_knob_defaults() {
        assert_eq!(parse_toolbar_mode(None), ToolbarMode::Design);
        assert_eq!(
            parse_toolbar_mode(Some("draw")),
            ToolbarMode::Design,
            "the retired Draw mode falls back to Design"
        );
        assert_eq!(parse_toolbar_mode(Some("MOTION")), ToolbarMode::Motion);
        assert_eq!(parse_toolbar_mode(Some("dev")), ToolbarMode::Dev);
        assert_eq!(parse_toolbar_mode(Some("unknown")), ToolbarMode::Design);

        assert_eq!(parse_toolbar_overlay(None), ToolbarOverlay::None);
        assert_eq!(
            parse_toolbar_overlay(Some("actions")),
            ToolbarOverlay::Actions
        );
        assert_eq!(parse_toolbar_overlay(Some("AGENT")), ToolbarOverlay::Agent);
        assert_eq!(parse_toolbar_overlay(Some("off")), ToolbarOverlay::None);
    }

    #[test]
    fn story_floor_hides_both_mock_chrome_columns() {
        // At the registered story floor the fixed-width mock columns must
        // both yield, leaving the whole surface to the dock's canvas host;
        // the inspector yields first as the less essential column.
        // The sidebar breakpoint is the lower of the two, so clearing it
        // clears the inspector breakpoint as well.
        let (floor_width, _) = StoryKind::Toolbar.descriptor().min_story_size();
        assert!(floor_width < STORY_SIDEBAR_MIN_WIDTH.min(STORY_INSPECTOR_MIN_WIDTH));
    }

    #[test]
    fn toolbar_mode_knob_covers_every_mode_once() {
        let mut seen = std::collections::HashSet::new();
        for mode in TOOLBAR_MODE_KNOB_ORDER {
            assert!(seen.insert(mode.label()));
        }
        assert_eq!(seen.len(), ToolbarMode::ALL.len());
    }

    #[test]
    fn story_chrome_controls_mirror_the_mock_sidebar_state() {
        let controls = toolbar_chrome_controls(true, false);
        assert_eq!(
            controls
                .iter()
                .map(|control| control.id.as_ref())
                .collect::<Vec<_>>(),
            [
                TOOLBAR_CHROME_FIT,
                TOOLBAR_CHROME_LEFT,
                TOOLBAR_CHROME_RIGHT
            ]
        );
        assert!(!controls[0].active);
        assert!(controls[1].active);
        assert!(!controls[2].active);
        assert_eq!(controls[1].icon_path(), "icons/panel-left-close.svg");
        assert_eq!(controls[2].icon_path(), "icons/panel-right-open.svg");
    }
}
