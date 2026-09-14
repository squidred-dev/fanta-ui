use std::borrow::Cow;

use gpui::{AssetSource, SharedString};
use gpui_component_assets::Assets;

pub(super) struct StorybookAssets;

impl AssetSource for StorybookAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Assets.list(path)
    }
}
