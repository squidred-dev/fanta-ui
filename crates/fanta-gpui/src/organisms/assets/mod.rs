//! Host-controlled project asset browser for the File Inspector.
//!
//! Assets and target pages are read models. The panel keeps only its search and
//! type filter; placement and page selection are typed requests to the host.

use std::{fmt, sync::Arc};

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Image,
    InteractiveElement as _, IntoElement, ObjectFit, ParentElement as _, Render, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled as _, StyledImage as _, Subscription,
    Window, div, img, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Disableable as _, Icon, IconName, IndexPath, Sizable as _,
    button::ButtonVariants as _,
    h_flex,
    input::{Input, InputEvent, InputState},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};

use crate::atoms::{
    ButtonControlExt as _, LucideIcon, render_lucide_icon, sidebar_style, tokens, ui_button,
};

pub const ASSETS_PANEL_KEY_CONTEXT: &str = "FantaAssetsPanel";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AssetKind {
    Image,
    Svg,
    Video,
    Audio,
    Other,
}

impl AssetKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Svg => "SVG vector",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Other => "Other",
        }
    }

    const fn icon(self) -> LucideIcon {
        match self {
            Self::Image => LucideIcon::Image,
            Self::Svg => LucideIcon::Shapes,
            Self::Video => LucideIcon::Video,
            Self::Audio => LucideIcon::Music2,
            Self::Other => LucideIcon::File,
        }
    }
}

#[derive(Clone)]
pub struct AssetThumbnail {
    pub key: SharedString,
    pub image: Arc<Image>,
}

impl AssetThumbnail {
    pub fn new(key: impl Into<SharedString>, image: Arc<Image>) -> Self {
        Self {
            key: key.into(),
            image,
        }
    }
}

impl fmt::Debug for AssetThumbnail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetThumbnail")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl PartialEq for AssetThumbnail {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for AssetThumbnail {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRow {
    pub id: SharedString,
    pub label: SharedString,
    pub kind: AssetKind,
    pub thumbnail: Option<AssetThumbnail>,
    /// Short host-supplied metadata, such as dimensions or duration.
    pub detail: SharedString,
    pub can_place: bool,
    /// Explains why a media type cannot be placed in the current workspace.
    pub disabled_reason: Option<SharedString>,
}

impl AssetRow {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        kind: AssetKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            thumbnail: None,
            detail: SharedString::default(),
            can_place: true,
            disabled_reason: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetPageTarget {
    pub id: SharedString,
    pub label: SharedString,
}

impl AssetPageTarget {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

impl SelectItem for AssetPageTarget {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn render(&self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .py_1()
            .text_color(sidebar_style(cx).text)
            .child(self.label.clone())
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AssetsViewData {
    pub assets: Vec<AssetRow>,
    pub pages: Vec<AssetPageTarget>,
    pub selected_page_id: Option<SharedString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetsPanelAction {
    TargetPageSelected {
        page_id: SharedString,
    },
    PlaceRequested {
        asset_id: SharedString,
        page_id: SharedString,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AssetFilter {
    All,
    Image,
    Svg,
    Video,
    Audio,
    Other,
}

impl AssetFilter {
    const ALL: [Self; 6] = [
        Self::All,
        Self::Image,
        Self::Svg,
        Self::Video,
        Self::Audio,
        Self::Other,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Image => "Images",
            Self::Svg => "SVG",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Other => "Other",
        }
    }

    const fn accepts(self, kind: AssetKind) -> bool {
        match self {
            Self::All => true,
            Self::Image => matches!(kind, AssetKind::Image),
            Self::Svg => matches!(kind, AssetKind::Svg),
            Self::Video => matches!(kind, AssetKind::Video),
            Self::Audio => matches!(kind, AssetKind::Audio),
            Self::Other => matches!(kind, AssetKind::Other),
        }
    }
}

pub struct AssetsPanel {
    id: SharedString,
    focus_handle: FocusHandle,
    data: AssetsViewData,
    search: Entity<InputState>,
    page_select: Entity<SelectState<SearchableVec<AssetPageTarget>>>,
    filter: AssetFilter,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<AssetsPanelAction> for AssetsPanel {}

impl AssetsPanel {
    pub fn new(
        id: impl Into<SharedString>,
        data: AssetsViewData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search project assets"));
        let selected = data
            .selected_page_id
            .as_ref()
            .and_then(|id| data.pages.iter().position(|page| &page.id == id))
            .map(|index| IndexPath::default().row(index));
        let page_select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(data.pages.clone()), selected, window, cx)
                .searchable(data.pages.len() > 8)
        });
        let search_subscription = cx.subscribe(&search, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        let page_subscription = cx.subscribe_in(
            &page_select,
            window,
            |_, _, event: &SelectEvent<SearchableVec<AssetPageTarget>>, _, cx| {
                if let SelectEvent::Confirm(Some(page_id)) = event {
                    cx.emit(AssetsPanelAction::TargetPageSelected {
                        page_id: page_id.clone(),
                    });
                }
            },
        );
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            data,
            search,
            page_select,
            filter: AssetFilter::All,
            scroll: ScrollHandle::new(),
            _subscriptions: vec![search_subscription, page_subscription],
        }
    }

    pub fn set_view_data(
        &mut self,
        data: AssetsViewData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.data == data {
            return;
        }
        let pages_changed =
            self.data.pages != data.pages || self.data.selected_page_id != data.selected_page_id;
        if pages_changed {
            let selected = data
                .selected_page_id
                .as_ref()
                .and_then(|id| data.pages.iter().position(|page| &page.id == id))
                .map(|index| IndexPath::default().row(index));
            self.page_select.update(cx, |select, cx| {
                select.set_items(SearchableVec::new(data.pages.clone()), window, cx);
                select.set_selected_index(selected, window, cx);
            });
        }
        self.data = data;
        cx.notify();
    }

    pub fn view_data(&self) -> &AssetsViewData {
        &self.data
    }

    fn visible_assets(&self, cx: &App) -> Vec<&AssetRow> {
        let query = self.search.read(cx).value().to_lowercase();
        let query = query.trim();
        self.data
            .assets
            .iter()
            .filter(|asset| {
                self.filter.accepts(asset.kind)
                    && (query.is_empty()
                        || asset.label.to_lowercase().contains(query)
                        || asset.kind.label().to_lowercase().contains(query))
            })
            .collect()
    }

    fn render_asset(&self, asset: &AssetRow, cx: &mut Context<Self>) -> AnyElement {
        let palette = sidebar_style(cx);
        let can_place = asset.can_place && self.data.selected_page_id.is_some();
        let tooltip = if !asset.can_place {
            asset
                .disabled_reason
                .clone()
                .unwrap_or_else(|| "Unavailable in this workspace".into())
        } else if self.data.selected_page_id.is_none() {
            "Choose a page to place this asset".into()
        } else {
            "Place on selected page".into()
        };
        let id = asset.id.clone();
        let page = self.data.selected_page_id.clone();
        let detail: SharedString = if asset.detail.is_empty() {
            asset.kind.label().into()
        } else {
            format!("{} · {}", asset.kind.label(), asset.detail).into()
        };
        let thumbnail = div()
            .w(px(42.))
            .h(px(42.))
            .flex_none()
            .rounded(px(4.))
            .border_1()
            .border_color(palette.border)
            .bg(palette.hover)
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center();
        let thumbnail = if let Some(preview) = &asset.thumbnail {
            thumbnail
                .child(
                    img(preview.image.clone())
                        .w_full()
                        .h_full()
                        .object_fit(ObjectFit::Contain),
                )
                .into_any_element()
        } else {
            thumbnail
                .child(render_lucide_icon(
                    asset.kind.icon(),
                    palette.muted_icon,
                    19.,
                ))
                .into_any_element()
        };
        h_flex()
            .debug_selector(|| format!("asset-row-{}", asset.id))
            .w_full()
            .min_w_0()
            .gap_2()
            .px_2()
            .py_2()
            .items_center()
            .hover(|row| row.bg(palette.hover))
            .child(thumbnail)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(palette.text)
                            .child(asset.label.clone()),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(px(10.))
                            .text_color(palette.muted_text)
                            .child(detail),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| format!("asset-place-{}", asset.id))
                    .child(
                        ui_button(SharedString::from(format!(
                            "{}-place-{}",
                            self.id, asset.id
                        )))
                        .ghost()
                        .xsmall()
                        .compact()
                        .icon(IconName::Plus)
                        .tooltip(tooltip)
                        .disabled(!can_place)
                        .on_activate(cx.listener(move |_, _, _, cx| {
                            if let Some(page_id) = &page {
                                cx.emit(AssetsPanelAction::PlaceRequested {
                                    asset_id: id.clone(),
                                    page_id: page_id.clone(),
                                });
                            }
                        })),
                    ),
            )
            .into_any_element()
    }
}

impl Focusable for AssetsPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AssetsPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = sidebar_style(cx);
        let visible = self.visible_assets(cx);
        let count = visible.len();
        let filter = self.filter;
        let filters = AssetFilter::ALL.into_iter().map(|option| {
            let selected = option == filter;
            ui_button(SharedString::from(format!(
                "{}-filter-{}",
                self.id,
                option.label()
            )))
            .label(option.label())
            .xsmall()
            .compact()
            .on_activate(cx.listener(move |this, _, _, cx| {
                this.filter = option;
                this.scroll.scroll_to_item(0);
                cx.notify();
            }))
            .when(!selected, |button| button.ghost())
        });
        v_flex()
            .id(self.id.clone())
            .key_context(ASSETS_PANEL_KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .min_h_0()
            .min_w_0()
            .bg(palette.background)
            .child(
                v_flex()
                    .w_full()
                    .flex_none()
                    .gap_2()
                    .p_2()
                    .child(
                        Input::new(&self.search)
                            .small()
                            .prefix(Icon::new(IconName::Search).small()),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-filters", self.id)))
                            .w_full()
                            .overflow_x_scroll()
                            .child(h_flex().gap_1().children(filters)),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .flex_none()
                                    .text_size(px(10.))
                                    .text_color(palette.muted_text)
                                    .child("Place in"),
                            )
                            .child(
                                Select::new(&self.page_select)
                                    .w_full()
                                    .h(px(tokens::RowHeight::FIELD))
                                    .placeholder("Choose page")
                                    .search_placeholder("Search pages…")
                                    .disabled(self.data.pages.is_empty()),
                            ),
                    ),
            )
            .child(div().w_full().h_px().bg(palette.border))
            .child(
                div()
                    .id(SharedString::from(format!("{}-list", self.id)))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .when(count == 0, |list| {
                        list.child(
                            div()
                                .p_3()
                                .text_size(px(11.))
                                .text_color(palette.muted_text)
                                .child(if self.data.assets.is_empty() {
                                    "No project assets yet. Add a creation from the gallery."
                                } else {
                                    "No assets match this search."
                                }),
                        )
                    })
                    .children(
                        visible
                            .into_iter()
                            .map(|asset| self.render_asset(asset, cx)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests;
