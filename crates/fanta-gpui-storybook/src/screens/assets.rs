//! The Assets story: mock host state, fixtures, reducer, and knobs.

use crate::*;

use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AssetsNamedState {
    Default,
    Empty,
}

impl AssetsNamedState {
    pub(crate) const ALL: [Self; 2] = [Self::Default, Self::Empty];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Empty => "Empty",
        }
    }
}

pub(crate) fn seed_assets_view_data(use_reference_icon_assets: bool) -> AssetsViewData {
    let view_data = AssetsViewData::new([
        AssetsLibrary::new(
            "current-file",
            "Created in this file",
            1435,
            AssetsLibraryKind::CurrentFile,
            0xf5f5f5,
        )
        .thumbnail_asset("storybook/assets-panel/current-file.png"),
        AssetsLibrary::new(
            "ios-18",
            "iOS 18 and iPadOS 18",
            156,
            AssetsLibraryKind::UiKit,
            0x23b7cf,
        )
        .thumbnail_asset("storybook/assets-panel/ios-18.png"),
        AssetsLibrary::new(
            "ios-26",
            "iOS and iPadOS 26",
            175,
            AssetsLibraryKind::UiKit,
            0x1659a8,
        )
        .thumbnail_asset("storybook/assets-panel/ios-26.png"),
        AssetsLibrary::new(
            "macos-26",
            "macOS 26",
            71,
            AssetsLibraryKind::UiKit,
            0x1f8ddb,
        )
        .thumbnail_asset("storybook/assets-panel/macos-26.png"),
        AssetsLibrary::new(
            "material-3",
            "Material 3 Design Kit",
            357,
            AssetsLibraryKind::UiKit,
            0xc36ff2,
        )
        .thumbnail_asset("storybook/assets-panel/material-3.png"),
        AssetsLibrary::new(
            "simple",
            "Simple Design System",
            1844,
            AssetsLibraryKind::UiKit,
            0x111111,
        )
        .thumbnail_asset("storybook/assets-panel/simple.png"),
        AssetsLibrary::new(
            "visionos-26",
            "visionOS 26",
            67,
            AssetsLibraryKind::UiKit,
            0x598cff,
        )
        .thumbnail_asset("storybook/assets-panel/visionos-26.png"),
        AssetsLibrary::new(
            "watchos-26",
            "watchOS 26",
            81,
            AssetsLibraryKind::UiKit,
            0x2578e8,
        )
        .thumbnail_asset("storybook/assets-panel/watchos-26.png"),
    ]);
    if use_reference_icon_assets {
        view_data.icon_assets(AssetsIconAssets {
            rail_file: Some("storybook/assets-panel/icons/rail-file.png".into()),
            rail_agents: Some("storybook/assets-panel/icons/rail-agents.png".into()),
            rail_assets: Some("storybook/assets-panel/icons/rail-assets.png".into()),
            rail_tools: Some("storybook/assets-panel/icons/rail-tools.png".into()),
            rail_variables: Some("storybook/assets-panel/icons/rail-variables.png".into()),
            header_library: Some("storybook/assets-panel/icons/header-library.png".into()),
            filters: Some("storybook/assets-panel/icons/filters.png".into()),
            ui_kit_badge: Some("storybook/assets-panel/icons/ui-kit-badge.png".into()),
        })
    } else {
        view_data
    }
}

pub(crate) fn empty_assets_view_data() -> AssetsViewData {
    AssetsViewData::new(Vec::<AssetsLibrary>::new())
}

pub(crate) struct AssetsScreen {
    pub(crate) panel: Entity<AssetsPanel>,
    pub(crate) view_data: AssetsViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: AssetsNamedState,
    reference_icon_assets: bool,
}

impl AssetsScreen {
    pub(crate) fn new(
        use_reference_icon_assets: bool,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) -> Self {
        let view_data = seed_assets_view_data(use_reference_icon_assets);
        let panel =
            cx.new(|cx| AssetsPanel::new("storybook-assets", view_data.clone(), window, cx));
        Self {
            panel,
            view_data,
            last_action: "Ready — search, filter, or select a component library".into(),
            named_state: AssetsNamedState::Default,
            reference_icon_assets: use_reference_icon_assets,
        }
    }

    fn fixture(state: AssetsNamedState, use_reference_icon_assets: bool) -> AssetsViewData {
        match state {
            AssetsNamedState::Default => seed_assets_view_data(use_reference_icon_assets),
            AssetsNamedState::Empty => empty_assets_view_data(),
        }
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: AssetsNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        self.view_data = Self::fixture(state, self.reference_icon_assets);
        let view_data = self.view_data.clone();
        self.panel
            .update(cx, |panel, cx| panel.set_view_data(view_data, cx));
        self.last_action = format!("Story applied the {} Assets state", state.label()).into();
        cx.notify();
    }

    pub(crate) fn handle_action(
        &mut self,
        panel: Entity<AssetsPanel>,
        action: &AssetsPanelAction,
        cx: &mut Context<Storybook>,
    ) {
        self.last_action = match action {
            AssetsPanelAction::SearchQueryChanged { query } if query.is_empty() => {
                "Cleared the library search".into()
            }
            AssetsPanelAction::SearchQueryChanged { query } => {
                format!("Filtered libraries by “{query}”").into()
            }
            AssetsPanelAction::LibrarySelected { library_id } => {
                let name = self
                    .view_data
                    .libraries
                    .iter()
                    .find(|library| library.id == *library_id)
                    .map_or(library_id.as_ref(), |library| library.name.as_ref());
                format!("Selected library {name}").into()
            }
            AssetsPanelAction::FiltersRequested => "Host opened Assets filters".into(),
            AssetsPanelAction::AddLibrariesRequested => "Host opened the library browser".into(),
            AssetsPanelAction::RailItemSelected { item } => {
                self.view_data.active_rail_item = *item;
                format!("Host selected the {item:?} editor rail item").into()
            }
        };
        panel.update(cx, |panel, cx| {
            panel.set_view_data(self.view_data.clone(), cx);
        });
        cx.notify();
    }
}

impl Storybook {
    pub(crate) fn render_assets_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-assets",
            self.assets_screen.panel.clone().into_any_element(),
            self.assets_screen.last_action.clone(),
            cx,
        )
    }

    pub(crate) fn render_assets_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "assets-story-knobs",
            vec![knobs::enum_knob_row(
                "assets-knob-state",
                "NAMED STATE",
                AssetsNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.assets_screen.named_state,
                |this, state, _, cx| this.assets_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_named_states_return_reseedable_fixtures() {
        assert!(
            !AssetsScreen::fixture(AssetsNamedState::Default, false)
                .libraries
                .is_empty()
        );
        assert!(
            AssetsScreen::fixture(AssetsNamedState::Empty, false)
                .libraries
                .is_empty()
        );
    }
}
