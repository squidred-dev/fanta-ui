use super::*;
use crate::molecules::sidebar_popup_surface;
use gpui::{PromptButton, PromptLevel};
use gpui_component::{
    Root, WindowExt as _,
    button::ButtonVariant,
    dialog::{Dialog, DialogButtonProps},
};

impl VariablesScreen {
    pub(super) fn request_delete_variable(
        &mut self,
        variable_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(variable) = self
            .view_data
            .variables
            .iter()
            .find(|variable| variable.id == variable_id)
        else {
            return;
        };
        self.open_delete_dialog(
            VariablesAction::VariableDeleteRequested { variable_id },
            "Delete variable",
            format!("Delete “{}”?", variable.name).into(),
            window,
            cx,
        );
    }

    pub(super) fn request_delete_mode(
        &mut self,
        mode_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(mode) = self.view_data.modes.iter().find(|mode| mode.id == mode_id) else {
            return;
        };
        self.open_delete_dialog(
            VariablesAction::ModeDeleteRequested { mode_id },
            "Delete mode",
            format!("Delete “{}” and its values?", mode.name).into(),
            window,
            cx,
        );
    }

    fn open_delete_dialog(
        &mut self,
        action: VariablesAction,
        title: &'static str,
        message: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.delete_prompt_open || window.has_active_dialog(cx) {
            return;
        }
        self.commit_edit(cx);
        self.settings_id = None;
        self.context_menu = None;
        self.alias_target = None;
        self.create_menu_open = false;
        self.filter_menu_open = false;
        self.close_color_picker(window, cx);
        let collection_id = self.view_data.selected_collection_id.clone();
        if Root::try_read(window, cx).is_none() {
            // Zed-style windows use MultiWorkspace as their root. Their prompt
            // builder paints a themed modal without changing the window root.
            self.delete_prompt_open = true;
            let answer = window.prompt(
                PromptLevel::Warning,
                title,
                Some(message.as_ref()),
                &[PromptButton::cancel("Cancel"), PromptButton::ok("Delete")],
                cx,
            );
            cx.spawn_in(window, async move |page, cx| {
                let confirmed = answer.await == Ok(1);
                _ = page.update(cx, |this, cx| {
                    this.delete_prompt_open = false;
                    if confirmed {
                        this.emit_confirmed_delete(&action, &collection_id, cx);
                    }
                    cx.notify();
                });
            })
            .detach();
            cx.notify();
            return;
        }
        let page = cx.entity();
        window.open_dialog(cx, move |dialog: Dialog, window, cx| {
            let page = page.clone();
            let collection_id = collection_id.clone();
            let action = action.clone();
            let palette = crate::atoms::sidebar_style(cx);
            dialog
                .title(title)
                .confirm()
                .footer(|ok, cancel, window, cx| {
                    vec![
                        div()
                            .id("variables-delete-dialog-cancel")
                            .debug_selector(|| "variables-delete-dialog-cancel".to_owned())
                            .child(cancel(window, cx)),
                        div()
                            .id("variables-delete-dialog-confirm")
                            .debug_selector(|| "variables-delete-dialog-confirm".to_owned())
                            .child(ok(window, cx)),
                    ]
                })
                .button_props(
                    DialogButtonProps::default()
                        .ok_text("Delete")
                        .ok_variant(ButtonVariant::Danger),
                )
                .w(popup_width(window, 400.))
                .p_3()
                .bg(palette.menu_background)
                .border_color(palette.border)
                .text_color(palette.text)
                .text_size(crate::atoms::sidebar_text_size())
                .on_ok(move |_, _, cx| {
                    page.update(cx, |this, cx| {
                        this.emit_confirmed_delete(&action, &collection_id, cx);
                    });
                    true
                })
                .child(
                    div()
                        .w_full()
                        .text_color(palette.muted_text)
                        .child(message.clone()),
                )
        });
        cx.notify();
    }

    fn emit_confirmed_delete(
        &self,
        action: &VariablesAction,
        collection_id: &SharedString,
        cx: &mut Context<Self>,
    ) {
        // A host may replace the snapshot while confirmation is open.
        let still_present = self.view_data.selected_collection_id == *collection_id
            && match action {
                VariablesAction::VariableDeleteRequested { variable_id } => self
                    .view_data
                    .variables
                    .iter()
                    .any(|variable| variable.id == *variable_id),
                VariablesAction::ModeDeleteRequested { mode_id } => {
                    self.view_data.modes.iter().any(|mode| mode.id == *mode_id)
                }
                _ => false,
            };
        if still_present {
            cx.emit(action.clone());
        }
    }

    pub(super) fn begin_edit(
        &mut self,
        target: EditTarget,
        value: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.edit_target.as_ref() == Some(&target) {
            return;
        }
        self.commit_edit(cx);
        self.edit_input = cx.new(|cx| {
            InputState::new(window, cx).validate({
                let numeric = matches!(target, EditTarget::Value(_, _, VariableKind::Number));
                move |value, _| {
                    !numeric
                        || value.is_empty()
                        || value == "-"
                        || value == "."
                        || value == "-."
                        || value.parse::<f64>().is_ok_and(f64::is_finite)
                }
            })
        });
        self.edit_subscription = Some(cx.subscribe(
            &self.edit_input,
            |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                    this.commit_edit(cx);
                }
            },
        ));
        self.edit_input
            .update(cx, |input, cx| input.set_value(value.clone(), window, cx));
        self.edit_input.read(cx).focus_handle(cx).focus(window, cx);
        self.edit_target = Some(target);
        cx.notify();
    }

    pub(super) fn commit_edit(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.edit_target.take() else {
            return;
        };
        let value = self.edit_input.read(cx).value();
        let action = match target {
            EditTarget::Name(variable_id) if !value.trim().is_empty() => {
                Some(VariablesAction::VariableRenameRequested {
                    variable_id,
                    name: value.trim().into(),
                })
            }
            EditTarget::Mode(mode_id) if !value.trim().is_empty() => {
                Some(VariablesAction::ModeRenameRequested {
                    mode_id,
                    name: value.trim().into(),
                })
            }
            EditTarget::Description(variable_id) => Some(VariablesAction::DescriptionChanged {
                variable_id,
                description: value,
            }),
            EditTarget::Value(variable_id, mode_id, kind)
                if kind != VariableKind::Color
                    && (kind != VariableKind::Number
                        || value.parse::<f64>().is_ok_and(f64::is_finite)) =>
            {
                Some(VariablesAction::ValueChanged {
                    variable_id,
                    mode_id,
                    value,
                })
            }
            _ => None,
        };
        if let Some(action) = action {
            cx.emit(action);
        }
        cx.notify();
    }

    pub(super) fn render_text(
        &self,
        target: EditTarget,
        text: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.edit_target.as_ref() == Some(&target) {
            let palette = crate::atoms::sidebar_style(cx);
            div()
                .flex_1()
                .min_w_0()
                .on_mouse_down_out(
                    cx.listener(|this, _: &MouseDownEvent, _, cx| this.commit_edit(cx)),
                )
                .h(px(tokens::RowHeight::FIELD))
                .child(
                    Input::new(&self.edit_input)
                        .xsmall()
                        .h_full()
                        .appearance(false)
                        .bordered(false)
                        .focus_bordered(false)
                        .px_0()
                        .text_size(crate::atoms::sidebar_text_size())
                        .text_color(palette.text),
                )
                .into_any_element()
        } else {
            div()
                .min_w(px(0.))
                .text_size(crate::atoms::sidebar_text_size())
                .truncate()
                .child(text)
                .into_any_element()
        }
    }

    pub(super) fn render_edit_overlays(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut overlays = Vec::new();
        if self.create_menu_open {
            let mut menu = sidebar_popup_surface("variables-create-menu", cx)
                .debug_selector(|| "variables-type-dropdown".to_owned())
                .w(popup_width(window, tokens::MenuWidth::NARROW))
                .max_h(popup_max_height(window))
                .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    this.create_menu_open = false;
                    cx.notify();
                }));
            for (kind, label) in [
                (VariableKind::Color, "Color"),
                (VariableKind::Number, "Number"),
                (VariableKind::String, "String"),
                (VariableKind::Boolean, "Boolean"),
            ] {
                menu = menu.child(
                    sidebar_menu_item(
                        SharedString::from(format!("create-{label}")),
                        px(tokens::RowHeight::FIELD),
                        true,
                        cx,
                    )
                    .debug_selector(move || format!("variables-create-{label}"))
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.create_menu_open = false;
                        cx.emit(VariablesAction::CreateTypedVariableRequested { kind });
                        cx.notify();
                    }))
                    .child(Self::render_kind_glyph(kind, cx))
                    .child(label),
                );
            }
            overlays.push(self.place_popup(
                &self.create_trigger,
                true,
                20,
                menu.into_any_element(),
            ));
        }
        if let Some(variable) = self
            .settings_id
            .as_ref()
            .and_then(|id| self.view_data.variables.iter().find(|v| v.id == *id))
        {
            let palette = crate::atoms::sidebar_style(cx);
            let mut settings = sidebar_popup_surface("variables-settings-popover", cx)
                .debug_selector(|| "variables-settings-popover".to_owned())
                .w(popup_width(window, tokens::MenuWidth::PICKER_WIDE))
                .max_h(popup_max_height(window))
                .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    if this.alias_target.is_none() && this.color_target.is_none() {
                        this.commit_edit(cx);
                        this.settings_id = None;
                        cx.notify();
                    }
                }));
            settings = settings.child(
                h_flex()
                    .w_full()
                    .h(px(tokens::RowHeight::FIELD))
                    .px_3()
                    .gap_2()
                    .child(Self::render_kind_glyph(variable.kind, cx))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .font_semibold()
                            .child(variable.name.clone()),
                    ),
            );
            settings = settings.child(sidebar_menu_separator(cx));
            for (label, target, text) in [
                (
                    "Name",
                    EditTarget::Name(variable.id.clone()),
                    variable.name.clone(),
                ),
                (
                    "Description",
                    EditTarget::Description(variable.id.clone()),
                    variable.description.clone(),
                ),
            ] {
                let draft = text.clone();
                let edit = target.clone();
                settings = settings.child(
                    v_flex()
                        .w_full()
                        .px_3()
                        .py_1()
                        .gap_1()
                        .child(
                            div()
                                .debug_selector(move || format!("variables-settings-label-{label}"))
                                .text_color(palette.muted_text)
                                .child(label),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("settings-{label}")))
                                .debug_selector(move || format!("variables-settings-field-{label}"))
                                .w_full()
                                .min_w_0()
                                .h(px(tokens::RowHeight::FIELD))
                                .px_2()
                                .rounded(px(tokens::Radius::CONTROL))
                                .border_1()
                                .border_color(palette.border)
                                .bg(palette.background)
                                .flex()
                                .items_center()
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.begin_edit(edit.clone(), draft.clone(), window, cx)
                                }))
                                .child(div().flex_1().min_w_0().child(self.render_text(
                                    target,
                                    if text.is_empty() {
                                        "How to use this variable".into()
                                    } else {
                                        text
                                    },
                                    cx,
                                ))),
                        ),
                );
            }
            settings = settings.child(sidebar_menu_separator(cx));
            settings = settings.child(
                div()
                    .px_3()
                    .py_1()
                    .text_color(palette.muted_text)
                    .child("Default value"),
            );
            if let Some(mode) = self.view_data.modes.first() {
                settings = settings.child(div().px_3().child(self.render_value(
                    variable,
                    mode,
                    tokens::MenuWidth::PICKER_WIDE - 2. * tokens::Space::MD,
                    false,
                    true,
                    cx,
                )));
            } else {
                settings = settings.child(
                    div()
                        .px_3()
                        .py_1()
                        .text_color(palette.muted_text)
                        .child("Add a mode to set a value"),
                );
            }
            settings = settings.child(sidebar_menu_separator(cx));
            let rename_id = variable.id.clone();
            let rename_name = variable.name.clone();
            settings = settings.child(
                sidebar_menu_item(
                    "variables-settings-rename-item",
                    px(tokens::RowHeight::FIELD),
                    true,
                    cx,
                )
                .debug_selector(|| "variables-settings-rename".to_owned())
                .on_activate(cx.listener(move |this, _, window, cx| {
                    this.begin_edit(
                        EditTarget::Name(rename_id.clone()),
                        rename_name.clone(),
                        window,
                        cx,
                    );
                }))
                .child(render_lucide_icon(LucideIcon::Pencil, palette.icon, 14.))
                .child("Rename variable"),
            );
            let delete_id = variable.id.clone();
            settings = settings.child(
                sidebar_menu_item(
                    "variables-settings-delete-item",
                    px(tokens::RowHeight::FIELD),
                    true,
                    cx,
                )
                .debug_selector(|| "variables-settings-delete".to_owned())
                .on_activate(cx.listener(move |this, _, window, cx| {
                    this.request_delete_variable(delete_id.clone(), window, cx);
                }))
                .child(render_lucide_icon(
                    LucideIcon::Trash,
                    crate::atoms::SemanticColor::IconDanger.resolve(cx),
                    14.,
                ))
                .child("Delete variable"),
            );
            overlays.push(self.place_popup(
                &format!("settings-{}", variable.id).into(),
                false,
                20,
                settings.into_any_element(),
            ));
        }
        if let Some((variable_id, mode_id)) = &self.alias_target {
            let palette = crate::atoms::sidebar_style(cx);
            let mut picker = sidebar_popup_surface("variables-alias-picker", cx)
                .w(popup_width(window, tokens::MenuWidth::POPOVER))
                .max_h(popup_max_height(window).min(px(tokens::Breakpoint::COMPACT)))
                .gap_1()
                .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    this.alias_target = None;
                    cx.notify();
                }))
                .child(
                    div()
                        .mx_2()
                        .my_1()
                        .px_2()
                        .border_1()
                        .border_color(palette.border)
                        .rounded(px(tokens::Radius::CONTROL))
                        .bg(palette.background)
                        .child(
                            Input::new(&self.alias_search)
                                .xsmall()
                                .h(px(tokens::RowHeight::FIELD))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false)
                                .text_size(crate::atoms::sidebar_text_size())
                                .prefix(Icon::new(IconName::Search).xsmall()),
                        ),
                );
            let kind = self
                .view_data
                .variables
                .iter()
                .find(|v| v.id == *variable_id)
                .map(|v| v.kind);
            let query = self.alias_search.read(cx).value().to_lowercase();
            let mut count = 0;
            for candidate in &self.view_data.variables {
                if candidate.id == *variable_id
                    || Some(candidate.kind) != kind
                    || !candidate.name.to_lowercase().contains(&query)
                    || self.alias_would_cycle(variable_id, &candidate.id, mode_id)
                {
                    continue;
                }
                count += 1;
                let v = variable_id.clone();
                let m = mode_id.clone();
                let alias = candidate.id.clone();
                picker = picker.child(
                    sidebar_menu_item(
                        SharedString::from(format!("alias-{alias}")),
                        px(tokens::RowHeight::FIELD),
                        true,
                        cx,
                    )
                    .debug_selector({
                        let alias = alias.clone();
                        move || format!("variables-alias-{alias}")
                    })
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.alias_target = None;
                        cx.emit(VariablesAction::AliasChanged {
                            variable_id: v.clone(),
                            mode_id: m.clone(),
                            alias_id: Some(alias.clone()),
                        });
                        cx.notify();
                    }))
                    .child(Self::render_kind_glyph(candidate.kind, cx))
                    .child(candidate.name.clone()),
                );
            }
            if count == 0 {
                picker = picker.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_size(crate::atoms::sidebar_text_size())
                        .text_color(palette.muted_text)
                        .child("No compatible variables"),
                );
            }
            overlays.push(self.place_popup(
                &self.alias_trigger,
                false,
                30,
                picker.into_any_element(),
            ));
        }
        if let Some((variable_id, mode_id, in_settings)) = &self.color_target {
            let nested_menu_open = self.color_picker.read(cx).has_open_menu(cx);
            let surface = div()
                .on_mouse_down_out(cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                    if !nested_menu_open {
                        this.close_color_picker(window, cx);
                    }
                }))
                .child(self.color_picker.clone());
            overlays.push(self.place_popup(
                &format!("color-{variable_id}-{mode_id}-{in_settings}").into(),
                false,
                40,
                surface.into_any_element(),
            ));
        }
        overlays
    }

    fn place_popup(
        &self,
        key: &SharedString,
        above: bool,
        priority: usize,
        surface: AnyElement,
    ) -> AnyElement {
        let bounds = self.overlay_bounds.get(key).copied().unwrap_or_default();
        let origin = if above {
            bounds.origin
        } else {
            bounds.bottom_left()
        };
        div()
            .absolute()
            .left(origin.x - self.page_origin.x)
            .top(origin.y - self.page_origin.y)
            .child(anchored_popup(
                if above {
                    Anchor::BottomLeft
                } else {
                    Anchor::TopLeft
                },
                point(
                    px(0.),
                    px(if above {
                        -tokens::Space::XS
                    } else {
                        tokens::Space::XS
                    }),
                ),
                priority,
                surface,
            ))
            .into_any_element()
    }

    pub(super) fn alias_would_cycle(
        &self,
        source: &SharedString,
        candidate: &SharedString,
        mode: &SharedString,
    ) -> bool {
        let mut current = candidate;
        let mut visited = std::collections::HashSet::new();
        loop {
            if current == source || !visited.insert(current) {
                return true;
            }
            let next = self
                .view_data
                .variables
                .iter()
                .find(|v| v.id == *current)
                .and_then(|v| v.values.iter().find(|v| v.mode_id == *mode))
                .and_then(|v| v.alias_id.as_ref());
            match next {
                Some(next) => current = next,
                None => return false,
            }
        }
    }
}
