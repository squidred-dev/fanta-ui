//! Section state and Transform-group repeat-modifier commands,
//! revalidated against the exact single-node target.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    node_index: usize,
    cx: &mut Context<Storybook>,
) -> bool {
    if story_is_section_or_transform_action(action) {
        let (current_target, can_edit) = {
            let panel = panel.read(cx);
            (
                story_design_target(panel.inspection_context()),
                panel.inspection_context().permissions().can_edit(),
            )
        };
        let accepted = apply_story_section_or_transform_action(
            &mut screen.nodes[node_index],
            current_target.as_ref(),
            can_edit,
            action,
        );
        screen.last_action = story_section_or_transform_action_status(action, accepted);
        if accepted {
            screen.apply_inspection_context(panel, cx);
        }
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn story_is_section_or_transform_action(action: &DesignPanelAction) -> bool {
    matches!(
        action,
        DesignPanelAction::SectionShareRequested { .. }
            | DesignPanelAction::SectionResolveChangedStatusRequested { .. }
            | DesignPanelAction::TransformModifierAddRequested { .. }
            | DesignPanelAction::TransformModifierRemoveRequested { .. }
            | DesignPanelAction::TransformModifierChangeRequested { .. }
            | DesignPanelAction::ApplyTransformModifiersRequested { .. }
    )
}

pub(crate) fn story_section_or_transform_action_status(
    action: &DesignPanelAction,
    accepted: bool,
) -> SharedString {
    if accepted {
        format!("Host accepted exact Section/Transform intent {action:?}").into()
    } else {
        format!(
            "Host rejected stale, permission-invalid, capability-disabled, or inapplicable Section/Transform intent {action:?}"
        )
        .into()
    }
}

pub(crate) fn story_target_is_exact_single_node(
    current_target: Option<&DesignPanelTarget>,
    node_id: &SharedString,
) -> bool {
    matches!(
        current_target,
        Some(DesignPanelTarget::Nodes { node_ids })
            if node_ids.len() == 1 && node_ids.first() == Some(node_id)
    )
}

pub(crate) fn apply_story_section_or_transform_action(
    node: &mut DesignPanelNode,
    current_target: Option<&DesignPanelTarget>,
    can_edit: bool,
    action: &DesignPanelAction,
) -> bool {
    let action_node_id = match action {
        DesignPanelAction::SectionShareRequested { node_id }
        | DesignPanelAction::SectionResolveChangedStatusRequested { node_id }
        | DesignPanelAction::TransformModifierAddRequested { node_id, .. }
        | DesignPanelAction::TransformModifierRemoveRequested { node_id, .. }
        | DesignPanelAction::TransformModifierChangeRequested { node_id, .. }
        | DesignPanelAction::ApplyTransformModifiersRequested { node_id } => node_id,
        _ => return false,
    };
    if action_node_id != &node.id
        || !story_target_is_exact_single_node(current_target, action_node_id)
    {
        return false;
    }

    match action {
        DesignPanelAction::SectionShareRequested { .. } => {
            node.supports_section(DesignPanelSection::Section)
                && node
                    .section
                    .as_ref()
                    .is_some_and(|section| section.capabilities.share)
        }
        DesignPanelAction::SectionResolveChangedStatusRequested { .. } => {
            let can_resolve = can_edit
                && node.supports_section(DesignPanelSection::Section)
                && node.section.as_ref().is_some_and(|section| {
                    section.capabilities.resolve_changed_status
                        && section
                            .dev_status
                            .as_ref()
                            .is_some_and(|status| status.changed)
                });
            if can_resolve
                && let Some(status) = node
                    .section
                    .as_mut()
                    .and_then(|section| section.dev_status.as_mut())
            {
                status.changed = false;
            }
            can_resolve
        }
        DesignPanelAction::TransformModifierAddRequested { repeat_type, .. } => {
            if !can_edit
                || node.kind != DesignPanelNodeKind::TransformGroup
                || !node.supports_section(DesignPanelSection::Transform)
            {
                return false;
            }
            let mut suffix = node.transform_modifiers.len();
            let modifier_id = loop {
                let candidate = format!("{}-repeat-host-{suffix}", node.id);
                if node
                    .transform_modifiers
                    .iter()
                    .all(|modifier| modifier.id.as_ref() != candidate.as_str())
                {
                    break candidate;
                }
                suffix += 1;
            };
            node.transform_modifiers.push(match repeat_type {
                DesignRepeatType::Linear => {
                    DesignRepeatModifier::linear(modifier_id, DesignRepeatAxis::Horizontal)
                }
                DesignRepeatType::Radial => DesignRepeatModifier::radial(modifier_id),
            });
            true
        }
        DesignPanelAction::TransformModifierRemoveRequested {
            modifier_id, index, ..
        } => {
            let can_remove = can_edit
                && node.kind == DesignPanelNodeKind::TransformGroup
                && node.supports_section(DesignPanelSection::Transform)
                && node
                    .transform_modifiers
                    .get(*index)
                    .is_some_and(|modifier| modifier.id == *modifier_id);
            if can_remove {
                node.transform_modifiers.remove(*index);
            }
            can_remove
        }
        DesignPanelAction::TransformModifierChangeRequested {
            modifier_id,
            index,
            change,
            phase,
            ..
        } => {
            let change_is_valid = match change {
                DesignTransformModifierChange::Count(count) => *count >= 1,
                DesignTransformModifierChange::Offset(offset) => offset.is_finite(),
                DesignTransformModifierChange::Mode(_) | DesignTransformModifierChange::Unit(_) => {
                    true
                }
            };
            let can_change = can_edit
                && node.kind == DesignPanelNodeKind::TransformGroup
                && node.supports_section(DesignPanelSection::Transform)
                && change_is_valid
                && node
                    .transform_modifiers
                    .get(*index)
                    .is_some_and(|modifier| modifier.id == *modifier_id);
            if !can_change {
                return false;
            }
            if *phase != DesignPanelEditPhase::Begin {
                let modifier = &mut node.transform_modifiers[*index];
                match change {
                    DesignTransformModifierChange::Mode(mode) => modifier.mode = *mode,
                    DesignTransformModifierChange::Count(count) => modifier.count = *count,
                    DesignTransformModifierChange::Unit(unit) => modifier.unit = *unit,
                    DesignTransformModifierChange::Offset(offset) => modifier.offset = *offset,
                }
            }
            true
        }
        DesignPanelAction::ApplyTransformModifiersRequested { .. } => {
            let can_apply = can_edit
                && node.kind == DesignPanelNodeKind::TransformGroup
                && node.supports_section(DesignPanelSection::Transform)
                && !node.transform_modifiers.is_empty();
            if can_apply {
                node.transform_modifiers.clear();
            }
            can_apply
        }
        _ => false,
    }
}
