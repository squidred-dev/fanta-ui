//! Composition access to the existing feature projection/controller boundary.
//! There is one dispatch path for legacy and composed surfaces.

use super::*;

pub(in super::super) trait DesignPanelCompositionController: Sized {
    fn prepare_composed_panels(&mut self, window: &mut Window, cx: &mut Context<Self>);
    fn deactivate_composed_panels(&mut self, window: &mut Window, cx: &mut Context<Self>);
    fn composed_panel_kinds(&self) -> Vec<DesignPropertyPanelKind>;
    fn render_feature_section(
        &self,
        section: DesignPanelSection,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement>;
    fn render_composed_panel(
        &self,
        kind: DesignPropertyPanelKind,
        cx: &mut Context<Self>,
    ) -> AnyElement;
    fn cancel_composed_interaction(&mut self, window: &mut Window, cx: &mut Context<Self>);
    fn composed_interaction_root(
        &self,
        id: SharedString,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div>;
}

impl DesignPanelCompositionController for DesignPanel {
    fn prepare_composed_panels(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.component_authoring.dialog_close_pending {
            self.component_authoring.dialog_close_pending = false;
            window.on_next_frame(|window, cx| {
                if window.has_active_dialog(cx) {
                    window.close_dialog(cx);
                }
            });
        }
        if self.shell.reset_after_render {
            self.shell.reset_after_render = false;
            let scroll_handle = self.shell.scroll_handle.clone();
            window.on_next_frame(move |window, _| {
                scroll_handle.set_offset(gpui::point(gpui::Pixels::ZERO, gpui::Pixels::ZERO));
                window.refresh();
            });
        }
        if self.renders_draw_workspace() {
            self.sync_draw_appearance_sliders(window, cx);
        }
        if self.host.inspection_context.selection().kind() != DesignPanelSelectionKind::None
            || self.can_export()
        {
            self.sync_option_states(window, cx);
        }
        if self.host.inspection_context.permissions().can_edit() {
            self.sync_typography_style_picker(cx);
            self.sync_paint_picker(window, cx);
        }
    }

    fn deactivate_composed_panels(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Keep the host snapshot and navigation intact. Only transient UI and
        // in-flight transactions belong to this lifecycle operation.
        self.cancel_dimension_limits_preview(cx);
        self.cancel_host_interactions_for_context_change(true, cx);
        self.cancel_component_authoring_for_workspace_change(cx);
        self.close_editor_only_overlays();
        self.overlays.clear_for_context_change();
        self.edit.clear_after_cancellation();
        if self.component_authoring.dialog_close_pending {
            self.component_authoring.dialog_close_pending = false;
            if window.has_active_dialog(cx) {
                window.close_dialog(cx);
            }
        }
        cx.notify();
    }

    fn composed_panel_kinds(&self) -> Vec<DesignPropertyPanelKind> {
        if self.host.inspection_context.selection().kind() == DesignPanelSelectionKind::None {
            let mut kinds = vec![DesignPropertyPanelKind::Page];
            if self.can_export() {
                kinds.push(DesignPropertyPanelKind::Export);
            }
            return kinds;
        }
        if !self.host.inspection_context.permissions().can_edit() {
            let mut kinds = vec![DesignPropertyPanelKind::Viewer];
            if self.can_export() {
                kinds.push(DesignPropertyPanelKind::Export);
            }
            return kinds;
        }
        let mut kinds = Vec::new();
        for projection in self.sections.resolve(
            self.host.inspected_node(),
            self.host.navigation.workspace_mode,
            self.can_export(),
        ) {
            if let Some(kind) = DesignPropertyPanelKind::from_section(projection.section)
                && !kinds.contains(&kind)
            {
                kinds.push(kind);
            }
        }
        kinds
    }

    fn render_feature_section(
        &self,
        section: DesignPanelSection,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        match section {
            DesignPanelSection::Selection => self.render_selection_colors(cx),
            DesignPanelSection::Component | DesignPanelSection::Instance => {
                self.render_component(cx)
            }
            DesignPanelSection::Position => {
                let projection = position::projection(self);
                let events = sections::position::PositionEventSink::new(cx.entity());
                Some(sections::position::render(&projection, self, &events, cx))
            }
            DesignPanelSection::Layout => Some(self.render_layout(cx)),
            DesignPanelSection::Constraints => {
                let projection = position::projection(self);
                Some(sections::position::render_constraints(
                    &projection,
                    self,
                    cx,
                ))
            }
            DesignPanelSection::Layer => Some(self.render_layer(cx)),
            DesignPanelSection::Section
            | DesignPanelSection::Transform
            | DesignPanelSection::Geometry => {
                let projection = sections::shape::ShapeProjection::from_node(
                    self.id.clone(),
                    self.can_edit(),
                    self.host.inspected_node(),
                );
                let events = sections::shape::ShapeEventSink::new(cx.entity());
                match section {
                    DesignPanelSection::Section => {
                        sections::shape::render_section_properties(&projection, self, events, cx)
                    }
                    DesignPanelSection::Transform => {
                        sections::shape::render_transform_modifiers(&projection, self, events, cx)
                    }
                    _ => sections::shape::render_geometry(&projection, self, cx),
                }
            }
            DesignPanelSection::Mask => self.render_mask(cx),
            DesignPanelSection::Typography => self.render_typography(cx),
            DesignPanelSection::Media => self.render_media(cx),
            DesignPanelSection::Fill => self.render_fill(cx),
            DesignPanelSection::Stroke => self.render_stroke(cx),
            DesignPanelSection::Effects => Some(self.render_effects(cx)),
            DesignPanelSection::LayoutGrid => self.render_layout_grids(cx),
            DesignPanelSection::Export => Some(self.render_export(cx)),
        }
    }

    fn render_composed_panel(
        &self,
        kind: DesignPropertyPanelKind,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if kind == DesignPropertyPanelKind::Page {
            return self.render_page(cx);
        }
        if kind == DesignPropertyPanelKind::Viewer {
            let permissions = self.host.inspection_context.permissions();
            let projection = sections::viewer::ViewerProjection::new(
                self.id.clone(),
                permissions.can_edit(),
                permissions.can_copy(),
                self.command_target(),
                self.host.projections.viewer_properties.clone(),
            );
            return sections::viewer::render(
                projection,
                sections::viewer::ViewerEventSink::new(cx.entity()),
                cx,
            );
        }
        if kind == DesignPropertyPanelKind::Export {
            return self.render_export(cx);
        }
        let sections = self.sections.resolve(
            self.host.inspected_node(),
            self.host.navigation.workspace_mode,
            self.can_export(),
        );
        v_flex()
            .w_full()
            .children(sections.into_iter().filter_map(|projection| {
                (DesignPropertyPanelKind::from_section(projection.section) == Some(kind))
                    .then(|| self.render_feature_section(projection.section, cx))
                    .flatten()
            }))
            .into_any_element()
    }

    fn cancel_composed_interaction(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.edit.component_authoring.has_name_editor() {
            self.finish_component_authoring_name_edit(false, window, cx);
        } else if self.edit.component_authoring.has_property_reorder() {
            self.finish_component_property_reorder(false, cx);
        } else if self.edit.component_authoring.has_variant_option_reorder() {
            self.finish_component_variant_option_reorder(false, cx);
        } else if self.component_authoring.create_draft.is_some() {
            self.cancel_component_property_create(window, cx);
        } else if let Some(property) = self.edit.draw_slider_property {
            self.finish_draw_appearance_slider(property, false, cx);
        } else if self.edit.variable_font_axis.is_some() {
            self.finish_variable_font_axis_edit(false, window, cx);
        } else if self.edit.variable_font_axis_scrub.is_some() {
            self.edit.variable_font_axis_scrub = None;
            cx.notify();
        } else if self.edit.property.is_some() {
            self.finish_property_edit(false, window, cx);
        } else if self.edit.component_multiline.is_some() {
            self.finish_component_multiline_editor(false, window, cx);
        } else if !self.dismiss_topmost_overlay(window, cx) {
            cx.propagate();
        }
    }

    fn composed_interaction_root(
        &self,
        id: SharedString,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        v_flex()
            .id(id.clone())
            .debug_selector(move || id.to_string())
            .key_context(DESIGN_PANEL_KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .relative()
            .occlude()
            .min_w_0()
            .on_action(
                cx.listener(|this, _: &CancelDesignInteraction, window, cx| {
                    this.cancel_composed_interaction(window, cx);
                }),
            )
            .on_key_down(cx.listener(Self::handle_property_key_down))
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .bg(crate::atoms::SemanticColor::BackgroundToolbar.resolve(cx))
            .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
            .typography(crate::atoms::TypographyToken::BodyLarge)
    }
}
