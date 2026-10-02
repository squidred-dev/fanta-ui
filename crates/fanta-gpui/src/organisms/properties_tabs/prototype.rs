use super::{PrototypeInspectorAction as Action, PrototypeInspectorViewData, controls::*};
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _, TypographyToken,
    icon_button, render_lucide_icon, tokens, truncating_label,
};
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, tooltip::Tooltip, v_flex};

pub struct PrototypeInspector {
    id: SharedString,
    data: PrototypeInspectorViewData,
    focus: FocusHandle,
    show_presentation_action: bool,
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
            show_presentation_action: true,
        }
    }
    /// Composed sidebars place the play control in their shared header.
    pub fn set_show_presentation_action(&mut self, show: bool, cx: &mut Context<Self>) {
        if self.show_presentation_action != show {
            self.show_presentation_action = show;
            cx.notify();
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
                    .border_color(Color::BorderPanel.resolve(cx))
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(tokens::Space::SM))
                            .child(
                                truncating_label(format!(
                                    "{} → {}",
                                    connection.trigger, connection.destination
                                ))
                                .typography(TypographyToken::PanelStrong),
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
                            .typography(TypographyToken::PanelCaption)
                            .text_color(Color::TextSecondary.resolve(cx))
                            .child(format!("{} · {}", connection.action, connection.animation)),
                    ),
            );
        }
        v_flex()
            .id(self.id.clone())
            .key_context(super::PROPERTIES_TABS_KEY_CONTEXT)
            .track_focus(&self.focus)
            .size_full()
            .min_w_0()
            .overflow_y_scroll()
            .occlude()
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .bg(Color::BackgroundPanel.resolve(cx))
            .text_color(crate::atoms::sidebar_style(cx).text)
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .when(
                !self.data.selection_name.is_empty() || self.show_presentation_action,
                |v| {
                    v.child(
                        h_flex()
                            .w_full()
                            .h(px(tokens::InspectorGeometry::SECTION_HEADER))
                            .px(px(tokens::Space::LG))
                            .gap(px(tokens::Space::SM))
                            .items_center()
                            .border_b_1()
                            .border_color(Color::BorderPanel.resolve(cx))
                            .child(render_lucide_icon(
                                LucideIcon::Workflow,
                                crate::atoms::sidebar_style(cx).muted_icon,
                                tokens::IconSize::SM,
                            ))
                            .child(
                                truncating_label(if self.data.selection_name.is_empty() {
                                    "Prototype".into()
                                } else {
                                    self.data.selection_name.clone()
                                })
                                .typography(TypographyToken::PanelStrong)
                                .flex_1(),
                            )
                            .when(self.show_presentation_action, |header| {
                                header.child(
                                    icon_button(
                                        format!("{}-present", self.id),
                                        px(tokens::ControlSize::CHROME),
                                        px(tokens::Radius::CONTROL),
                                        cx,
                                    )
                                    .debug_selector({
                                        let id = format!("{}-present", self.id);
                                        move || id.clone()
                                    })
                                    .tab_index(if self.data.can_present { 0 } else { -1 })
                                    .opacity(if self.data.can_present { 1. } else { 0.45 })
                                    .tooltip(|window, cx| {
                                        Tooltip::new("Play prototype").build(window, cx)
                                    })
                                    .on_activate(cx.listener(|this, _, _, cx| {
                                        if this.data.can_present {
                                            cx.emit(Action::PresentRequested);
                                        }
                                    }))
                                    .child(render_lucide_icon(
                                        LucideIcon::Play,
                                        crate::atoms::sidebar_style(cx).icon,
                                        tokens::IconSize::MD,
                                    )),
                                )
                            }),
                    )
                },
            )
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
                                .bg(Color::BackgroundPanelField.resolve(cx))
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
                        .pb(px(tokens::InspectorGeometry::BODY_BOTTOM))
                        .child(
                            action(
                                format!("{}-add", self.id).into(),
                                "Add interaction",
                                LucideIcon::Plus,
                                !disabled && !self.data.selection_name.is_empty(),
                                cx,
                            )
                            .w_full()
                            .bg(Color::BackgroundPanelField.resolve(cx))
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
