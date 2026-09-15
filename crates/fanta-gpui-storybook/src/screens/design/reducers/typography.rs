//! Typography properties, styles, fonts, variable axes, and
//! OpenType features on the selected text node.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    node_index: usize,
    inside_auto_layout: bool,
    cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.host.nodes[node_index];
    match action {
        DesignPanelAction::TypographyPropertyChangeRequested {
            node_id,
            target,
            property,
            value,
        } => {
            apply_design_property_with_parent(node, *property, value, inside_auto_layout);
            screen.harness.last_action =
                format!("Host applied {property:?} = {value:?} to {target:?} on {node_id}").into();
        }
        DesignPanelAction::TypographyPropertyEditRequested {
            node_id,
            target,
            property,
            value,
            phase,
        } => {
            if *phase != DesignPanelEditPhase::Begin {
                apply_design_property_with_parent(node, *property, value, inside_auto_layout);
            }
            screen.harness.last_action = format!(
                "Host observed {phase:?} for {property:?} = {value:?} on {target:?} in {node_id}"
            )
            .into();
        }
        DesignPanelAction::TypographyVariableAxisEditRequested {
            node_id,
            target,
            tag,
            value,
            phase,
        } => {
            let changed = if *phase == DesignPanelEditPhase::Begin {
                true
            } else {
                node.typography.as_mut().is_some_and(|typography| {
                    typography
                        .variable_axes
                        .iter_mut()
                        .find(|axis| axis.tag == *tag)
                        .is_some_and(|axis| {
                            if axis.is_editable()
                                && value.is_finite()
                                && (axis.min..=axis.max).contains(value)
                            {
                                axis.value = *value;
                                true
                            } else {
                                false
                            }
                        })
                })
            };
            screen.harness.last_action = if changed {
                format!(
                        "Host observed {phase:?} for variable axis {tag} = {value} on {target:?} in {node_id}"
                    )
                    .into()
            } else {
                format!("Host rejected stale variable axis {tag} on {node_id}").into()
            };
        }
        DesignPanelAction::TypographyStyleApplyRequested {
            node_id,
            target,
            style,
        } => {
            let resolved_style = screen.host.typography_styles.style(style).cloned();
            let applied = node
                .typography
                .as_mut()
                .zip(resolved_style.as_ref())
                .is_some_and(|(typography, resolved)| {
                    typography.family = resolved.family.clone();
                    typography.style = resolved.font_style.clone();
                    typography.size = resolved.size;
                    typography.style_binding = Some(DesignTypographyStyleBinding::new(
                        style.clone(),
                        resolved.name.clone(),
                    ));
                    true
                });
            screen.harness.last_action = if applied {
                format!(
                    "Host applied text style {} to {target:?} on {node_id}",
                    style.style_id
                )
                .into()
            } else {
                format!("Host rejected unavailable text style on {node_id}").into()
            };
        }
        DesignPanelAction::TypographyStyleDetachRequested {
            node_id,
            target,
            style,
        } => {
            let detached = node.typography.as_mut().is_some_and(|typography| {
                let can_detach = typography
                    .style_binding
                    .as_ref()
                    .is_some_and(|binding| binding.can_detach && binding.selection == *style);
                if can_detach {
                    typography.style_binding = None;
                }
                can_detach
            });
            screen.harness.last_action = if detached {
                format!(
                    "Host detached text style {} from {target:?} on {node_id}",
                    style.style_id
                )
                .into()
            } else {
                format!("Host rejected stale or locked text-style detach on {node_id}").into()
            };
        }
        DesignPanelAction::TypographyFontApplyRequested {
            node_id,
            target,
            font,
        } => {
            let resolved = screen
                .host
                .fonts
                .font(font)
                .filter(|(_, style)| style.availability.can_apply())
                .map(|(family, style)| {
                    (
                        family.name.clone(),
                        style.name.clone(),
                        style.weight.map(f32::from),
                    )
                });
            let applied = node.typography.as_mut().zip(resolved).is_some_and(
                |(typography, (family, style, weight))| {
                    typography.family = family;
                    typography.style = style;
                    if let Some(weight) = weight {
                        typography.weight = weight;
                    }
                    typography.style_binding = None;
                    true
                },
            );
            screen.harness.last_action = if applied {
                format!("Host applied font {font:?} to {target:?} on {node_id}").into()
            } else {
                format!("Host rejected unavailable font on {node_id}").into()
            };
        }
        DesignPanelAction::TypographyFontImportRequested {
            node_id,
            target,
            font,
        } => {
            let imported = screen
                .host
                .fonts
                .families
                .iter_mut()
                .find(|family| family.id == font.family_id && family.source == font.source)
                .and_then(|family| {
                    family
                        .styles
                        .iter_mut()
                        .find(|style| style.id == font.style_id)
                })
                .is_some_and(|style| {
                    if style.availability.can_import() {
                        style.availability = DesignFontAvailability::Imported;
                        true
                    } else {
                        false
                    }
                });
            if imported {
                screen.apply_inspection_context(panel, cx);
            }
            screen.harness.last_action = if imported {
                format!(
                    "Host imported font {} for {target:?} on {node_id}; choose it again to apply",
                    font.style_id
                )
                .into()
            } else {
                format!("Host rejected stale font import on {node_id}").into()
            };
        }
        DesignPanelAction::TypographyOpenTypeFeatureChangeRequested {
            node_id,
            target,
            tag,
            enabled,
        } => {
            let changed = node.typography.as_mut().is_some_and(|typography| {
                typography
                    .open_type_features
                    .iter_mut()
                    .find(|feature| feature.tag == *tag)
                    .is_some_and(|feature| {
                        if feature.availability.is_available() && feature.enabled != *enabled {
                            feature.enabled = *enabled;
                            true
                        } else {
                            false
                        }
                    })
            });
            screen.harness.last_action = if changed {
                format!(
                    "Host set OpenType {} to {enabled} for {target:?} on {node_id}",
                    tag.api_name()
                )
                .into()
            } else {
                format!("Host rejected stale OpenType feature on {node_id}").into()
            };
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}
