use super::{
    CommentsFilter, CommentsInspectorAction as Action, CommentsInspectorViewData, InspectorChoice,
    controls::*,
};
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _, TypographyToken,
    tokens, truncating_label,
};
use gpui::AppContext as _;
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Sizable as _, h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

pub struct CommentsInspector {
    id: SharedString,
    data: CommentsInspectorViewData,
    focus: FocusHandle,
    filter: Entity<Picker>,
    composer: Option<Entity<InputState>>,
    subscriptions: Vec<Subscription>,
}
impl EventEmitter<Action> for CommentsInspector {}
impl CommentsInspector {
    pub fn new(
        id: impl Into<SharedString>,
        data: CommentsInspectorViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        Self {
            filter: picker(&format!("{id}-filter"), cx, |_, event, cx| {
                if let Some(filter) = CommentsFilter::ALL
                    .into_iter()
                    .find(|v| v.label() == event.0.as_ref())
                {
                    cx.emit(Action::FilterChangeRequested { filter });
                }
            }),
            id,
            data,
            focus: cx.focus_handle(),
            composer: None,
            subscriptions: Vec::new(),
        }
    }
    pub fn view_data(&self) -> &CommentsInspectorViewData {
        &self.data
    }
    pub fn set_view_data(&mut self, data: CommentsInspectorViewData, cx: &mut Context<Self>) {
        self.data = data;
        cx.notify();
    }
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.data.can_comment {
            return;
        }
        let Some(input) = &self.composer else {
            return;
        };
        let body: SharedString = input.read(cx).value().trim().to_owned().into();
        if body.is_empty() {
            return;
        }
        if let Some(thread_id) = &self.data.selected_thread {
            cx.emit(Action::ReplyRequested {
                thread_id: thread_id.clone(),
                body,
            });
        } else {
            cx.emit(Action::CommentAddRequested { body });
        }
        input.update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }
}
impl Focusable for CommentsInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for CommentsInspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.composer.is_none() {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Leave a comment…")
                    .multi_line(true)
                    .auto_grow(2, 4)
            });
            self.subscriptions
                .push(cx.subscribe(&input, |_, _, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Change) {
                        cx.notify();
                    }
                }));
            self.composer = Some(input);
        }
        sync_picker(
            &self.filter,
            self.data.filter.label(),
            CommentsFilter::ALL
                .into_iter()
                .map(|v| InspectorChoice::new(v.label(), v.label()))
                .collect(),
            false,
            cx,
        );
        let visible = self
            .data
            .threads
            .iter()
            .filter(|thread| match self.data.filter {
                CommentsFilter::All => true,
                CommentsFilter::Open => !thread.resolved,
                CommentsFilter::Resolved => thread.resolved,
            })
            .collect::<Vec<_>>();
        let mut threads = v_flex().p(px(tokens::Space::LG)).gap(px(tokens::Space::MD));
        for thread in &visible {
            let id = thread.id.clone();
            let resolve_id = id.clone();
            let selected = self.data.selected_thread.as_ref() == Some(&id);
            let resolved = thread.resolved;
            let mut card = v_flex()
                .p(px(tokens::Space::MD))
                .gap(px(tokens::Space::SM))
                .rounded(px(tokens::Radius::CONTROL))
                .border_1()
                .border_color(
                    if selected {
                        Color::BorderSelected
                    } else {
                        Color::Border
                    }
                    .resolve(cx),
                )
                .bg(Color::Background.resolve(cx));
            card = card.child(
                h_flex()
                    .items_center()
                    .gap(px(tokens::Space::XS))
                    .child(
                        truncating_label(thread.location.clone())
                            .typography(TypographyToken::BodySmall)
                            .text_color(Color::TextSecondary.resolve(cx)),
                    )
                    .when(thread.unread, |v| {
                        v.child(
                            div()
                                .size(px(tokens::Space::XS))
                                .rounded_full()
                                .bg(Color::BackgroundSelected.resolve(cx)),
                        )
                    })
                    .child(
                        action(
                            format!("{}-resolve-{}", self.id, id).into(),
                            "",
                            if resolved {
                                LucideIcon::RotateCcw
                            } else {
                                LucideIcon::Check
                            },
                            self.data.can_comment,
                            cx,
                        )
                        .on_activate(cx.listener(move |this, _, _, cx| {
                            if this.data.can_comment {
                                cx.emit(Action::ResolveChangeRequested {
                                    id: resolve_id.clone(),
                                    resolved: !resolved,
                                });
                            }
                        })),
                    ),
            );
            for comment in thread
                .comments
                .iter()
                .take(if selected { usize::MAX } else { 1 })
            {
                let initials = comment
                    .author
                    .split_whitespace()
                    .filter_map(|s| s.chars().next())
                    .take(2)
                    .collect::<String>();
                card = card.child(
                    h_flex()
                        .gap(px(tokens::Space::SM))
                        .items_start()
                        .child(
                            div()
                                .size(px(tokens::ControlSize::CHROME))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .bg(Color::BackgroundSecondary.resolve(cx))
                                .typography(TypographyToken::BodySmallStrong)
                                .child(initials),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap(px(tokens::Space::XS))
                                .child(
                                    h_flex()
                                        .gap(px(tokens::Space::XS))
                                        .child(
                                            truncating_label(comment.author.clone())
                                                .typography(TypographyToken::BodyMediumStrong),
                                        )
                                        .child(
                                            div()
                                                .flex_none()
                                                .typography(TypographyToken::BodySmall)
                                                .text_color(Color::TextTertiary.resolve(cx))
                                                .child(comment.time_label.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .typography(TypographyToken::BodyMedium)
                                        .child(comment.body.clone()),
                                ),
                        ),
                );
            }
            card = card.child(
                action(
                    format!("{}-thread-{}", self.id, id).into(),
                    if selected {
                        "Reply to thread".to_owned()
                    } else if thread.comments.len() > 1 {
                        format!(
                            "{} {}",
                            thread.comments.len() - 1,
                            if thread.comments.len() == 2 {
                                "reply"
                            } else {
                                "replies"
                            }
                        )
                    } else {
                        "Reply".to_owned()
                    },
                    LucideIcon::MessageCircle,
                    true,
                    cx,
                )
                .on_activate(cx.listener(move |_, _, _, cx| {
                    cx.emit(Action::ThreadSelectRequested { id: id.clone() })
                })),
            );
            threads = threads.child(card);
        }
        let send_enabled = self.data.can_comment
            && !self
                .composer
                .as_ref()
                .unwrap()
                .read(cx)
                .value()
                .trim()
                .is_empty();
        v_flex()
            .id(self.id.clone())
            .key_context(super::PROPERTIES_TABS_KEY_CONTEXT)
            .track_focus(&self.focus)
            .size_full()
            .min_w_0()
            .bg(Color::Background.resolve(cx))
            .text_color(Color::Text.resolve(cx))
            .occlude()
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(section("Comments", cx).child(body().child(self.filter.clone())))
            .child(
                v_flex()
                    .id(format!("{}-threads", self.id))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .child(threads)
                    .when(visible.is_empty(), |v| {
                        v.child(empty(
                            LucideIcon::MessageCircle,
                            "No comments here",
                            "Comments and replies for this view will appear here.",
                            cx,
                        ))
                    }),
            )
            .child(
                v_flex()
                    .p(px(tokens::Space::LG))
                    .gap(px(tokens::Space::SM))
                    .border_t_1()
                    .border_color(Color::Border.resolve(cx))
                    .when(self.data.selected_thread.is_some(), |v| {
                        v.child(
                            h_flex()
                                .items_center()
                                .gap(px(tokens::Space::SM))
                                .child(
                                    truncating_label("Replying to thread")
                                        .typography(TypographyToken::BodySmallStrong)
                                        .text_color(Color::TextSecondary.resolve(cx)),
                                )
                                .child(
                                    action(
                                        format!("{}-new-comment", self.id).into(),
                                        "New comment",
                                        LucideIcon::Plus,
                                        true,
                                        cx,
                                    )
                                    .on_activate(cx.listener(
                                        |_, _, _, cx| cx.emit(Action::ThreadClearRequested),
                                    )),
                                ),
                        )
                    })
                    .child(
                        Input::new(self.composer.as_ref().unwrap())
                            .typography(TypographyToken::BodyMedium)
                            .small()
                            .disabled(!self.data.can_comment),
                    )
                    .child(
                        action(
                            format!("{}-send", self.id).into(),
                            if self.data.selected_thread.is_some() {
                                "Send reply"
                            } else {
                                "Post comment"
                            },
                            LucideIcon::Send,
                            send_enabled,
                            cx,
                        )
                        .w_full()
                        .bg(Color::BackgroundSecondary.resolve(cx))
                        .on_activate(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
            )
    }
}
