//! Media input and transform validation kept separate from retained picker UI state.

use crate::atoms::TypographyExt as _;
use std::path::PathBuf;

use crate::design::{
    DesignMediaCropAction, DesignMediaCropToolState, DesignMediaDroppedFile,
    DesignMediaPaintCapabilities, DesignPaintTransform,
};

pub(super) const CROP_ROTATION_STEP_DEGREES: f32 = 15.;

pub(super) fn rotated_crop_preview(crop_tool: DesignMediaCropToolState) -> DesignMediaCropAction {
    DesignMediaCropAction::Preview {
        transform: crop_tool.transform.rotated(CROP_ROTATION_STEP_DEGREES),
        zoom: crop_tool.zoom,
        aspect_ratio: crop_tool.aspect_ratio,
    }
}

pub(crate) fn media_drop_from_paths(
    paths: &[PathBuf],
    capabilities: DesignMediaPaintCapabilities,
) -> Option<DesignMediaDroppedFile> {
    capabilities
        .can_upload_source
        .then(|| DesignMediaDroppedFile::from_paths(paths, capabilities.accepted_drop_file_kinds))
        .flatten()
}

pub(super) fn media_transform_is_finite(transform: DesignPaintTransform) -> bool {
    [
        transform.m11,
        transform.m12,
        transform.m21,
        transform.m22,
        transform.tx,
        transform.ty,
    ]
    .into_iter()
    .all(f32::is_finite)
}

fn media_asset_matches(asset: &DesignMediaPaintAsset, paint_type: DesignPaintType) -> bool {
    !asset.source.id.is_empty()
        && matches!(
            (asset.kind, paint_type),
            (DesignMediaKind::Image, DesignPaintType::Image)
                | (DesignMediaKind::Video, DesignPaintType::Video)
        )
}

// Retained media-editor state and rendering are kept with the pure media
// validation helpers so the parent picker remains an orchestration shell.
use super::*;

impl PaintPicker {
    pub(super) fn select_media_asset(&mut self, source_id: &SharedString, cx: &mut Context<Self>) {
        if self.base_editing_disabled() {
            return;
        }
        let Some(paint_type) = self.paint.as_ref().map(DesignPaint::paint_type) else {
            return;
        };
        let Some(asset) =
            self.media_view_data.assets.iter().find(|asset| {
                asset.source.id == *source_id && media_asset_matches(asset, paint_type)
            })
        else {
            return;
        };
        if self
            .emit_edit(
                DesignPaintEdit {
                    property: DesignPaintProperty::Source,
                    value: DesignPaintValue::Source(asset.source.clone()),
                },
                DesignPanelEditPhase::Commit,
                cx,
            )
            .is_some()
        {
            self.media_tab = MediaPickerTab::Adjust;
            cx.notify();
        }
    }

    pub(super) fn render_media_tabs(&self, cx: &mut Context<Self>) -> AnyElement {
        crate::atoms::InspectorTabs::new(
            format!("{}-media-tabs", self.id),
            ["Source", "Assets", "Adjust"].map(|label| {
                (
                    format!("{}-media-tab-{}", self.id, label.to_lowercase()).into(),
                    label.into(),
                )
            }),
        )
        .selected_index(match self.media_tab {
            MediaPickerTab::Source => 0,
            MediaPickerTab::Assets => 1,
            MediaPickerTab::Adjust => 2,
        })
        .on_change(
            cx.listener(|this, selection: &crate::atoms::TabSelection, _, cx| {
                this.media_tab = match selection.index {
                    0 => MediaPickerTab::Source,
                    1 => MediaPickerTab::Assets,
                    _ => MediaPickerTab::Adjust,
                };
                this.scroll_handle.set_offset(Point::default());
                cx.notify();
            }),
        )
        .into_any_element()
    }

    fn render_media_assets(&self, paint: &DesignPaint, cx: &mut Context<Self>) -> AnyElement {
        let assets: Vec<_> = self
            .media_view_data
            .assets
            .iter()
            .filter(|asset| media_asset_matches(asset, paint.paint_type()))
            .collect();
        let source_id = match &paint.payload {
            DesignPaintPayload::Image(image) => Some(&image.source.id),
            DesignPaintPayload::Video(video) => Some(&video.source.id),
            _ => None,
        };
        let mut content = v_flex()
            .w_full()
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Reuse media from this project"),
            );
        if assets.is_empty() {
            return content
                .child(
                    div()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .child("No compatible assets available"),
                )
                .into_any_element();
        }
        for asset in assets {
            let id = asset.source.id.clone();
            let selector = format!("{}-media-asset-{}", self.id, id);
            content = content.child(
                crate::atoms::ui_button(SharedString::from(selector.clone()))
                    .debug_selector(move || selector.clone())
                    .child(picker_menu_option(
                        format!("{}-media-asset-{id}", self.id),
                        asset.source.name.clone(),
                        Some(if asset.kind == DesignMediaKind::Image {
                            LucideIcon::Image
                        } else {
                            LucideIcon::Video
                        }),
                        source_id == Some(&id),
                        cx,
                    ))
                    .xsmall()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .compact()
                    .ghost()
                    .w_full()
                    .h(px(tokens::RowHeight::LIST))
                    .px(px(tokens::Space::SM))
                    .min_w_0()
                    .justify_start()
                    .selected(source_id == Some(&id))
                    .disabled(self.base_editing_disabled())
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.select_media_asset(&id, cx);
                    })),
            );
        }
        content.into_any_element()
    }
    pub(super) fn current_media_view(&self) -> Option<&DesignMediaPaintView> {
        let target = self.target.as_ref()?;
        self.media_view_data
            .paint(target.collection, &target.paint_id, target.index)
    }

    pub(super) fn current_media_capabilities(&self) -> DesignMediaPaintCapabilities {
        self.current_media_view()
            .map_or_else(DesignMediaPaintCapabilities::default, |view| {
                view.capabilities
            })
    }

    pub(super) fn base_editing_disabled(&self) -> bool {
        self.disabled || self.paint.as_ref().is_some_and(paint_picker_locked)
    }

    pub(super) fn editing_disabled(&self) -> bool {
        self.base_editing_disabled()
            || self.paint.as_ref().is_some_and(|paint| {
                matches!(
                    &paint.payload,
                    DesignPaintPayload::Image(_) | DesignPaintPayload::Video(_)
                ) && !self.current_media_capabilities().can_edit_properties
            })
    }

    #[cfg(test)]
    pub(super) fn source_replacement_disabled(&self) -> bool {
        if self.base_editing_disabled() {
            return true;
        }
        match self.paint.as_ref().map(|paint| &paint.payload) {
            Some(DesignPaintPayload::Pattern(_)) => false,
            Some(DesignPaintPayload::Image(_) | DesignPaintPayload::Video(_)) => {
                self.media_source_action_disabled(DesignMediaSourceAction::Upload)
            }
            _ => true,
        }
    }

    pub(super) fn media_source_action_disabled(&self, action: DesignMediaSourceAction) -> bool {
        if self.base_editing_disabled() {
            return true;
        }
        let Some(paint_type) = self.paint.as_ref().map(DesignPaint::paint_type) else {
            return true;
        };
        if action == DesignMediaSourceAction::EditImage && self.paint.as_ref().is_some_and(|paint| matches!(&paint.payload, DesignPaintPayload::Image(image) if image.source.id.is_empty())) {
            return true;
        }
        !action.is_applicable_to(paint_type)
            || !self
                .current_media_capabilities()
                .allows_source_action(action)
    }

    pub(super) fn select_pattern_source(
        &mut self,
        source_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.base_editing_disabled()
            || source_id.is_empty()
            || self
                .target
                .as_ref()
                .is_some_and(|target| target.node_id == source_id)
            || self.media_view_data.pattern_source(&source_id).is_none()
            || !matches!(
                self.paint.as_ref().map(|paint| &paint.payload),
                Some(DesignPaintPayload::Pattern(pattern)) if pattern.source_node_id != source_id
            )
        {
            return;
        }
        if self
            .emit_edit(
                DesignPaintEdit {
                    property: DesignPaintProperty::PatternSourceNode,
                    value: DesignPaintValue::PatternSourceNode(source_id),
                },
                DesignPanelEditPhase::Commit,
                cx,
            )
            .is_some()
        {
            self.close_nested_overlay(PaintPickerOverlay::PatternSource, window, cx);
        }
    }

    fn render_pattern_source_selector(
        &self,
        sources: Vec<DesignPatternSource>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let picker_for_open = cx.entity();
        let picker_for_trigger = picker_for_open.clone();
        let picker_for_content = picker_for_open.clone();
        let picker_id = self.id.clone();
        let selected_source_id = self.paint.as_ref().and_then(|paint| match &paint.payload {
            DesignPaintPayload::Pattern(pattern) => Some(pattern.source_node_id.clone()),
            _ => None,
        });
        let disabled = self.base_editing_disabled() || sources.is_empty();
        let trigger = crate::atoms::ui_button(SharedString::from(format!(
            "{}-pattern-source-trigger",
            self.id
        )))
        .label(
            if selected_source_id.as_ref().is_some_and(|id| !id.is_empty()) {
                "Change"
            } else {
                "Choose"
            },
        )
        .tooltip(if sources.is_empty() {
            "No eligible layers on this page"
        } else {
            "Choose a layer for the pattern tile"
        })
        .xsmall()
        .typography(crate::atoms::TypographyToken::Panel)
        .compact()
        .outline()
        .selected(self.nested_overlay_is_open(PaintPickerOverlay::PatternSource))
        .disabled(disabled)
        .on_keyboard_activate(move |window, cx| {
            picker_for_trigger.update(cx, |this, cx| {
                if this.nested_overlay_is_open(PaintPickerOverlay::PatternSource) {
                    this.close_nested_overlay(PaintPickerOverlay::PatternSource, window, cx);
                } else {
                    this.open_nested_overlay(PaintPickerOverlay::PatternSource, window, cx);
                    cx.notify();
                }
            });
        });

        let selector = format!("{}-pattern-source-trigger", self.id);
        let popover = Popover::new(SharedString::from(format!(
            "{}-pattern-source-menu",
            self.id
        )))
        .anchor(Anchor::BottomRight)
        .appearance(false)
        .open(self.nested_overlay_is_open(PaintPickerOverlay::PatternSource))
        .overlay_closable(true)
        .on_open_change(move |open, window, cx| {
            picker_for_open.update(cx, |this, cx| {
                if *open {
                    this.open_nested_overlay(PaintPickerOverlay::PatternSource, window, cx);
                    cx.notify();
                } else {
                    this.dismiss_nested_overlay(
                        PaintPickerOverlay::PatternSource,
                        InspectorOverlayDismissCause::OutsideClick,
                        window,
                        cx,
                    );
                }
            });
        })
        .trigger(trigger)
        .content(move |_, window, cx| {
            crate::molecules::sidebar_popup_surface(
                SharedString::from(format!("{picker_id}-pattern-source-options")),
                cx,
            )
            .w(popup_width(window, tokens::MenuWidth::STANDARD))
            .max_h(popup_height(window, 300.))
            .overflow_y_scroll()
            .p(px(tokens::Space::XS))
            .child(
                div()
                    .px(px(tokens::Space::SM))
                    .py(px(tokens::Space::XS))
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Layers on this page"),
            )
            .children(sources.clone().into_iter().map(|source| {
                let picker = picker_for_content.clone();
                let source_id = source.id.clone();
                let selector = format!("{}-pattern-source-{}", picker_id, source.id);
                let button = crate::atoms::ui_button(SharedString::from(format!(
                    "{}-pattern-source-{}",
                    picker_id, source.id
                )))
                .child(picker_menu_option(
                    selector.clone(),
                    source.name,
                    Some(LucideIcon::Layers),
                    selected_source_id.as_ref() == Some(&source_id),
                    cx,
                ))
                .tooltip(source.id)
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .w_full()
                .h(px(tokens::RowHeight::MENU))
                .px(px(tokens::Space::SM))
                .selected(selected_source_id.as_ref() == Some(&source_id))
                .on_activate(move |_, window, cx| {
                    picker.update(cx, |this, cx| {
                        this.select_pattern_source(source_id.clone(), window, cx);
                    });
                });
                div()
                    .id(SharedString::from(format!("{selector}-row")))
                    .debug_selector(move || selector.clone())
                    .w_full()
                    .min_w_0()
                    .child(button)
            }))
        });
        div()
            .id(SharedString::from(format!("{selector}-anchor")))
            .debug_selector(move || selector.clone())
            .child(popover)
            .into_any_element()
    }

    pub(super) fn request_media_source_action(
        &self,
        action: DesignMediaSourceAction,
        cx: &mut Context<Self>,
    ) {
        if self.media_source_action_disabled(action) {
            return;
        }
        let (Some(target), Some(paint)) = (self.target.clone(), self.paint.as_ref()) else {
            return;
        };
        let source_id = match &paint.payload {
            DesignPaintPayload::Image(image) => image.source.id.clone(),
            DesignPaintPayload::Video(video) => video.source.id.clone(),
            _ => return,
        };
        cx.emit(PaintPickerEvent::MediaSourceActionRequested {
            target,
            source_id,
            action,
        });
    }

    pub(super) fn request_media_source_drop(&self, paths: &[PathBuf], cx: &mut Context<Self>) {
        if self.base_editing_disabled() {
            return;
        }
        let (Some(target), Some(paint)) = (self.target.clone(), self.paint.as_ref()) else {
            return;
        };
        let capabilities = self.current_media_capabilities();
        let Some(file) = media_drop_from_paths(paths, capabilities) else {
            return;
        };
        let (expected_media_kind, expected_source_id) = match &paint.payload {
            DesignPaintPayload::Image(image) => (DesignMediaKind::Image, image.source.id.clone()),
            DesignPaintPayload::Video(video) => (DesignMediaKind::Video, video.source.id.clone()),
            _ => return,
        };
        cx.emit(PaintPickerEvent::MediaSourceDropRequested {
            target,
            expected_source_id,
            expected_media_kind,
            file,
        });
    }

    pub(super) fn request_media_crop_action(
        &self,
        action: DesignMediaCropAction,
        cx: &mut Context<Self>,
    ) {
        if self.editing_disabled()
            || !self.paint.as_ref().is_some_and(|paint| {
                matches!(
                    &paint.payload,
                    DesignPaintPayload::Image(DesignImagePaint {
                        placement: DesignMediaPaintPlacement::Crop { .. },
                        ..
                    }) | DesignPaintPayload::Video(DesignVideoPaint {
                        placement: DesignMediaPaintPlacement::Crop { .. },
                        ..
                    })
                )
            })
        {
            return;
        }
        let valid = match &action {
            DesignMediaCropAction::Preview {
                transform, zoom, ..
            }
            | DesignMediaCropAction::Commit {
                transform, zoom, ..
            } => media_transform_is_finite(*transform) && zoom.is_finite() && *zoom > 0.,
            DesignMediaCropAction::Begin
            | DesignMediaCropAction::Cancel
            | DesignMediaCropAction::ResizeToFit => true,
        };
        if !valid {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        cx.emit(PaintPickerEvent::MediaCropActionRequested { target, action });
    }

    pub(super) fn current_video_preview(&self) -> Option<&DesignVideoPreviewState> {
        self.current_media_view()?.video_preview.as_ref()
    }

    pub(super) fn request_video_preview_action(
        &self,
        action: DesignVideoPreviewAction,
        cx: &mut Context<Self>,
    ) {
        if !matches!(
            self.paint.as_ref().map(|paint| &paint.payload),
            Some(DesignPaintPayload::Video(_))
        ) || !matches!(
            self.current_video_preview().map(|preview| &preview.status),
            Some(DesignVideoPreviewStatus::Ready)
        ) {
            return;
        }
        let valid = match action {
            DesignVideoPreviewAction::Play | DesignVideoPreviewAction::Pause => true,
            DesignVideoPreviewAction::Seek { seconds }
            | DesignVideoPreviewAction::Scrub { seconds, .. } => {
                seconds.is_finite() && seconds >= 0.
            }
        };
        if !valid {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        cx.emit(PaintPickerEvent::VideoPreviewActionRequested { target, action });
    }

    pub(super) fn request_video_scrub(&self, seconds: f32, cx: &mut Context<Self>) {
        for phase in [
            DesignPanelEditPhase::Begin,
            DesignPanelEditPhase::Preview,
            DesignPanelEditPhase::Commit,
        ] {
            self.request_video_preview_action(
                DesignVideoPreviewAction::Scrub { seconds, phase },
                cx,
            );
        }
    }

    pub(super) fn render_source_state(
        &self,
        paint: &DesignPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (source_name, icon) = match &paint.payload {
            DesignPaintPayload::Pattern(pattern) => (
                self.media_view_data
                    .pattern_source(&pattern.source_node_id)
                    .map_or_else(
                        || {
                            if pattern.source_node_id.is_empty() {
                                "Choose a layer".into()
                            } else {
                                pattern.source_node_id.clone()
                            }
                        },
                        |source| source.name.clone(),
                    ),
                IconName::LayoutDashboard,
            ),
            DesignPaintPayload::Image(image) => {
                (image.source.name.clone(), IconName::GalleryVerticalEnd)
            }
            DesignPaintPayload::Video(video) => (video.source.name.clone(), IconName::File),
            DesignPaintPayload::Shader(shader) => {
                return self.render_shader_state(shader, cx);
            }
            DesignPaintPayload::Unsupported(opaque) => {
                return v_flex()
                    .w_full()
                    .p(px(tokens::Space::MD))
                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::Panel)
                            .font_semibold()
                            .child(opaque.type_name.clone()),
                    )
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::Panel)
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child("This payload is preserved losslessly and is read-only here."),
                    )
                    .into_any_element();
            }
            DesignPaintPayload::Solid(_) | DesignPaintPayload::Gradient(_) => {
                return div().into_any_element();
            }
        };

        let pattern_source = matches!(&paint.payload, DesignPaintPayload::Pattern(_));
        let pattern_sources = if pattern_source {
            self.media_view_data
                .pattern_sources
                .iter()
                .filter(|source| {
                    self.target
                        .as_ref()
                        .is_none_or(|target| source.id != target.node_id)
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        let no_pattern_sources = pattern_sources.is_empty();
        let media_source = matches!(
            &paint.payload,
            DesignPaintPayload::Image(_) | DesignPaintPayload::Video(_)
        );
        let media_capabilities = self.current_media_capabilities();
        let media_drop_enabled =
            media_source && !self.base_editing_disabled() && media_capabilities.can_upload_source;
        let media_drop_background = crate::atoms::SemanticColor::BackgroundSelected
            .resolve(cx)
            .opacity(0.18);
        let media_drop_border = crate::atoms::SemanticColor::BackgroundSelected.resolve(cx);
        let source_card = h_flex()
            .w_full()
            .min_h(px(tokens::RowHeight::LIST))
            .px(px(tokens::Space::SM))
            .py(px(tokens::Space::XS))
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .rounded(px(tokens::Radius::CONTROL))
            .border_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
            .bg(crate::atoms::SemanticColor::BackgroundPanelField.resolve(cx))
            .when(pattern_source, |source| {
                source.child(
                    div()
                        .typography(crate::atoms::TypographyToken::PanelCaption)
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child("Source"),
                )
            })
            .when(!pattern_source, |source| {
                source.child(
                    Icon::new(icon)
                        .small()
                        .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx)),
                )
            })
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .child(source_name),
            )
            .when(pattern_source, |source| {
                source.child(self.render_pattern_source_selector(pattern_sources, cx))
            })
            .when(media_drop_enabled, |source| {
                source
                    .can_drop(move |candidate, _, _| {
                        candidate
                            .downcast_ref::<ExternalPaths>()
                            .and_then(|paths| {
                                media_drop_from_paths(paths.paths(), media_capabilities)
                            })
                            .is_some()
                    })
                    .drag_over::<ExternalPaths>(move |style, paths, _, _| {
                        if media_drop_from_paths(paths.paths(), media_capabilities).is_some() {
                            style
                                .bg(media_drop_background)
                                .border_color(media_drop_border)
                        } else {
                            style
                        }
                    })
                    .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                        this.request_media_source_drop(paths.paths(), cx);
                    }))
            });
        let source_selected = match &paint.payload {
            DesignPaintPayload::Image(image) => !image.source.id.is_empty(),
            DesignPaintPayload::Video(video) => !video.source.id.is_empty(),
            _ => true,
        };
        let mut content = v_flex()
            .w_full()
            .gap(px(tokens::Space::MD))
            .when(source_selected, |content| content.child(source_card));
        if pattern_source && no_pattern_sources {
            content = content.child(
                div()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("No eligible layers on this page"),
            );
        }
        if media_source {
            match self.media_tab {
                MediaPickerTab::Source => {
                    return content
                        .child(self.render_media_source_actions(paint, cx))
                        .into_any_element();
                }
                MediaPickerTab::Assets => {
                    return content
                        .child(self.render_media_assets(paint, cx))
                        .into_any_element();
                }
                MediaPickerTab::Adjust => {}
            }
        }

        match &paint.payload {
            DesignPaintPayload::Pattern(pattern) => {
                content = content.child(self.render_pattern_settings(pattern, cx));
            }
            DesignPaintPayload::Image(image) => {
                content = content.child(self.render_media_settings(
                    MediaSettingsView {
                        placement: image.placement,
                        filters: image.filters,
                        crop_tool: self.current_media_view().map_or_else(
                            || DesignMediaCropToolState {
                                transform: image.placement.crop_transform().unwrap_or_default(),
                                ..DesignMediaCropToolState::default()
                            },
                            |view| view.crop_tool,
                        ),
                    },
                    cx,
                ));
            }
            DesignPaintPayload::Video(video) => {
                content = content
                    .child(self.render_media_settings(
                        MediaSettingsView {
                            placement: video.placement,
                            filters: video.filters,
                            crop_tool: self.current_media_view().map_or_else(
                                || DesignMediaCropToolState {
                                    transform: video.placement.crop_transform().unwrap_or_default(),
                                    ..DesignMediaCropToolState::default()
                                },
                                |view| view.crop_tool,
                            ),
                        },
                        cx,
                    ))
                    .child(self.render_video_preview(cx));
            }
            _ => {}
        }

        content.into_any_element()
    }

    pub(super) fn render_media_source_actions(
        &self,
        paint: &DesignPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let media_label = if paint.paint_type() == DesignPaintType::Video {
            "video"
        } else {
            "image"
        };
        let caps = self.current_media_capabilities();
        let drop_enabled = !self.base_editing_disabled() && caps.can_upload_source;
        let drop_border = crate::atoms::SemanticColor::BackgroundSelected.resolve(cx);
        let upload_panel = v_flex()
            .w_full()
            .items_center()
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .p(px(tokens::Space::LG))
            .rounded(px(tokens::Radius::CONTROL))
            .border_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
            .bg(crate::atoms::SemanticColor::BackgroundPanelField.resolve(cx))
            .child(render_lucide_icon(
                LucideIcon::Upload,
                crate::atoms::SemanticColor::TextTertiary.resolve(cx),
                tokens::ControlSize::SWATCH,
            ))
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::PanelStrong)
                    .child(format!("Upload {media_label}")),
            )
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Drag a file here"),
            )
            .child(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-media-source-upload",
                    self.id
                )))
                .label("Choose file…")
                .debug_selector(|| "media-source-upload".to_owned())
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .outline()
                .disabled(self.media_source_action_disabled(DesignMediaSourceAction::Upload))
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.request_media_source_action(DesignMediaSourceAction::Upload, cx);
                })),
            )
            .when(drop_enabled, |panel| {
                panel
                    .can_drop(move |candidate, _, _| {
                        candidate
                            .downcast_ref::<ExternalPaths>()
                            .and_then(|paths| media_drop_from_paths(paths.paths(), caps))
                            .is_some()
                    })
                    .drag_over::<ExternalPaths>(move |style, paths, _, _| {
                        if media_drop_from_paths(paths.paths(), caps).is_some() {
                            style.border_color(drop_border)
                        } else {
                            style
                        }
                    })
                    .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                        this.request_media_source_drop(paths.paths(), cx);
                    }))
            });
        let mut actions = v_flex()
            .w_full()
            .gap(px(tokens::Space::SM))
            .child(upload_panel)
            .child(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-media-browse-assets",
                    self.id
                )))
                .label("Browse assets")
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .w_full()
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.media_tab = MediaPickerTab::Assets;
                    cx.notify();
                })),
            );
        if paint.paint_type() == DesignPaintType::Image {
            let mut secondary = h_flex().w_full().gap(px(tokens::Space::SM));
            for (action, label, icon) in [
                (
                    DesignMediaSourceAction::MakeImage,
                    "Generate",
                    LucideIcon::Sparkles,
                ),
                (
                    DesignMediaSourceAction::EditImage,
                    "Edit image",
                    LucideIcon::Pencil,
                ),
            ] {
                secondary = secondary.child(
                    crate::atoms::ui_button(SharedString::from(format!(
                        "{}-media-source-{}",
                        self.id,
                        action.slug()
                    )))
                    .label(label)
                    .debug_selector(move || format!("media-source-{}", action.slug()))
                    .tooltip(action.label())
                    .child(render_lucide_icon(
                        icon,
                        crate::atoms::SemanticColor::TextTertiary.resolve(cx),
                        tokens::IconSize::SM,
                    ))
                    .xsmall()
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .compact()
                    .ghost()
                    .flex_1()
                    .min_w_0()
                    .disabled(self.media_source_action_disabled(action))
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.request_media_source_action(action, cx);
                    })),
                );
            }
            actions = actions.child(secondary);
        }
        actions.into_any_element()
    }

    pub(super) fn render_media_settings(
        &self,
        settings: MediaSettingsView,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let disabled = self.editing_disabled();
        let current_mode = settings.placement.mode();
        let rotation = settings.placement.rotation().unwrap_or_default();
        let tile_scale = settings.placement.tile_scaling_factor().unwrap_or(1.);
        let filters = settings.filters;
        let crop_tool = settings.crop_tool;
        let mut modes = h_flex().w_full().gap(px(tokens::Space::XS));
        for mode in DesignMediaPaintScaleMode::ALL {
            modes = modes.child(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-media-mode-{}",
                    self.id,
                    mode.label().to_lowercase()
                )))
                .label(mode.label())
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .flex_1()
                .selected(current_mode == mode)
                .disabled(disabled)
                .on_activate(cx.listener(move |this, _, _, cx| {
                    let _ = this.emit_edit(
                        DesignPaintEdit {
                            property: DesignPaintProperty::MediaScaleMode,
                            value: DesignPaintValue::MediaScaleMode(mode),
                        },
                        DesignPanelEditPhase::Commit,
                        cx,
                    );
                })),
            );
        }
        v_flex()
            .w_full()
            .gap(px(tokens::InspectorGeometry::GROUP_GAP))
            .child(modes)
            .child(
                h_flex()
                    .w_full()
                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-rotation",
                            self.id
                        )))
                        .label(format!("Rotation {}°", rotation.degrees()))
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .disabled(disabled || current_mode == DesignMediaPaintScaleMode::Crop)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                let _ = this.emit_edit(
                                    DesignPaintEdit {
                                        property: DesignPaintProperty::MediaQuarterTurn,
                                        value: DesignPaintValue::MediaQuarterTurn(
                                            rotation.rotated_clockwise(),
                                        ),
                                    },
                                    DesignPanelEditPhase::Commit,
                                    cx,
                                );
                            },
                        )),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-tile-scale",
                            self.id
                        )))
                        .label(format!("Tile {}%", format_decimal(tile_scale * 100.)))
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .disabled(disabled || current_mode != DesignMediaPaintScaleMode::Tile)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                let _ = this.emit_edit(
                                    DesignPaintEdit {
                                        property: DesignPaintProperty::MediaTileScalingFactor,
                                        value: DesignPaintValue::Number(tile_scale + 0.1),
                                    },
                                    DesignPanelEditPhase::Commit,
                                    cx,
                                );
                            },
                        )),
                    ),
            )
            .when(
                current_mode == DesignMediaPaintScaleMode::Crop,
                |settings| settings.child(self.render_crop_tool(crop_tool, disabled, cx)),
            )
            .child(self.render_image_filter_rows(filters, disabled, cx))
            .into_any_element()
    }

    pub(super) fn render_crop_tool(
        &self,
        crop_tool: DesignMediaCropToolState,
        disabled: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !crop_tool.active {
            return crate::atoms::ui_button(SharedString::from(format!("{}-media-crop", self.id)))
                .label("Crop image")
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .outline()
                .w_full()
                .disabled(disabled)
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.request_media_crop_action(DesignMediaCropAction::Begin, cx);
                }))
                .into_any_element();
        }

        let mut nudged_transform = crop_tool.transform;
        nudged_transform.tx += 0.05;
        let next_zoom = (crop_tool.zoom + 0.1).min(64.);
        let next_aspect_ratio = crop_tool.aspect_ratio.next();
        v_flex()
            .w_full()
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .child(
                h_flex()
                    .w_full()
                    .gap(px(tokens::Space::XS))
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-crop-nudge",
                            self.id
                        )))
                        .label("Nudge crop")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .flex_1()
                        .disabled(disabled)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                this.request_media_crop_action(
                                    DesignMediaCropAction::Preview {
                                        transform: nudged_transform,
                                        zoom: crop_tool.zoom,
                                        aspect_ratio: crop_tool.aspect_ratio,
                                    },
                                    cx,
                                );
                            },
                        )),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-crop-zoom",
                            self.id
                        )))
                        .label(format!("Zoom {}×", format_decimal(crop_tool.zoom)))
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .flex_1()
                        .disabled(disabled)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                this.request_media_crop_action(
                                    DesignMediaCropAction::Preview {
                                        transform: crop_tool.transform,
                                        zoom: next_zoom,
                                        aspect_ratio: crop_tool.aspect_ratio,
                                    },
                                    cx,
                                );
                            },
                        )),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap(px(tokens::Space::XS))
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-crop-aspect",
                            self.id
                        )))
                        .label(format!("Ratio {}", crop_tool.aspect_ratio.label()))
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .flex_1()
                        .disabled(disabled)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                this.request_media_crop_action(
                                    DesignMediaCropAction::Preview {
                                        transform: crop_tool.transform,
                                        zoom: crop_tool.zoom,
                                        aspect_ratio: next_aspect_ratio,
                                    },
                                    cx,
                                );
                            },
                        )),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "{}-media-crop-rotate-row",
                                self.id
                            )))
                            .debug_selector({
                                let selector = format!("{}-media-crop-rotate", self.id);
                                move || selector.clone()
                            })
                            .flex_1()
                            .child(
                                crate::atoms::ui_button(SharedString::from(format!(
                                    "{}-media-crop-rotate",
                                    self.id
                                )))
                                .label("Rotate 15°")
                                .xsmall()
                                .typography(crate::atoms::TypographyToken::Panel)
                                .compact()
                                .outline()
                                .w_full()
                                .disabled(
                                    disabled || !self.current_media_capabilities().can_rotate_crop,
                                )
                                .on_activate(cx.listener(
                                    move |this, _, _, cx| {
                                        this.request_media_crop_action(
                                            rotated_crop_preview(crop_tool),
                                            cx,
                                        );
                                    },
                                )),
                            ),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-crop-resize-to-fit",
                            self.id
                        )))
                        .label("Resize to fit")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .flex_1()
                        .disabled(disabled)
                        .on_activate(cx.listener(|this, _, _, cx| {
                            this.request_media_crop_action(DesignMediaCropAction::ResizeToFit, cx);
                        })),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap(px(tokens::Space::XS))
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-crop-cancel",
                            self.id
                        )))
                        .label("Cancel")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .ghost()
                        .flex_1()
                        .disabled(disabled)
                        .on_activate(cx.listener(|this, _, _, cx| {
                            this.request_media_crop_action(DesignMediaCropAction::Cancel, cx);
                        })),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-crop-apply",
                            self.id
                        )))
                        .label("Apply")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .primary()
                        .flex_1()
                        .disabled(disabled)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                this.request_media_crop_action(
                                    DesignMediaCropAction::Commit {
                                        transform: crop_tool.transform,
                                        zoom: crop_tool.zoom,
                                        aspect_ratio: crop_tool.aspect_ratio,
                                    },
                                    cx,
                                );
                            },
                        )),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn render_image_filter_rows(
        &self,
        filters: DesignImageFilters,
        disabled: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut rows = v_flex()
            .w_full()
            .gap(px(tokens::InspectorGeometry::ROW_GAP));
        for filter in DesignImageFilter::ALL {
            let value = filters.value(filter);
            let slug = filter.label().to_ascii_lowercase();
            rows = rows.child(
                h_flex()
                    .w_full()
                    .gap(px(tokens::Space::XS))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .typography(crate::atoms::TypographyToken::Panel)
                            .child(filter.label()),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-filter-{slug}-minus",
                            self.id
                        )))
                        .label("−")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .disabled(disabled || value <= -1.)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                let _ = this.emit_edit(
                                    DesignPaintEdit {
                                        property: DesignPaintProperty::MediaFilter(filter),
                                        value: DesignPaintValue::Number(value - 0.1),
                                    },
                                    DesignPanelEditPhase::Commit,
                                    cx,
                                );
                            },
                        )),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-filter-{slug}-reset",
                            self.id
                        )))
                        .label(format!("{:+.0}", value * 100.))
                        .tooltip("Reset")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .ghost()
                        .w(px(46.))
                        .disabled(disabled)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                let _ = this.emit_edit(
                                    DesignPaintEdit {
                                        property: DesignPaintProperty::MediaFilter(filter),
                                        value: DesignPaintValue::Number(0.),
                                    },
                                    DesignPanelEditPhase::Commit,
                                    cx,
                                );
                            },
                        )),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-media-filter-{slug}-plus",
                            self.id
                        )))
                        .label("+")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .disabled(disabled || value >= 1.)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                let _ = this.emit_edit(
                                    DesignPaintEdit {
                                        property: DesignPaintProperty::MediaFilter(filter),
                                        value: DesignPaintValue::Number(value + 0.1),
                                    },
                                    DesignPanelEditPhase::Commit,
                                    cx,
                                );
                            },
                        )),
                    ),
            );
        }
        rows.into_any_element()
    }

    pub(super) fn render_video_preview(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(preview) = self.current_video_preview() else {
            return div()
                .typography(crate::atoms::TypographyToken::Panel)
                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child("Preview unavailable")
                .into_any_element();
        };
        match &preview.status {
            DesignVideoPreviewStatus::Idle => div()
                .typography(crate::atoms::TypographyToken::Panel)
                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child("Preview idle")
                .into_any_element(),
            DesignVideoPreviewStatus::Loading => div()
                .typography(crate::atoms::TypographyToken::Panel)
                .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child("Loading preview…")
                .into_any_element(),
            DesignVideoPreviewStatus::Error { message } => div()
                .typography(crate::atoms::TypographyToken::Panel)
                .text_color(crate::atoms::SemanticColor::TextDanger.resolve(cx))
                .child(format!("Preview error: {message}"))
                .into_any_element(),
            DesignVideoPreviewStatus::Ready => {
                let duration = preview.duration_seconds.max(0.);
                let current = preview.current_seconds.clamp(0., duration);
                let seek_back = (current - 1.).max(0.);
                let seek_forward = (current + 1.).min(duration);
                let scrub_forward = (current + duration * 0.1).min(duration);
                v_flex()
                    .w_full()
                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .child(
                                div()
                                    .typography(crate::atoms::TypographyToken::Panel)
                                    .font_semibold()
                                    .child("Video preview"),
                            )
                            .child(
                                div()
                                    .typography(crate::atoms::TypographyToken::Panel)
                                    .child(format!(
                                        "{} / {}s",
                                        format_decimal(current),
                                        format_decimal(duration)
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap(px(tokens::Space::XS))
                            .child(
                                crate::atoms::ui_button(SharedString::from(format!(
                                    "{}-video-preview-play",
                                    self.id
                                )))
                                .label(if preview.playing { "Pause" } else { "Play" })
                                .xsmall()
                                .typography(crate::atoms::TypographyToken::Panel)
                                .compact()
                                .primary()
                                .disabled(duration <= 0.)
                                .on_activate({
                                    let playing = preview.playing;
                                    cx.listener(move |this, _, _, cx| {
                                        this.request_video_preview_action(
                                            if playing {
                                                DesignVideoPreviewAction::Pause
                                            } else {
                                                DesignVideoPreviewAction::Play
                                            },
                                            cx,
                                        );
                                    })
                                }),
                            )
                            .child(
                                crate::atoms::ui_button(SharedString::from(format!(
                                    "{}-video-preview-back",
                                    self.id
                                )))
                                .label("−1s")
                                .xsmall()
                                .typography(crate::atoms::TypographyToken::Panel)
                                .compact()
                                .outline()
                                .disabled(current <= 0.)
                                .on_activate(cx.listener(
                                    move |this, _, _, cx| {
                                        this.request_video_preview_action(
                                            DesignVideoPreviewAction::Seek { seconds: seek_back },
                                            cx,
                                        );
                                    },
                                )),
                            )
                            .child(
                                crate::atoms::ui_button(SharedString::from(format!(
                                    "{}-video-preview-forward",
                                    self.id
                                )))
                                .label("+1s")
                                .xsmall()
                                .typography(crate::atoms::TypographyToken::Panel)
                                .compact()
                                .outline()
                                .disabled(current >= duration)
                                .on_activate(cx.listener(
                                    move |this, _, _, cx| {
                                        this.request_video_preview_action(
                                            DesignVideoPreviewAction::Seek {
                                                seconds: seek_forward,
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            ),
                    )
                    .child(
                        crate::atoms::ui_button(SharedString::from(format!(
                            "{}-video-preview-scrub",
                            self.id
                        )))
                        .label("Scrub +10%")
                        .tooltip("Simulate a begin/preview/commit scrub")
                        .xsmall()
                        .typography(crate::atoms::TypographyToken::Panel)
                        .compact()
                        .outline()
                        .disabled(duration <= 0. || current >= duration)
                        .on_activate(cx.listener(
                            move |this, _, _, cx| {
                                this.request_video_scrub(scrub_forward, cx);
                            },
                        )),
                    )
                    .into_any_element()
            }
        }
    }
}
