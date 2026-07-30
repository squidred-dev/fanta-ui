//! A composed, simulated editor shell for exercising Fanta GPUI components.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};

use crate::controls::{ActivateControl, CONTROL_KEY_CONTEXT};
use crate::{
    assets::AssetsPanel, design::DesignPanel, layers::LayersPanel, pages::PagesPanel,
    prototype::PrototypePanel, timeline::Timeline, toolbar::EditorToolbar,
    variables::VariablesPage,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PseudoEditorLeftSurface {
    Pages,
    Layers,
    #[default]
    Assets,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PseudoEditorRightSurface {
    Design,
    #[default]
    Prototype,
}

#[derive(Clone)]
pub struct PseudoEditorChildren {
    pub pages: Entity<PagesPanel>,
    pub layers: Entity<LayersPanel>,
    pub assets: Entity<AssetsPanel>,
    pub design: Entity<DesignPanel>,
    pub prototype: Entity<PrototypePanel>,
    pub timeline: Entity<Timeline>,
    pub toolbar: Entity<EditorToolbar>,
    pub variables: Entity<VariablesPage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PseudoEditorAction {
    LeftSurfaceChanged { surface: PseudoEditorLeftSurface },
    RightSurfaceChanged { surface: PseudoEditorRightSurface },
    VariablesVisibilityChanged { visible: bool },
    PresentRequested,
    ShareRequested,
}

/// Presentation-only editor shell. Domain data remains owned by each child
/// component's host adapter.
pub struct PseudoEditor {
    id: SharedString,
    focus_handle: FocusHandle,
    children: PseudoEditorChildren,
    left_surface: PseudoEditorLeftSurface,
    right_surface: PseudoEditorRightSurface,
    variables_visible: bool,
}

impl EventEmitter<PseudoEditorAction> for PseudoEditor {}

impl PseudoEditor {
    pub fn new(
        id: impl Into<SharedString>,
        children: PseudoEditorChildren,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            children,
            left_surface: PseudoEditorLeftSurface::default(),
            right_surface: PseudoEditorRightSurface::default(),
            variables_visible: false,
        }
    }

    pub fn set_left_surface(&mut self, surface: PseudoEditorLeftSurface, cx: &mut Context<Self>) {
        self.left_surface = surface;
        cx.notify();
    }

    pub fn set_right_surface(&mut self, surface: PseudoEditorRightSurface, cx: &mut Context<Self>) {
        self.right_surface = surface;
        cx.notify();
    }

    pub fn set_variables_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.variables_visible = visible;
        cx.notify();
    }

    fn small_tab<T: EventEmitter<PseudoEditorAction>>(
        &self,
        id: SharedString,
        label: &'static str,
        selected: bool,
        on_click: impl Fn(&mut T, &mut Context<T>) + 'static,
        cx: &mut Context<T>,
    ) -> AnyElement {
        let on_click = Rc::new(on_click);
        let on_action = on_click.clone();
        div()
            .id(id)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(26.))
            .px(px(8.))
            .items_center()
            .rounded(px(5.))
            .cursor_pointer()
            .text_size(px(10.))
            .when(selected, |tab| {
                tab.bg(cx.theme().tab_active)
                    .text_color(cx.theme().tab_active_foreground)
                    .font_semibold()
            })
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| style.border_1().border_color(cx.theme().selection))
            .on_action(cx.listener(move |this, _: &ActivateControl, _, cx| on_action(this, cx)))
            .on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
            .child(label)
            .into_any_element()
    }

    fn render_left(&self, cx: &mut Context<Self>) -> AnyElement {
        let panel = match self.left_surface {
            PseudoEditorLeftSurface::Pages => self.children.pages.clone().into_any_element(),
            PseudoEditorLeftSurface::Layers => self.children.layers.clone().into_any_element(),
            PseudoEditorLeftSurface::Assets => self.children.assets.clone().into_any_element(),
        };
        div()
            .w(px(337.))
            .h_full()
            .flex_none()
            .min_h(px(0.))
            .overflow_hidden()
            .border_r_1()
            .border_color(cx.theme().border)
            .child(panel)
            .into_any_element()
    }

    fn render_right(&self, cx: &mut Context<Self>) -> AnyElement {
        let panel = match self.right_surface {
            PseudoEditorRightSurface::Design => self.children.design.clone().into_any_element(),
            PseudoEditorRightSurface::Prototype => {
                self.children.prototype.clone().into_any_element()
            }
        };
        div()
            .w(px(473.))
            .h_full()
            .flex_none()
            .min_h(px(0.))
            .overflow_hidden()
            .border_l_1()
            .border_color(cx.theme().border)
            .child(panel)
            .into_any_element()
    }

    fn render_canvas(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .debug_selector(|| "pseudo-editor-canvas".to_owned())
            .relative()
            .flex_1()
            .h_full()
            .min_w(px(0.))
            .items_center()
            .justify_center()
            .overflow_hidden()
            .bg(cx.theme().muted)
            .child(
                v_flex()
                    .w(px(360.))
                    .h(px(240.))
                    .items_center()
                    .justify_center()
                    .gap(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(cx.theme().selection)
                    .bg(cx.theme().background)
                    .text_color(cx.theme().foreground)
                    .when(cx.theme().shadow, |card| card.shadow_lg())
                    .child(div().text_size(px(20.)).font_semibold().child("Fanta"))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child("A simulated editor canvas"),
                    ),
            )
            .child(self.children.toolbar.clone())
            .into_any_element()
    }
}

impl Focusable for PseudoEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PseudoEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let variables = self.children.variables.clone();
        v_flex()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .min_h(px(0.))
            .overflow_hidden()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .h(px(44.))
                    .w_full()
                    .flex_none()
                    .px(px(12.))
                    .gap(px(10.))
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(div().font_semibold().text_size(px(12.)).child("Untitled"))
                    .child(self.small_tab(
                        SharedString::from(format!("{}-left-pages", self.id)),
                        "Pages",
                        self.left_surface == PseudoEditorLeftSurface::Pages,
                        |this, cx| {
                            this.left_surface = PseudoEditorLeftSurface::Pages;
                            cx.emit(PseudoEditorAction::LeftSurfaceChanged {
                                surface: PseudoEditorLeftSurface::Pages,
                            });
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(self.small_tab(
                        SharedString::from(format!("{}-left-layers", self.id)),
                        "Layers",
                        self.left_surface == PseudoEditorLeftSurface::Layers,
                        |this, cx| {
                            this.left_surface = PseudoEditorLeftSurface::Layers;
                            cx.emit(PseudoEditorAction::LeftSurfaceChanged {
                                surface: PseudoEditorLeftSurface::Layers,
                            });
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(self.small_tab(
                        SharedString::from(format!("{}-left-assets", self.id)),
                        "Assets",
                        self.left_surface == PseudoEditorLeftSurface::Assets,
                        |this, cx| {
                            this.left_surface = PseudoEditorLeftSurface::Assets;
                            cx.emit(PseudoEditorAction::LeftSurfaceChanged {
                                surface: PseudoEditorLeftSurface::Assets,
                            });
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(div().flex_1())
                    .child(self.small_tab(
                        SharedString::from(format!("{}-right-design", self.id)),
                        "Design",
                        self.right_surface == PseudoEditorRightSurface::Design,
                        |this, cx| {
                            this.right_surface = PseudoEditorRightSurface::Design;
                            cx.emit(PseudoEditorAction::RightSurfaceChanged {
                                surface: PseudoEditorRightSurface::Design,
                            });
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(self.small_tab(
                        SharedString::from(format!("{}-right-prototype", self.id)),
                        "Prototype",
                        self.right_surface == PseudoEditorRightSurface::Prototype,
                        |this, cx| {
                            this.right_surface = PseudoEditorRightSurface::Prototype;
                            cx.emit(PseudoEditorAction::RightSurfaceChanged {
                                surface: PseudoEditorRightSurface::Prototype,
                            });
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-variables", self.id)))
                            .key_context(CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .h(px(28.))
                            .px(px(9.))
                            .items_center()
                            .rounded(px(5.))
                            .cursor_pointer()
                            .text_size(px(10.))
                            .hover(|style| style.bg(cx.theme().accent))
                            .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                                this.variables_visible = true;
                                cx.emit(PseudoEditorAction::VariablesVisibilityChanged {
                                    visible: true,
                                });
                                cx.notify();
                            }))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.variables_visible = true;
                                cx.emit(PseudoEditorAction::VariablesVisibilityChanged {
                                    visible: true,
                                });
                                cx.notify();
                            }))
                            .child("Variables"),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-present", self.id)))
                            .debug_selector(|| "pseudo-editor-present".to_owned())
                            .key_context(CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .h(px(28.))
                            .px(px(9.))
                            .items_center()
                            .rounded(px(5.))
                            .cursor_pointer()
                            .text_size(px(10.))
                            .hover(|style| style.bg(cx.theme().accent))
                            .on_action(cx.listener(|_, _: &ActivateControl, _, cx| {
                                cx.emit(PseudoEditorAction::PresentRequested);
                            }))
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(PseudoEditorAction::PresentRequested);
                            }))
                            .child("Present"),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-share", self.id)))
                            .key_context(CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .h(px(28.))
                            .px(px(12.))
                            .items_center()
                            .rounded(px(6.))
                            .bg(cx.theme().primary)
                            .text_color(cx.theme().primary_foreground)
                            .cursor_pointer()
                            .font_semibold()
                            .text_size(px(10.))
                            .on_action(cx.listener(|_, _: &ActivateControl, _, cx| {
                                cx.emit(PseudoEditorAction::ShareRequested);
                            }))
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(PseudoEditorAction::ShareRequested);
                            }))
                            .child("Share"),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_start()
                    .child(self.render_left(cx))
                    .child(self.render_canvas(cx))
                    .child(self.render_right(cx)),
            )
            .child(
                div()
                    .h(px(333.))
                    .w_full()
                    .flex_none()
                    .child(self.children.timeline.clone()),
            )
            .when(self.variables_visible, |root| {
                root.child(
                    v_flex()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top_0()
                        .bottom_0()
                        .bg(cx.theme().background)
                        .child(variables)
                        .child(
                            h_flex().absolute().right(px(82.)).top(px(9.)).child(
                                div()
                                    .id(SharedString::from(format!("{}-close-variables", self.id)))
                                    .key_context(CONTROL_KEY_CONTEXT)
                                    .tab_index(0)
                                    .size(px(30.))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(6.))
                                    .bg(cx.theme().secondary)
                                    .cursor_pointer()
                                    .hover(|style| style.bg(cx.theme().secondary_hover))
                                    .on_action(cx.listener(|this, _: &ActivateControl, _, cx| {
                                        this.variables_visible = false;
                                        cx.emit(PseudoEditorAction::VariablesVisibilityChanged {
                                            visible: false,
                                        });
                                        cx.notify();
                                    }))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.variables_visible = false;
                                        cx.emit(PseudoEditorAction::VariablesVisibilityChanged {
                                            visible: false,
                                        });
                                        cx.notify();
                                    }))
                                    .child("×"),
                            ),
                        ),
                )
            })
    }
}

#[cfg(test)]
mod composed_host_flow_tests;
#[cfg(test)]
mod interaction_tests;
