//! Mock document adapters for the independent properties inspector tabs.
//!
//! Reusable tabs emit requests; this Storybook owner validates them, updates
//! local fixtures, and echoes the accepted snapshots back into each organism.
use super::knobs::{self, KnobOption};
use crate::*;
use fanta_gpui::properties_tabs::*;
use fanta_gpui::timeline::{TimelineEasing, TimelinePlayback};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PropertiesTabsNamedState {
    Populated,
    Empty,
    ReadOnly,
}
impl PropertiesTabsNamedState {
    const ALL: [Self; 3] = [Self::Populated, Self::Empty, Self::ReadOnly];
    const fn label(self) -> &'static str {
        match self {
            Self::Populated => "Populated",
            Self::Empty => "Empty",
            Self::ReadOnly => "Read only",
        }
    }
}

pub(crate) struct PropertiesTabsScreen {
    pub(crate) motion: Entity<MotionInspector>,
    pub(crate) motion_data: MotionInspectorViewData,
    pub(crate) draw: Entity<DrawInspector>,
    pub(crate) draw_data: DrawInspectorViewData,
    pub(crate) code: Entity<CodeInspector>,
    pub(crate) code_data: CodeInspectorViewData,
    pub(crate) prototype: Entity<PrototypeInspector>,
    pub(crate) prototype_data: PrototypeInspectorViewData,
    pub(crate) comments: Entity<CommentsInspector>,
    pub(crate) comments_data: CommentsInspectorViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: PropertiesTabsNamedState,
    next_id: u64,
    keyframe_counts: HashMap<SharedString, u32>,
}
impl PropertiesTabsScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let motion_data = motion_fixture();
        let draw_data = draw_fixture();
        let code_data = code_fixture();
        let prototype_data = prototype_fixture();
        let comments_data = comments_fixture();
        Self {
            motion: cx.new(|cx| {
                MotionInspector::new("storybook-motion-inspector", motion_data.clone(), cx)
            }),
            draw: cx
                .new(|cx| DrawInspector::new("storybook-draw-inspector", draw_data.clone(), cx)),
            code: cx
                .new(|cx| CodeInspector::new("storybook-code-inspector", code_data.clone(), cx)),
            prototype: cx.new(|cx| {
                PrototypeInspector::new("storybook-prototype-inspector", prototype_data.clone(), cx)
            }),
            comments: cx.new(|cx| {
                CommentsInspector::new("storybook-comments-inspector", comments_data.clone(), cx)
            }),
            motion_data,
            draw_data,
            code_data,
            prototype_data,
            comments_data,
            last_action: "Ready — inspect and edit the local sample project".into(),
            named_state: PropertiesTabsNamedState::Populated,
            next_id: 0,
            keyframe_counts: HashMap::new(),
        }
    }

    pub(crate) fn subscriptions(
        &self,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) -> Vec<Subscription> {
        vec![
            cx.subscribe_in(
                &self.motion,
                window,
                |story, _, action: &MotionInspectorAction, window, cx| {
                    story
                        .properties_tabs_screen
                        .handle_motion_action(action, cx);
                    if matches!(action, MotionInspectorAction::TimelineOpenRequested) {
                        story.activate_gallery_story(StoryKind::Timeline, window, cx);
                    }
                },
            ),
            cx.subscribe(&self.draw, |story, _, action: &DrawInspectorAction, cx| {
                story.properties_tabs_screen.handle_draw_action(action, cx);
            }),
            cx.subscribe(&self.code, |story, _, action: &CodeInspectorAction, cx| {
                story.properties_tabs_screen.handle_code_action(action, cx);
            }),
            cx.subscribe(
                &self.prototype,
                |story, _, action: &PrototypeInspectorAction, cx| {
                    story
                        .properties_tabs_screen
                        .handle_prototype_action(action, cx);
                },
            ),
            cx.subscribe(
                &self.comments,
                |story, _, action: &CommentsInspectorAction, cx| {
                    story
                        .properties_tabs_screen
                        .handle_comments_action(action, cx);
                },
            ),
        ]
    }

    fn fresh_id(&mut self, prefix: &str) -> SharedString {
        self.next_id += 1;
        format!("story-{prefix}-{}", self.next_id).into()
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: PropertiesTabsNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        self.motion_data = motion_fixture();
        self.draw_data = draw_fixture();
        self.code_data = code_fixture();
        self.prototype_data = prototype_fixture();
        self.comments_data = comments_fixture();
        self.keyframe_counts.clear();
        if state == PropertiesTabsNamedState::Empty {
            self.motion_data.selection_name = "No selection".into();
            self.motion_data.animated_properties.clear();
            self.motion_data.selected_preset = None;
            self.code_data.selection_name = "No selection".into();
            self.code_data.code = SharedString::default();
            self.code_data.properties.clear();
            self.prototype_data.selection_name = "No selection".into();
            self.prototype_data.connections.clear();
            self.prototype_data.flow_name = None;
            self.comments_data.threads.clear();
            self.comments_data.selected_thread = None;
        }
        let read_only = state == PropertiesTabsNamedState::ReadOnly;
        self.motion_data.read_only = read_only;
        self.draw_data.read_only = read_only;
        self.prototype_data.read_only = read_only;
        self.comments_data.can_comment = !read_only;
        self.last_action = format!("Loaded {} inspector fixtures", state.label()).into();
        self.echo(cx);
    }

    fn echo(&self, cx: &mut Context<Storybook>) {
        self.motion.update(cx, |panel, cx| {
            panel.set_view_data(self.motion_data.clone(), cx)
        });
        self.draw.update(cx, |panel, cx| {
            panel.set_view_data(self.draw_data.clone(), cx)
        });
        self.code.update(cx, |panel, cx| {
            panel.set_view_data(self.code_data.clone(), cx)
        });
        self.prototype.update(cx, |panel, cx| {
            panel.set_view_data(self.prototype_data.clone(), cx)
        });
        self.comments.update(cx, |panel, cx| {
            panel.set_view_data(self.comments_data.clone(), cx)
        });
        cx.notify();
    }

    pub(crate) fn handle_motion_action(
        &mut self,
        action: &MotionInspectorAction,
        cx: &mut Context<Storybook>,
    ) {
        let presentation_only = matches!(
            action,
            MotionInspectorAction::PlayingChangeRequested { .. }
                | MotionInspectorAction::TimelineOpenRequested
        );
        if self.motion_data.read_only && !presentation_only {
            self.last_action = "Motion change refused: the sample is read only".into();
            cx.notify();
            return;
        }
        self.last_action = format!("{action:?}").into();
        match action {
            MotionInspectorAction::DurationChangeRequested { duration_ms } => {
                self.motion_data.duration_ms = (*duration_ms).clamp(1, 3_600_000)
            }
            MotionInspectorAction::DelayChangeRequested { delay_ms } => {
                self.motion_data.delay_ms = (*delay_ms).min(3_600_000)
            }
            MotionInspectorAction::EasingChangeRequested { easing } => {
                self.motion_data.easing = easing.clone()
            }
            MotionInspectorAction::PlaybackChangeRequested { playback } => {
                self.motion_data.playback = *playback
            }
            MotionInspectorAction::PlayingChangeRequested { playing } => {
                self.motion_data.playing = *playing
            }
            MotionInspectorAction::AutoKeyframeChangeRequested { enabled } => {
                self.motion_data.auto_keyframe = *enabled
            }
            MotionInspectorAction::PresetApplyRequested { id } => {
                if self
                    .motion_data
                    .presets
                    .iter()
                    .any(|preset| preset.id == *id)
                {
                    self.motion_data.selected_preset = Some(id.clone());
                    match id.as_ref() {
                        "spring" => {
                            self.motion_data.duration_ms = 800;
                            self.motion_data.easing = TimelineEasing::Spring { bounce: 0.3 };
                        }
                        "slide-up" => {
                            self.motion_data.duration_ms = 500;
                            self.motion_data.easing = TimelineEasing::EaseOut;
                        }
                        _ => {
                            self.motion_data.duration_ms = 300;
                            self.motion_data.easing = TimelineEasing::EaseInOut;
                        }
                    }
                }
            }
            MotionInspectorAction::KeyframeAddRequested { property_id } => {
                if let Some(property) = self
                    .motion_data
                    .animated_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                {
                    let count = self.keyframe_counts.entry(property_id.clone()).or_insert(2);
                    *count += 1;
                    let name = property.label.split(" · ").next().unwrap_or("Property");
                    property.label = format!("{name} · {count} keyframes").into();
                }
            }
            MotionInspectorAction::TimelineOpenRequested => {}
        }
        self.motion.update(cx, |panel, cx| {
            panel.set_view_data(self.motion_data.clone(), cx)
        });
        cx.notify();
    }

    pub(crate) fn handle_draw_action(
        &mut self,
        action: &DrawInspectorAction,
        cx: &mut Context<Storybook>,
    ) {
        if self.draw_data.read_only {
            self.last_action = "Draw change refused: the sample is read only".into();
            cx.notify();
            return;
        }
        self.last_action = format!("{action:?}").into();
        match action {
            DrawInspectorAction::OptionsChangeRequested { options } => {
                self.draw_data.options = options.clone().normalized()
            }
            DrawInspectorAction::ColorChangeRequested { hex } => {
                if let Some(hex) = normalize_hex(hex) {
                    self.draw_data.color_hex = hex;
                }
            }
            DrawInspectorAction::BlendModeChangeRequested { id } => {
                if self.draw_data.blend_modes.iter().any(|mode| mode.id == *id) {
                    self.draw_data.blend_mode = id.clone();
                }
            }
            DrawInspectorAction::BrushPresetSaveRequested => {
                let name = format!(
                    "Saved brush {}",
                    self.draw_data.options.brush_tips.len().saturating_sub(5)
                );
                self.draw_data.options.brush_tips.push(name.clone().into());
                self.draw_data.options.brush_tip = name.clone().into();
                self.last_action = format!("Saved {name} in the local brush catalog").into();
            }
        }
        self.draw.update(cx, |panel, cx| {
            panel.set_view_data(self.draw_data.clone(), cx)
        });
        cx.notify();
    }

    pub(crate) fn handle_code_action(
        &mut self,
        action: &CodeInspectorAction,
        cx: &mut Context<Storybook>,
    ) {
        self.last_action = format!("{action:?}").into();
        match action {
            CodeInspectorAction::LanguageChangeRequested { language } => {
                if self
                    .code_data
                    .languages
                    .iter()
                    .any(|option| option.id == *language)
                {
                    self.code_data.language = language.clone();
                    if !self.code_data.properties.is_empty() {
                        self.code_data.code = sample_code(language).into();
                    }
                }
            }
            CodeInspectorAction::CopyRequested { code } => {
                if *code == self.code_data.code {
                    cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                }
            }
            CodeInspectorAction::CopyPropertyRequested { property_id } => {
                if let Some(property) = self
                    .code_data
                    .properties
                    .iter()
                    .find(|property| property.id == *property_id)
                {
                    cx.write_to_clipboard(ClipboardItem::new_string(format!(
                        "{}: {};",
                        property.id, property.label
                    )));
                }
            }
            CodeInspectorAction::WrapLinesChangeRequested { enabled } => {
                self.code_data.wrap_lines = *enabled
            }
        }
        self.code.update(cx, |panel, cx| {
            panel.set_view_data(self.code_data.clone(), cx)
        });
        cx.notify();
    }

    pub(crate) fn handle_prototype_action(
        &mut self,
        action: &PrototypeInspectorAction,
        cx: &mut Context<Storybook>,
    ) {
        if self.prototype_data.read_only
            && !matches!(action, PrototypeInspectorAction::PresentRequested)
        {
            self.last_action = "Prototype change refused: the sample is read only".into();
            cx.notify();
            return;
        }
        self.last_action = format!("{action:?}").into();
        match action {
            PrototypeInspectorAction::DeviceChangeRequested { id } => {
                if self
                    .prototype_data
                    .devices
                    .iter()
                    .any(|device| device.id == *id)
                {
                    self.prototype_data.device = id.clone();
                }
            }
            PrototypeInspectorAction::BackgroundChangeRequested { hex } => {
                if let Some(hex) = normalize_hex(hex) {
                    self.prototype_data.background_hex = hex;
                }
            }
            PrototypeInspectorAction::ConnectionAddRequested => {
                let id = self.fresh_id("connection");
                self.prototype_data.connections.push(PrototypeConnection {
                    id,
                    trigger: "On click".into(),
                    action: "Navigate to".into(),
                    destination: "Checkout".into(),
                    animation: "Smart animate · 300 ms".into(),
                });
            }
            PrototypeInspectorAction::ConnectionEditRequested { id } => {
                if let Some(connection) = self
                    .prototype_data
                    .connections
                    .iter_mut()
                    .find(|connection| connection.id == *id)
                {
                    let overlay = connection.action.as_ref() != "Open overlay";
                    connection.action = if overlay {
                        "Open overlay"
                    } else {
                        "Navigate to"
                    }
                    .into();
                    connection.destination = if overlay { "Quick view" } else { "Checkout" }.into();
                    connection.animation = if overlay {
                        "Dissolve · 200 ms"
                    } else {
                        "Smart animate · 300 ms"
                    }
                    .into();
                    self.last_action = format!(
                        "Mock host changed interaction to {} {}",
                        connection.action, connection.destination
                    )
                    .into();
                }
            }
            PrototypeInspectorAction::ConnectionRemoveRequested { id } => self
                .prototype_data
                .connections
                .retain(|connection| connection.id != *id),
            PrototypeInspectorAction::FlowStartRequested => {
                self.prototype_data.flow_name = Some("Shopping flow".into())
            }
            PrototypeInspectorAction::FlowRenameRequested { name } => {
                if !name.trim().is_empty() && self.prototype_data.flow_name.is_some() {
                    self.prototype_data.flow_name = Some(name.trim().to_owned().into());
                }
            }
            PrototypeInspectorAction::PresentRequested => {
                self.last_action = format!(
                    "Mock host requested presentation of {}",
                    self.prototype_data
                        .flow_name
                        .as_deref()
                        .unwrap_or("the selected frame")
                )
                .into()
            }
        }
        self.prototype.update(cx, |panel, cx| {
            panel.set_view_data(self.prototype_data.clone(), cx)
        });
        cx.notify();
    }

    pub(crate) fn handle_comments_action(
        &mut self,
        action: &CommentsInspectorAction,
        cx: &mut Context<Storybook>,
    ) {
        let presentation_only = matches!(
            action,
            CommentsInspectorAction::FilterChangeRequested { .. }
                | CommentsInspectorAction::ThreadSelectRequested { .. }
                | CommentsInspectorAction::ThreadClearRequested
        );
        if !self.comments_data.can_comment && !presentation_only {
            self.last_action = "Comment change refused: commenting is disabled".into();
            cx.notify();
            return;
        }
        self.last_action = format!("{action:?}").into();
        match action {
            CommentsInspectorAction::ThreadClearRequested => {
                self.comments_data.selected_thread = None;
            }
            CommentsInspectorAction::FilterChangeRequested { filter } => {
                self.comments_data.filter = *filter
            }
            CommentsInspectorAction::ThreadSelectRequested { id } => {
                if let Some(thread) = self
                    .comments_data
                    .threads
                    .iter_mut()
                    .find(|thread| thread.id == *id)
                {
                    thread.unread = false;
                    self.comments_data.selected_thread = Some(id.clone());
                }
            }
            CommentsInspectorAction::ResolveChangeRequested { id, resolved } => {
                if let Some(thread) = self
                    .comments_data
                    .threads
                    .iter_mut()
                    .find(|thread| thread.id == *id)
                {
                    thread.resolved = *resolved;
                }
            }
            CommentsInspectorAction::CommentAddRequested { body } => {
                if !body.trim().is_empty() {
                    let id = self.fresh_id("thread");
                    let comment_id = self.fresh_id("comment");
                    self.comments_data.selected_thread = Some(id.clone());
                    self.comments_data.filter = CommentsFilter::Open;
                    self.comments_data.threads.push(InspectorCommentThread {
                        id,
                        location: "Product card".into(),
                        resolved: false,
                        unread: false,
                        comments: vec![InspectorComment {
                            id: comment_id,
                            author: "You".into(),
                            time_label: "Just now".into(),
                            body: body.trim().to_owned().into(),
                        }],
                    });
                }
            }
            CommentsInspectorAction::ReplyRequested { thread_id, body } => {
                if !body.trim().is_empty()
                    && self
                        .comments_data
                        .threads
                        .iter()
                        .any(|thread| thread.id == *thread_id)
                {
                    let id = self.fresh_id("comment");
                    if let Some(thread) = self
                        .comments_data
                        .threads
                        .iter_mut()
                        .find(|thread| thread.id == *thread_id)
                    {
                        thread.comments.push(InspectorComment {
                            id,
                            author: "You".into(),
                            time_label: "Just now".into(),
                            body: body.trim().to_owned().into(),
                        });
                        thread.unread = false;
                        self.comments_data.selected_thread = Some(thread_id.clone());
                    }
                }
            }
        }
        self.comments.update(cx, |panel, cx| {
            panel.set_view_data(self.comments_data.clone(), cx)
        });
        cx.notify();
    }
}

fn normalize_hex(value: &str) -> Option<SharedString> {
    let value = value.trim().trim_start_matches('#');
    (matches!(value.len(), 6 | 8) && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| value.to_ascii_uppercase().into())
}
fn motion_fixture() -> MotionInspectorViewData {
    MotionInspectorViewData {
        selection_name: "Product card".into(),
        duration_ms: 600,
        delay_ms: 100,
        easing: TimelineEasing::EaseOut,
        playback: TimelinePlayback::Once,
        presets: vec![
            InspectorChoice::new("fade-in", "Fade in"),
            InspectorChoice::new("slide-up", "Slide up"),
            InspectorChoice::new("spring", "Spring"),
        ],
        selected_preset: Some("slide-up".into()),
        animated_properties: vec![
            InspectorChoice::new("position", "Position"),
            InspectorChoice::new("opacity", "Opacity"),
            InspectorChoice::new("scale", "Scale"),
        ],
        can_edit_timing: true,
        can_preview: true,
        ..Default::default()
    }
}
fn draw_fixture() -> DrawInspectorViewData {
    let mut data = DrawInspectorViewData {
        color_hex: "6366F1".into(),
        ..Default::default()
    };
    data.options.size = 48;
    data.options.hardness = 80;
    data.options.smoothing = 20;
    data
}
fn code_fixture() -> CodeInspectorViewData {
    CodeInspectorViewData {
        selection_name: "Product card".into(),
        code: sample_code("css").into(),
        properties: vec![
            InspectorChoice::new("width", "320px"),
            InspectorChoice::new("height", "400px"),
            InspectorChoice::new("border-radius", "16px"),
            InspectorChoice::new("background", "#FFFFFF"),
        ],
        ..Default::default()
    }
}
fn sample_code(language: &str) -> &'static str {
    match language {
        "swiftui" => {
            "VStack(alignment: .leading, spacing: 16) {\n    Image(\"product\")\n        .resizable()\n        .aspectRatio(contentMode: .fill)\n    Text(\"Everyday essentials\")\n        .font(.headline)\n}\n.padding(24)\n.frame(width: 320, height: 400)\n.background(.white)\n.clipShape(RoundedRectangle(cornerRadius: 16))"
        }
        "jsx" => {
            "<article className=\"product-card\">\n  <img src=\"/product.png\" alt=\"Product\" />\n  <h2>Everyday essentials</h2>\n  <p>Made for the way you live.</p>\n  <button>Add to bag</button>\n</article>"
        }
        _ => {
            ".product-card {\n  display: flex;\n  flex-direction: column;\n  gap: 16px;\n  padding: 24px;\n  width: 320px;\n  height: 400px;\n  border-radius: 16px;\n  background: #ffffff;\n  box-shadow: 0 8px 24px #00000014;\n}"
        }
    }
}
fn prototype_fixture() -> PrototypeInspectorViewData {
    PrototypeInspectorViewData {
        selection_name: "Product card".into(),
        devices: vec![
            InspectorChoice::new("none", "No device"),
            InspectorChoice::new("phone", "Phone · 390 × 844"),
            InspectorChoice::new("desktop", "Desktop · 1440 × 900"),
        ],
        device: "phone".into(),
        background_hex: "F4F4F5".into(),
        connections: vec![PrototypeConnection {
            id: "product-checkout".into(),
            trigger: "On click".into(),
            action: "Navigate to".into(),
            destination: "Checkout".into(),
            animation: "Smart animate · 300 ms".into(),
        }],
        flow_name: Some("Shopping flow".into()),
        can_start_flow: true,
        can_present: true,
        read_only: false,
    }
}
fn comments_fixture() -> CommentsInspectorViewData {
    CommentsInspectorViewData {
        threads: vec![
            InspectorCommentThread { id: "thread-card".into(), location: "Product card".into(), resolved: false, unread: true, comments: vec![
                InspectorComment { id: "comment-1".into(), author: "Alex Morgan".into(), time_label: "12 min ago".into(), body: "Can we give the product image a little more room? The card feels dense on smaller screens.".into() },
                InspectorComment { id: "comment-2".into(), author: "Sam Chen".into(), time_label: "5 min ago".into(), body: "Agreed. I'll try a taller image and keep the spacing at 16 px.".into() },
            ] },
            InspectorCommentThread { id: "thread-checkout".into(), location: "Checkout · Primary button".into(), resolved: false, unread: false, comments: vec![InspectorComment { id: "comment-3".into(), author: "Taylor Reed".into(), time_label: "1 hr ago".into(), body: "The button contrast looks good. Please use the same treatment in the confirmation screen.".into() }] },
            InspectorCommentThread { id: "thread-type".into(), location: "Typography".into(), resolved: true, unread: false, comments: vec![InspectorComment { id: "comment-4".into(), author: "Alex Morgan".into(), time_label: "Yesterday".into(), body: "Updated the heading style to match the shared library.".into() }] },
        ], filter: CommentsFilter::Open, selected_thread: None, can_comment: true,
    }
}
impl Storybook {
    pub(crate) fn render_properties_tabs_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "properties-tabs-story-knobs",
            vec![knobs::enum_knob_row(
                "properties-tabs-state",
                "INSPECTION STATE",
                PropertiesTabsNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.properties_tabs_screen.named_state,
                |story, state, _, cx| story.properties_tabs_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}
