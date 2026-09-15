//! Component properties, swaps, slots, instance commands, and
//! main-component authoring (definitions and Variant options).

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.host.nodes[node_index];
    match action {
        DesignPanelAction::ComponentPropertyChangeRequested {
            node_id,
            property_id,
            value,
        } => {
            let instance_role = node.component_context.as_ref().is_some_and(|context| {
                matches!(
                    context.role,
                    DesignComponentRole::Instance | DesignComponentRole::SlotInstance
                )
            });
            let applied = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
                .is_some_and(|property| {
                    if !property.set_resolved_value(value.clone()) {
                        return false;
                    }
                    if instance_role {
                        if property.resolved_value == property.definition.default_value() {
                            property.reset_to_default();
                        } else {
                            property.mark_overridden();
                        }
                    }
                    true
                });
            refresh_story_component_override_summary(node);
            screen.harness.last_action = if applied {
                format!("Host applied component property {property_id} on {node_id}").into()
            } else {
                format!("Host rejected component property {property_id} on {node_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyEditRequested {
            node_id,
            property_id,
            value,
            phase,
        } => {
            if *phase != DesignPanelEditPhase::Begin {
                let instance_role = node.component_context.as_ref().is_some_and(|context| {
                    matches!(
                        context.role,
                        DesignComponentRole::Instance | DesignComponentRole::SlotInstance
                    )
                });
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    && property.set_resolved_value(value.clone())
                    && instance_role
                {
                    if property.resolved_value == property.definition.default_value() {
                        property.reset_to_default();
                    } else {
                        property.mark_overridden();
                    }
                }
                refresh_story_component_override_summary(node);
            }
            screen.harness.last_action = format!(
                "Host observed {phase:?} for component property {property_id} on {node_id}"
            )
            .into();
        }
        DesignPanelAction::ComponentPropertyResetRequested {
            node_id,
            property_id,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
            {
                property.reset_to_default();
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host reset component property {property_id} on {node_id}").into();
        }
        DesignPanelAction::ComponentPropertyVariableApplyRequested {
            node_id,
            target,
            variable_id,
        } => {
            let role = node.component_context.as_ref().map(|context| context.role);
            let variable = screen
                .host
                .property_variables
                .variable(variable_id.as_ref())
                .filter(|variable| {
                    variable.resolved_type == target.resolved_type
                        && variable.disabled_reason.is_none()
                        && variable.import_state != DesignVariableImportState::Available
                        && variable.resolved_value.is_some()
                })
                .cloned();
            let applied = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == target.property_id)
                .zip(role)
                .filter(|(property, role)| property.variable_target(*role).as_ref() == Some(target))
                .zip(variable.as_ref())
                .is_some_and(|((property, _), variable)| {
                    let Some(resolved_value) = variable.resolved_value.clone() else {
                        return false;
                    };
                    let binding = DesignComponentPropertyVariableBinding::new(
                        variable.id.clone(),
                        format!("{} / {}", variable.collection_name, variable.name),
                        resolved_value,
                    );
                    match target.field {
                        DesignComponentPropertyVariableField::DefinitionDefaultValue => {
                            property.default_value_binding = Some(binding);
                        }
                        DesignComponentPropertyVariableField::InstanceValue => {
                            property.resolved_value_binding = Some(binding);
                        }
                    }
                    true
                });
            screen.harness.last_action = if applied {
                format!(
                    "Host bound {variable_id} to component property {}.{} on {node_id}",
                    target.property_id,
                    target.field.api_name()
                )
                .into()
            } else {
                format!("Host rejected stale component variable apply {variable_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyVariableImportRequested {
            node_id,
            target,
            variable_id,
        } => {
            let target_is_current = node
                .component_context
                .as_ref()
                .and_then(|context| {
                    node.component_properties
                        .iter()
                        .find(|property| property.id == target.property_id)
                        .and_then(|property| property.variable_target(context.role))
                })
                .as_ref()
                == Some(target);
            let imported = target_is_current
                && screen
                    .host
                    .property_variables
                    .variables
                    .iter_mut()
                    .find(|variable| {
                        variable.id == *variable_id
                            && variable.resolved_type == target.resolved_type
                            && variable.disabled_reason.is_none()
                            && variable.import_state == DesignVariableImportState::Available
                    })
                    .is_some_and(|variable| {
                        variable.import_state = DesignVariableImportState::Imported;
                        true
                    });
            screen.harness.last_action = if imported {
                format!(
                        "Host imported {variable_id} for component property {}.{} on {node_id}; choose it again to apply",
                        target.property_id,
                        target.field.api_name()
                    )
                    .into()
            } else {
                format!("Host rejected stale component variable import {variable_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyVariableDetachRequested {
            node_id,
            target,
            variable_id,
        } => {
            let role = node.component_context.as_ref().map(|context| context.role);
            let detached = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == target.property_id)
                .zip(role)
                .filter(|(property, role)| property.variable_target(*role).as_ref() == Some(target))
                .is_some_and(|(property, _)| {
                    let binding = match target.field {
                        DesignComponentPropertyVariableField::DefinitionDefaultValue => {
                            &mut property.default_value_binding
                        }
                        DesignComponentPropertyVariableField::InstanceValue => {
                            &mut property.resolved_value_binding
                        }
                    };
                    let can_detach = binding.as_ref().is_some_and(|binding| {
                        binding.variable_id == *variable_id && binding.can_detach
                    });
                    if can_detach {
                        *binding = None;
                    }
                    can_detach
                });
            screen.harness.last_action = if detached {
                format!(
                    "Host detached {variable_id} from component property {}.{} on {node_id}",
                    target.property_id,
                    target.field.api_name()
                )
                .into()
            } else {
                format!("Host rejected stale component variable detach {variable_id}").into()
            };
        }
        DesignPanelAction::ComponentSwapApplyRequested {
            node_id,
            property_id,
            selection,
        } => {
            let replacement = selection.as_ref().and_then(|selection| {
                screen
                    .host
                    .component_swaps
                    .candidate(selection)
                    .filter(|candidate| candidate.can_apply())
                    .map(|candidate| candidate.reference.clone())
            });
            let selection_is_valid = selection.is_none() || replacement.is_some();
            let instance_role = node.component_context.as_ref().is_some_and(|context| {
                matches!(
                    context.role,
                    DesignComponentRole::Instance | DesignComponentRole::SlotInstance
                )
            });
            let applied = selection_is_valid
                && node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    .is_some_and(|property| {
                        if !property.set_resolved_value(DesignComponentPropertyValue::InstanceSwap(
                            replacement,
                        )) {
                            return false;
                        }
                        if instance_role {
                            if property.resolved_value == property.definition.default_value() {
                                property.reset_to_default();
                            } else {
                                property.mark_overridden();
                            }
                        }
                        true
                    });
            refresh_story_component_override_summary(node);
            screen.harness.last_action = if applied {
                format!("Host applied component swap {selection:?} to {property_id} on {node_id}")
                    .into()
            } else {
                format!("Host rejected stale component swap on {property_id}").into()
            };
        }
        DesignPanelAction::ComponentSwapImportRequested {
            node_id,
            property_id,
            selection,
        } => {
            let property_accepts_swap = node
                .component_properties
                .iter()
                .find(|property| property.id == *property_id)
                .is_some_and(|property| {
                    property.kind == fanta_gpui::prelude::DesignComponentPropertyKind::InstanceSwap
                });
            let imported = property_accepts_swap
                && screen
                    .host
                    .component_swaps
                    .candidates
                    .iter_mut()
                    .find(|candidate| candidate.selection() == *selection)
                    .filter(|candidate| candidate.can_import())
                    .is_some_and(|candidate| {
                        candidate.import_state = DesignComponentImportState::Imported;
                        true
                    });
            screen.harness.last_action = if imported {
                format!(
                    "Host imported {} for {property_id} on {node_id}; choose it again to apply",
                    selection.component_key
                )
                .into()
            } else {
                format!(
                    "Host rejected stale component import {}",
                    selection.component_key
                )
                .into()
            };
        }
        DesignPanelAction::ComponentSwapPreviewRequested {
            node_id,
            property_id,
            selection,
        } => {
            let valid = selection.as_ref().is_none_or(|selection| {
                screen
                    .host
                    .component_swaps
                    .candidate(selection)
                    .is_some_and(|candidate| candidate.can_apply())
            });
            screen.harness.last_action = if valid {
                format!(
                    "Host {} component-swap preview for {property_id} on {node_id}: {selection:?}",
                    if selection.is_some() {
                        "showed"
                    } else {
                        "cleared"
                    }
                )
                .into()
            } else {
                format!("Host rejected stale component-swap preview on {property_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyNestedInstanceSelectRequested {
            node_id,
            property_id,
            instance_id,
        } => {
            let valid = node
                .component_properties
                .iter()
                .find(|property| property.id == *property_id)
                .is_some_and(|property| {
                    matches!(
                        &property.origin,
                        DesignComponentPropertyOrigin::NestedInstance {
                            instance_id: current,
                            ..
                        } if current == instance_id
                    )
                });
            screen.harness.last_action = if valid {
                format!(
                    "Host selected nested instance {instance_id} from {property_id} on {node_id}"
                )
                .into()
            } else {
                format!("Host rejected stale nested-instance selection {instance_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyNestedInstanceGoToMainRequested {
            node_id,
            property_id,
            instance_id,
            main_component_id,
        } => {
            let valid = node
                .component_properties
                .iter()
                .find(|property| property.id == *property_id)
                .is_some_and(|property| {
                    matches!(
                        &property.origin,
                        DesignComponentPropertyOrigin::NestedInstance {
                            instance_id: current,
                            main_component: Some(main),
                            ..
                        } if current == instance_id && main.id == *main_component_id
                    )
                });
            screen.harness.last_action = if valid {
                format!(
                        "Host opened nested main component {main_component_id} for {instance_id} on {node_id}"
                    )
                    .into()
            } else {
                format!("Host rejected stale nested-main navigation {main_component_id}").into()
            };
        }
        DesignPanelAction::SlotSettingsChangeRequested {
            node_id,
            property_id,
            expected_settings,
            change,
            phase,
        } => {
            let replacement = node
                .component_properties
                .iter()
                .find(|property| property.id == *property_id)
                .filter(|property| property.slot_settings() == Some(expected_settings))
                .cloned()
                .and_then(|mut property| {
                    property
                        .apply_slot_settings_change(change)
                        .then_some(property)
                })
                .filter(|property| {
                    property.slot_settings().is_some_and(|settings| {
                        settings.has_valid_child_range()
                            && story_component_references_are_current(
                                &screen.host.component_swaps,
                                &settings.preferred_values,
                            )
                    })
                });
            let accepted = replacement.is_some();
            if *phase != DesignPanelEditPhase::Begin
                && let Some(replacement) = replacement
                && let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
            {
                *property = replacement;
            }
            screen.harness.last_action = if accepted {
                format!("Host observed {phase:?} for slot settings {property_id} on {node_id}")
                    .into()
            } else {
                format!("Host rejected stale slot settings {property_id} on {node_id}").into()
            };
        }
        DesignPanelAction::SlotResetRequested {
            node_id,
            property_id,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
            {
                property.reset_to_default();
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host reset slot {property_id} on {node_id}").into();
        }
        DesignPanelAction::SlotClearRequested {
            node_id,
            property_id,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
                && property
                    .set_resolved_value(DesignComponentPropertyValue::Slot(Default::default()))
            {
                property.mark_overridden();
                property.refresh_slot_violations();
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host cleared slot {property_id} on {node_id}").into();
        }
        DesignPanelAction::SlotAddInstanceRequested {
            node_id,
            property_id,
            preferred_component,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
            {
                let component = preferred_component.clone().or_else(|| {
                    property
                        .slot_settings()
                        .and_then(|settings| {
                            settings
                                .preferred_values
                                .iter()
                                .find(|component| component.availability.is_available())
                        })
                        .cloned()
                });
                if let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value() {
                    let instance_index = slot.children.len();
                    let mut instance = if let Some(component) = component {
                        let mut instance = DesignSlotChild::instance(
                            format!("{node_id}-{property_id}-slot-{instance_index}"),
                            component.name.clone(),
                        );
                        instance.main_component = Some(component);
                        instance
                    } else {
                        DesignSlotChild::instance(
                            format!("{node_id}-{property_id}-slot-{instance_index}"),
                            "Inserted layer",
                        )
                    };
                    if instance.name.is_empty() {
                        instance.name = "Inserted layer".into();
                    }
                    slot.children.push(instance);
                    if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                        property.mark_overridden();
                        property.refresh_slot_violations();
                    }
                }
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host added an instance to slot {property_id} on {node_id}").into();
        }
        DesignPanelAction::SlotChildSelectRequested {
            node_id,
            property_id,
            child_node_id,
        } => {
            screen.harness.last_action =
                format!("Host selected slot child {child_node_id} in {property_id} on {node_id}")
                    .into();
        }
        DesignPanelAction::SlotLimitLayersSelectRequested {
            node_id,
            property_id,
            child_node_ids,
        } => {
            let expected = node
                .component_properties
                .iter()
                .find(|property| property.id == *property_id)
                .filter(|property| {
                    property
                        .slot_settings()
                        .is_some_and(|settings| settings.preferred_values_only)
                })
                .and_then(|property| {
                    let state = property.slot_state.as_ref()?;
                    let violating_ids = state
                        .violations
                        .iter()
                        .filter_map(|violation| match violation {
                            DesignSlotViolation::NonPreferredValue { instance_id, .. }
                            | DesignSlotViolation::MissingMainComponent { instance_id } => {
                                Some(instance_id.clone())
                            }
                            DesignSlotViolation::NonPreferredChild { child_id, .. } => {
                                Some(child_id.clone())
                            }
                            DesignSlotViolation::BelowMinimum { .. }
                            | DesignSlotViolation::AboveMaximum { .. } => None,
                        })
                        .collect::<HashSet<_>>();
                    let DesignComponentPropertyValue::Slot(value) = property.effective_value()
                    else {
                        return None;
                    };
                    Some(
                        value
                            .children
                            .iter()
                            .filter(|child| violating_ids.contains(&child.node_id))
                            .map(|child| child.node_id.clone())
                            .collect::<Vec<_>>(),
                    )
                })
                .unwrap_or_default();
            let all_selectable =
                node.component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .and_then(|property| {
                        let DesignComponentPropertyValue::Slot(value) = property.effective_value()
                        else {
                            return None;
                        };
                        Some(expected.iter().all(|expected_id| {
                            value.children.iter().any(|child| {
                                child.node_id == *expected_id && child.capabilities.select
                            })
                        }))
                    })
                    .unwrap_or(false);
            let accepted = !expected.is_empty() && expected == *child_node_ids && all_selectable;
            screen.harness.last_action = if accepted {
                format!(
                    "Host selected {} non-preferred slot layers in {property_id} on {node_id}",
                    child_node_ids.len()
                )
                .into()
            } else {
                format!("Host rejected stale slot-limit layers for {property_id} on {node_id}")
                    .into()
            };
        }
        DesignPanelAction::SlotChildRemoveRequested {
            node_id,
            property_id,
            child_node_id,
            index,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
                && let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                && let Some(resolved_index) = slot
                    .children
                    .iter()
                    .position(|child| child.node_id == *child_node_id)
            {
                let _index_hint_matches = resolved_index == *index;
                slot.children.remove(resolved_index);
                if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                    property.mark_overridden();
                    property.refresh_slot_violations();
                }
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host removed slot child {child_node_id} from {property_id} on {node_id}")
                    .into();
        }
        DesignPanelAction::SlotChildReorderRequested {
            node_id,
            property_id,
            child_node_id,
            from_index,
            to_index,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
                && let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                && let Some(resolved_index) = slot
                    .children
                    .iter()
                    .position(|child| child.node_id == *child_node_id)
                && *to_index < slot.children.len()
            {
                let _index_hint_matches = resolved_index == *from_index;
                let child = slot.children.remove(resolved_index);
                let destination = (*to_index).min(slot.children.len());
                slot.children.insert(destination, child);
                if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                    property.mark_overridden();
                    property.refresh_slot_violations();
                }
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host reordered slot child {child_node_id} in {property_id} on {node_id}")
                    .into();
        }
        DesignPanelAction::SlotChildReplaceRequested {
            node_id,
            property_id,
            child_node_id,
            replacement,
        } => {
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| property.id == *property_id)
                && let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                && let Some(child) = slot
                    .children
                    .iter_mut()
                    .find(|child| child.node_id == *child_node_id)
            {
                child.name = replacement.name.clone();
                child.kind = DesignPanelNodeKind::Instance;
                child.main_component = Some(replacement.clone());
                if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                    property.mark_overridden();
                    property.refresh_slot_violations();
                }
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action =
                format!("Host swapped slot child {child_node_id} in {property_id} on {node_id}")
                    .into();
        }
        DesignPanelAction::ResetInstanceOverridesRequested { node_id } => {
            for property in &mut node.component_properties {
                if property.reset_state.is_overridden() {
                    property.reset_to_default();
                }
            }
            if let Some(context) = node.component_context.as_mut() {
                context.overrides.nested_override_count = 0;
            }
            refresh_story_component_override_summary(node);
            screen.harness.last_action = format!("Host reset overrides for {node_id}").into();
        }
        DesignPanelAction::GoToMainComponentRequested { node_id } => {
            screen.harness.last_action =
                format!("Host navigated to the main component for {node_id}").into();
        }
        DesignPanelAction::DetachInstanceRequested { node_id } => {
            screen.harness.last_action = format!("Host detached instance {node_id}").into();
        }
        DesignPanelAction::ComponentPropertyDefinitionCreateRequested {
            node_id,
            kind,
            name,
            description,
            documentation_links,
            definition,
            default_variable_id,
            partition,
            expected_property_order,
            after_property_id,
        } => {
            if !story_component_property_create_is_current(
                node,
                *kind,
                name,
                description,
                documentation_links,
                definition,
                *partition,
                expected_property_order,
                after_property_id.as_ref(),
            ) || !story_component_definition_references_are_current(
                &screen.host.component_swaps,
                definition,
            ) {
                screen.harness.last_action =
                    format!("Host rejected stale component-property create on {node_id}").into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            }
            let Some(default_value_binding) = story_component_default_variable(
                &screen.host.property_variables,
                *kind,
                default_variable_id.as_ref(),
            ) else {
                screen.harness.last_action = format!(
                        "Host rejected stale default-variable binding for component-property create on {node_id}"
                    )
                    .into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            };
            let property_id = next_story_component_property_id(node, *kind);
            let mut property = match definition {
                DesignComponentPropertyDefinition::Variant {
                    default_value,
                    options,
                } => DesignComponentProperty::variant(
                    property_id.clone(),
                    name.clone(),
                    default_value.clone(),
                    default_value.clone(),
                    options.clone(),
                ),
                DesignComponentPropertyDefinition::Boolean { default_value } => {
                    DesignComponentProperty::boolean(
                        property_id.clone(),
                        name.clone(),
                        *default_value,
                        *default_value,
                    )
                }
                DesignComponentPropertyDefinition::Text {
                    default_value,
                    multiline,
                } => DesignComponentProperty::text(
                    property_id.clone(),
                    name.clone(),
                    default_value.clone(),
                    default_value.clone(),
                )
                .with_multiline(*multiline),
                DesignComponentPropertyDefinition::InstanceSwap {
                    default_value,
                    preferred_values,
                } => DesignComponentProperty::instance_swap(
                    property_id.clone(),
                    name.clone(),
                    default_value.clone(),
                    default_value.clone(),
                    preferred_values.clone(),
                ),
                DesignComponentPropertyDefinition::Slot {
                    default_value,
                    settings,
                } => DesignComponentProperty::slot(
                    property_id.clone(),
                    name.clone(),
                    default_value.clone(),
                    default_value.clone(),
                    settings.clone(),
                    DesignSlotState::default(),
                ),
            };
            property.description = description.clone();
            property.documentation_links = documentation_links.clone();
            property.default_value_binding = default_value_binding;
            property.refresh_slot_violations();
            let insert_index = if *kind == DesignComponentPropertyKind::Variant {
                node.component_properties
                    .iter()
                    .take_while(|property| {
                        property.definition.kind() == DesignComponentPropertyKind::Variant
                    })
                    .count()
            } else {
                node.component_properties.len()
            };
            let variant_option_labels = property.definition.option_labels();
            node.component_properties.insert(insert_index, property);
            if let Some(authoring) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
            {
                let mut definition =
                    DesignComponentPropertyDefinitionAuthoring::editable(property_id.clone());
                if *kind == DesignComponentPropertyKind::Variant {
                    definition = definition.with_variant_options(
                        variant_option_labels.clone().into_iter().enumerate().map(
                            |(index, option)| {
                                DesignComponentVariantOptionAuthoring::editable(
                                    format!("{property_id}-option-{index}"),
                                    option,
                                )
                            },
                        ),
                    );
                }
                authoring.definitions.push(definition);
            }
            screen.harness.last_action =
                format!("Host created {} on {node_id}", kind.label()).into();
        }
        DesignPanelAction::ComponentPropertyDefinitionRenameRequested {
            node_id,
            property_id,
            original_name,
            expected_name,
            name,
            phase,
        } => {
            let key = (node_id.clone(), property_id.clone());
            let renamed = node
                .component_properties
                .iter_mut()
                .find(|property| &property.id == property_id)
                .is_some_and(|property| {
                    apply_story_component_authoring_name_edit(
                        &mut property.name,
                        &mut screen.edits.component_property_name_edits,
                        key,
                        original_name,
                        expected_name,
                        name,
                        *phase,
                    )
                });
            screen.harness.last_action = if renamed {
                format!("Host echoed {phase:?} rename for {property_id} on {node_id}").into()
            } else {
                format!("Host rejected stale rename for {property_id} on {node_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyDefinitionMetadataEditRequested {
            node_id,
            property_id,
            expected_description,
            description,
            expected_documentation_links,
            documentation_links,
        } => {
            let can_edit = node
                .component_context
                .as_ref()
                .and_then(|context| context.authoring.as_ref())
                .and_then(|authoring| authoring.definition(property_id.as_ref()))
                .is_some_and(|definition| definition.capabilities.edit_metadata);
            let metadata_is_valid = description
                .as_ref()
                .is_none_or(|description| !description.trim().is_empty())
                && documentation_links
                    .iter()
                    .all(|link| !link.label.trim().is_empty() && !link.url.trim().is_empty());
            let edited = can_edit
                && metadata_is_valid
                && node
                    .component_properties
                    .iter_mut()
                    .find(|property| &property.id == property_id)
                    .filter(|property| {
                        property.description == *expected_description
                            && property.documentation_links == *expected_documentation_links
                    })
                    .is_some_and(|property| {
                        property.description = description.clone();
                        property.documentation_links = documentation_links.clone();
                        true
                    });
            screen.harness.last_action = if edited {
                format!("Host edited metadata for {property_id} on {node_id}").into()
            } else {
                format!("Host rejected stale metadata edit for {property_id} on {node_id}").into()
            };
        }
        DesignPanelAction::ComponentPropertyDefinitionEditRequested {
            node_id,
            property_id,
            expected_description,
            description,
            expected_documentation_links,
            documentation_links,
            expected_definition,
            definition,
        } => {
            let edited = apply_story_component_property_definition_edit(
                node,
                &screen.host.component_swaps,
                property_id,
                expected_description,
                description,
                expected_documentation_links,
                documentation_links,
                expected_definition,
                definition,
            );
            screen.harness.last_action = if edited {
                format!("Host atomically edited component property {property_id} on {node_id}")
                    .into()
            } else {
                format!(
                        "Host rejected stale atomic component-property edit for {property_id} on {node_id}"
                    )
                    .into()
            };
        }
        DesignPanelAction::ComponentPropertyDefinitionDeleteRequested {
            node_id,
            property_id,
            expected_name,
        } => {
            if !story_component_property_delete_is_current(
                node,
                property_id.as_ref(),
                expected_name.as_ref(),
            ) {
                screen.harness.last_action =
                    format!("Host rejected stale delete for {property_id} on {node_id}").into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            }
            node.component_properties
                .retain(|property| &property.id != property_id);
            screen
                .edits
                .component_property_name_edits
                .remove(&(node_id.clone(), property_id.clone()));
            invalidate_story_component_property_reorders(
                &mut screen.edits.component_property_reorders,
                node_id,
                property_id,
            );
            screen.edits.component_variant_option_name_edits.retain(
                |(active_node_id, active_property_id, _), _| {
                    active_node_id != node_id || active_property_id != property_id
                },
            );
            screen.edits.component_variant_option_reorders.retain(
                |(active_node_id, active_property_id, _), _| {
                    active_node_id != node_id || active_property_id != property_id
                },
            );
            if let Some(authoring) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
            {
                authoring
                    .definitions
                    .retain(|definition| &definition.property_id != property_id);
                for control in &mut authoring.applied_properties {
                    control
                        .candidates
                        .retain(|candidate| &candidate.property_id != property_id);
                    if control.applied_property_id.as_ref() == Some(property_id) {
                        control.applied_property_id = None;
                    }
                }
            }
            screen.harness.last_action =
                format!("Host deleted {property_id} from {node_id}").into();
        }
        DesignPanelAction::ComponentPropertyDefinitionReorderRequested {
            node_id,
            property_id,
            partition,
            original_property_order,
            expected_property_order,
            before_property_id,
            phase,
        } => {
            let current_order = node
                .component_properties
                .iter()
                .filter(|property| {
                    DesignComponentPropertyPartition::for_kind(property.definition.kind())
                        == *partition
                })
                .map(|property| property.id.clone())
                .collect::<Vec<_>>();
            let key = (node_id.clone(), property_id.clone());
            let Some(desired_order) = apply_story_component_authoring_reorder(
                &current_order,
                &mut screen.edits.component_property_reorders,
                key,
                property_id,
                original_property_order,
                expected_property_order,
                before_property_id.as_ref(),
                *phase,
            ) else {
                screen.harness.last_action =
                        format!("Host rejected stale or unbalanced {phase:?} reorder for {property_id} on {node_id}").into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            };
            if echo_story_component_property_order(node, *partition, &desired_order) {
                screen.harness.last_action =
                    format!("Host echoed {phase:?} reorder for {property_id} on {node_id}").into();
            } else {
                screen.harness.last_action =
                    format!("Host failed to echo reorder for {property_id} on {node_id}").into();
            }
        }
        DesignPanelAction::ComponentVariantOptionCreateRequested {
            node_id,
            property_id,
            name,
            expected_option_order,
            after_option_id,
        } => {
            if !story_component_variant_option_create_is_current(
                node,
                property_id.as_ref(),
                name,
                expected_option_order,
                after_option_id.as_ref(),
            ) {
                screen.harness.last_action = format!(
                    "Host rejected stale Variant-value create on {property_id} in {node_id}"
                )
                .into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            }
            let option_id = next_story_component_variant_option_id(node, property_id.as_ref());
            let option_name = name.clone();
            if let Some(property) = node
                .component_properties
                .iter_mut()
                .find(|property| &property.id == property_id)
                && let DesignComponentPropertyDefinition::Variant { options, .. } =
                    &mut property.definition
            {
                options.push(option_name.clone());
                property.preferred_values = property.definition.option_labels();
            }
            if let Some(definition) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
                .and_then(|authoring| {
                    authoring
                        .definitions
                        .iter_mut()
                        .find(|definition| &definition.property_id == property_id)
                })
            {
                definition
                    .variant_options
                    .push(DesignComponentVariantOptionAuthoring::editable(
                        option_id,
                        option_name,
                    ));
            }
            screen.harness.last_action =
                format!("Host created a Variant option on {property_id} in {node_id}").into();
        }
        DesignPanelAction::ComponentVariantOptionRenameRequested {
            node_id,
            property_id,
            option_id,
            original_name,
            expected_name,
            name,
            phase,
        } => {
            let key = (node_id.clone(), property_id.clone(), option_id.clone());
            let edited = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
                .and_then(|authoring| {
                    authoring
                        .definitions
                        .iter_mut()
                        .find(|definition| &definition.property_id == property_id)
                })
                .and_then(|definition| {
                    definition
                        .variant_options
                        .iter_mut()
                        .find(|option| &option.id == option_id)
                })
                .is_some_and(|option| {
                    apply_story_component_authoring_name_edit(
                        &mut option.name,
                        &mut screen.edits.component_variant_option_name_edits,
                        key,
                        original_name,
                        expected_name,
                        name,
                        *phase,
                    )
                });
            if edited {
                let option_index_and_name = node
                    .component_context
                    .as_ref()
                    .and_then(|context| context.authoring.as_ref())
                    .and_then(|authoring| authoring.definition(property_id.as_ref()))
                    .and_then(|definition| {
                        definition
                            .variant_options
                            .iter()
                            .position(|option| &option.id == option_id)
                            .map(|index| (index, definition.variant_options[index].name.clone()))
                    });
                if let Some((option_index, next_name)) = option_index_and_name
                    && let Some(property) = node
                        .component_properties
                        .iter_mut()
                        .find(|property| &property.id == property_id)
                    && let DesignComponentPropertyDefinition::Variant { options, .. } =
                        &mut property.definition
                    && let Some(option) = options.get_mut(option_index)
                {
                    *option = next_name;
                    property.preferred_values = property.definition.option_labels();
                }
                screen.harness.last_action =
                    format!("Host echoed {phase:?} rename for {option_id} on {node_id}").into();
            } else {
                screen.harness.last_action =
                    format!("Host rejected stale Variant-value rename for {option_id}").into();
            }
        }
        DesignPanelAction::ComponentVariantOptionDeleteRequested {
            node_id,
            property_id,
            option_id,
            expected_name,
        } => {
            if !story_component_variant_option_delete_is_current(
                node,
                property_id.as_ref(),
                option_id.as_ref(),
                expected_name.as_ref(),
            ) {
                screen.harness.last_action =
                    format!("Host rejected stale Variant-value delete for {option_id}").into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            }
            let option_index = node
                .component_context
                .as_ref()
                .and_then(|context| context.authoring.as_ref())
                .and_then(|authoring| authoring.definition(property_id.as_ref()))
                .and_then(|definition| {
                    definition
                        .variant_options
                        .iter()
                        .position(|option| &option.id == option_id)
                });
            if let Some(option_index) = option_index {
                screen.edits.component_variant_option_name_edits.remove(&(
                    node_id.clone(),
                    property_id.clone(),
                    option_id.clone(),
                ));
                invalidate_story_component_variant_option_reorders(
                    &mut screen.edits.component_variant_option_reorders,
                    node_id,
                    property_id,
                );
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| &property.id == property_id)
                    && let DesignComponentPropertyDefinition::Variant { options, .. } =
                        &mut property.definition
                    && option_index < options.len()
                {
                    options.remove(option_index);
                    property.preferred_values = property.definition.option_labels();
                }
                if let Some(definition) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .definitions
                            .iter_mut()
                            .find(|definition| &definition.property_id == property_id)
                    })
                    && option_index < definition.variant_options.len()
                {
                    definition.variant_options.remove(option_index);
                }
                screen.harness.last_action =
                    format!("Host deleted {option_id} on {node_id}").into();
            }
        }
        DesignPanelAction::ComponentVariantOptionReorderRequested {
            node_id,
            property_id,
            option_id,
            original_option_order,
            expected_option_order,
            before_option_id,
            phase,
        } => {
            let current_order = node
                .component_context
                .as_ref()
                .and_then(|context| context.authoring.as_ref())
                .and_then(|authoring| authoring.definition(property_id.as_ref()))
                .map(|definition| {
                    definition
                        .variant_options
                        .iter()
                        .map(|option| option.id.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let key = (node_id.clone(), property_id.clone(), option_id.clone());
            let Some(desired_order) = apply_story_component_authoring_reorder(
                &current_order,
                &mut screen.edits.component_variant_option_reorders,
                key,
                option_id,
                original_option_order,
                expected_option_order,
                before_option_id.as_ref(),
                *phase,
            ) else {
                screen.harness.last_action =
                        format!("Host rejected stale or unbalanced {phase:?} Variant-value reorder for {option_id}").into();
                cx.notify();
                return Some(NodeOutcome::Complete);
            };
            if echo_story_component_variant_option_order(node, property_id.as_ref(), &desired_order)
            {
                screen.harness.last_action =
                    format!("Host echoed {phase:?} reorder for {option_id} on {node_id}").into();
            } else {
                screen.harness.last_action =
                    format!("Host failed to echo Variant-value reorder for {option_id}").into();
            }
        }
        DesignPanelAction::ComponentPropertyApplyToLayerRequested {
            node_id,
            control_id,
            property_id,
            ..
        }
        | DesignPanelAction::ComponentPropertySwitchOnLayerRequested {
            node_id,
            control_id,
            to_property_id: property_id,
            ..
        } => {
            if let Some(control) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
                .and_then(|authoring| {
                    authoring
                        .applied_properties
                        .iter_mut()
                        .find(|control| &control.control_id == control_id)
                })
            {
                control.applied_property_id = Some(property_id.clone());
                screen.harness.last_action =
                    format!("Host applied {property_id} to a layer in {node_id}").into();
            }
        }
        DesignPanelAction::ComponentPropertyDetachFromLayerRequested {
            node_id,
            control_id,
            ..
        } => {
            if let Some(control) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
                .and_then(|authoring| {
                    authoring
                        .applied_properties
                        .iter_mut()
                        .find(|control| &control.control_id == control_id)
                })
            {
                control.applied_property_id = None;
                screen.harness.last_action =
                    format!("Host detached a property from a layer in {node_id}").into();
            }
        }
        DesignPanelAction::NestedComponentPropertyExposeRequested {
            node_id,
            candidate_id,
            ..
        } => {
            if let Some(candidate) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
                .and_then(|authoring| {
                    authoring
                        .exposure_candidates
                        .iter_mut()
                        .find(|candidate| &candidate.candidate_id == candidate_id)
                })
            {
                candidate.exposed_property_id = Some(format!("exposed-{candidate_id}").into());
                screen.harness.last_action =
                    format!("Host exposed {candidate_id} on {node_id}").into();
            }
        }
        DesignPanelAction::NestedComponentPropertyUnexposeRequested {
            node_id,
            candidate_id,
            ..
        } => {
            if let Some(candidate) = node
                .component_context
                .as_mut()
                .and_then(|context| context.authoring.as_mut())
                .and_then(|authoring| {
                    authoring
                        .exposure_candidates
                        .iter_mut()
                        .find(|candidate| &candidate.candidate_id == candidate_id)
                })
            {
                candidate.exposed_property_id = None;
                screen.harness.last_action =
                    format!("Host unexposed {candidate_id} on {node_id}").into();
            }
        }
        DesignPanelAction::NestedComponentPropertyPreviewRequested {
            node_id,
            candidate_id,
            preview,
            ..
        } => {
            screen.harness.last_action = format!(
                "Host {} preview for {candidate_id} on {node_id}",
                if *preview { "started" } else { "ended" }
            )
            .into();
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

pub(crate) fn refresh_story_component_override_summary(node: &mut DesignPanelNode) {
    let overridden_property_count = node
        .component_properties
        .iter()
        .filter(|property| property.reset_state.is_overridden())
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    if let Some(context) = node.component_context.as_mut() {
        context.overrides.overridden_property_count = overridden_property_count;
        context.overrides.reset_state =
            if overridden_property_count > 0 || context.overrides.nested_override_count > 0 {
                DesignComponentResetState::Resettable
            } else {
                DesignComponentResetState::Clean
            };
    }
}
