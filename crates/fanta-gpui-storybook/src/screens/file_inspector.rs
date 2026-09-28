//! The File inspector story: Pages, Layers, and a host-controlled asset library.

use crate::*;
use fanta_gpui::assets::{
    AssetKind, AssetPageTarget, AssetRow, AssetThumbnail, AssetsPanel, AssetsPanelAction,
    AssetsViewData,
};
use gpui::{Image, ImageFormat};
use std::sync::Arc;

use super::harness;

pub(crate) struct FileInspectorScreen {
    pub(crate) sidebar: Entity<FileInspectorSidebar>,
    pub(crate) pages: PagesScreen,
    pub(crate) layers: LayersScreen,
    pub(crate) assets: Entity<AssetsPanel>,
    pub(crate) last_asset_action: SharedString,
    _asset_subscription: Subscription,
}

impl FileInspectorScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let pages = PagesScreen::new(window, cx);
        let layers = LayersScreen::new(window, cx);
        let image = Arc::new(Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("../../assets/generation/product-study.png").to_vec(),
        ));
        let vector = Arc::new(Image::from_bytes(
            ImageFormat::Svg,
            include_bytes!("../../assets/generation/svg-compass.svg").to_vec(),
        ));
        let waveform = Arc::new(Image::from_bytes(
            ImageFormat::Svg,
            include_bytes!("../../assets/generation/audio-music.svg").to_vec(),
        ));
        let mut picture = AssetRow::new("asset-product", "Amber Product Study", AssetKind::Image);
        picture.thumbnail = Some(AssetThumbnail::new("product-study", image));
        picture.detail = "2048 × 2048".into();
        let mut svg = AssetRow::new("asset-compass", "North Star Monogram", AssetKind::Svg);
        svg.thumbnail = Some(AssetThumbnail::new("svg-compass", vector));
        svg.detail = "Editable vector".into();
        let mut video = AssetRow::new("asset-video", "Coastal Motion Sequence", AssetKind::Video);
        video.detail = "8 seconds · Plays in Motion or Prototype".into();
        let mut audio = AssetRow::new("asset-audio", "Soft Ambient Theme", AssetKind::Audio);
        audio.thumbnail = Some(AssetThumbnail::new("audio-music", waveform));
        audio.detail = "24 seconds · Plays in Motion or Prototype".into();
        let mut font = AssetRow::new("asset-font", "Display Typeface", AssetKind::Other);
        font.detail = "Font file".into();
        font.can_place = false;
        font.disabled_reason = Some("This asset cannot be placed on the canvas".into());
        let assets = cx.new(|cx| {
            AssetsPanel::new(
                "storybook-project-assets",
                AssetsViewData {
                    assets: vec![picture, svg, video, audio, font],
                    pages: vec![
                        AssetPageTarget::new("page-home", "Home"),
                        AssetPageTarget::new("page-campaign", "Campaign"),
                    ],
                    selected_page_id: Some("page-home".into()),
                },
                window,
                cx,
            )
        });
        let asset_subscription = cx.subscribe_in(
            &assets,
            window,
            |this, _, action: &AssetsPanelAction, window, cx| {
                match action {
                    AssetsPanelAction::TargetPageSelected { page_id } => {
                        let assets = this.file_inspector_screen.assets.clone();
                        assets.update(cx, |panel, cx| {
                            let mut data = panel.view_data().clone();
                            data.selected_page_id = Some(page_id.clone());
                            panel.set_view_data(data, window, cx);
                        });
                        this.file_inspector_screen.last_asset_action =
                            format!("Target page: {page_id}").into();
                    }
                    AssetsPanelAction::PlaceRequested { asset_id, page_id } => {
                        this.file_inspector_screen.last_asset_action =
                            format!("Place {asset_id} on {page_id}").into();
                    }
                }
                cx.notify();
            },
        );
        let sidebar = cx.new(|cx| {
            let mut sidebar = FileInspectorSidebar::new(
                "storybook-file-inspector",
                pages.panel.clone(),
                layers.panel.clone(),
                cx,
            );
            sidebar.set_project_name("Fanta Design", cx);
            sidebar.set_assets_panel(Some(assets.clone()), cx);
            sidebar
        });
        Self {
            sidebar,
            pages,
            layers,
            assets,
            last_asset_action: "Select Assets to browse project media".into(),
            _asset_subscription: asset_subscription,
        }
    }
}

impl Storybook {
    pub(crate) fn file_inspector_last_action(&self) -> SharedString {
        format!(
            "Pages: {}  ·  Layers: {}  ·  Assets: {}",
            self.file_inspector_screen.pages.last_action,
            self.file_inspector_screen.layers.last_action,
            self.file_inspector_screen.last_asset_action,
        )
        .into()
    }

    pub(crate) fn render_file_inspector_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_story_shell(
            "storybook-reference-file-inspector",
            harness::ReferenceStoryCopy {
                eyebrow: "FILE INSPECTOR",
                title: "File inspector",
                description: "Expand the Assets section to browse project media, choose a target page, and place an asset. Audio and video appear as static previews in Design and play in Motion or Prototype.",
                adapter_description: "Pages, Layers and Assets receive host data. Choosing a page and placing an asset emit typed intents; the mock host echoes the chosen page without changing a document.",
            },
            self.file_inspector_last_action(),
            self.file_inspector_screen.sidebar.clone().into_any_element(),
            cx,
        )
    }
}
