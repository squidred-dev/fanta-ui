//! A collapsible project sidebar composing the Pages and Layers panels.

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, v_flex};

use crate::{
    atoms::{
        ControlExt as _, LucideIcon, SemanticColor, TypographyExt as _, TypographyToken,
        icon_button, render_fanta_logo, render_lucide_icon, tokens,
    },
    layers::LayersPanel,
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
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            pages,
            layers,
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

    fn header(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .debug_selector(|| "file-inspector-project-header".to_owned())
            .w_full()
            .min_w_0()
            .flex_none()
            .p_3()
            .gap_3()
            .items_center()
            .typography(TypographyToken::BodyMedium)
            .text_color(SemanticColor::Text.resolve(cx))
            .when(self.collapsed, |header| {
                header.child(div().flex_none().child(render_fanta_logo(
                    SemanticColor::Text.resolve(cx),
                    tokens::RowHeight::FIELD,
                )))
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
                    SemanticColor::Text.resolve(cx),
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
                        .shadow_md()
                        .bg(SemanticColor::BackgroundToolbar.resolve(cx))
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
            .bg(SemanticColor::BackgroundToolbar.resolve(cx))
            .child(
                div()
                    .w_full()
                    .flex_none()
                    .border_b_1()
                    .border_color(SemanticColor::Border.resolve(cx))
                    .child(self.header(cx)),
            )
            .child(
                div()
                    .debug_selector(|| "file-inspector-pages".to_owned())
                    .w_full()
                    .flex_none()
                    .child(self.pages.clone()),
            )
            .child(
                div()
                    .debug_selector(|| "file-inspector-layers".to_owned())
                    .border_t_1()
                    .border_color(SemanticColor::Border.resolve(cx))
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .child(self.layers.clone()),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod interaction_tests;
