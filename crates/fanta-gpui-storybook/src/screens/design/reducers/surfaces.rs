//! Surface change echoes, menu previews, viewer copies, and
//! dimension-limit previews — presentation echoes that never mutate a node.

use super::*;

pub(crate) fn echo_surface_change(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::SurfaceChangeRequested { current, requested } = action {
        let (active, can_edit) = {
            let panel = panel.read(cx);
            (
                panel.active_surface(),
                panel.inspection_context().permissions().can_edit(),
            )
        };
        let mut echoed = active;
        let mut accepted =
            apply_story_surface_change_request(&mut echoed, can_edit, *current, *requested);
        if accepted {
            accepted = panel.update(cx, |panel, cx| panel.set_active_surface(echoed, cx));
        }
        screen.last_action = if accepted {
            format!(
                "Host echoed {} → {} without changing the document",
                current.label(),
                requested.label()
            )
            .into()
        } else {
            format!(
                "Host rejected stale or permission-invalid {} → {} request",
                current.label(),
                requested.label()
            )
            .into()
        };
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn apply_menu_preview_transaction(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::MenuPreviewRequested { preview, phase } = action {
        let (current_target, can_edit) = {
            let panel = panel.read(cx);
            (
                story_design_target(panel.inspection_context()),
                panel.inspection_context().permissions().can_edit(),
            )
        };
        let current_paint_target = match preview {
            DesignMenuPreview::PaintProperty {
                node_id,
                collection,
                ..
            } => Some(screen.paint_target_for(node_id, *collection)),
            DesignMenuPreview::NodeProperty { .. } | DesignMenuPreview::EffectProperty { .. } => {
                None
            }
        };
        let accepted = apply_story_menu_preview(
            &mut screen.menu_preview,
            StoryMenuPreviewContext {
                nodes: &screen.nodes,
                bindings: &screen.property_bindings,
                current_target: current_target.as_ref(),
                current_paint_target,
                can_edit,
            },
            preview,
            *phase,
        );
        screen.last_action = if accepted {
            format!("Host accepted {phase:?} for exact menu preview {preview:?}").into()
        } else {
            format!("Host rejected stale or unbalanced menu preview {preview:?}").into()
        };
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn copy_property(
    screen: &mut DesignScreen,
    _panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::PropertyCopyRequested {
        target,
        property,
        displayed_value,
    } = action
    {
        screen.last_action = story_property_copy_status(target, *property, displayed_value);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn preview_dimension_limits(
    screen: &mut DesignScreen,
    _panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::DimensionLimitsPreviewRequested {
        node_id,
        axis,
        minimum,
        maximum,
        preview,
    } = action
    {
        let node = screen.nodes.iter().find(|node| node.id == *node_id);
        let current = node
            .and_then(|node| node.layout.as_ref())
            .map(|layout| match axis {
                fanta_gpui::prelude::DesignLayoutDimensionAxis::Width => {
                    (layout.item.min_width, layout.item.max_width)
                }
                fanta_gpui::prelude::DesignLayoutDimensionAxis::Height => {
                    (layout.item.min_height, layout.item.max_height)
                }
            });
        let accepted = if *preview {
            current == Some((*minimum, *maximum))
        } else {
            node.is_some()
        };
        screen.last_action = if accepted {
            format!(
                "Host {} {:?} limits {:?}…{:?} on {node_id}",
                if *preview { "previewed" } else { "cleared" },
                axis,
                minimum,
                maximum
            )
            .into()
        } else {
            format!("Host rejected stale {axis:?} limit preview on {node_id}").into()
        };
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn copy_viewer_section(
    screen: &mut DesignScreen,
    _panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::ViewerSectionCopyRequested {
        target,
        section_id,
        copy_value,
    } = action
    {
        let can_copy = screen.inspection_context().0.permissions().can_copy();
        let current = match target {
            DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                screen
                    .viewer_properties
                    .get(&node_ids[0])
                    .filter(|view_data| &view_data.target == target)
                    .and_then(|view_data| view_data.section(section_id.as_ref()))
                    .and_then(|section| section.copy_value.as_ref())
                    == Some(copy_value)
            }
            DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => false,
        };
        screen.last_action = if can_copy && current {
            format!("Host copied viewer section {section_id} from {target:?}: “{copy_value}”")
                .into()
        } else {
            format!("Host rejected stale or restricted viewer copy for {section_id}").into()
        };
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn change_viewer_representation(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::ViewerSectionRepresentationChangeRequested {
        target,
        section_id,
        representation,
    } = action
    {
        let can_copy = screen.inspection_context().0.permissions().can_copy();
        let node_id = match target {
            DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                Some(node_ids[0].clone())
            }
            DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => None,
        };
        let accepted = node_id.as_ref().is_some_and(|node_id| {
            can_copy
                && screen
                    .viewer_properties
                    .get(node_id)
                    .is_some_and(|view_data| {
                        &view_data.target == target
                            && view_data
                                .section(section_id.as_ref())
                                .is_some_and(|section| {
                                    section.color_representation.is_some()
                                        && section.color_representation != Some(*representation)
                                })
                    })
        });
        if accepted
            && let Some(node_id) = node_id
            && let Some(node) = screen.nodes.iter().find(|node| node.id == node_id).cloned()
        {
            screen.viewer_properties.insert(
                node_id,
                fixtures::viewer_properties_for_node(&node, *representation),
            );
            screen.apply_inspection_context(panel, cx);
            screen.last_action = format!(
                "Host represented viewer section {section_id} as {}",
                representation.label()
            )
            .into();
        } else {
            screen.last_action =
                format!("Host rejected stale viewer representation for {section_id}").into();
        }
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn apply_story_surface_change_request(
    active: &mut DesignPanelSurface,
    can_edit: bool,
    current: DesignPanelSurface,
    requested: DesignPanelSurface,
) -> bool {
    if *active != current
        || current == requested
        || !current.is_available(can_edit)
        || !requested.is_available(can_edit)
    {
        return false;
    }
    *active = requested;
    true
}

pub(crate) fn story_menu_property_value(
    node: &DesignPanelNode,
    property: DesignPanelProperty,
) -> Option<DesignPanelValue> {
    match property {
        DesignPanelProperty::BlendMode => Some(DesignPanelValue::BlendMode(node.blend_mode)),
        DesignPanelProperty::HorizontalSizing => Some(DesignPanelValue::SizingMode(
            node.layout.as_ref()?.horizontal_sizing,
        )),
        DesignPanelProperty::VerticalSizing => Some(DesignPanelValue::SizingMode(
            node.layout.as_ref()?.vertical_sizing,
        )),
        DesignPanelProperty::StrokeAlign => {
            Some(DesignPanelValue::StrokeAlign(node.stroke.as_ref()?.align))
        }
        DesignPanelProperty::EffectKind(index) => Some(DesignPanelValue::EffectKind(
            node.effects.get(index)?.settings.kind(),
        )),
        DesignPanelProperty::EffectShadowBlendMode(index) => {
            let blend_mode = match &node.effects.get(index)?.settings {
                DesignEffectSettings::DropShadow(settings) => settings.blend_mode,
                DesignEffectSettings::InnerShadow(settings) => settings.blend_mode,
                _ => return None,
            };
            Some(DesignPanelValue::BlendMode(blend_mode))
        }
        DesignPanelProperty::EffectNoiseBlendMode(index) => {
            let DesignEffectSettings::Noise(settings) = &node.effects.get(index)?.settings else {
                return None;
            };
            Some(DesignPanelValue::BlendMode(settings.blend_mode))
        }
        DesignPanelProperty::EffectBlurType(index) => {
            let settings = match &node.effects.get(index)?.settings {
                DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) => settings,
                _ => return None,
            };
            Some(DesignPanelValue::EffectBlurType(settings.blur_type()))
        }
        DesignPanelProperty::EffectNoiseType(index) => {
            let DesignEffectSettings::Noise(settings) = &node.effects.get(index)?.settings else {
                return None;
            };
            Some(DesignPanelValue::EffectNoiseType(
                settings.colors.noise_type(),
            ))
        }
        _ => None,
    }
}

pub(crate) fn story_menu_paint_value(
    node: &DesignPanelNode,
    collection: DesignPanelCollection,
    paint_id: &SharedString,
    index: usize,
    property: DesignPaintProperty,
) -> Option<DesignPaintValue> {
    let paints = match collection {
        DesignPanelCollection::Fill => node.fills.as_slice(),
        DesignPanelCollection::Stroke => node.stroke.as_ref()?.paints.as_slice(),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => return None,
    };
    let paint = paints.get(index)?;
    if paint.read_only || (!paint_id.is_empty() && paint.id != *paint_id) {
        return None;
    }
    match property {
        DesignPaintProperty::BlendMode => Some(DesignPaintValue::BlendMode(paint.blend_mode)),
        _ => None,
    }
}

pub(crate) struct StoryMenuPreviewContext<'a> {
    pub(crate) nodes: &'a [DesignPanelNode],
    pub(crate) bindings: &'a HashMap<
        (SharedString, DesignPanelProperty),
        DesignPanelPropertyBinding<DesignPanelValue>,
    >,
    pub(crate) current_target: Option<&'a DesignPanelTarget>,
    pub(crate) current_paint_target: Option<DesignPaintTarget>,
    pub(crate) can_edit: bool,
}

pub(crate) fn apply_story_menu_preview(
    active: &mut Option<DesignMenuPreview>,
    context: StoryMenuPreviewContext<'_>,
    preview: &DesignMenuPreview,
    phase: DesignMenuPreviewPhase,
) -> bool {
    let StoryMenuPreviewContext {
        nodes,
        bindings,
        current_target,
        current_paint_target,
        can_edit,
    } = context;
    if phase == DesignMenuPreviewPhase::End {
        if active.as_ref() != Some(preview) {
            return false;
        }
        *active = None;
        return true;
    }
    if active.is_some() || !can_edit {
        return false;
    }
    let valid = match preview {
        DesignMenuPreview::NodeProperty {
            target,
            property,
            original,
            candidate,
        } => {
            let DesignPanelTarget::Nodes { node_ids } = target else {
                return false;
            };
            current_target == Some(target)
                && original != candidate
                && !node_ids.is_empty()
                && node_ids.iter().all(|node_id| {
                    !bindings.contains_key(&(node_id.clone(), *property))
                        && nodes
                            .iter()
                            .find(|node| node.id == *node_id)
                            .and_then(|node| story_menu_property_value(node, *property))
                            .as_ref()
                            == Some(original)
                })
        }
        DesignMenuPreview::EffectProperty {
            node_id,
            effect_id,
            index,
            property,
            original,
            candidate,
        } => {
            let exact_target = DesignPanelTarget::Nodes {
                node_ids: vec![node_id.clone()],
            };
            let node = nodes.iter().find(|node| node.id == *node_id);
            let effect = node.and_then(|node| node.effects.get(*index));
            current_target == Some(&exact_target)
                && original != candidate
                && !bindings.contains_key(&(node_id.clone(), *property))
                && effect.is_some_and(|effect| effect_id.is_empty() || effect.id == *effect_id)
                && node
                    .and_then(|node| story_menu_property_value(node, *property))
                    .as_ref()
                    == Some(original)
        }
        DesignMenuPreview::PaintProperty {
            node_id,
            collection,
            target,
            paint_id,
            index,
            property,
            original,
            candidate,
        } => {
            let exact_target = DesignPanelTarget::Nodes {
                node_ids: vec![node_id.clone()],
            };
            let node = nodes.iter().find(|node| node.id == *node_id);
            current_target == Some(&exact_target)
                && current_paint_target == Some(*target)
                && original != candidate
                && node.is_some_and(|node| {
                    let style_is_bound = match collection {
                        DesignPanelCollection::Fill => node.fill_style_binding.is_some(),
                        DesignPanelCollection::Stroke => node.stroke_style_binding.is_some(),
                        DesignPanelCollection::Effect
                        | DesignPanelCollection::LayoutGrid
                        | DesignPanelCollection::Export => true,
                    };
                    !style_is_bound
                        && story_menu_paint_value(
                            node,
                            *collection,
                            paint_id,
                            *index,
                            property.clone(),
                        )
                        .as_ref()
                            == Some(original)
                })
        }
    };
    if valid {
        *active = Some(preview.clone());
    }
    valid
}
