//! Shader discovery, binding, and property-editor rendering.

use super::*;

impl PaintPicker {
    pub(super) fn reset_shader_input_edit_sessions(&mut self) {
        self.shader_active_number_property = None;
        self.shader_active_color_property = None;
        self.shader_number_edit_session = None;
        self.shader_color_edit_session = None;
        self.shader_number_invalid = false;
        self.shader_color_invalid = false;
    }

    fn shader_property_value(&self, definition_id: &str) -> Option<DesignShaderPropertyValue> {
        let DesignPaintPayload::Shader(shader) = &self.paint.as_ref()?.payload else {
            return None;
        };
        let definition = self.current_shader_definition()?.property(definition_id)?;
        shader
            .property(definition_id)
            .or(definition.default_value.as_ref())
            .cloned()
    }

    fn shader_property_edit(
        &self,
        definition_id: &SharedString,
        value: DesignShaderPropertyValue,
    ) -> Option<DesignPaintEdit> {
        let definition = self.current_shader_definition()?.property(definition_id)?;
        if !value.is_compatible_with(definition.kind) {
            return None;
        }
        Some(DesignPaintEdit {
            property: DesignPaintProperty::ShaderProperty {
                definition_id: definition_id.clone(),
            },
            value: DesignPaintValue::ShaderProperty(value),
        })
    }

    pub(super) fn sync_shader_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let number = self
            .shader_active_number_property
            .as_ref()
            .and_then(|id| self.shader_property_value(id))
            .and_then(|value| match value {
                DesignShaderPropertyValue::Number(value) => Some(format_decimal(value)),
                _ => None,
            });
        let color = self
            .shader_active_color_property
            .as_ref()
            .and_then(|id| self.shader_property_value(id))
            .and_then(|value| match value {
                DesignShaderPropertyValue::Color(value) => Some(rgba_hex(value)),
                _ => None,
            });
        self.suppress_input_events = true;
        if let Some(number) = number
            && !self.shader_number_input.focus_handle(cx).is_focused(window)
        {
            self.shader_number_input.update(cx, |input, cx| {
                input.set_value(number, window, cx);
            });
        }
        if let Some(color) = color
            && !self.shader_color_input.focus_handle(cx).is_focused(window)
        {
            self.shader_color_input.update(cx, |input, cx| {
                input.set_value(color, window, cx);
            });
        }
        self.suppress_input_events = false;
    }

    fn open_shader_number_editor(
        &mut self,
        definition_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled() {
            return;
        }
        let Some(DesignShaderPropertyValue::Number(value)) =
            self.shader_property_value(&definition_id)
        else {
            return;
        };
        self.cancel_text_input_edit_sessions(cx);
        self.shader_active_number_property = Some(definition_id);
        self.shader_active_color_property = None;
        self.shader_number_invalid = false;
        self.suppress_input_events = true;
        self.shader_number_input.update(cx, |input, cx| {
            input.set_value(format_decimal(value), window, cx);
        });
        self.suppress_input_events = false;
        self.shader_number_input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    fn open_shader_color_editor(
        &mut self,
        definition_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled() {
            return;
        }
        let Some(DesignShaderPropertyValue::Color(color)) =
            self.shader_property_value(&definition_id)
        else {
            return;
        };
        self.cancel_text_input_edit_sessions(cx);
        self.shader_active_color_property = Some(definition_id);
        self.shader_active_number_property = None;
        self.shader_color_invalid = false;
        self.suppress_input_events = true;
        self.shader_color_input.update(cx, |input, cx| {
            input.set_value(rgba_hex(color), window, cx);
        });
        self.suppress_input_events = false;
        self.shader_color_input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    fn restore_shader_input(
        &mut self,
        number: bool,
        session: Option<&DesignPaintEdit>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let property = if number {
            self.shader_active_number_property.as_ref()
        } else {
            self.shader_active_color_property.as_ref()
        };
        let session_value = session.and_then(|edit| match &edit.value {
            DesignPaintValue::ShaderProperty(value) => Some(value.clone()),
            _ => None,
        });
        let Some(value) =
            session_value.or_else(|| property.and_then(|id| self.shader_property_value(id)))
        else {
            return;
        };
        let text = match value {
            DesignShaderPropertyValue::Number(value) if number => format_decimal(value),
            DesignShaderPropertyValue::Color(value) if !number => rgba_hex(value),
            _ => return,
        };
        self.suppress_input_events = true;
        if number {
            self.shader_number_input.update(cx, |input, cx| {
                input.set_value(text, window, cx);
            });
            self.shader_number_invalid = false;
        } else {
            self.shader_color_input.update(cx, |input, cx| {
                input.set_value(text, window, cx);
            });
            self.shader_color_invalid = false;
        }
        self.suppress_input_events = false;
        cx.notify();
    }

    fn parsed_shader_color(&self, text: &str, definition_id: &SharedString) -> Option<DesignColor> {
        let parsed = parse_hex_color_input(text)?;
        let mut color = parsed.color;
        if !parsed.explicit_alpha {
            let DesignShaderPropertyValue::Color(previous) =
                self.shader_property_value(definition_id)?
            else {
                return None;
            };
            color.alpha = previous.alpha;
        }
        Some(color)
    }

    pub(super) fn handle_shader_number_input(
        &mut self,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Focus) {
            self.dismissal_event_guard = false;
        }
        if self.dismissal_event_guard || self.suppress_input_events || self.editing_disabled() {
            return;
        }
        let Some(definition_id) = self.shader_active_number_property.clone() else {
            return;
        };
        let value = self.shader_number_input.read(cx).value();
        let parsed = value
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|number| number.is_finite());
        let candidate = parsed.and_then(|number| {
            self.shader_property_edit(&definition_id, DesignShaderPropertyValue::Number(number))
        });
        match event {
            InputEvent::Change => {
                if !self.shader_number_input.focus_handle(cx).is_focused(window) {
                    return;
                }
                if self.shader_number_edit_session.is_none()
                    && let Some(original) = self
                        .shader_property_value(&definition_id)
                        .and_then(|value| self.shader_property_edit(&definition_id, value))
                {
                    self.shader_number_edit_session = self.begin_text_input_edit(original, cx);
                }
                self.shader_number_invalid = candidate.is_none();
                if let Some(candidate) = candidate {
                    self.emit_edit(candidate, DesignPanelEditPhase::Preview, cx);
                } else {
                    cx.notify();
                }
            }
            InputEvent::PressEnter { .. } | InputEvent::Blur => {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    window.prevent_default();
                }
                let session = self.shader_number_edit_session.take();
                if self.shader_number_invalid || candidate.is_none() {
                    self.restore_shader_input(true, session.as_ref(), window, cx);
                }
                if let Some(session) = session {
                    self.finish_text_input_edit(session, candidate, cx);
                }
            }
            _ => {}
        }
    }

    pub(super) fn handle_shader_color_input(
        &mut self,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Focus) {
            self.dismissal_event_guard = false;
        }
        if self.dismissal_event_guard || self.suppress_input_events || self.editing_disabled() {
            return;
        }
        let Some(definition_id) = self.shader_active_color_property.clone() else {
            return;
        };
        let value = self.shader_color_input.read(cx).value();
        let candidate = self
            .parsed_shader_color(value.as_ref(), &definition_id)
            .and_then(|color| {
                self.shader_property_edit(&definition_id, DesignShaderPropertyValue::Color(color))
            });
        match event {
            InputEvent::Change => {
                if !self.shader_color_input.focus_handle(cx).is_focused(window) {
                    return;
                }
                if self.shader_color_edit_session.is_none()
                    && let Some(original) = self
                        .shader_property_value(&definition_id)
                        .and_then(|value| self.shader_property_edit(&definition_id, value))
                {
                    self.shader_color_edit_session = self.begin_text_input_edit(original, cx);
                }
                self.shader_color_invalid = candidate.is_none();
                if let Some(candidate) = candidate {
                    self.emit_edit(candidate, DesignPanelEditPhase::Preview, cx);
                } else {
                    cx.notify();
                }
            }
            InputEvent::PressEnter { .. } | InputEvent::Blur => {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    window.prevent_default();
                }
                let session = self.shader_color_edit_session.take();
                if self.shader_color_invalid || candidate.is_none() {
                    self.restore_shader_input(false, session.as_ref(), window, cx);
                }
                if let Some(session) = session {
                    self.finish_text_input_edit(session, candidate, cx);
                }
            }
            _ => {}
        }
    }

    fn choose_shader_color(
        &mut self,
        definition_id: SharedString,
        color: DesignColor,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled() {
            return;
        }
        self.cancel_text_input_edit_sessions(cx);
        if let Some(edit) =
            self.shader_property_edit(&definition_id, DesignShaderPropertyValue::Color(color))
        {
            self.emit_edit(edit, DesignPanelEditPhase::Commit, cx);
        }
    }

    pub(super) fn render_shader_row(
        &self,
        shader: &DesignShaderDefinition,
        selection: DesignShaderSelection,
        selected_shader_id: Option<&SharedString>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = selected_shader_id.is_some_and(|shader_id| *shader_id == shader.id);
        let selection_for_click = selection.clone();
        h_flex()
            .w_full()
            .min_h(px(34.))
            .gap_2()
            .px_2()
            .rounded(px(6.))
            .when(selected, |row| {
                row.bg(crate::atoms::SemanticColor::BackgroundHover
                    .resolve(cx)
                    .opacity(0.12))
            })
            .child(
                Icon::new(IconName::Asterisk)
                    .xsmall()
                    .text_color(if selected {
                        crate::atoms::SemanticColor::BackgroundHover.resolve(cx)
                    } else {
                        crate::atoms::SemanticColor::TextTertiary.resolve(cx)
                    }),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .child(
                        div()
                            .truncate()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .font_semibold()
                            .child(shader.name.clone()),
                    )
                    .child(
                        div()
                            .truncate()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(if shader.imported {
                                format!(
                                    "Imported · {} properties",
                                    shader.property_definitions.len()
                                )
                            } else {
                                "Available · import required".to_owned()
                            }),
                    ),
            )
            .child(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-shader-{}-{}",
                    self.id,
                    if shader.imported { "apply" } else { "import" },
                    shader.id
                )))
                .label(if shader.imported { "Apply" } else { "Import" })
                .xsmall()
                .compact()
                .outline()
                .disabled(self.editing_disabled() || (selected && shader.imported))
                .on_activate(cx.listener(move |this, _, _, cx| {
                    this.request_shader_selection(selection_for_click.clone(), cx);
                })),
            )
            .into_any_element()
    }

    pub(super) fn render_shader_browser(
        &self,
        paint: &DesignPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected_shader_id = match &paint.payload {
            DesignPaintPayload::Shader(shader) => Some(&shader.shader_id),
            _ => None,
        };
        if self.shader_view_data.page_shaders.is_empty()
            && self.shader_view_data.libraries.is_empty()
        {
            return v_flex()
                .w_full()
                .min_h(px(180.))
                .items_center()
                .justify_center()
                .gap_2()
                .p_4()
                .child(
                    Icon::new(IconName::Asterisk)
                        .small()
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx)),
                )
                .child(
                    div()
                        .typography(crate::atoms::TypographyToken::BodyLarge)
                        .font_semibold()
                        .child("Shaders"),
                )
                .child(
                    div()
                        .text_center()
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child("No fill shaders supplied by the host."),
                )
                .into_any_element();
        }

        let mut content = v_flex().w_full().gap_3().p_4();
        if !self.shader_view_data.page_shaders.is_empty() {
            let mut page = v_flex().w_full().gap_1().child(
                div()
                    .typography(crate::atoms::TypographyToken::BodyMedium)
                    .font_semibold()
                    .child("In this file"),
            );
            for shader in &self.shader_view_data.page_shaders {
                page = page.child(self.render_shader_row(
                    shader,
                    DesignShaderSelection::page(shader.id.clone()),
                    selected_shader_id,
                    cx,
                ));
            }
            content = content.child(page);
        }
        for library in &self.shader_view_data.libraries {
            let library_id = library.id.clone();
            let mut group = v_flex().w_full().gap_1().child(
                h_flex()
                    .w_full()
                    .h(px(24.))
                    .gap_2()
                    .child(
                        Icon::new(IconName::BookOpen)
                            .xsmall()
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .font_semibold()
                            .child(library.name.clone()),
                    )
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(library.shaders.len().to_string()),
                    ),
            );
            for shader in &library.shaders {
                group = group.child(self.render_shader_row(
                    shader,
                    DesignShaderSelection::library(library_id.clone(), shader.id.clone()),
                    selected_shader_id,
                    cx,
                ));
            }
            content = content.child(group);
        }
        content.into_any_element()
    }

    pub(super) fn current_shader_definition(&self) -> Option<&DesignShaderDefinition> {
        let DesignPaintPayload::Shader(shader) = &self.paint.as_ref()?.payload else {
            return None;
        };
        self.shader_view_data.definition(&shader.shader_id)
    }

    pub(super) fn request_shader_selection(
        &self,
        shader: DesignShaderSelection,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled() {
            return;
        }
        let (Some(target), Some(definition)) =
            (self.target.clone(), self.shader_view_data.shader(&shader))
        else {
            return;
        };
        if definition.imported {
            cx.emit(PaintPickerEvent::ShaderApplyRequested { target, shader });
        } else {
            cx.emit(PaintPickerEvent::ShaderImportRequested { target, shader });
        }
    }

    pub(super) fn request_shader_property_bind(
        &self,
        definition_id: SharedString,
        cx: &mut Context<Self>,
    ) {
        if !self.shader_variable_binding_enabled
            || self.editing_disabled()
            || self
                .current_shader_definition()
                .is_none_or(|shader| shader.property(&definition_id).is_none())
        {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        cx.emit(PaintPickerEvent::ShaderPropertyBindRequested {
            target,
            definition_id,
        });
    }

    pub(super) fn request_shader_property_editor(
        &self,
        definition_id: SharedString,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled()
            || self
                .current_shader_definition()
                .is_none_or(|shader| shader.property(&definition_id).is_none())
        {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        cx.emit(PaintPickerEvent::ShaderPropertyEditorRequested {
            target,
            definition_id,
        });
    }

    pub(super) fn request_shader_property_detach(
        &self,
        definition_id: SharedString,
        variable_id: SharedString,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled() {
            return;
        }
        let Some(DesignPaintPayload::Shader(shader)) =
            self.paint.as_ref().map(|paint| &paint.payload)
        else {
            return;
        };
        if shader
            .property(&definition_id)
            .and_then(DesignShaderPropertyValue::variable_alias_id)
            .is_none_or(|current| current != &variable_id)
        {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        cx.emit(PaintPickerEvent::ShaderPropertyDetachRequested {
            target,
            definition_id,
            variable_id,
        });
    }

    pub(super) fn render_shader_property(
        &self,
        shader: &DesignShaderPaint,
        definition: &DesignShaderPropertyDefinition,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let value = shader
            .property(&definition.id)
            .or(definition.default_value.as_ref());
        let summary = value.map_or_else(|| "No value".into(), DesignShaderPropertyValue::summary);
        let disabled = self.editing_disabled();
        let definition_id = definition.id.clone();
        let variable_id = value
            .and_then(DesignShaderPropertyValue::variable_alias_id)
            .cloned();

        let value_control = match value {
            Some(DesignShaderPropertyValue::Boolean(current)) if variable_id.is_none() => {
                let next = !current;
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-shader-property-{}-boolean",
                    self.id, definition.id
                )))
                .label(if *current { "On" } else { "Off" })
                .xsmall()
                .compact()
                .outline()
                .disabled(disabled)
                .on_activate({
                    let definition_id = definition_id.clone();
                    cx.listener(move |this, _, _, cx| {
                        let _ = this.emit_edit(
                            DesignPaintEdit {
                                property: DesignPaintProperty::ShaderProperty {
                                    definition_id: definition_id.clone(),
                                },
                                value: DesignPaintValue::ShaderProperty(
                                    DesignShaderPropertyValue::Boolean(next),
                                ),
                            },
                            DesignPanelEditPhase::Commit,
                            cx,
                        );
                    })
                })
                .into_any_element()
            }
            Some(DesignShaderPropertyValue::Number(current))
                if variable_id.is_none() && current.is_finite() =>
            {
                if self.shader_active_number_property.as_ref() == Some(&definition.id) {
                    div()
                        .debug_selector(|| "shader-number-input".to_owned())
                        .child(
                            Input::new(&self.shader_number_input)
                                .typography(crate::atoms::TypographyToken::BodyMedium)
                                .xsmall()
                                .h(px(26.))
                                .w(px(74.))
                                .disabled(disabled)
                                .when(self.shader_number_invalid, |input| {
                                    input.border_color(
                                        crate::atoms::SemanticColor::TextDanger.resolve(cx),
                                    )
                                }),
                        )
                        .into_any_element()
                } else {
                    let button_id = SharedString::from(format!(
                        "{}-shader-property-{}-number",
                        self.id, definition.id
                    ));
                    let debug_id = button_id.to_string();
                    crate::atoms::ui_button(button_id)
                        .debug_selector(move || debug_id.clone())
                        .label(format_decimal(*current))
                        .xsmall()
                        .compact()
                        .outline()
                        .disabled(disabled)
                        .on_activate({
                            let definition_id = definition_id.clone();
                            cx.listener(move |this, _, window, cx| {
                                this.open_shader_number_editor(definition_id.clone(), window, cx);
                            })
                        })
                        .into_any_element()
                }
            }
            Some(DesignShaderPropertyValue::Color(current)) if variable_id.is_none() => {
                let button_id = SharedString::from(format!(
                    "{}-shader-property-{}-color",
                    self.id, definition.id
                ));
                let debug_id = button_id.to_string();
                crate::atoms::ui_button(button_id)
                    .debug_selector(move || debug_id.clone())
                    .label(format!("#{}", current.hex()))
                    .child(
                        div()
                            .size(px(16.))
                            .rounded(px(3.))
                            .border_1()
                            .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
                            .bg(color_to_hsla(*current)),
                    )
                    .xsmall()
                    .compact()
                    .outline()
                    .disabled(disabled)
                    .on_activate({
                        let definition_id = definition_id.clone();
                        cx.listener(move |this, _, window, cx| {
                            this.open_shader_color_editor(definition_id.clone(), window, cx);
                        })
                    })
                    .into_any_element()
            }
            _ => crate::atoms::ui_button(SharedString::from(format!(
                "{}-shader-property-{}-edit",
                self.id, definition.id
            )))
            .label(summary)
            .xsmall()
            .compact()
            .outline()
            .disabled(disabled || variable_id.is_some())
            .on_activate({
                let definition_id = definition_id.clone();
                cx.listener(move |this, _, _, cx| {
                    this.request_shader_property_editor(definition_id.clone(), cx);
                })
            })
            .into_any_element(),
        };

        let variable_control = if let Some(variable_id) = variable_id {
            Some(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-shader-property-{}-detach",
                    self.id, definition.id
                )))
                .label("Detach")
                .tooltip(format!("Detach variable {variable_id}"))
                .xsmall()
                .compact()
                .ghost()
                .disabled(disabled)
                .on_activate({
                    let definition_id = definition_id.clone();
                    cx.listener(move |this, _, _, cx| {
                        this.request_shader_property_detach(
                            definition_id.clone(),
                            variable_id.clone(),
                            cx,
                        );
                    })
                })
                .into_any_element(),
            )
        } else if self.shader_variable_binding_enabled {
            let button_id = SharedString::from(format!(
                "{}-shader-property-{}-bind",
                self.id, definition.id
            ));
            let debug_id = button_id.to_string();
            Some(
                crate::atoms::ui_button(button_id)
                    .debug_selector(move || debug_id.clone())
                    .label("Bind")
                    .xsmall()
                    .compact()
                    .ghost()
                    .disabled(disabled)
                    .on_activate({
                        let definition_id = definition_id.clone();
                        cx.listener(move |this, _, _, cx| {
                            this.request_shader_property_bind(definition_id.clone(), cx);
                        })
                    })
                    .into_any_element(),
            )
        } else {
            None
        };

        v_flex()
            .w_full()
            .gap_1()
            .py_1()
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w(px(0.))
                            .child(
                                div()
                                    .truncate()
                                    .typography(crate::atoms::TypographyToken::BodyMedium)
                                    .font_semibold()
                                    .child(definition.name.clone()),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .typography(crate::atoms::TypographyToken::BodyMedium)
                                    .text_color(
                                        crate::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(format!(
                                        "{} · {}",
                                        definition.kind.label(),
                                        definition.id
                                    )),
                            ),
                    )
                    .child(value_control)
                    .children(variable_control),
            )
            .when(
                self.shader_active_color_property.as_ref() == Some(&definition.id),
                |row| row.child(self.render_shader_color_editor(definition, value, cx)),
            )
            .when_some(definition.description.clone(), |row, description| {
                row.child(
                    div()
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(description),
                )
            })
            .into_any_element()
    }

    fn render_shader_color_editor(
        &self,
        definition: &DesignShaderPropertyDefinition,
        value: Option<&DesignShaderPropertyValue>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let current = match value {
            Some(DesignShaderPropertyValue::Color(current)) => *current,
            _ => DesignColor::BLACK,
        };
        let palette = [
            DesignColor::WHITE,
            DesignColor::rgb(204, 204, 204),
            DesignColor::rgb(102, 102, 102),
            DesignColor::BLACK,
            DesignColor::rgb(230, 54, 54),
            DesignColor::rgb(239, 141, 38),
            DesignColor::rgb(245, 213, 47),
            DesignColor::rgb(69, 184, 94),
            DesignColor::rgb(53, 166, 220),
            DesignColor::rgb(65, 96, 223),
            DesignColor::rgb(139, 80, 208),
            DesignColor::rgb(222, 83, 157),
        ];
        let mut swatches = h_flex().w_full().gap_1().flex_wrap();
        for (index, palette_color) in palette.into_iter().enumerate() {
            let color = DesignColor::rgba(
                palette_color.red,
                palette_color.green,
                palette_color.blue,
                current.alpha,
            );
            let definition_id = definition.id.clone();
            swatches = swatches.child(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-shader-property-{}-swatch-{index}",
                    self.id, definition.id
                )))
                .tooltip(format!("Set #{}", color.hex()))
                .xsmall()
                .compact()
                .ghost()
                .selected(color == current)
                .disabled(self.editing_disabled())
                .child(
                    div()
                        .size(px(20.))
                        .rounded(px(3.))
                        .border_1()
                        .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
                        .bg(color_to_hsla(color)),
                )
                .on_activate(cx.listener(move |this, _, _, cx| {
                    this.choose_shader_color(definition_id.clone(), color, cx);
                })),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .p_2()
            .rounded(px(6.))
            .border_1()
            .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .child("Hex"),
                    )
                    .child(
                        div()
                            .debug_selector(|| "shader-color-input".to_owned())
                            .flex_1()
                            .min_w(px(0.))
                            .child(
                                Input::new(&self.shader_color_input)
                                    .typography(crate::atoms::TypographyToken::BodyMedium)
                                    .xsmall()
                                    .h(px(26.))
                                    .flex_1()
                                    .disabled(self.editing_disabled())
                                    .when(self.shader_color_invalid, |input| {
                                        input.border_color(
                                            crate::atoms::SemanticColor::TextDanger.resolve(cx),
                                        )
                                    }),
                            ),
                    ),
            )
            .child(swatches)
            .into_any_element()
    }

    pub(super) fn render_shader_state(
        &self,
        shader: &DesignShaderPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut content = v_flex().w_full().gap_2().child(
            h_flex()
                .w_full()
                .p_2()
                .gap_2()
                .rounded(px(7.))
                .border_1()
                .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
                .child(
                    Icon::new(IconName::Asterisk)
                        .small()
                        .text_color(crate::atoms::SemanticColor::BackgroundHover.resolve(cx)),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w(px(0.))
                        .child(
                            div()
                                .truncate()
                                .typography(crate::atoms::TypographyToken::BodyLarge)
                                .font_semibold()
                                .child(shader.name.clone()),
                        )
                        .child(
                            div()
                                .truncate()
                                .typography(crate::atoms::TypographyToken::BodyMedium)
                                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                                .child(shader.shader_id.clone()),
                        ),
                )
                .child(
                    crate::atoms::ui_button(SharedString::from(format!(
                        "{}-choose-shader",
                        self.id
                    )))
                    .label("Choose")
                    .xsmall()
                    .compact()
                    .outline()
                    .disabled(self.editing_disabled())
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.active_tab = PaintPickerTab::Libraries;
                        this.scroll_handle.set_offset(Point::default());
                        cx.notify();
                    })),
                ),
        );

        let Some(definition) = self.shader_view_data.definition(&shader.shader_id) else {
            return content
                .child(
                    div()
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(
                            "Shader metadata is unavailable. The host still preserves every assignment.",
                        ),
                )
                .into_any_element();
        };
        if definition.property_definitions.is_empty() {
            return content
                .child(
                    div()
                        .typography(crate::atoms::TypographyToken::BodyMedium)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(
                            "Import metadata is pending; no property definitions are available.",
                        ),
                )
                .into_any_element();
        }
        content = content.child(
            div()
                .typography(crate::atoms::TypographyToken::BodyMedium)
                .font_semibold()
                .child("Properties"),
        );
        for property in &definition.property_definitions {
            content = content.child(self.render_shader_property(shader, property, cx));
        }
        content.into_any_element()
    }
}
