//! The Design story reducer: a thin dispatcher that preserves the exact
//! ARCHITECTURE §11 preflight order — compatibility rejection, surface and
//! menu echoes, the targeted multi-node replay, page/export/selection phases,
//! then single-node resolution — delegating each action family to its module.

pub(crate) mod arrange;
pub(crate) mod components;
pub(crate) mod effects;
pub(crate) mod exports;
pub(crate) mod layout;
pub(crate) mod media;
pub(crate) mod paints;
pub(crate) mod properties;
pub(crate) mod sections;
pub(crate) mod selection;
pub(crate) mod smart_selection;
pub(crate) mod styles;
pub(crate) mod surfaces;
pub(crate) mod targeted;
pub(crate) mod typography;
pub(crate) mod vector;

// Re-export every family so sibling modules, the design screen, and the
// storybook tests can share the partitioned helpers without tracking
// which module owns each one.
#[allow(unused_imports)]
pub(crate) use {
    arrange::*, components::*, effects::*, exports::*, layout::*, media::*, paints::*,
    properties::*, sections::*, selection::*, smart_selection::*, styles::*, surfaces::*,
    targeted::*, typography::*, vector::*,
};

use super::*;

/// How one single-node family module resolved an action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeOutcome {
    /// The family applied the action; the shared node-edit finish and
    /// inspection-context echo tail still runs.
    Applied,
    /// The family completed the action itself (usually a stale-target
    /// rejection) and the shared tail must not run.
    Complete,
}

impl DesignScreen {
    #[allow(deprecated)]
    pub(crate) fn handle_action(
        &mut self,
        panel: Entity<DesignPanel>,
        action: &DesignPanelAction,
        cx: &mut Context<Storybook>,
    ) {
        if let Some(path) = action.compatibility_path() {
            self.harness.last_action = format!(
                "Ignored compatibility-only Design action; use {}",
                path.replacement()
            )
            .into();
            cx.notify();
            return;
        }
        if surfaces::echo_surface_change(self, &panel, action, cx) {
            return;
        }
        if surfaces::apply_menu_preview_transaction(self, &panel, action, cx) {
            return;
        }
        if matches!(
            action,
            DesignPanelAction::PropertyChangeRequested { .. }
                | DesignPanelAction::EffectEditRequested {
                    phase: DesignPanelEditPhase::Commit,
                    ..
                }
                | DesignPanelAction::PaintEditRequested {
                    phase: DesignPanelEditPhase::Commit,
                    ..
                }
        ) {
            self.edits.menu_preview = None;
        }
        if targeted::replay_preflighted_target(self, &panel, action, cx) {
            return;
        }
        if smart_selection::reduce_spacing_and_arrange(self, &panel, action, cx) {
            return;
        }
        if smart_selection::add_auto_layout(self, &panel, action, cx) {
            return;
        }
        if surfaces::copy_property(self, &panel, action, cx) {
            return;
        }
        if surfaces::preview_dimension_limits(self, &panel, action, cx) {
            return;
        }
        if surfaces::copy_viewer_section(self, &panel, action, cx) {
            return;
        }
        if surfaces::change_viewer_representation(self, &panel, action, cx) {
            return;
        }
        if styles::reduce_page_and_local_styles(self, &panel, action, cx) {
            return;
        }
        if exports::reduce(self, &panel, action, cx) {
            return;
        }
        if selection::acknowledge_header_command(self, &panel, action, cx) {
            return;
        }
        if selection::select_color_occurrences(self, &panel, action, cx) {
            return;
        }
        if selection::edit_selection_color_paint(self, &panel, action, cx) {
            return;
        }
        if selection::edit_selection_color(self, &panel, action, cx) {
            return;
        }
        if selection::reduce_color_resources(self, &panel, action, cx) {
            return;
        }
        if arrange::reduce(self, &panel, action, cx) {
            return;
        }
        let action_node_id = match action {
            DesignPanelAction::TypographyPropertyChangeRequested { node_id, .. }
            | DesignPanelAction::TypographyPropertyEditRequested { node_id, .. }
            | DesignPanelAction::TypographyVariableAxisEditRequested { node_id, .. }
            | DesignPanelAction::TypographyStyleApplyRequested { node_id, .. }
            | DesignPanelAction::TypographyStyleDetachRequested { node_id, .. }
            | DesignPanelAction::TypographyFontApplyRequested { node_id, .. }
            | DesignPanelAction::TypographyFontImportRequested { node_id, .. }
            | DesignPanelAction::TypographyOpenTypeFeatureChangeRequested { node_id, .. }
            | DesignPanelAction::TextPathFlipOrientationRequested { node_id }
            | DesignPanelAction::TextPathStartChangeRequested { node_id, .. }
            | DesignPanelAction::VectorVertexSelectionEditRequested { node_id, .. }
            | DesignPanelAction::VectorVertexPositionEditRequested { node_id, .. }
            | DesignPanelAction::VectorVertexCornerRadiusEditRequested { node_id, .. }
            | DesignPanelAction::VectorHandleMirroringEditRequested { node_id, .. }
            | DesignPanelAction::PropertyChangeRequested { node_id, .. }
            | DesignPanelAction::PropertyEditRequested { node_id, .. }
            | DesignPanelAction::PropertyVariableApplyRequested { node_id, .. }
            | DesignPanelAction::PropertyVariableImportRequested { node_id, .. }
            | DesignPanelAction::PropertyVariableDetachRequested { node_id, .. }
            | DesignPanelAction::SectionShareRequested { node_id }
            | DesignPanelAction::SectionResolveChangedStatusRequested { node_id }
            | DesignPanelAction::TransformModifierAddRequested { node_id, .. }
            | DesignPanelAction::TransformModifierRemoveRequested { node_id, .. }
            | DesignPanelAction::TransformModifierChangeRequested { node_id, .. }
            | DesignPanelAction::ApplyTransformModifiersRequested { node_id }
            | DesignPanelAction::ComponentPropertyDefinitionCreateRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionRenameRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionMetadataEditRequested {
                node_id, ..
            }
            | DesignPanelAction::ComponentPropertyDefinitionEditRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionDeleteRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionReorderRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionCreateRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionRenameRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionDeleteRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionReorderRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyApplyToLayerRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertySwitchOnLayerRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDetachFromLayerRequested { node_id, .. }
            | DesignPanelAction::NestedComponentPropertyExposeRequested { node_id, .. }
            | DesignPanelAction::NestedComponentPropertyUnexposeRequested { node_id, .. }
            | DesignPanelAction::NestedComponentPropertyPreviewRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyChangeRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyEditRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyResetRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyVariableApplyRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyVariableImportRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyVariableDetachRequested { node_id, .. }
            | DesignPanelAction::ComponentSwapApplyRequested { node_id, .. }
            | DesignPanelAction::ComponentSwapImportRequested { node_id, .. }
            | DesignPanelAction::ComponentSwapPreviewRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyNestedInstanceSelectRequested {
                node_id, ..
            }
            | DesignPanelAction::ComponentPropertyNestedInstanceGoToMainRequested {
                node_id, ..
            }
            | DesignPanelAction::SlotSettingsChangeRequested { node_id, .. }
            | DesignPanelAction::SlotResetRequested { node_id, .. }
            | DesignPanelAction::SlotClearRequested { node_id, .. }
            | DesignPanelAction::SlotAddInstanceRequested { node_id, .. }
            | DesignPanelAction::SlotChildSelectRequested { node_id, .. }
            | DesignPanelAction::SlotLimitLayersSelectRequested { node_id, .. }
            | DesignPanelAction::SlotChildRemoveRequested { node_id, .. }
            | DesignPanelAction::SlotChildReorderRequested { node_id, .. }
            | DesignPanelAction::SlotChildReplaceRequested { node_id, .. }
            | DesignPanelAction::CollectionItemAddRequested { node_id, .. }
            | DesignPanelAction::CollectionItemRemoveRequested { node_id, .. }
            | DesignPanelAction::EffectAddRequested { node_id, .. }
            | DesignPanelAction::EffectRemoveRequested { node_id, .. }
            | DesignPanelAction::EffectReorderRequested { node_id, .. }
            | DesignPanelAction::EffectEditRequested { node_id, .. }
            | DesignPanelAction::EffectShaderChooseRequested { node_id, .. }
            | DesignPanelAction::EffectShaderPropertyEditorRequested { node_id, .. }
            | DesignPanelAction::EffectShaderPropertyVariableDetachRequested { node_id, .. }
            | DesignPanelAction::EffectStyleApplyRequested { node_id, .. }
            | DesignPanelAction::EffectStyleCreateRequested { node_id, .. }
            | DesignPanelAction::EffectStyleDetachRequested { node_id, .. }
            | DesignPanelAction::EffectVariableApplyRequested { node_id, .. }
            | DesignPanelAction::EffectVariableDetachRequested { node_id, .. }
            | DesignPanelAction::GridDimensionsEditRequested { node_id, .. }
            | DesignPanelAction::GridTrackAddRequested { node_id, .. }
            | DesignPanelAction::GridTrackDeleteRequested { node_id, .. }
            | DesignPanelAction::GridTracksReorderRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleCreateRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleDetachRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleImportRequested { node_id, .. }
            | DesignPanelAction::LayoutGridPropertyEditRequested { node_id, .. }
            | DesignPanelAction::LayoutGridRemoveRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableImportRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableDetachRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableCreateRequested { node_id, .. }
            | DesignPanelAction::LayoutGridCountVariableApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridCountVariableDetachRequested { node_id, .. }
            | DesignPanelAction::PaintEditRequested { node_id, .. }
            | DesignPanelAction::PaintReorderRequested { node_id, .. }
            | DesignPanelAction::PaintSourceReplaceRequested { node_id, .. }
            | DesignPanelAction::PaintMediaSourceActionRequested { node_id, .. }
            | DesignPanelAction::PaintMediaSourceDropRequested { node_id, .. }
            | DesignPanelAction::PaintMediaCropActionRequested { node_id, .. }
            | DesignPanelAction::PaintVideoPreviewActionRequested { node_id, .. }
            | DesignPanelAction::PaintShaderImportRequested { node_id, .. }
            | DesignPanelAction::PaintShaderApplyRequested { node_id, .. }
            | DesignPanelAction::PaintShaderPropertyBindRequested { node_id, .. }
            | DesignPanelAction::PaintShaderPropertyEditorRequested { node_id, .. }
            | DesignPanelAction::PaintShaderPropertyDetachRequested { node_id, .. }
            | DesignPanelAction::PaintStyleApplyRequested { node_id, .. }
            | DesignPanelAction::PaintStyleImportRequested { node_id, .. }
            | DesignPanelAction::PaintStyleCreateRequested { node_id, .. }
            | DesignPanelAction::PaintStyleDetachRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableApplyRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableImportRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableDetachRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableCreateRequested { node_id, .. }
            | DesignPanelAction::PaintColorStyleSampleRequested { node_id, .. }
            | DesignPanelAction::PaintColorStyleApplyRequested { node_id, .. }
            | DesignPanelAction::PaintColorStyleCreateRequested { node_id, .. }
            | DesignPanelAction::PaintEyedropperRequested { node_id, .. }
            | DesignPanelAction::SwapStrokeEndpointsRequested { node_id }
            | DesignPanelAction::ResetInstanceOverridesRequested { node_id }
            | DesignPanelAction::GoToMainComponentRequested { node_id }
            | DesignPanelAction::DetachInstanceRequested { node_id } => node_id,
            DesignPanelAction::PropertyCopyRequested { .. }
            | DesignPanelAction::DimensionLimitsPreviewRequested { .. }
            | DesignPanelAction::MenuPreviewRequested { .. }
            | DesignPanelAction::ViewerSectionCopyRequested { .. }
            | DesignPanelAction::ViewerSectionRepresentationChangeRequested { .. }
            | DesignPanelAction::SurfaceChangeRequested { .. }
            | DesignPanelAction::SelectionHeaderCommandRequested { .. }
            | DesignPanelAction::AddAutoLayoutRequested { .. }
            | DesignPanelAction::SmartSelectionSpacingEditRequested { .. }
            | DesignPanelAction::SmartSelectionArrangeRequested { .. }
            | DesignPanelAction::TargetedNodeActionRequested { .. }
            | DesignPanelAction::ArrangeRequested { .. }
            | DesignPanelAction::TransformRequested { .. }
            | DesignPanelAction::ResizeToFitRequested { .. }
            | DesignPanelAction::FramePresetApplyRequested { .. }
            | DesignPanelAction::ExportConfigurationAddRequested { .. }
            | DesignPanelAction::ExportConfigurationRemoveRequested { .. }
            | DesignPanelAction::ExportConfigurationChangeRequested { .. }
            | DesignPanelAction::ExportModeChangeRequested { .. }
            | DesignPanelAction::AnimatedExportChangeRequested { .. }
            | DesignPanelAction::AnimatedExportRequested { .. }
            | DesignPanelAction::ExportAllRequested { .. }
            | DesignPanelAction::ExportPreviewRequested { .. }
            | DesignPanelAction::PageBackgroundEditRequested { .. }
            | DesignPanelAction::PageBackgroundChangeRequested { .. }
            | DesignPanelAction::LocalResourceBrowseRequested { .. }
            | DesignPanelAction::LocalResourceOpenRequested { .. }
            | DesignPanelAction::LocalResourceCreateRequested { .. }
            | DesignPanelAction::LocalResourceImportRequested { .. }
            | DesignPanelAction::LocalStyleCommandRequested { .. }
            | DesignPanelAction::LocalStyleCreateRequested { .. }
            | DesignPanelAction::LocalStyleFolderCreateRequested { .. }
            | DesignPanelAction::LocalStylesDeleteRequested { .. }
            | DesignPanelAction::LocalStylesMoveRequested { .. }
            | DesignPanelAction::VariablesViewOpenRequested { .. }
            | DesignPanelAction::VariableModeApplyRequested { .. }
            | DesignPanelAction::VariableModeClearRequested { .. }
            | DesignPanelAction::SelectionColorEditRequested { .. }
            | DesignPanelAction::SelectionColorPaintEditRequested { .. }
            | DesignPanelAction::SelectionColorOccurrencesSelectRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleApplyRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleImportRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleCreateRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleDetachRequested { .. }
            | DesignPanelAction::SelectionColorVariableApplyRequested { .. }
            | DesignPanelAction::SelectionColorVariableImportRequested { .. }
            | DesignPanelAction::SelectionColorVariableCreateRequested { .. }
            | DesignPanelAction::SelectionColorVariableDetachRequested { .. } => unreachable!(),
            DesignPanelAction::PaintChangeRequested { .. }
            | DesignPanelAction::ExportRequested { .. }
            | DesignPanelAction::ReplaceMediaRequested { .. } => {
                unreachable!("compatibility actions return before node resolution")
            }
        };
        let Some(node_index) = self
            .host
            .nodes
            .iter()
            .position(|node| node.id == *action_node_id)
        else {
            self.harness.last_action =
                format!("Ignored stale Design action for {action_node_id}").into();
            cx.notify();
            return;
        };
        let inside_auto_layout = node_index == self.harness.selected_node
            && self.inspection_context().0.parent_layout().is_auto_layout();
        if sections::reduce(self, &panel, action, node_index, cx) {
            return;
        }
        if paints::rejects_stale_paint_target(self, action, action_node_id, cx) {
            return;
        }
        if paints::rejects_out_of_order_lifecycle(self, action, action_node_id, cx) {
            return;
        }
        let media_drop_capabilities = match action {
            DesignPanelAction::PaintMediaSourceDropRequested {
                collection,
                paint_id,
                index,
                ..
            } => self
                .host
                .media_paint_views
                .get(action_node_id)
                .and_then(|views| views.paint(*collection, paint_id, *index))
                .map_or_else(DesignMediaPaintCapabilities::default, |view| {
                    view.capabilities
                }),
            _ => DesignMediaPaintCapabilities::viewer(),
        };
        let node_edit_transaction = story_node_edit_transaction(action);
        if let Some((target, phase)) = &node_edit_transaction {
            begin_story_node_edit(
                &self.host.nodes[node_index],
                &mut self.edits.node_edit_snapshots,
                target,
                *phase,
            );
        }
        let mut outcome =
            typography::reduce(self, &panel, action, node_index, inside_auto_layout, cx);
        if outcome.is_none() {
            outcome = vector::reduce(self, action, node_index, cx);
        }
        if outcome.is_none() {
            outcome = properties::reduce(self, action, node_index, inside_auto_layout, cx);
        }
        if outcome.is_none() {
            outcome = components::reduce(self, action, node_index, cx);
        }
        if outcome.is_none() {
            outcome = effects::reduce(self, action, node_index, cx);
        }
        if outcome.is_none() {
            outcome = layout::reduce(self, action, node_index, cx);
        }
        if outcome.is_none() {
            outcome = paints::reduce(self, action, node_index, cx);
        }
        if outcome.is_none() {
            outcome = media::reduce(self, action, node_index, media_drop_capabilities, cx);
        }
        let Some(outcome) = outcome else {
            unreachable!("every single-node Design action is reduced by exactly one family module")
        };
        if outcome == NodeOutcome::Complete {
            return;
        }
        if let Some((target, phase)) = node_edit_transaction {
            finish_story_node_edit(
                &mut self.host.nodes[node_index],
                &mut self.edits.node_edit_snapshots,
                target,
                phase,
            );
        }
        self.apply_inspection_context(&panel, cx);
        cx.notify();
    }
}
