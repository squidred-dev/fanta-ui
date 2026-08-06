//! The Pseudo editor story: layout shell state, reducer, and knobs.

use crate::*;

use super::knobs;

pub(crate) struct PseudoEditorScreen {
    pub(crate) editor: Entity<PseudoEditor>,
    pub(crate) left_surface: PseudoEditorLeftSurface,
    pub(crate) right_surface: PseudoEditorRightSurface,
    pub(crate) variables_visible: bool,
    pub(crate) last_action: SharedString,
}

impl PseudoEditorScreen {
    pub(crate) fn new(children: PseudoEditorChildren, cx: &mut Context<Storybook>) -> Self {
        let editor = cx.new(|cx| PseudoEditor::new("storybook-pseudo-editor", children, cx));
        Self {
            editor,
            left_surface: PseudoEditorLeftSurface::default(),
            right_surface: PseudoEditorRightSurface::default(),
            variables_visible: false,
            last_action: "Ready — switch panels, open Variables, present, or share".into(),
        }
    }

    pub(crate) fn handle_action(
        &mut self,
        editor: Entity<PseudoEditor>,
        action: &PseudoEditorAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            PseudoEditorAction::LeftSurfaceChanged { surface } => {
                self.left_surface = *surface;
                editor.update(cx, |editor, cx| editor.set_left_surface(*surface, cx));
                self.last_action = format!("Selected the {surface:?} left panel").into();
            }
            PseudoEditorAction::RightSurfaceChanged { surface } => {
                self.right_surface = *surface;
                editor.update(cx, |editor, cx| editor.set_right_surface(*surface, cx));
                self.last_action = format!("Selected the {surface:?} right panel").into();
            }
            PseudoEditorAction::VariablesVisibilityChanged { visible } => {
                self.variables_visible = *visible;
                editor.update(cx, |editor, cx| {
                    editor.set_variables_visible(*visible, cx);
                });
                self.last_action = if *visible {
                    "Opened the full Variables manager".into()
                } else {
                    "Returned from the Variables manager".into()
                };
            }
            PseudoEditorAction::PresentRequested => {
                self.last_action = "Host started prototype presentation".into();
            }
            PseudoEditorAction::ShareRequested => {
                self.last_action = "Host opened editor sharing".into();
            }
        }
        cx.notify();
    }
}

impl Storybook {
    pub(crate) fn render_pseudo_editor_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-pseudo-editor",
            self.pseudo_screen.editor.clone().into_any_element(),
            self.pseudo_screen.last_action.clone(),
            cx,
        )
    }

    pub(crate) fn render_pseudo_editor_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "pseudo-editor-story-knobs",
            vec![knobs::bool_knob_row(
                "pseudo-editor-knob-variables",
                "VARIABLES MANAGER",
                "Variables manager",
                self.pseudo_screen.variables_visible,
                |this, visible, _, cx| {
                    let editor = this.pseudo_screen.editor.clone();
                    this.pseudo_screen.handle_action(
                        editor,
                        &PseudoEditorAction::VariablesVisibilityChanged { visible },
                        cx,
                    );
                },
                cx,
            )],
            cx,
        )
    }
}
