//! Host adapter for the standalone color picker.
use super::*;

impl VariablesScreen {
    fn variable_color(
        &self,
        variable_id: &SharedString,
        mode_id: &SharedString,
    ) -> Option<PickerColor> {
        let variable = self
            .view_data
            .variables
            .iter()
            .find(|v| v.id == *variable_id && v.kind == VariableKind::Color)?;
        let value = variable
            .values
            .iter()
            .find(|v| v.mode_id == *mode_id && v.alias_id.is_none())?;
        let [r, g, b, a] = rgba_channels(parse_hex_rgba(
            value.color_hex.as_deref().unwrap_or(&value.value),
        )?);
        Some(PickerColor::rgba(r, g, b, a))
    }
    pub(super) fn sync_color_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((variable_id, mode_id, _)) = &self.color_target else {
            return;
        };
        if let Some(color) = self.variable_color(variable_id, mode_id) {
            self.color_picker
                .update(cx, |picker, cx| picker.set_color(color, window, cx));
        } else {
            self.close_color_picker(window, cx);
        }
    }
    pub(super) fn close_color_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.color_target.take().is_some() {
            self.color_picker
                .update(cx, |picker, cx| picker.cancel(window, cx));
            cx.notify();
        }
    }
    pub(super) fn handle_color_event(
        &mut self,
        event: &ColorPickerAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((variable_id, mode_id, _)) = self.color_target.clone() else {
            return;
        };
        match event {
            ColorPickerAction::Edit {
                color,
                phase: ColorPickerPhase::Commit,
            } => cx.emit(VariablesAction::ValueChanged {
                variable_id,
                mode_id,
                value: color.hex(),
            }),
            ColorPickerAction::CloseRequested => self.close_color_picker(window, cx),
            ColorPickerAction::EyedropperRequested => {
                cx.emit(VariablesAction::ColorEyedropperRequested {
                    variable_id,
                    mode_id,
                })
            }
            _ => {}
        }
    }
    pub(super) fn render_color_trigger(
        &self,
        variable: &VariableRow,
        mode: &VariablesMode,
        in_settings: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let target = (variable.id.clone(), mode.id.clone(), in_settings);
        let key: SharedString = format!("color-{}-{}-{in_settings}", variable.id, mode.id).into();
        let value = variable.values.iter().find(|v| v.mode_id == mode.id);
        let hex = value.map_or_else(SharedString::default, |v| v.value.clone());
        let color = value
            .and_then(|v| Self::parse_hex(v.color_hex.as_deref().unwrap_or(&v.value)))
            .unwrap_or(crate::atoms::SemanticColor::Background.resolve(cx));
        div()
            .relative()
            .flex_1()
            .min_w_0()
            .h(px(tokens::RowHeight::FIELD))
            .child(track_bounds(cx.entity(), move |this, bounds| {
                this.overlay_bounds.insert(key.clone(), bounds);
            }))
            .child(
                h_flex()
                    .id(SharedString::from(format!(
                        "variable-color-{}-{}-{in_settings}",
                        variable.id, mode.id
                    )))
                    .debug_selector({
                        let v = variable.id.clone();
                        let m = mode.id.clone();
                        move || format!("variables-color-{v}-{m}-{in_settings}")
                    })
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .w_full()
                    .h_full()
                    .gap_2()
                    .rounded(px(tokens::Radius::CONTROL))
                    .border_1()
                    .border_color(cx.theme().transparent)
                    .cursor_pointer()
                    .hover(|style| {
                        style.bg(crate::atoms::SemanticColor::BackgroundHover.resolve(cx))
                    })
                    .focus(|style| {
                        style.border_color(
                            crate::atoms::SemanticColor::BackgroundSelected.resolve(cx),
                        )
                    })
                    .on_activate(cx.listener(move |this, _, window, cx| {
                        if this.color_target.as_ref() == Some(&target) {
                            this.close_color_picker(window, cx);
                            return;
                        }
                        this.close_color_picker(window, cx);
                        this.commit_edit(cx);
                        this.create_menu_open = false;
                        this.alias_target = None;
                        if !in_settings {
                            this.settings_id = None;
                        }
                        this.color_target = Some(target.clone());
                        this.sync_color_picker(window, cx);
                        this.color_picker
                            .read(cx)
                            .focus_handle(cx)
                            .focus(window, cx);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .size(px(tokens::ControlSize::INLINE))
                            .flex_none()
                            .rounded(px(tokens::Radius::CONTROL))
                            .border_1()
                            .border_color(crate::atoms::SemanticColor::Border.resolve(cx))
                            .bg(color),
                    )
                    .child(
                        div()
                            .truncate()
                            .typography(crate::atoms::TypographyToken::BodyMedium)
                            .child(hex),
                    ),
            )
            .into_any_element()
    }
}
