//! The File inspector story: one sidebar composed from the existing Pages
//! and Layers mock hosts.

use crate::*;

use super::harness;

pub(crate) struct FileInspectorScreen {
    pub(crate) sidebar: Entity<FileInspectorSidebar>,
    pub(crate) pages: PagesScreen,
    pub(crate) layers: LayersScreen,
}

impl FileInspectorScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let pages = PagesScreen::new(window, cx);
        let layers = LayersScreen::new(window, cx);
        let sidebar = cx.new(|cx| {
            let mut sidebar = FileInspectorSidebar::new(
                "storybook-file-inspector",
                pages.panel.clone(),
                layers.panel.clone(),
                cx,
            );
            sidebar.set_project_name("Fanta Design", cx);
            sidebar
        });
        Self {
            sidebar,
            pages,
            layers,
        }
    }
}

impl Storybook {
    pub(crate) fn file_inspector_last_action(&self) -> SharedString {
        format!(
            "Pages: {}  ·  Layers: {}",
            self.file_inspector_screen.pages.last_action,
            self.file_inspector_screen.layers.last_action
        )
        .into()
    }

    pub(crate) fn render_file_inspector_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_story_shell(
            "storybook-reference-file-inspector",
            harness::ReferenceStoryCopy {
                eyebrow: "FILE INSPECTOR",
                title: "File inspector",
                description: "Collapse the project sidebar into a floating Fanta card, then reopen it without losing Pages or Layers state.",
                adapter_description: "The composition owns placement, project title and collapse state. The Pages and Layers mock hosts still receive their original typed intents and echo updated read models into each child panel.",
            },
            self.file_inspector_last_action(),
            self.file_inspector_screen.sidebar.clone().into_any_element(),
            cx,
        )
    }
}
