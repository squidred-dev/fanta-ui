use std::borrow::Cow;

use fanta_gpui::prelude::{
    AssetsIconAssets, AssetsLibrary, AssetsLibraryKind, AssetsViewData, LayersPanelItem,
    PagesPanelElementKind, PagesPanelItem, PagesPanelSearchResult, VariableKind, VariableModeValue,
    VariableRow, VariablesCollection, VariablesGroup, VariablesMode, VariablesViewData,
};
use gpui::{AssetSource, SharedString};
use gpui_component_assets::Assets;

pub(super) struct StorybookAssets;

impl AssetSource for StorybookAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "storybook/assets-panel/current-file.png" => {
                Some(include_bytes!("../assets/assets-panel/current-file.png"))
            }
            "storybook/assets-panel/ios-18.png" => {
                Some(include_bytes!("../assets/assets-panel/ios-18.png"))
            }
            "storybook/assets-panel/ios-26.png" => {
                Some(include_bytes!("../assets/assets-panel/ios-26.png"))
            }
            "storybook/assets-panel/macos-26.png" => {
                Some(include_bytes!("../assets/assets-panel/macos-26.png"))
            }
            "storybook/assets-panel/material-3.png" => {
                Some(include_bytes!("../assets/assets-panel/material-3.png"))
            }
            "storybook/assets-panel/simple.png" => {
                Some(include_bytes!("../assets/assets-panel/simple.png"))
            }
            "storybook/assets-panel/visionos-26.png" => {
                Some(include_bytes!("../assets/assets-panel/visionos-26.png"))
            }
            "storybook/assets-panel/watchos-26.png" => {
                Some(include_bytes!("../assets/assets-panel/watchos-26.png"))
            }
            "storybook/assets-panel/icons/rail-file.png" => {
                Some(include_bytes!("../assets/assets-panel/icons/rail-file.png"))
            }
            "storybook/assets-panel/icons/rail-agents.png" => Some(include_bytes!(
                "../assets/assets-panel/icons/rail-agents.png"
            )),
            "storybook/assets-panel/icons/rail-assets.png" => Some(include_bytes!(
                "../assets/assets-panel/icons/rail-assets.png"
            )),
            "storybook/assets-panel/icons/rail-tools.png" => Some(include_bytes!(
                "../assets/assets-panel/icons/rail-tools.png"
            )),
            "storybook/assets-panel/icons/rail-variables.png" => Some(include_bytes!(
                "../assets/assets-panel/icons/rail-variables.png"
            )),
            "storybook/assets-panel/icons/header-library.png" => Some(include_bytes!(
                "../assets/assets-panel/icons/header-library.png"
            )),
            "storybook/assets-panel/icons/filters.png" => {
                Some(include_bytes!("../assets/assets-panel/icons/filters.png"))
            }
            "storybook/assets-panel/icons/ui-kit-badge.png" => Some(include_bytes!(
                "../assets/assets-panel/icons/ui-kit-badge.png"
            )),
            _ => return Assets.load(path),
        };
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Assets.list(path)
    }
}

#[derive(Clone)]
pub(super) struct MockElement {
    pub(super) result: PagesPanelSearchResult,
    pub(super) page_id: SharedString,
}

pub(super) fn seed_variables_view_data() -> VariablesViewData {
    VariablesViewData {
        document_name: "Untitled".into(),
        collections: vec![
            VariablesCollection::new("collection-1", "Collection 1", 1),
            VariablesCollection::new("collection-2", "Collection 2", 0),
        ],
        selected_collection_id: "collection-1".into(),
        groups: vec![VariablesGroup::new("all", "All", 1).aggregate()],
        selected_group_id: "all".into(),
        modes: vec![
            VariablesMode::new("mode-1", "Mode 1"),
            VariablesMode::new("mode-2", "Mode 2"),
        ],
        variables: vec![VariableRow::new(
            "color",
            "Color",
            "all",
            VariableKind::Color,
            [
                VariableModeValue::new("mode-1", "FFFFFF").color("FFFFFF"),
                VariableModeValue::new("mode-2", "FFFFFF").color("FFFFFF"),
            ],
        )],
    }
}

pub(super) fn seed_assets_view_data(use_reference_icon_assets: bool) -> AssetsViewData {
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

pub(super) fn seed_pages() -> Vec<PagesPanelItem> {
    vec![
        PagesPanelItem::new("page-1", "Page 1"),
        PagesPanelItem::new("page-3", "Page 3"),
        PagesPanelItem::new("page-4", "Page 4"),
        PagesPanelItem::new("page-2", "Page 2"),
        PagesPanelItem::new("page-5", "Page 5"),
    ]
}

pub(super) fn seed_elements() -> Vec<MockElement> {
    use PagesPanelElementKind::{Component, FrameGroup, Image, Instance, Shape, Text};

    [
        ("image-1", "Homepage hero", "Hero frame", Image, "page-5"),
        ("text-1", "Hello designers", "Hero frame", Text, "page-5"),
        ("frame-1", "Header", "Homepage", FrameGroup, "page-5"),
        (
            "component-1",
            "Primary button",
            "Components",
            Component,
            "page-5",
        ),
        (
            "instance-1",
            "Primary button instance",
            "Header",
            Instance,
            "page-5",
        ),
        ("text-2", "Hello again", "Footer", Text, "page-5"),
        ("shape-1", "Background glow", "Hero frame", Shape, "page-5"),
        ("image-2", "Team portrait", "About section", Image, "page-5"),
        ("text-3", "Page title", "Frame 2", Text, "page-1"),
        (
            "image-3",
            "Screenshot 2026-07-25 at 23.54.15",
            "Frame 2",
            Image,
            "page-1",
        ),
        (
            "image-4",
            "Screenshot 2026-07-25 at 23.54.31",
            "Frame 2",
            Image,
            "page-3",
        ),
        ("text-4", "Pricing heading", "Pricing", Text, "page-4"),
    ]
    .into_iter()
    .map(|(id, title, parent, kind, page_id)| MockElement {
        result: PagesPanelSearchResult::new(id, title, kind).parent(parent),
        page_id: page_id.into(),
    })
    .collect()
}

pub(super) fn seed_layers() -> Vec<LayersPanelItem> {
    use fanta_gpui::prelude::LayersPanelNodeKind as Kind;

    vec![
        LayersPanelItem::new("checkout", "Checkout", Kind::Frame).children(vec![
            LayersPanelItem::new("header", "Header", Kind::Group).children(vec![
                LayersPanelItem::new("brand-image", "Brand mark", Kind::Image),
                LayersPanelItem::new("hero-title", "Checkout title", Kind::Text),
                LayersPanelItem::new("divider", "Divider", Kind::Line),
                LayersPanelItem::new("flow-arrow", "Flow arrow", Kind::Arrow),
            ]),
            LayersPanelItem::new("controls", "Controls", Kind::ComponentSet).children(vec![
                LayersPanelItem::new("button-component", "Primary button", Kind::Component)
                    .children(vec![
                        LayersPanelItem::new("button-label", "Label", Kind::Text),
                        LayersPanelItem::new("button-icon", "Icon", Kind::Instance),
                    ]),
            ]),
            LayersPanelItem::new("hero-video", "Product preview", Kind::Video).visible(false),
            LayersPanelItem::new("background", "Background", Kind::Rectangle).locked(true),
            LayersPanelItem::new("avatar", "Avatar", Kind::Ellipse),
            LayersPanelItem::new("badge", "Badge", Kind::Polygon),
            LayersPanelItem::new("favorite", "Favorite", Kind::Star),
            LayersPanelItem::new("vector-art", "Vector artwork", Kind::BooleanOperation).children(
                vec![
                    LayersPanelItem::new("pen-path", "Pen path", Kind::Pen),
                    LayersPanelItem::new("pencil-path", "Pencil path", Kind::Pencil),
                    LayersPanelItem::new("vector-path", "Vector path", Kind::Vector),
                ],
            ),
            LayersPanelItem::new("mask", "Avatar mask", Kind::Mask),
        ]),
        LayersPanelItem::new("variants", "Variants", Kind::Section),
        LayersPanelItem::new("export-slice", "Export slice", Kind::Slice),
        LayersPanelItem::new("other", "Imported node", Kind::Other),
    ]
}
