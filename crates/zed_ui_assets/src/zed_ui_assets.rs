use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

/// The reusable visual assets shipped with the Zed UI baseline.
#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "fonts/**/*"]
#[include = "icons/**/*"]
#[include = "images/**/*"]
#[include = "themes/**/*"]
#[include = "licenses.md"]
#[exclude = "themes/src/*"]
pub struct Assets;

/// Returns a bundled asset without requiring a GPUI application.
pub fn bundled_asset(path: &str) -> Option<Cow<'static, [u8]>> {
    Assets::get(path).map(|file| file.data)
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(bundled_asset(path))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|asset_path| asset_path.starts_with(path))
            .map(Into::into)
            .collect())
    }
}
