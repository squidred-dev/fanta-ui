use std::borrow::Cow;

use gpui::{AssetSource, SharedString};
use gpui_component_assets::Assets as ComponentAssets;
use zed_ui_assets::Assets as ZedAssets;

pub(super) struct StorybookAssets;

impl AssetSource for StorybookAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = ZedAssets.load(path)? {
            return Ok(Some(bytes));
        }
        ComponentAssets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut paths = ZedAssets.list(path)?;
        for fallback_path in ComponentAssets.list(path)? {
            if !paths.contains(&fallback_path) {
                paths.push(fallback_path);
            }
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_zed_icons_take_precedence() {
        for path in [
            "icons/check.svg",
            "icons/arrow-down.svg",
            "icons/folder.svg",
        ] {
            let zed = ZedAssets
                .load(path)
                .expect("Zed assets load")
                .expect("Zed icon exists");
            let fallback = ComponentAssets
                .load(path)
                .expect("component assets load")
                .expect("fallback icon exists");
            assert_eq!(
                StorybookAssets.load(path).expect("storybook assets load"),
                Some(zed)
            );
            if path != "icons/arrow-down.svg" {
                assert_ne!(
                    StorybookAssets.load(path).expect("storybook assets load"),
                    Some(fallback)
                );
            }
            assert_eq!(
                StorybookAssets
                    .list(path)
                    .expect("asset list")
                    .iter()
                    .filter(|candidate| candidate.as_ref() == path)
                    .count(),
                1
            );
        }
    }
}
