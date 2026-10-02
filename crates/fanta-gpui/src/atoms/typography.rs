//! Semantic typography roles based on the Figma UI3 hierarchy.

use gpui::{FontWeight, Pixels, px};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypographyToken {
    Display,
    HeadingLarge,
    HeadingMedium,
    HeadingSmall,
    BodyLarge,
    BodyLargeStrong,
    BodyMedium,
    BodyMediumStrong,
    BodySmall,
    BodySmallStrong,
    /// Editor panel copy follows Zed's UI scale and the host's rem sizing.
    Panel,
    PanelStrong,
    PanelCaption,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyStyle {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub font_weight: FontWeight,
}

impl TypographyToken {
    pub const ALL: [Self; 13] = [
        Self::Display,
        Self::HeadingLarge,
        Self::HeadingMedium,
        Self::HeadingSmall,
        Self::BodyLarge,
        Self::BodyLargeStrong,
        Self::BodyMedium,
        Self::BodyMediumStrong,
        Self::BodySmall,
        Self::BodySmallStrong,
        Self::Panel,
        Self::PanelStrong,
        Self::PanelCaption,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Display => "heading/display",
            Self::HeadingLarge => "heading/heading.large",
            Self::HeadingMedium => "heading/heading.medium",
            Self::HeadingSmall => "heading/heading.small",
            Self::BodyLarge | Self::BodyLargeStrong => "body/body.large",
            Self::BodyMedium | Self::BodyMediumStrong => "body/body.medium",
            Self::BodySmall | Self::BodySmallStrong => "body/body.small",
            Self::Panel | Self::PanelStrong => "panel/body",
            Self::PanelCaption => "panel/caption",
        }
    }

    pub fn style(self) -> TypographyStyle {
        let strong = FontWeight(550.);
        match self {
            Self::Display => TypographyStyle {
                font_size: px(48.),
                line_height: px(56.),
                font_weight: FontWeight::MEDIUM,
            },
            Self::HeadingLarge => TypographyStyle {
                font_size: px(24.),
                line_height: px(32.),
                font_weight: strong,
            },
            Self::HeadingMedium => TypographyStyle {
                font_size: px(15.),
                line_height: px(25.),
                font_weight: strong,
            },
            Self::HeadingSmall => TypographyStyle {
                font_size: px(13.),
                line_height: px(22.),
                font_weight: strong,
            },
            Self::BodyLarge => TypographyStyle {
                font_size: px(13.),
                line_height: px(22.),
                font_weight: FontWeight(450.),
            },
            Self::BodyLargeStrong => TypographyStyle {
                font_size: px(13.),
                line_height: px(22.),
                font_weight: strong,
            },
            Self::BodyMedium => TypographyStyle {
                font_size: px(11.),
                line_height: px(16.),
                font_weight: FontWeight(450.),
            },
            Self::BodyMediumStrong => TypographyStyle {
                font_size: px(11.),
                line_height: px(16.),
                font_weight: strong,
            },
            Self::BodySmall => TypographyStyle {
                font_size: px(9.),
                line_height: px(14.),
                font_weight: FontWeight(450.),
            },
            Self::BodySmallStrong => TypographyStyle {
                font_size: px(9.),
                line_height: px(14.),
                font_weight: strong,
            },
            Self::Panel | Self::PanelStrong => TypographyStyle {
                font_size: px(super::tokens::TypeScale::TITLE),
                line_height: px(super::tokens::TypeScale::DISPLAY),
                font_weight: if self == Self::PanelStrong {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                },
            },
            Self::PanelCaption => TypographyStyle {
                font_size: px(super::tokens::TypeScale::BODY),
                line_height: px(super::tokens::RowHeight::FIELD - super::tokens::Space::SM),
                font_weight: FontWeight::NORMAL,
            },
        }
    }

    pub const fn is_strong(self) -> bool {
        matches!(
            self,
            Self::BodyLargeStrong
                | Self::BodyMediumStrong
                | Self::BodySmallStrong
                | Self::PanelStrong
        )
    }
}

pub trait TypographyExt: gpui::Styled + Sized {
    fn typography(mut self, token: TypographyToken) -> Self {
        let style = token.style();
        self = if matches!(
            token,
            TypographyToken::Panel | TypographyToken::PanelStrong | TypographyToken::PanelCaption
        ) {
            self.text_size(ui::rems_from_px(f32::from(style.font_size)))
                .line_height(ui::rems_from_px(f32::from(style.line_height)))
        } else {
            self.text_size(style.font_size)
                .line_height(style.line_height)
        };
        self.font_weight(style.font_weight)
    }
}

impl<T: gpui::Styled> TypographyExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ui3_type_scale_has_stable_sizes_and_strong_pairs() {
        assert_eq!(TypographyToken::BodyMedium.style().font_size, px(11.));
        assert_eq!(TypographyToken::HeadingLarge.style().line_height, px(32.));
        assert_eq!(
            TypographyToken::BodyLarge.style().font_size,
            TypographyToken::BodyLargeStrong.style().font_size
        );
        assert!(TypographyToken::BodySmallStrong.is_strong());
    }
}
