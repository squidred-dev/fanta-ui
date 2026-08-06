use std::borrow::Cow;

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
