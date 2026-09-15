//! Ordinary node property edits, the single-node mock property
//! reducer, and property variable bindings.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    inside_auto_layout: bool,
    _cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.host.nodes[node_index];
    match action {
        DesignPanelAction::PropertyChangeRequested {
            node_id,
            property,
            value,
        } => {
            apply_design_property_with_parent(node, *property, value, inside_auto_layout);
            screen.harness.last_action =
                format!("Host applied {property:?} = {value:?} to {node_id}").into();
        }
        DesignPanelAction::PropertyEditRequested {
            node_id,
            property,
            value,
            phase,
        } => {
            if *phase != DesignPanelEditPhase::Begin {
                apply_design_property_with_parent(node, *property, value, inside_auto_layout);
            }
            screen.harness.last_action =
                format!("Host observed {phase:?} for {property:?} = {value:?} on {node_id}").into();
        }
        DesignPanelAction::PropertyVariableApplyRequested {
            node_id,
            target,
            variable_id,
        } => {
            let variable = screen
                .host
                .property_variables
                .variable(variable_id.as_ref())
                .filter(|variable| {
                    variable.is_compatible_with(target)
                        && variable.disabled_reason.is_none()
                        && variable.import_state != DesignVariableImportState::Available
                })
                .cloned();
            if let Some(variable) = variable {
                let resolved = variable.resolved_value.as_ref().map_or_else(
                    || DesignPanelValue::Text("Resolved by host".into()),
                    DesignVariableResolvedValue::panel_value,
                );
                if matches!(
                    target.property,
                    DesignPanelProperty::FontFamily
                        | DesignPanelProperty::FontStyle
                        | DesignPanelProperty::FontWeight
                ) {
                    apply_design_property(node, target.property, &resolved);
                }
                let name = format!("{} / {}", variable.collection_name, variable.name);
                screen.host.property_bindings.insert(
                    (node_id.clone(), target.property),
                    DesignPanelPropertyBinding::new(
                        variable.id.clone(),
                        name,
                        DesignPanelBindingKind::Variable,
                        resolved,
                    ),
                );
                screen.harness.last_action = format!(
                    "Host bound {variable_id} to {:?} on {node_id}",
                    target.fields
                )
                .into();
            } else {
                screen.harness.last_action =
                    format!("Host rejected unavailable variable {variable_id}").into();
            }
        }
        DesignPanelAction::PropertyVariableImportRequested {
            node_id,
            target,
            variable_id,
        } => {
            let imported = screen
                .host
                .property_variables
                .variables
                .iter_mut()
                .find(|variable| {
                    variable.id == *variable_id
                        && variable.is_compatible_with(target)
                        && variable.disabled_reason.is_none()
                        && variable.import_state == DesignVariableImportState::Available
                })
                .is_some_and(|variable| {
                    variable.import_state = DesignVariableImportState::Imported;
                    true
                });
            screen.harness.last_action = if imported {
                format!(
                    "Host imported {variable_id} for {:?} on {node_id}; choose it again to apply",
                    target.property
                )
                .into()
            } else {
                format!("Host rejected stale variable import {variable_id}").into()
            };
        }
        DesignPanelAction::PropertyVariableDetachRequested {
            node_id,
            target,
            variable_id,
        } => {
            let key = (node_id.clone(), target.property);
            let can_detach = screen
                .host
                .property_bindings
                .get(&key)
                .is_some_and(|binding| {
                    binding.kind() == DesignPanelBindingKind::Variable
                        && binding.id() == variable_id
                });
            if can_detach {
                screen.host.property_bindings.remove(&key);
            }
            screen.harness.last_action = if can_detach {
                format!(
                    "Host detached {variable_id} from {:?} on {node_id}",
                    target.fields
                )
                .into()
            } else {
                format!("Host rejected stale variable detach {variable_id}").into()
            };
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

pub(crate) fn apply_design_property(
    node: &mut DesignPanelNode,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
) {
    apply_design_property_with_parent(node, property, value, false);
}

pub(crate) fn apply_design_property_with_parent(
    node: &mut DesignPanelNode,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
    inside_auto_layout: bool,
) {
    match (property, value) {
        (DesignPanelProperty::X, DesignPanelValue::Number(value)) => node.x = *value,
        (DesignPanelProperty::Y, DesignPanelValue::Number(value)) => node.y = *value,
        (DesignPanelProperty::Width, DesignPanelValue::Number(value)) => {
            if node.lock_aspect_ratio && node.width.abs() > f32::EPSILON {
                node.height *= *value / node.width;
            }
            node.width = *value;
        }
        (DesignPanelProperty::Height, DesignPanelValue::Number(value)) => {
            if node.lock_aspect_ratio && node.height.abs() > f32::EPSILON {
                node.width *= *value / node.height;
            }
            node.height = *value;
        }
        (DesignPanelProperty::Rotation, DesignPanelValue::Number(value)) => node.rotation = *value,
        (DesignPanelProperty::LockAspectRatio, DesignPanelValue::Bool(value)) => {
            if !node.is_component_instance_child {
                node.lock_aspect_ratio = *value;
            }
        }
        (DesignPanelProperty::HorizontalConstraint, DesignPanelValue::Constraint(constraint)) => {
            node.horizontal_constraint = *constraint
        }
        (DesignPanelProperty::VerticalConstraint, DesignPanelValue::Constraint(constraint)) => {
            node.vertical_constraint = *constraint
        }
        (DesignPanelProperty::LayoutMode, DesignPanelValue::LayoutMode(mode)) => {
            let supports_auto_layout = node.supports_auto_layout_container();
            let supports_grid = node.supports_grid_auto_layout();
            if let Some(layout) = node.layout.as_mut() {
                layout.set_mode_for_capabilities(supports_auto_layout, supports_grid, *mode);
            }
        }
        (DesignPanelProperty::HorizontalSizing, DesignPanelValue::SizingMode(mode)) => {
            if let Some(layout) = node.layout.as_mut()
                && !(layout.mode == DesignLayoutMode::Grid
                    && *mode == DesignSizingMode::Hug
                    && layout.grid_columns.iter().any(|track| {
                        track.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction
                    }))
            {
                layout.horizontal_sizing = *mode;
            }
        }
        (DesignPanelProperty::VerticalSizing, DesignPanelValue::SizingMode(mode)) => {
            if let Some(layout) = node.layout.as_mut()
                && !(layout.mode == DesignLayoutMode::Grid
                    && *mode == DesignSizingMode::Hug
                    && layout.grid_rows.iter().any(|track| {
                        track.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction
                    }))
            {
                layout.vertical_sizing = *mode;
            }
            node.normalize_text_max_lines(inside_auto_layout);
        }
        (
            DesignPanelProperty::AutoLayoutAlignment,
            DesignPanelValue::AutoLayoutAlignment(alignment),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.alignment_x = alignment.x;
                layout.alignment_y = alignment.y;
            }
        }
        (DesignPanelProperty::Wrap, DesignPanelValue::Bool(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.wrap = *value && layout.mode == DesignLayoutMode::Horizontal;
            }
        }
        (DesignPanelProperty::Gap, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.gap = *value;
            }
        }
        (DesignPanelProperty::ItemSpacingMode, DesignPanelValue::ItemSpacingMode(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item_spacing_mode = *value;
            }
        }
        (
            DesignPanelProperty::CounterAxisAlignContent,
            DesignPanelValue::CounterAxisAlignContent(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                let _ = layout.set_counter_axis_align_content(*value);
            }
        }
        (DesignPanelProperty::CounterAxisGap, DesignPanelValue::OptionalNumber(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                if layout.mode == DesignLayoutMode::Grid {
                    if value.is_some_and(|gap| gap.is_finite() && gap >= 0.) {
                        layout.counter_axis_gap = *value;
                    }
                } else {
                    let _ = layout.set_counter_axis_gap(*value);
                }
            }
        }
        (DesignPanelProperty::PaddingVertical, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value >= 0.
            {
                layout.padding[0] = *value;
                layout.padding[2] = *value;
            }
        }
        (DesignPanelProperty::PaddingHorizontal, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value >= 0.
            {
                layout.padding[1] = *value;
                layout.padding[3] = *value;
            }
        }
        (DesignPanelProperty::PaddingShorthand, DesignPanelValue::NumberList(values))
            if values.iter().all(|value| value.is_finite() && *value >= 0.) =>
        {
            let expanded = match values.as_slice() {
                [all] => Some([*all; 4]),
                [vertical, horizontal] => Some([*vertical, *horizontal, *vertical, *horizontal]),
                [top, horizontal, bottom] => Some([*top, *horizontal, *bottom, *horizontal]),
                [top, right, bottom, left] => Some([*top, *right, *bottom, *left]),
                _ => None,
            };
            if let (Some(layout), Some(expanded)) = (node.layout.as_mut(), expanded) {
                layout.padding = expanded;
            }
        }
        (DesignPanelProperty::PaddingTop, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[0] = *value;
            }
        }
        (DesignPanelProperty::PaddingRight, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[1] = *value;
            }
        }
        (DesignPanelProperty::PaddingBottom, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[2] = *value;
            }
        }
        (DesignPanelProperty::PaddingLeft, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[3] = *value;
            }
        }
        (DesignPanelProperty::ClipContent, DesignPanelValue::Bool(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.clip_content = *value;
            }
        }
        (DesignPanelProperty::IncludeStrokes, DesignPanelValue::Bool(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.include_strokes = *value;
            }
        }
        (DesignPanelProperty::StackingOrder, DesignPanelValue::StackingOrder(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.stacking_order = *value;
            }
        }
        (DesignPanelProperty::BaselineAlignment, DesignPanelValue::BaselineAlignment(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.baseline_alignment = *value;
            }
        }
        (DesignPanelProperty::MinWidth, DesignPanelValue::OptionalNumber(value)) => {
            let linked_minimum = (node.lock_aspect_ratio && node.width.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.height / node.width));
            if let Some(layout) = node.layout.as_mut() {
                layout.item.min_width = *value;
                if let Some(linked_minimum) = linked_minimum {
                    layout.item.min_height = linked_minimum;
                }
            }
        }
        (DesignPanelProperty::MaxWidth, DesignPanelValue::OptionalNumber(value)) => {
            let linked_maximum = (node.lock_aspect_ratio && node.width.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.height / node.width));
            if let Some(layout) = node.layout.as_mut() {
                layout.item.max_width = *value;
            }
            if let Some(linked_maximum) = linked_maximum {
                let _ = node.set_layout_max_height(linked_maximum);
            }
        }
        (DesignPanelProperty::MinHeight, DesignPanelValue::OptionalNumber(value)) => {
            let linked_minimum = (node.lock_aspect_ratio && node.height.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.width / node.height));
            if let Some(layout) = node.layout.as_mut() {
                layout.item.min_height = *value;
                if let Some(linked_minimum) = linked_minimum {
                    layout.item.min_width = linked_minimum;
                }
            }
        }
        (DesignPanelProperty::MaxHeight, DesignPanelValue::OptionalNumber(value)) => {
            let linked_maximum = (node.lock_aspect_ratio && node.height.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.width / node.height));
            let _ = node.set_layout_max_height(*value);
            if let Some(linked_maximum) = linked_maximum
                && let Some(layout) = node.layout.as_mut()
            {
                layout.item.max_width = linked_maximum;
            }
        }
        (DesignPanelProperty::LayoutPositioning, DesignPanelValue::LayoutPositioning(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.positioning = *value;
            }
        }
        (DesignPanelProperty::LayoutAlignSelf, DesignPanelValue::LayoutAlignSelf(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.align_self = *value;
            }
        }
        (DesignPanelProperty::LayoutGrow, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.layout_grow = *value;
            }
        }
        (DesignPanelProperty::GridAutoTracks, DesignPanelValue::GridAutoTracks(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && layout.mode == DesignLayoutMode::Grid
            {
                layout.grid_auto_tracks = *value;
            }
        }
        (
            DesignPanelProperty::GridItemsPositioning,
            DesignPanelValue::GridItemsPositioning(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.grid_items_positioning = *value;
            }
        }
        (DesignPanelProperty::GridColumnCount, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && layout.mode == DesignLayoutMode::Grid
                && let Ok(count) = usize::try_from(*value)
                && count >= 1
            {
                layout.grid_columns.resize(count, DesignGridTrack::hug());
            }
        }
        (DesignPanelProperty::GridRowCount, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && layout.mode == DesignLayoutMode::Grid
                && layout.grid_auto_tracks == DesignGridAutoTracks::None
                && let Ok(count) = usize::try_from(*value)
                && count >= 1
            {
                layout.grid_rows.resize(count, DesignGridTrack::hug());
            }
        }
        (DesignPanelProperty::GridColumnTrack(index), DesignPanelValue::GridTrack(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_valid()
                && !(layout.horizontal_sizing == DesignSizingMode::Hug
                    && value.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction)
                && let Some(track) = layout.grid_columns.get_mut(index)
            {
                *track = *value;
            }
        }
        (DesignPanelProperty::GridRowTrack(index), DesignPanelValue::GridTrack(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_valid()
                && !(layout.vertical_sizing == DesignSizingMode::Hug
                    && value.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction)
                && let Some(track) = layout.grid_rows.get_mut(index)
            {
                *track = *value;
            }
        }
        (DesignPanelProperty::GridColumnTrackValue(index), DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value > 0.
                && let Some(track) = layout.grid_columns.get_mut(index)
                && track.sizing != fanta_gpui::prelude::DesignGridTrackSizing::Hug
            {
                track.value = *value;
            }
        }
        (DesignPanelProperty::GridRowTrackValue(index), DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value > 0.
                && let Some(track) = layout.grid_rows.get_mut(index)
                && track.sizing != fanta_gpui::prelude::DesignGridTrackSizing::Hug
            {
                track.value = *value;
            }
        }
        (DesignPanelProperty::GridRowIndex, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_row_index =
                    usize::try_from(value.saturating_sub(1).max(0)).unwrap_or(usize::MAX);
            }
        }
        (DesignPanelProperty::GridColumnIndex, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_column_index =
                    usize::try_from(value.saturating_sub(1).max(0)).unwrap_or(usize::MAX);
            }
        }
        (DesignPanelProperty::GridRowSpan, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_row_span = (*value).clamp(1, i64::from(u16::MAX)) as u16;
            }
        }
        (DesignPanelProperty::GridColumnSpan, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_column_span = (*value).clamp(1, i64::from(u16::MAX)) as u16;
            }
        }
        (
            DesignPanelProperty::GridHorizontalAlignment,
            DesignPanelValue::GridItemAlignment(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_horizontal_alignment = *value;
            }
        }
        (
            DesignPanelProperty::GridVerticalAlignment,
            DesignPanelValue::GridItemAlignment(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_vertical_alignment = *value;
            }
        }
        (DesignPanelProperty::Visible, DesignPanelValue::Bool(value)) => node.visible = *value,
        (DesignPanelProperty::FillShowsInExports, DesignPanelValue::Bool(value)) => {
            if let Some(show_in_exports) = node.fill_shows_in_exports.as_mut() {
                *show_in_exports = *value;
            }
        }
        (DesignPanelProperty::Opacity, DesignPanelValue::Number(value)) => node.opacity = *value,
        (DesignPanelProperty::BlendMode, DesignPanelValue::BlendMode(mode)) => {
            node.blend_mode = *mode;
        }
        (DesignPanelProperty::CornerRadius, DesignPanelValue::Number(value)) => {
            node.corner_radii = [*value; 4];
        }
        (DesignPanelProperty::IndependentCorners, DesignPanelValue::Bool(value)) => {
            node.independent_corners = *value;
        }
        (DesignPanelProperty::CornerRadiusTopLeft, DesignPanelValue::Number(value)) => {
            node.corner_radii[0] = *value
        }
        (DesignPanelProperty::CornerRadiusTopRight, DesignPanelValue::Number(value)) => {
            node.corner_radii[1] = *value
        }
        (DesignPanelProperty::CornerRadiusBottomRight, DesignPanelValue::Number(value)) => {
            node.corner_radii[2] = *value
        }
        (DesignPanelProperty::CornerRadiusBottomLeft, DesignPanelValue::Number(value)) => {
            node.corner_radii[3] = *value
        }
        (DesignPanelProperty::CornerSmoothing, DesignPanelValue::Ratio(value)) => {
            node.corner_smoothing = value.clamp(0., 1.);
        }
        (DesignPanelProperty::ComponentProperty(index), DesignPanelValue::Text(value)) => {
            if let Some(property) = node.component_properties.get_mut(index) {
                property.value = value.clone();
            }
        }
        (DesignPanelProperty::FontFamily, DesignPanelValue::Text(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.family = value.clone();
            }
        }
        (DesignPanelProperty::FontStyle, DesignPanelValue::Text(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.style = value.clone();
            }
        }
        (DesignPanelProperty::FontWeight, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut()
                && value.is_finite()
            {
                typography.weight = value.clamp(1., 1000.);
            }
        }
        (DesignPanelProperty::FontSize, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.size = value.max(1.);
            }
        }
        (DesignPanelProperty::LineHeight, DesignPanelValue::LineHeight(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.line_height = *value;
            }
        }
        (DesignPanelProperty::LetterSpacing, DesignPanelValue::LetterSpacing(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.letter_spacing = *value;
            }
        }
        (DesignPanelProperty::TextLeadingTrim, DesignPanelValue::TextLeadingTrim(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.leading_trim = *value;
            }
        }
        (DesignPanelProperty::ParagraphSpacing, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.paragraph_spacing = *value;
            }
        }
        (DesignPanelProperty::ParagraphIndent, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut()
                && typography.horizontal_alignment
                    == fanta_gpui::prelude::DesignTextHorizontalAlignment::Left
            {
                typography.paragraph_indent = *value;
            }
        }
        (DesignPanelProperty::ListSpacing, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut()
                && typography.list != DesignTextList::None
                && value.is_finite()
                && *value >= 0.
            {
                typography.list_spacing = *value;
            }
        }
        (DesignPanelProperty::TextHangingPunctuation, DesignPanelValue::Bool(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.hanging_punctuation = *value;
            }
        }
        (DesignPanelProperty::TextHangingLists, DesignPanelValue::Bool(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.hanging_lists = *value;
            }
        }
        (
            DesignPanelProperty::HorizontalTextAlignment,
            DesignPanelValue::TextHorizontalAlignment(value),
        ) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.horizontal_alignment = *value;
            }
        }
        (
            DesignPanelProperty::VerticalTextAlignment,
            DesignPanelValue::TextVerticalAlignment(value),
        ) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.vertical_alignment = *value;
            }
        }
        (DesignPanelProperty::TextResize, DesignPanelValue::TextResize(value)) => {
            let _ = node.set_text_resize(*value);
            node.normalize_text_max_lines(inside_auto_layout);
        }
        (DesignPanelProperty::TextTruncate, DesignPanelValue::Bool(value)) => {
            let _ = node.set_text_truncation(*value);
            node.normalize_text_max_lines(inside_auto_layout);
        }
        (DesignPanelProperty::TextMaxLines, DesignPanelValue::OptionalNumber(value)) => {
            let max_lines = value
                .and_then(|value| (value.is_finite() && value >= 1.).then(|| value.round() as u32));
            if value.is_none() || max_lines.is_some() {
                let _ = node.set_text_max_lines(max_lines, inside_auto_layout);
            }
        }
        (DesignPanelProperty::TextDecoration, DesignPanelValue::TextDecoration(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.decoration = *value;
                if *value == DesignTextDecoration::None {
                    typography.decoration_details = None;
                } else if typography.decoration_details.is_none() {
                    typography.decoration_details = Some(DesignTextDecorationDetails::default());
                }
            }
        }
        (
            DesignPanelProperty::TextDecorationStyle,
            DesignPanelValue::TextDecorationStyle(value),
        ) => {
            if let Some(details) = node
                .typography
                .as_mut()
                .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.style = *value;
            }
        }
        (
            DesignPanelProperty::TextDecorationOffset,
            DesignPanelValue::TextDecorationMetric(value),
        ) => {
            if value.is_finite()
                && let Some(details) = node
                    .typography
                    .as_mut()
                    .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.offset = *value;
            }
        }
        (
            DesignPanelProperty::TextDecorationThickness,
            DesignPanelValue::TextDecorationMetric(value),
        ) => {
            if value.is_non_negative()
                && let Some(details) = node
                    .typography
                    .as_mut()
                    .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.thickness = *value;
            }
        }
        (
            DesignPanelProperty::TextDecorationColor,
            DesignPanelValue::TextDecorationColor(value),
        ) => {
            if let Some(details) = node
                .typography
                .as_mut()
                .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.color = *value;
            }
        }
        (DesignPanelProperty::TextDecorationSkipInk, DesignPanelValue::Bool(value)) => {
            if let Some(details) = node
                .typography
                .as_mut()
                .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.skip_ink = *value;
            }
        }
        (DesignPanelProperty::TextCase, DesignPanelValue::TextCase(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.case = *value;
            }
        }
        (DesignPanelProperty::TextList, DesignPanelValue::TextList(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.list = *value;
            }
        }
        (DesignPanelProperty::PolygonCount, DesignPanelValue::Integer(value)) => {
            if let DesignShapeGeometry::Polygon(geometry) = &mut node.shape_geometry {
                geometry.point_count = (*value).clamp(3, 60) as u16;
            }
        }
        (DesignPanelProperty::StarPointCount, DesignPanelValue::Integer(value)) => {
            if let DesignShapeGeometry::Star(geometry) = &mut node.shape_geometry {
                geometry.point_count = (*value).clamp(3, 60) as u16;
            }
        }
        (DesignPanelProperty::StarInnerRadius, DesignPanelValue::Ratio(value)) => {
            if let DesignShapeGeometry::Star(geometry) = &mut node.shape_geometry {
                geometry.inner_radius = value.clamp(0., 1.);
            }
        }
        (DesignPanelProperty::ArcStartingAngle, DesignPanelValue::AngleRadians(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                let sweep = arc.sweep_angle();
                arc.starting_angle = *value;
                arc.ending_angle = *value + sweep;
            }
        }
        (DesignPanelProperty::ArcSweep, DesignPanelValue::AngleRadians(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                arc.ending_angle = arc.starting_angle + *value;
            }
        }
        (DesignPanelProperty::ArcEndingAngle, DesignPanelValue::AngleRadians(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                arc.ending_angle = *value;
            }
        }
        (DesignPanelProperty::ArcInnerRadius, DesignPanelValue::Ratio(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                arc.inner_radius = value.clamp(0., 1.);
            }
        }
        (DesignPanelProperty::BooleanOperation, DesignPanelValue::BooleanOperation(operation)) => {
            node.shape_geometry = DesignShapeGeometry::Boolean(*operation);
        }
        (DesignPanelProperty::IsMask, DesignPanelValue::Bool(value)) => {
            node.is_mask = *value;
            node.mask_type = value.then(|| node.mask_mode.label().into());
        }
        (DesignPanelProperty::MaskType, DesignPanelValue::MaskType(value)) => {
            node.is_mask = true;
            node.mask_mode = *value;
            node.mask_type = Some(value.label().into());
        }
        (DesignPanelProperty::SectionContentsHidden, DesignPanelValue::Bool(value)) => {
            if let Some(section) = node.section.as_mut() {
                section.contents_hidden = *value;
            }
        }
        (DesignPanelProperty::SectionDevStatus, DesignPanelValue::SectionDevStatus(value)) => {
            if let Some(section) = node.section.as_mut() {
                section.dev_status = value.map(DesignSectionDevStatus::new);
            }
        }
        (
            DesignPanelProperty::TransformRepeatType(index),
            DesignPanelValue::RepeatType(repeat_type),
        ) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.mode = match repeat_type {
                    DesignRepeatType::Linear => DesignRepeatMode::Linear(
                        modifier.mode.axis().unwrap_or(DesignRepeatAxis::Horizontal),
                    ),
                    DesignRepeatType::Radial => DesignRepeatMode::Radial,
                };
            }
        }
        (DesignPanelProperty::TransformRepeatAxis(index), DesignPanelValue::RepeatAxis(axis)) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.mode = DesignRepeatMode::Linear(*axis);
            }
        }
        (DesignPanelProperty::TransformRepeatCount(index), DesignPanelValue::Integer(value)) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.count = u32::try_from((*value).max(1)).unwrap_or(u32::MAX);
            }
        }
        (
            DesignPanelProperty::TransformRepeatUnit(index),
            DesignPanelValue::TransformUnit(unit),
        ) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.unit = *unit;
            }
        }
        (DesignPanelProperty::TransformRepeatOffset(index), DesignPanelValue::Number(value)) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.offset = *value;
            }
        }
        (
            DesignPanelProperty::PaintOpacity { collection, index },
            DesignPanelValue::Number(value),
        ) => {
            let paint = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    node.selection_colors.get_mut(index)
                }
                DesignPanelCollection::Fill => node.fills.get_mut(index),
                DesignPanelCollection::Stroke => node
                    .stroke
                    .as_mut()
                    .and_then(|stroke| stroke.paints.get_mut(index)),
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            if let Some(paint) = paint {
                paint.opacity = *value;
            }
        }
        (
            DesignPanelProperty::PaintVisible { collection, index },
            DesignPanelValue::Bool(value),
        ) => {
            let paint = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    node.selection_colors.get_mut(index)
                }
                DesignPanelCollection::Fill => node.fills.get_mut(index),
                DesignPanelCollection::Stroke => node
                    .stroke
                    .as_mut()
                    .and_then(|stroke| stroke.paints.get_mut(index)),
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            if let Some(paint) = paint {
                paint.visible = *value;
            }
        }
        (DesignPanelProperty::StrokeWeight, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.set_active(*value);
            }
        }
        (DesignPanelProperty::StrokeWeightMode, DesignPanelValue::StrokeWeightMode(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.set_mode(*value);
            }
        }
        (DesignPanelProperty::StrokeWeightTop, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.top = *value;
            }
        }
        (DesignPanelProperty::StrokeWeightRight, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.right = *value;
            }
        }
        (DesignPanelProperty::StrokeWeightBottom, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.bottom = *value;
            }
        }
        (DesignPanelProperty::StrokeWeightLeft, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.left = *value;
            }
        }
        (DesignPanelProperty::StrokeAlign, DesignPanelValue::StrokeAlign(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && stroke.alignment_options().contains(value)
            {
                stroke.align = *value;
            }
        }
        (DesignPanelProperty::StrokeStartCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.start_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeEndCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.end_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeEndpointCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.endpoint_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeDashMode, DesignPanelValue::StrokeDashMode(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_dash_mode(*value);
            }
        }
        (DesignPanelProperty::StrokeDashPattern, DesignPanelValue::NumberList(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_dash_pattern(value.clone());
            }
        }
        (DesignPanelProperty::StrokeDashCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.dash_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeJoin, DesignPanelValue::StrokeJoin(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.join = *value;
            }
        }
        (DesignPanelProperty::StrokeMiterAngle, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.miter_angle = value.clamp(0., 180.);
            }
        }
        (
            DesignPanelProperty::StrokeVariableWidth,
            DesignPanelValue::StrokeVariableWidth(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_variable_width(value.clone());
            }
        }
        (
            DesignPanelProperty::StrokeVariableWidthPointPosition(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut()
                && stroke.supports_variable_width()
                && let Some(variable_width) = stroke.variable_width.as_mut()
            {
                variable_width.set_point_position(index, *value);
            }
        }
        (
            DesignPanelProperty::StrokeVariableWidthPointWidth(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut()
                && stroke.supports_variable_width()
                && let Some(variable_width) = stroke.variable_width.as_mut()
            {
                variable_width.set_point_width(index, *value);
            }
        }
        (DesignPanelProperty::StrokeType, DesignPanelValue::StrokeType(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_type(*value);
            }
        }
        (DesignPanelProperty::StrokeStretchBrush, DesignPanelValue::StrokeStretchBrush(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::StretchBrush(stretch) = &mut stroke.complex_stroke
            {
                stretch.brush = *value;
            }
        }
        (
            DesignPanelProperty::StrokeBrushDirection,
            DesignPanelValue::StrokeBrushDirection(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::StretchBrush(stretch) = &mut stroke.complex_stroke
            {
                stretch.direction = *value;
            }
        }
        (DesignPanelProperty::StrokeScatterBrush, DesignPanelValue::StrokeScatterBrush(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                scatter.brush = *value;
            }
        }
        (DesignPanelProperty::StrokeScatterGap, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    gap: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterWiggle, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    wiggle: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterSizeJitter, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    size_jitter: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterAngularJitter, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    angular_jitter: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterRotation, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    rotation: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeDynamicFrequency, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::Dynamic(dynamic) = &mut stroke.complex_stroke
            {
                let candidate = DesignDynamicStroke {
                    frequency: *value,
                    ..*dynamic
                };
                if candidate.is_valid() {
                    *dynamic = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeDynamicWiggle, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::Dynamic(dynamic) = &mut stroke.complex_stroke
            {
                let candidate = DesignDynamicStroke {
                    wiggle: *value,
                    ..*dynamic
                };
                if candidate.is_valid() {
                    *dynamic = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeDynamicSmoothen, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::Dynamic(dynamic) = &mut stroke.complex_stroke
            {
                let candidate = DesignDynamicStroke {
                    smoothen: *value,
                    ..*dynamic
                };
                if candidate.is_valid() {
                    *dynamic = candidate;
                }
            }
        }
        (DesignPanelProperty::EffectKind(index), DesignPanelValue::EffectKind(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                effect.set_kind(*value);
            }
        }
        (DesignPanelProperty::EffectVisible(index), DesignPanelValue::Bool(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                effect.visible = *value;
            }
        }
        (DesignPanelProperty::EffectShadowColor(index), DesignPanelValue::Color(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.color = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.color = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowBlendMode(index), DesignPanelValue::BlendMode(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.blend_mode = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.blend_mode = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowBlur(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.radius = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.radius = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowSpread(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.spread = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.spread = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowOffsetX(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.offset.x = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.offset.x = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowOffsetY(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.offset.y = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.offset.y = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (
            DesignPanelProperty::EffectDropShadowShowBehindNode(index),
            DesignPanelValue::Bool(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::DropShadow(settings) = &mut effect.settings
            {
                settings.show_behind_node = *value;
            }
        }
        (DesignPanelProperty::EffectBlurType(index), DesignPanelValue::EffectBlurType(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::LayerBlur(settings)
                    | DesignEffectSettings::BackgroundBlur(settings) => {
                        settings.set_blur_type(*value);
                    }
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (
            DesignPanelProperty::EffectBlurRadius(index)
            | DesignPanelProperty::EffectProgressiveBlurEndRadius(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::LayerBlur(settings)
                    | DesignEffectSettings::BackgroundBlur(settings) => {
                        settings.set_end_radius(*value);
                    }
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurStartRadius(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { start_radius, .. } =
                    settings
            {
                *start_radius = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurStartOffsetX(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { start_offset, .. } =
                    settings
            {
                start_offset.x = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurStartOffsetY(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { start_offset, .. } =
                    settings
            {
                start_offset.y = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurEndOffsetX(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { end_offset, .. } =
                    settings
            {
                end_offset.x = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurEndOffsetY(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { end_offset, .. } =
                    settings
            {
                end_offset.y = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseType(index), DesignPanelValue::EffectNoiseType(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.colors.set_noise_type(*value);
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectNoisePrimaryColor(index), DesignPanelValue::Color(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                match &mut settings.colors {
                    DesignNoiseColors::Monotone { color }
                    | DesignNoiseColors::Duotone { color, .. } => *color = *value,
                    DesignNoiseColors::Multitone { .. } => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectNoiseSecondaryColor(index), DesignPanelValue::Color(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
                && let DesignNoiseColors::Duotone {
                    secondary_color, ..
                } = &mut settings.colors
            {
                *secondary_color = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseOpacity(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
                && let DesignNoiseColors::Multitone { opacity } = &mut settings.colors
            {
                *opacity = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseBlendMode(index), DesignPanelValue::BlendMode(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.blend_mode = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseSizeX(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.size.x = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseSizeY(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.size.y = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseDensity(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.density = *value;
            }
        }
        (DesignPanelProperty::EffectTextureSizeX(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.size.x = *value;
            }
        }
        (DesignPanelProperty::EffectTextureSizeY(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.size.y = *value;
            }
        }
        (DesignPanelProperty::EffectTextureRadius(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.radius = *value;
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectTextureClipToShape(index), DesignPanelValue::Bool(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.clip_to_shape = *value;
            }
        }
        (
            DesignPanelProperty::EffectGlassLightIntensity(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.light_intensity = *value;
            }
        }
        (DesignPanelProperty::EffectGlassLightAngle(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.light_angle = *value;
            }
        }
        (DesignPanelProperty::EffectGlassRefraction(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.refraction = *value;
            }
        }
        (DesignPanelProperty::EffectGlassDepth(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.depth = *value;
            }
        }
        (DesignPanelProperty::EffectGlassDispersion(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.dispersion = *value;
            }
        }
        (DesignPanelProperty::EffectGlassFrost(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.frost = *value;
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectGlassSplay(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.splay = *value;
            }
        }
        (
            DesignPanelProperty::EffectShaderProperty(index, property_index),
            DesignPanelValue::ShaderProperty(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Shader(shader) = &mut effect.settings
                && let Some(property) = shader.properties.get_mut(property_index)
                && !property.read_only
                && value.is_compatible_with(property.kind)
            {
                property.value = value.clone();
            }
        }
        (DesignPanelProperty::ExportSuffix(index), DesignPanelValue::Text(value)) => {
            if let Some(export) = node.export_settings.get_mut(index) {
                export.suffix = value.clone();
            }
        }
        (DesignPanelProperty::ExportFormat(index), DesignPanelValue::ExportFormat(value)) => {
            if let Some(export) = node.export_settings.get_mut(index) {
                export.format = *value;
            }
        }
        (unhandled_property, unhandled_value) => {
            debug_assert!(
                false,
                "Storybook host has no reducer for {unhandled_property:?} with {unhandled_value:?}"
            );
        }
    }
}
