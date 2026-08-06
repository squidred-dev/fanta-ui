//! Effect collections, effect shaders, effect styles, and
//! effect variables on the selected node.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    _cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.nodes[node_index];
    match action {
        DesignPanelAction::EffectAddRequested { node_id, kind } => {
            let added = node.can_use_effect_kind(*kind, None);
            if added {
                let effect_id = format!(
                    "{node_id}-effect-{}-{}",
                    kind.label().to_ascii_lowercase().replace(' ', "-"),
                    node.effects.len()
                );
                node.effects
                    .push(DesignEffect::new(*kind).with_id(effect_id));
            }
            screen.last_action = if added {
                format!("Host added {} to {node_id}", kind.label()).into()
            } else {
                format!(
                    "Host rejected unavailable or maxed {} on {node_id}",
                    kind.label()
                )
                .into()
            };
        }
        DesignPanelAction::EffectRemoveRequested {
            node_id,
            effect_id,
            index,
        } => {
            let resolved = if effect_id.is_empty() {
                (*index < node.effects.len()).then_some(*index)
            } else {
                node.effect_index_by_id(effect_id.as_ref())
            };
            let removed = resolved.map(|index| node.effects.remove(index));
            screen.last_action = removed.map_or_else(
                || format!("Host ignored stale effect removal on {node_id}").into(),
                |effect| format!("Host removed {} from {node_id}", effect.kind.label()).into(),
            );
        }
        DesignPanelAction::EffectReorderRequested {
            node_id,
            effect_id,
            from_index,
            to_index,
        } => {
            let resolved = if effect_id.is_empty() {
                (*from_index < node.effects.len()).then_some(*from_index)
            } else {
                node.effect_index_by_id(effect_id.as_ref())
            };
            let moved = resolved.is_some_and(|from_index| {
                if from_index == *to_index || *to_index >= node.effects.len() {
                    return false;
                }
                let effect = node.effects.remove(from_index);
                node.effects.insert(*to_index, effect);
                true
            });
            screen.last_action = if moved {
                format!("Host reordered effect {effect_id} on {node_id}").into()
            } else {
                format!("Host ignored stale effect reorder on {node_id}").into()
            };
        }
        DesignPanelAction::EffectEditRequested {
            node_id,
            effect_id,
            index,
            property,
            shader_property_id,
            value,
            phase,
        } => {
            let resolved = if effect_id.is_empty() {
                (*index < node.effects.len()).then_some(*index)
            } else {
                node.effect_index_by_id(effect_id.as_ref())
            };
            let applied = resolved.is_some_and(|resolved_index| {
                if *phase == DesignPanelEditPhase::Begin {
                    return true;
                }
                let mut property = property.with_effect_index(resolved_index);
                if let DesignPanelProperty::EffectShaderProperty(_, _) = property
                    && let Some(definition_id) = shader_property_id.as_ref()
                {
                    let Some(effect) = node.effects.get(resolved_index) else {
                        return false;
                    };
                    let DesignEffectSettings::Shader(shader) = &effect.settings else {
                        return false;
                    };
                    let Some(property_index) = shader
                        .properties
                        .iter()
                        .position(|property| property.definition_id == *definition_id)
                    else {
                        return false;
                    };
                    property =
                        DesignPanelProperty::EffectShaderProperty(resolved_index, property_index);
                }
                apply_design_property(node, property, value);
                true
            });
            screen.last_action = if applied {
                format!("Host observed {phase:?} effect edit {property:?} on {node_id}").into()
            } else {
                format!("Host ignored stale effect edit on {node_id}").into()
            };
        }
        DesignPanelAction::EffectShaderChooseRequested {
            node_id,
            effect_id,
            index,
        } => {
            let resolved = if effect_id.is_empty() {
                (*index < node.effects.len()).then_some(*index)
            } else {
                node.effect_index_by_id(effect_id.as_ref())
            };
            let applied = resolved.is_some_and(|index| {
                let Some(effect) = node.effects.get_mut(index) else {
                    return false;
                };
                effect.set_settings(DesignEffectSettings::Shader(DesignShaderEffect::new(
                    "shader:effect:storybook",
                    "Storybook distortion",
                    [
                        DesignShaderProperty::new(
                            "amount",
                            "Amount",
                            DesignShaderPropertyKind::Number,
                            DesignShaderPropertyValue::Number(0.5),
                        ),
                        DesignShaderProperty::new(
                            "active",
                            "Active",
                            DesignShaderPropertyKind::Boolean,
                            DesignShaderPropertyValue::Boolean(true),
                        ),
                    ],
                )));
                true
            });
            screen.last_action = if applied {
                format!("Host applied an imported Shader effect on {node_id}").into()
            } else {
                format!("Host ignored stale Shader chooser request on {node_id}").into()
            };
        }
        DesignPanelAction::EffectShaderPropertyEditorRequested {
            node_id,
            effect_id,
            index,
            shader_property_id,
            property_index,
            property_kind,
            target,
            editor,
            current_value,
        } => {
            let applied = apply_story_effect_shader_property_editor(
                node,
                effect_id,
                *index,
                shader_property_id,
                *property_index,
                *property_kind,
                *target,
                *editor,
                current_value,
            );
            screen.last_action = if applied {
                format!(
                        "Host opened the {editor:?} editor for Shader property {shader_property_id} on {node_id}"
                    )
                    .into()
            } else {
                format!("Host rejected a stale Shader property editor on {node_id}").into()
            };
        }
        DesignPanelAction::EffectShaderPropertyVariableDetachRequested {
            node_id,
            effect_id,
            index,
            shader_property_id,
            property_index,
            target,
            variable_id,
        } => {
            let detached = apply_story_effect_shader_variable_detach(
                node,
                effect_id,
                *index,
                shader_property_id,
                *property_index,
                *target,
                variable_id,
            );
            screen.last_action = if detached {
                format!(
                        "Host detached variable {variable_id} from Shader property {shader_property_id} on {node_id}"
                    )
                    .into()
            } else {
                format!("Host rejected a stale Shader variable detach on {node_id}").into()
            };
        }
        DesignPanelAction::EffectStyleApplyRequested { node_id, style } => {
            let resolved = screen.effect_styles.style(style).cloned();
            let applied = resolved.is_some_and(|style_data| {
                node.effects = style_data
                    .effect_kinds
                    .iter()
                    .enumerate()
                    .map(|(index, kind)| {
                        DesignEffect::new(*kind).with_id(format!("{node_id}-style-effect-{index}"))
                    })
                    .collect();
                node.effect_style_binding = Some(DesignEffectStyleBinding::new(
                    style.clone(),
                    style_data.name,
                ));
                true
            });
            screen.last_action = if applied {
                format!("Host applied Effect style {} on {node_id}", style.style_id).into()
            } else {
                format!("Host rejected unavailable Effect style on {node_id}").into()
            };
        }
        DesignPanelAction::EffectStyleCreateRequested { node_id, effects } => {
            screen.last_action = format!(
                "Host opened Effect-style creation for {} ordered effects on {node_id}",
                effects.len()
            )
            .into();
        }
        DesignPanelAction::EffectStyleDetachRequested { node_id, style } => {
            let detached = node
                .effect_style_binding
                .as_ref()
                .is_some_and(|binding| binding.can_detach && binding.selection == *style);
            if detached {
                node.effect_style_binding = None;
            }
            screen.last_action = if detached {
                format!("Host detached Effect style {} on {node_id}", style.style_id).into()
            } else {
                format!("Host rejected stale Effect-style detach on {node_id}").into()
            };
        }
        DesignPanelAction::EffectVariableApplyRequested {
            node_id,
            effect_id,
            index,
            field,
            variable_id,
        } => {
            let variable = screen
                .effect_variables
                .variable(variable_id.as_ref())
                .cloned();
            let resolved = if effect_id.is_empty() {
                (*index < node.effects.len()).then_some(*index)
            } else {
                node.effect_index_by_id(effect_id.as_ref())
            };
            let spread_supported = node.effect_capabilities.shadow_spread;
            let applied = resolved
                .and_then(|index| node.effects.get_mut(index))
                .zip(variable.as_ref())
                .is_some_and(|(effect, variable)| {
                    if !effect.variable_fields(spread_supported).contains(field)
                        || variable.kind != field.value_kind()
                    {
                        return false;
                    }
                    effect.set_variable_binding(DesignEffectVariableBinding::new(
                        *field,
                        variable.id.clone(),
                        variable.name.clone(),
                        variable.collection_name.clone(),
                    ));
                    true
                });
            screen.last_action = if applied {
                format!("Host bound {field:?} to {variable_id} on {node_id}").into()
            } else {
                format!("Host rejected incompatible effect variable on {node_id}").into()
            };
        }
        DesignPanelAction::EffectVariableDetachRequested {
            node_id,
            effect_id,
            index,
            field,
            variable_id,
        } => {
            let resolved = if effect_id.is_empty() {
                (*index < node.effects.len()).then_some(*index)
            } else {
                node.effect_index_by_id(effect_id.as_ref())
            };
            let detached = resolved
                .and_then(|index| node.effects.get_mut(index))
                .is_some_and(|effect| {
                    let can_detach = effect.variable_binding(*field).is_some_and(|binding| {
                        binding.can_detach && binding.variable_id == *variable_id
                    });
                    if can_detach {
                        effect.remove_variable_binding(*field);
                    }
                    can_detach
                });
            screen.last_action = if detached {
                format!("Host detached {variable_id} from {field:?} on {node_id}").into()
            } else {
                format!("Host rejected stale effect-variable detach on {node_id}").into()
            };
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

pub(crate) fn story_effect_shader_property_mut<'a>(
    node: &'a mut DesignPanelNode,
    effect_id: &SharedString,
    effect_index: usize,
    shader_property_id: &SharedString,
    property_index: usize,
) -> Option<&'a mut DesignShaderProperty> {
    let effect_index = if effect_id.is_empty() {
        (effect_index < node.effects.len()).then_some(effect_index)
    } else {
        node.effect_index_by_id(effect_id.as_ref())
    }?;
    let effect = node.effects.get_mut(effect_index)?;
    let DesignEffectSettings::Shader(shader) = &mut effect.settings else {
        return None;
    };
    let property_index = if shader_property_id.is_empty() {
        (property_index < shader.properties.len()).then_some(property_index)
    } else {
        shader
            .properties
            .iter()
            .position(|property| property.definition_id == *shader_property_id)
    }?;
    shader.properties.get_mut(property_index)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_story_effect_shader_property_editor(
    node: &mut DesignPanelNode,
    effect_id: &SharedString,
    effect_index: usize,
    shader_property_id: &SharedString,
    property_index: usize,
    property_kind: DesignShaderPropertyKind,
    target: DesignShaderPropertyEditorTarget,
    editor: DesignShaderPropertyEditorKind,
    current_value: &DesignShaderPropertyValue,
) -> bool {
    let Some(property) = story_effect_shader_property_mut(
        node,
        effect_id,
        effect_index,
        shader_property_id,
        property_index,
    ) else {
        return false;
    };
    if property.kind != property_kind || property.read_only || property.value != *current_value {
        return false;
    }
    match (editor, target) {
        (DesignShaderPropertyEditorKind::Resource, DesignShaderPropertyEditorTarget::Value) => {
            let asset_id = match property.kind {
                DesignShaderPropertyKind::Image => "image:storybook-replacement",
                DesignShaderPropertyKind::InstanceSwap => "component:storybook-replacement",
                DesignShaderPropertyKind::Slot => "slot:storybook-replacement",
                _ => return false,
            };
            property.value = DesignShaderPropertyValue::AssetId(asset_id.into());
        }
        (DesignShaderPropertyEditorKind::Variable, DesignShaderPropertyEditorTarget::Value) => {
            property.value = DesignShaderPropertyValue::VariableAlias {
                variable_id: format!("variable:shader:{}", property.definition_id).into(),
            };
        }
        (
            DesignShaderPropertyEditorKind::Variable,
            DesignShaderPropertyEditorTarget::ColorPointColor,
        ) => {
            let DesignShaderPropertyValue::ColorPoint { variable_id, .. } = &mut property.value
            else {
                return false;
            };
            *variable_id = Some(format!("variable:shader:{}", property.definition_id).into());
        }
        (
            DesignShaderPropertyEditorKind::Variable,
            DesignShaderPropertyEditorTarget::GradientStopColor(stop_index),
        ) => {
            let DesignShaderPropertyValue::Gradient(stops) = &mut property.value else {
                return false;
            };
            let Some(stop) = stops.get_mut(stop_index) else {
                return false;
            };
            stop.variable_id = Some(
                format!(
                    "variable:shader:{}:stop:{stop_index}",
                    property.definition_id
                )
                .into(),
            );
        }
        (DesignShaderPropertyEditorKind::Resource, _) => return false,
    }
    true
}

pub(crate) fn apply_story_effect_shader_variable_detach(
    node: &mut DesignPanelNode,
    effect_id: &SharedString,
    effect_index: usize,
    shader_property_id: &SharedString,
    property_index: usize,
    target: DesignShaderPropertyEditorTarget,
    variable_id: &SharedString,
) -> bool {
    let Some(property) = story_effect_shader_property_mut(
        node,
        effect_id,
        effect_index,
        shader_property_id,
        property_index,
    ) else {
        return false;
    };
    match target {
        DesignShaderPropertyEditorTarget::Value => {
            let DesignShaderPropertyValue::VariableAlias {
                variable_id: current,
            } = &property.value
            else {
                return false;
            };
            if current != variable_id {
                return false;
            }
            property.value = story_shader_property_fallback(property.kind);
        }
        DesignShaderPropertyEditorTarget::ColorPointColor => {
            let DesignShaderPropertyValue::ColorPoint {
                variable_id: current,
                ..
            } = &mut property.value
            else {
                return false;
            };
            if current.as_ref() != Some(variable_id) {
                return false;
            }
            *current = None;
        }
        DesignShaderPropertyEditorTarget::GradientStopColor(stop_index) => {
            let DesignShaderPropertyValue::Gradient(stops) = &mut property.value else {
                return false;
            };
            let Some(stop) = stops.get_mut(stop_index) else {
                return false;
            };
            if stop.variable_id.as_ref() != Some(variable_id) {
                return false;
            }
            stop.variable_id = None;
        }
    }
    true
}
