use super::{PrototypeInspectorAction as Action, PrototypeInspectorViewData, controls::*};
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticButtonIconAlignment, SemanticButtonSize,
    SemanticButtonVariant, SemanticColor as Color, TypographyExt as _, TypographyToken,
    semantic_button, tokens, truncating_label,
};
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, v_flex};

pub struct PrototypeInspector {
    id: SharedString,
    data: PrototypeInspectorViewData,
    focus: FocusHandle,
    device: Entity<Picker>,
    background: Entity<Entry>,
    flow: Entity<Entry>,
}
impl EventEmitter<Action> for PrototypeInspector {}
impl PrototypeInspector {
    pub fn new(
        id: impl Into<SharedString>,
        data: PrototypeInspectorViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        Self {
            device: picker(&format!("{id}-device"), cx, |this, event, cx| {
                if !this.data.read_only {
                    cx.emit(Action::DeviceChangeRequested {
                        id: event.0.clone(),
                    });
                }
            }),
            background: entry(
                &format!("{id}-background"),
                EntryKind::Color,
                cx,
                |this, event, cx| {
                    if !this.data.read_only {
                        cx.emit(Action::BackgroundChangeRequested {
                            hex: event.0.clone(),
                        });
                    }
                },
            ),
            flow: entry(
                &format!("{id}-flow"),
                EntryKind::Text,
                cx,
                |this, event, cx| {
                    if !this.data.read_only {
                        cx.emit(Action::FlowRenameRequested {
                            name: event.0.clone(),
                        });
                    }
                },
            ),
            id,
            data,
            focus: cx.focus_handle(),
        }
    }
    pub fn view_data(&self) -> &PrototypeInspectorViewData {
        &self.data
    }
    pub fn set_view_data(&mut self, data: PrototypeInspectorViewData, cx: &mut Context<Self>) {
        self.data = data;
        cx.notify();
    }
}
impl Focusable for PrototypeInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for PrototypeInspector {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = self.data.read_only;
        sync_picker(
            &self.device,
            self.data.device.clone(),
            self.data.devices.clone(),
            disabled,
            cx,
        );
        sync_entry(
            &self.background,
            self.data.background_hex.clone(),
            disabled,
            cx,
        );
        sync_entry(
            &self.flow,
            self.data.flow_name.clone().unwrap_or_default(),
            disabled,
            cx,
        );
        let mut interactions = body();
        for connection in &self.data.connections {
            let edit_id = connection.id.clone();
            let remove_id = connection.id.clone();
            interactions = interactions.child(
                v_flex()
                    .p(px(tokens::Space::SM))
                    .gap(px(tokens::Space::XS))
                    .rounded(px(tokens::Radius::CONTROL))
                    .border_1()
                    .border_color(Color::Border.resolve(cx))
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(tokens::Space::SM))
                            .child(
                                truncating_label(format!(
                                    "{} → {}",
                                    connection.trigger, connection.destination
                                ))
                                .typography(TypographyToken::BodyMediumStrong),
                            )
                            .child(
                                action(
                                    format!("{}-edit-{}", self.id, connection.id).into(),
                                    "",
                                    LucideIcon::Settings2,
                                    !disabled,
                                    cx,
                                )
                                .on_activate(cx.listener(
                                    move |this, _, _, cx| {
                                        if !this.data.read_only {
                                            cx.emit(Action::ConnectionEditRequested {
                                                id: edit_id.clone(),
                                            });
                                        }
                                    },
                                )),
                            )
                            .child(
                                action(
                                    format!("{}-remove-{}", self.id, connection.id).into(),
                                    "",
                                    LucideIcon::Minus,
                                    !disabled,
                                    cx,
                                )
                                .on_activate(cx.listener(
                                    move |this, _, _, cx| {
                                        if !this.data.read_only {
                                            cx.emit(Action::ConnectionRemoveRequested {
                                                id: remove_id.clone(),
                                            });
                                        }
                                    },
                                )),
                            ),
                    )
                    .child(
                        div()
                            .typography(TypographyToken::BodySmall)
                            .text_color(Color::TextSecondary.resolve(cx))
                            .child(format!("{} · {}", connection.action, connection.animation)),
                    ),
            );
        }
        v_flex()
            .id(self.id.clone())
            .track_focus(&self.focus)
            .size_full()
            .min_w_0()
            .overflow_y_scroll()
            .occlude()
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .bg(Color::Background.resolve(cx))
            .text_color(Color::Text.resolve(cx))
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .when(!self.data.selection_name.is_empty(), |v| {
                v.child(selection(
                    self.data.selection_name.clone(),
                    LucideIcon::Workflow,
                    cx,
                ))
            })
            .child(
                section("Flow starting point", cx).child(
                    body()
                        .when(self.data.flow_name.is_some(), |v| {
                            v.child(row("Name", self.flow.clone(), cx))
                        })
                        .when(self.data.flow_name.is_none(), |v| {
                            v.child(
                                action(
                                    format!("{}-flow-start", self.id).into(),
                                    "Set as starting point",
                                    LucideIcon::Flag,
                                    !disabled && self.data.can_start_flow,
                                    cx,
                                )
                                .w_full()
                                .bg(Color::BackgroundSecondary.resolve(cx))
                                .on_activate(cx.listener(
                                    |this, _, _, cx| {
                                        if !this.data.read_only && this.data.can_start_flow {
                                            cx.emit(Action::FlowStartRequested);
                                        }
                                    },
                                )),
                            )
                        }),
                ),
            )
            .child(
                section("Interactions", cx).child(interactions).child(
                    div()
                        .px(px(tokens::Space::LG))
                        .pb(px(tokens::Space::LG))
                        .child(
                            action(
                                format!("{}-add", self.id).into(),
                                "Add interaction",
                                LucideIcon::Plus,
                                !disabled && !self.data.selection_name.is_empty(),
                                cx,
                            )
                            .w_full()
                            .bg(Color::BackgroundSecondary.resolve(cx))
                            .on_activate(cx.listener(
                                |this, _, _, cx| {
                                    if !this.data.read_only && !this.data.selection_name.is_empty()
                                    {
                                        cx.emit(Action::ConnectionAddRequested);
                                    }
                                },
                            )),
                        ),
                ),
            )
            .child(
                section("Prototype settings", cx).child(
                    body()
                        .child(row("Device", self.device.clone(), cx))
                        .child(row("Background", self.background.clone(), cx)),
                ),
            )
            .child(
                div().p(px(tokens::Space::LG)).child(
                    semantic_button(
                        format!("{}-present", self.id),
                        SemanticButtonVariant::Primary,
                        SemanticButtonSize::Large,
                    )
                    .label("Present prototype")
                    .icon(LucideIcon::Play, SemanticButtonIconAlignment::Left)
                    .full_width(true)
                    .disabled(!self.data.can_present)
                    .on_activate(cx.listener(|this, _, _, cx| {
                        if this.data.can_present {
                            cx.emit(Action::PresentRequested);
                        }
                    })),
                ),
            )
            .when(self.data.selection_name.is_empty(), |v| {
                v.child(empty(
                    LucideIcon::Workflow,
                    "Connect your screens",
                    "Select a frame to add interactions or define a flow starting point.",
                    cx,
                ))
            })
    }
}
