//! A collapsible project sidebar composing Pages, Layers, and project Assets.

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, v_flex};

use crate::{
    assets::AssetsPanel,
    atoms::{
        CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, icon_button, render_fanta_logo,
        render_lucide_icon, sidebar_style, tokens,
    },
    layers::{LayersPanel, LayersPanelAction},
    pages::PagesPanel,
};

pub const FILE_INSPECTOR_MIN_WIDTH: f32 = 240.;
pub const FILE_INSPECTOR_MIN_HEIGHT: f32 = 400.;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileInspectorAction {
    CollapsedChanged { collapsed: bool },
}

/// Owns sidebar presentation only. Hosts supply the project name and continue
/// handling document intents on the dedicated child panel entities.
pub struct FileInspectorSidebar {
    id: SharedString,
    focus_handle: FocusHandle,
    pages: Entity<PagesPanel>,
    layers: Entity<LayersPanel>,
    _layer_expansion_subscription: Subscription,
    assets: Option<Entity<AssetsPanel>>,
    assets_expanded: bool,
    project_name: SharedString,
    collapsed: bool,
}

impl EventEmitter<FileInspectorAction> for FileInspectorSidebar {}

impl FileInspectorSidebar {
    /// Child panels belong to this sidebar and are rendered without outer borders.
    pub fn new(
        id: impl Into<SharedString>,
        pages: Entity<PagesPanel>,
        layers: Entity<LayersPanel>,
        cx: &mut Context<Self>,
    ) -> Self {
        pages.update(cx, |panel, cx| panel.set_bordered(false, cx));
        layers.update(cx, |panel, cx| panel.set_bordered(false, cx));
        let layer_expansion_subscription = cx.subscribe(&layers, |_, _, event, cx| {
            if matches!(event, LayersPanelAction::PanelExpansionChanged { .. }) {
                cx.notify();
            }
        });
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            pages,
            layers,
            _layer_expansion_subscription: layer_expansion_subscription,
            assets: None,
            assets_expanded: false,
            project_name: "Untitled".into(),
            collapsed: false,
        }
    }

    pub fn set_project_name(&mut self, name: impl Into<SharedString>, cx: &mut Context<Self>) {
        let name = name.into();
        if self.project_name != name {
            self.project_name = name;
            cx.notify();
        }
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.collapsed != collapsed {
            self.collapsed = collapsed;
            cx.notify();
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Adds the host-controlled asset browser without changing the existing
    /// Pages/Layers constructor contract. The child emits its own intents.
    pub fn set_assets_panel(
        &mut self,
        assets: Option<Entity<AssetsPanel>>,
        cx: &mut Context<Self>,
    ) {
        let previously_available = self.assets.is_some();
        self.assets = assets;
        if self.assets.is_none() {
            self.assets_expanded = false;
        } else if !previously_available {
            self.assets_expanded = true;
        }
        cx.notify();
    }

    /// Reveals Assets after a gallery import or a user opening the section.
    pub fn expand_assets(&mut self, cx: &mut Context<Self>) {
        if self.assets.is_some() && !self.assets_expanded {
            self.assets_expanded = true;
            cx.notify();
        }
    }

    pub fn collapse_assets(&mut self, cx: &mut Context<Self>) {
        if self.assets_expanded {
            self.assets_expanded = false;
            cx.notify();
        }
    }

    pub fn assets_expanded(&self) -> bool {
        self.assets_expanded && self.assets.is_some()
    }

    fn header(&self, cx: &mut Context<Self>) -> AnyElement {
        let style = sidebar_style(cx);
        h_flex()
            .debug_selector(|| "file-inspector-project-header".to_owned())
            .w_full()
            .min_w_0()
            .flex_none()
            .h(px(tokens::RowHeight::SECTION_HEADER))
            .px_2()
            .gap_2()
            .items_center()
            .text_size(crate::atoms::sidebar_text_size())
            .text_color(style.text)
            .when(self.collapsed, |header| {
                header.child(
                    div()
                        .flex_none()
                        .child(render_fanta_logo(style.icon, tokens::IconSize::MD)),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(self.project_name.clone()),
            )
            .child(
                icon_button(
                    SharedString::from(format!("{}-toggle", self.id)),
                    px(tokens::RowHeight::FIELD),
                    px(tokens::Space::XS),
                    cx,
                )
                .debug_selector(|| "file-inspector-toggle".to_owned())
                .on_activate(cx.listener(|this, _, _, cx| {
                    let collapsed = !this.collapsed;
                    this.set_collapsed(collapsed, cx);
                    cx.emit(FileInspectorAction::CollapsedChanged { collapsed });
                }))
                .child(render_lucide_icon(
                    LucideIcon::PanelLeft,
                    style.muted_icon,
                    tokens::IconSize::MD,
                )),
            )
            .into_any_element()
    }
}

impl Focusable for FileInspectorSidebar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileInspectorSidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = sidebar_style(cx);
        let zed_theme = cx.try_global::<theme::GlobalTheme>().is_some();
        let layers_expanded = self.layers.read(cx).is_expanded();
        let asset_count = self
            .assets
            .as_ref()
            .map(|panel| panel.read(cx).view_data().assets.len());
        if self.collapsed {
            return div()
                .id(self.id.clone())
                .track_focus(&self.focus_handle)
                .max_w_full()
                .min_w_0()
                .p_2()
                .child(
                    div()
                        .debug_selector(|| "file-inspector-floating-card".to_owned())
                        .w(px(FILE_INSPECTOR_MIN_WIDTH))
                        .max_w_full()
                        .min_w_0()
                        .rounded(px(tokens::Radius::MENU))
                        .border_1()
                        .border_color(style.border)
                        .shadow_md()
                        .bg(style.background)
                        .child(self.header(cx)),
                )
                .into_any_element();
        }
        v_flex()
            .id(self.id.clone())
            .debug_selector(|| "file-inspector-sidebar".to_owned())
            .track_focus(&self.focus_handle)
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .bg(style.background)
            .text_color(style.text)
            .child(self.header(cx))
            .child(if zed_theme {
                ui::Divider::horizontal().into_any_element()
            } else {
                div().w_full().h_px().bg(style.border).into_any_element()
            })
            .child(
                div()
                    .debug_selector(|| "file-inspector-pages".to_owned())
                    .w_full()
                    .flex_none()
                    .child(self.pages.clone()),
            )
            .child(if zed_theme {
                ui::Divider::horizontal().into_any_element()
            } else {
                div().w_full().h_px().bg(style.border).into_any_element()
            })
            .child(
                div()
                    .debug_selector(|| "file-inspector-layers".to_owned())
                    .w_full()
                    .when(layers_expanded, |section| section.flex_1())
                    .when(!layers_expanded, |section| section.flex_none())
                    .min_h_0()
                    .child(self.layers.clone()),
            )
            .when_some(asset_count, |sidebar, count| {
                sidebar
                    .child(if zed_theme {
                        ui::Divider::horizontal().into_any_element()
                    } else {
                        div().w_full().h_px().bg(style.border).into_any_element()
                    })
                    .child(
                        h_flex()
                            .id(SharedString::from(format!("{}-assets-section", self.id)))
                            .debug_selector(|| "file-inspector-assets-header".to_owned())
                            .key_context(CONTROL_KEY_CONTEXT)
                            .role(Role::Button)
                            .aria_label("Project assets")
                            .aria_expanded(self.assets_expanded)
                            .tab_index(0)
                            .w_full()
                            .flex_none()
                            .h(px(tokens::RowHeight::SECTION_HEADER))
                            .px_3()
                            .gap_1()
                            .items_center()
                            .cursor_pointer()
                            .hover(|header| header.bg(style.hover))
                            .focus(|header| header.bg(style.hover))
                            .when(!self.assets_expanded, |header| {
                                header.child(render_lucide_icon(
                                    LucideIcon::ChevronRight,
                                    style.muted_icon,
                                    tokens::IconSize::SM,
                                ))
                            })
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(crate::atoms::sidebar_text_size())
                                    .text_color(style.muted_text)
                                    .child("Assets"),
                            )
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(style.muted_text)
                                    .child(count.to_string()),
                            )
                            .on_activate(cx.listener(|this, _, _, cx| {
                                if this.assets_expanded {
                                    this.collapse_assets(cx);
                                } else {
                                    this.expand_assets(cx);
                                }
                            })),
                    )
            })
            .when_some(
                self.assets.clone().filter(|_| self.assets_expanded),
                |sidebar, assets| {
                    sidebar.child(
                        div()
                            .debug_selector(|| "file-inspector-assets".to_owned())
                            .w_full()
                            .flex_1()
                            .min_h_0()
                            .child(assets),
                    )
                },
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod interaction_tests;
