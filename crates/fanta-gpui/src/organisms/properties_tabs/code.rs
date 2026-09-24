use super::{CodeInspectorAction as Action, CodeInspectorViewData, controls::*};
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _, TypographyToken,
    tokens, truncating_label,
};
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

pub struct CodeInspector {
    id: SharedString,
    data: CodeInspectorViewData,
    focus: FocusHandle,
    language: Entity<Picker>,
}
impl EventEmitter<Action> for CodeInspector {}
impl CodeInspector {
    pub fn new(
        id: impl Into<SharedString>,
        data: CodeInspectorViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        Self {
            language: picker(&format!("{id}-language"), cx, |_, event, cx| {
                cx.emit(Action::LanguageChangeRequested {
                    language: event.0.clone(),
                })
            }),
            id,
            data,
            focus: cx.focus_handle(),
        }
    }
    pub fn view_data(&self) -> &CodeInspectorViewData {
        &self.data
    }
    pub fn set_view_data(&mut self, data: CodeInspectorViewData, cx: &mut Context<Self>) {
        self.data = data;
        cx.notify();
    }
}
impl Focusable for CodeInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for CodeInspector {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_picker(
            &self.language,
            self.data.language.clone(),
            self.data.languages.clone(),
            self.data.languages.is_empty(),
            cx,
        );
        let mut lines = v_flex()
            .gap(px(tokens::Space::XS))
            .py(px(tokens::Space::MD));
        for (index, line) in self.data.code.lines().enumerate() {
            lines = lines.child(
                h_flex()
                    .items_start()
                    .gap(px(tokens::Space::MD))
                    .child(
                        div()
                            .w(px(tokens::ControlSize::TOOL))
                            .flex_none()
                            .text_right()
                            .text_color(Color::TextTertiary.resolve(cx))
                            .child((index + 1).to_string()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .font_family(cx.theme().mono_font_family.clone())
                            .when(!self.data.wrap_lines, |v| v.whitespace_nowrap())
                            .child(line.to_owned()),
                    ),
            );
        }
        let mut properties = body();
        for property in &self.data.properties {
            let id = property.id.clone();
            properties = properties.child(
                h_flex()
                    .items_center()
                    .gap(px(tokens::Space::SM))
                    .child(
                        truncating_label(property.id.clone())
                            .typography(TypographyToken::BodyMedium)
                            .text_color(Color::TextSecondary.resolve(cx)),
                    )
                    .child(
                        truncating_label(property.label.clone())
                            .typography(TypographyToken::BodyMedium),
                    )
                    .child(
                        action(
                            format!("{}-copy-{}", self.id, property.id).into(),
                            "",
                            LucideIcon::Copy,
                            true,
                            cx,
                        )
                        .on_activate(cx.listener(move |_, _, _, cx| {
                            cx.emit(Action::CopyPropertyRequested {
                                property_id: id.clone(),
                            })
                        })),
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
            .bg(Color::Background.resolve(cx))
            .text_color(Color::Text.resolve(cx))
            .typography(TypographyToken::BodyMedium)
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .when(!self.data.selection_name.is_empty(), |v| {
                v.child(selection(
                    self.data.selection_name.clone(),
                    LucideIcon::Code,
                    cx,
                ))
            })
            .child(
                section("Code", cx).child(
                    body().child(self.language.clone()).child(
                        h_flex()
                            .gap(px(tokens::Space::SM))
                            .child(
                                action(
                                    format!("{}-copy", self.id).into(),
                                    "Copy code",
                                    LucideIcon::Copy,
                                    !self.data.code.is_empty(),
                                    cx,
                                )
                                .flex_1()
                                .bg(Color::BackgroundSecondary.resolve(cx))
                                .on_activate(cx.listener(
                                    |this, _, _, cx| {
                                        if !this.data.code.is_empty() {
                                            cx.emit(Action::CopyRequested {
                                                code: this.data.code.clone(),
                                            });
                                        }
                                    },
                                )),
                            )
                            .child(
                                action(
                                    format!("{}-wrap", self.id).into(),
                                    "Wrap",
                                    LucideIcon::TextWrap,
                                    true,
                                    cx,
                                )
                                .when(self.data.wrap_lines, |v| {
                                    v.bg(Color::BackgroundSecondary.resolve(cx))
                                })
                                .on_activate(cx.listener(
                                    |this, _, _, cx| {
                                        cx.emit(Action::WrapLinesChangeRequested {
                                            enabled: !this.data.wrap_lines,
                                        })
                                    },
                                )),
                            ),
                    ),
                ),
            )
            .when(!self.data.code.is_empty(), |v| {
                v.child(
                    div()
                        .id(format!("{}-source", self.id))
                        .flex_none()
                        .w_full()
                        .overflow_x_scroll()
                        .bg(Color::BackgroundSecondary.resolve(cx))
                        .child(lines),
                )
            })
            .when(self.data.code.is_empty(), |v| {
                v.child(empty(
                    LucideIcon::Code,
                    "Inspect implementation",
                    "Select a layer to see code supplied by your project.",
                    cx,
                ))
            })
            .when(!self.data.properties.is_empty(), |v| {
                v.child(section("Properties", cx).child(properties))
            })
    }
}
