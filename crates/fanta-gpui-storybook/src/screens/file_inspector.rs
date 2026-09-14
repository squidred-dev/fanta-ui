//! The File inspector story: one sidebar composed from the existing Pages
//! and Layers mock hosts.

use crate::*;

use super::harness;

pub(crate) struct FileInspectorScreen {
    pub(crate) sidebar: Entity<FileInspectorSidebar>,
}

impl FileInspectorScreen {
    pub(crate) fn new(
        pages: Entity<PagesPanel>,
        layers: Entity<LayersPanel>,
        cx: &mut Context<Storybook>,
    ) -> Self {
        Self {
            sidebar: cx
                .new(|cx| FileInspectorSidebar::new("storybook-file-inspector", pages, layers, cx)),
        }
    }
}

impl Storybook {
    pub(crate) fn file_inspector_last_action(&self) -> SharedString {
        format!(
            "Pages: {}  ·  Layers: {}",
            self.pages_screen.last_action, self.layers_screen.last_action
        )
        .into()
    }

    pub(crate) fn render_file_inspector_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_story_shell(
            "storybook-reference-file-inspector",
            harness::ReferenceStoryCopy {
                eyebrow: "FILE INSPECTOR",
                title: "File inspector",
                description: "Pages and Layers share one continuous left sidebar: Pages keeps its compact content height while Layers fills the remaining space.",
                adapter_description: "The composition owns placement only. The Pages and Layers mock hosts still receive their original typed intents and echo updated read models into each child panel.",
            },
            self.file_inspector_last_action(),
            self.file_inspector_screen.sidebar.clone().into_any_element(),
            cx,
        )
    }
}
