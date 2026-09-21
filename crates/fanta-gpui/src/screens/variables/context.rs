use super::*;
use crate::atoms::{Dropdown, SemanticColor, TypographyToken};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariablesChoice {
    pub id: Option<SharedString>,
    pub label: SharedString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariablesModeScope {
    pub id: SharedString,
    pub label: SharedString,
    pub selected: Option<SharedString>,
    pub choices: Vec<VariablesChoice>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariablesBindingProperty {
    pub id: SharedString,
    pub label: SharedString,
    pub selected: Option<SharedString>,
    pub choices: Vec<VariablesChoice>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariablesLayerBindings {
    pub node_id: SharedString,
    pub name: SharedString,
    pub properties: Vec<VariablesBindingProperty>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VariablesContextData {
    pub mode_scopes: Vec<VariablesModeScope>,
    pub bindings: Option<VariablesLayerBindings>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariablesContextAction {
    ModeSelected {
        collection_id: SharedString,
        scope_id: SharedString,
        mode_id: Option<SharedString>,
    },
    BindingSelected {
        node_id: SharedString,
        property_id: SharedString,
        variable_id: Option<SharedString>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ContextMenuTarget {
    Mode(SharedString),
    Binding(SharedString),
}

impl EventEmitter<VariablesContextAction> for VariablesScreen {}

impl VariablesScreen {
    pub fn set_context_data(&mut self, data: VariablesContextData, cx: &mut Context<Self>) {
        if self.context_data != data {
            self.context_data = data;
            self.context_menu = None;
            cx.notify();
        }
    }

    fn context_dropdown(
        &self,
        id: SharedString,
        label: SharedString,
        disabled: bool,
        target: ContextMenuTarget,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selector = id.clone();
        let bounds_id = id.clone();
        let owner = cx.entity();
        div()
            .relative()
            .w_full()
            .debug_selector(move || selector.to_string())
            .child(track_bounds(owner, move |this, bounds| {
                this.overlay_bounds.insert(bounds_id.clone(), bounds);
            }))
            .child(
                Dropdown::new(id.clone(), label)
                    .full_width(true)
                    .disabled(disabled)
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.context_menu = if this
                            .context_menu
                            .as_ref()
                            .is_some_and(|(open, _)| *open == target)
                        {
                            None
                        } else {
                            Some((target.clone(), id.clone()))
                        };
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    pub(super) fn render_mode_scopes(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .id("variables-mode-scopes")
            .w_full()
            .flex_none()
            .flex_wrap()
            .px_3()
            .py_2()
            .gap_3()
            .items_center()
            .border_b_1()
            .border_color(SemanticColor::Border.resolve(cx))
            .typography(TypographyToken::BodyMedium)
            .child(div().font_semibold().child("Modes"))
            .children(self.context_data.mode_scopes.iter().map(|scope| {
                let label = scope
                    .choices
                    .iter()
                    .find(|choice| choice.id == scope.selected)
                    .map(|choice| choice.label.clone())
                    .unwrap_or_else(|| "Inherit".into());
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_color(SemanticColor::TextSecondary.resolve(cx))
                            .child(scope.label.clone()),
                    )
                    .child(
                        div()
                            .w(px(tokens::VariablesGeometry::MODE_DROPDOWN_WIDTH))
                            .child(self.context_dropdown(
                                format!("variables-mode-scope-{}", scope.id).into(),
                                label,
                                scope.choices.is_empty(),
                                ContextMenuTarget::Mode(scope.id.clone()),
                                cx,
                            )),
                    )
            }))
            .into_any_element()
    }

    pub(super) fn render_layer_bindings(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(bindings) = &self.context_data.bindings else {
            return div().into_any_element();
        };
        v_flex()
            .id("variables-layer-bindings")
            .debug_selector(|| "variables-layer-bindings".to_owned())
            .w(px(tokens::VariablesGeometry::BINDINGS_WIDTH))
            .h_full()
            .flex_none()
            .min_h_0()
            .overflow_y_scroll()
            // The screen's header owns the horizontal separator. This panel owns only its left edge.
            .border_l_1()
            .border_color(SemanticColor::Border.resolve(cx))
            .bg(SemanticColor::Background.resolve(cx))
            .typography(TypographyToken::BodyMedium)
            .child(
                v_flex()
                    .px_3()
                    .py_3()
                    .gap_2()
                    .child(div().font_semibold().child("Bind selected layer"))
                    .child(div().truncate().child(bindings.name.clone())),
            )
            .children(bindings.properties.iter().map(|property| {
                let label = property
                    .choices
                    .iter()
                    .find(|choice| choice.id == property.selected)
                    .map(|choice| choice.label.clone())
                    .unwrap_or_else(|| {
                        if property.choices.is_empty() {
                            "No compatible variables".into()
                        } else {
                            "Choose variable".into()
                        }
                    });
                v_flex()
                    .px_3()
                    .pb_3()
                    .gap_1()
                    .child(
                        div()
                            .text_color(SemanticColor::TextSecondary.resolve(cx))
                            .child(property.label.clone()),
                    )
                    .child(self.context_dropdown(
                        format!("variables-binding-{}", property.id).into(),
                        label,
                        property.choices.is_empty(),
                        ContextMenuTarget::Binding(property.id.clone()),
                        cx,
                    ))
            }))
            .into_any_element()
    }

    pub(super) fn render_context_menu(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (target, trigger) = self.context_menu.as_ref()?;
        let choices = match target {
            ContextMenuTarget::Mode(id) => {
                &self
                    .context_data
                    .mode_scopes
                    .iter()
                    .find(|scope| scope.id == *id)?
                    .choices
            }
            ContextMenuTarget::Binding(id) => {
                &self
                    .context_data
                    .bindings
                    .as_ref()?
                    .properties
                    .iter()
                    .find(|property| property.id == *id)?
                    .choices
            }
        };
        let bounds = self.overlay_bounds.get(trigger)?;
        let mut menu = popup_surface("variables-context-menu", px(tokens::Radius::MENU), cx)
            .w(popup_width(window, 240.))
            .max_h(popup_max_height(window))
            .py_1()
            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                this.context_menu = None;
                cx.notify();
            }));
        for (index, choice) in choices.iter().enumerate() {
            let target = target.clone();
            let choice = choice.clone();
            menu = menu.child(
                menu_item(
                    ("variables-context-choice", index),
                    px(tokens::RowHeight::FIELD),
                    cx,
                )
                .debug_selector(move || format!("variables-context-choice-{index}"))
                .px_2()
                .child(choice.label.clone())
                .on_activate(cx.listener(move |this, _, _, cx| {
                    let action = match &target {
                        ContextMenuTarget::Mode(scope_id) => {
                            Some(VariablesContextAction::ModeSelected {
                                collection_id: this.view_data.selected_collection_id.clone(),
                                scope_id: scope_id.clone(),
                                mode_id: choice.id.clone(),
                            })
                        }
                        ContextMenuTarget::Binding(property_id) => {
                            this.context_data.bindings.as_ref().map(|bindings| {
                                VariablesContextAction::BindingSelected {
                                    node_id: bindings.node_id.clone(),
                                    property_id: property_id.clone(),
                                    variable_id: choice.id.clone(),
                                }
                            })
                        }
                    };
                    this.context_menu = None;
                    if let Some(action) = action {
                        cx.emit(action);
                    }
                    cx.notify();
                })),
            );
        }
        Some(
            div()
                .absolute()
                .left(bounds.origin.x - self.page_origin.x)
                .top(bounds.bottom_left().y - self.page_origin.y)
                .child(anchored_popup(
                    Anchor::TopLeft,
                    point(px(0.), px(tokens::Space::XS)),
                    3,
                    menu,
                ))
                .into_any_element(),
        )
    }
}
