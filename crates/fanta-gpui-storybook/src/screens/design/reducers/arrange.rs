//! General arrange, distribute, transform, and resize-to-fit intents.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    match action {
        DesignPanelAction::ArrangeRequested { target, operation } => {
            screen.last_action = format!("Host applied {operation:?} to {target:?}").into();
            cx.notify();
            return true;
        }
        DesignPanelAction::TransformRequested { target, operation } => {
            let (current_target, can_edit) = {
                let panel = panel.read(cx);
                (
                    story_design_target(panel.inspection_context()),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            let accepted = apply_story_transform_request(
                &mut screen.nodes,
                current_target.as_ref(),
                target,
                can_edit,
                *operation,
            );
            screen.last_action = if accepted {
                match operation {
                        fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90 => {
                            format!("Host atomically applied {operation:?} to {target:?}").into()
                        }
                        fanta_gpui::prelude::DesignTransformOperation::FlipHorizontal
                        | fanta_gpui::prelude::DesignTransformOperation::FlipVertical => format!(
                            "Host accepted exact {operation:?} for {target:?}; the Storybook node model does not reflect flip state"
                        )
                        .into(),
                    }
            } else {
                format!(
                        "Host rejected stale, permission-invalid, or inapplicable {operation:?} for {target:?}"
                    )
                    .into()
            };
            if accepted {
                screen.apply_inspection_context(panel, cx);
            }
            cx.notify();
            return true;
        }
        DesignPanelAction::ResizeToFitRequested { target } => {
            screen.last_action = format!("Host resized {target:?} to fit").into();
            cx.notify();
            return true;
        }
        _ => {}
    }
    false
}

pub(crate) fn apply_story_transform_request(
    nodes: &mut [DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
    operation: fanta_gpui::prelude::DesignTransformOperation,
) -> bool {
    let Some(node_indices) =
        story_exact_editable_node_indices(nodes, current_target, requested_target, can_edit)
    else {
        return false;
    };
    if !node_indices
        .iter()
        .all(|index| nodes[*index].supports_section(DesignPanelSection::Position))
    {
        return false;
    }
    if operation == fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90 {
        for index in node_indices {
            let node = &mut nodes[index];
            node.rotation = (node.rotation + 90.).rem_euclid(360.);
        }
    }
    true
}
