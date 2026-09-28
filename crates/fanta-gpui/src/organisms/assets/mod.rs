//! Host-controlled project asset browser for the File Inspector.
//!
//! Assets and target pages are read models. The panel keeps only transient
//! search presentation state; placement and page selection are typed requests.

use std::{fmt, sync::Arc};

use gpui::{
    AnyElement, App, AppContext as _, Bounds, Context, Entity, EventEmitter, FocusHandle,
    Focusable, Image, InteractiveElement as _, IntoElement, KeyBinding, ObjectFit,
    ParentElement as _, Pixels, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, StyledImage as _, Subscription, Window, actions,
    deferred, div, img, point, prelude::FluentBuilder as _, px, size,
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
    ButtonControlExt as _, CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, render_lucide_icon,
    sidebar_style, tokens, track_bounds, ui_button,
};
use crate::molecules::{
    SIDEBAR_MENU_ITEM_HEIGHT, clamp_menu_origin, sidebar_menu_height, sidebar_menu_item,
    sidebar_menu_surface,
};

pub const ASSETS_PANEL_KEY_CONTEXT: &str = "FantaAssetsPanel";
const ASSETS_TEXT_ENTRY_KEY_CONTEXT: &str = "FantaAssetsPanelTextEntry";
const FILTER_MENU_WIDTH: f32 = tokens::MenuWidth::STANDARD;

actions!(
    fanta_assets,
    [
        FindInAssets,
        CloseAssetsSearch,
        ConfirmAssetsSearchEntry,
        PreviousAssetResult,
        NextAssetResult
    ]
);

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-f", FindInAssets, Some(ASSETS_PANEL_KEY_CONTEXT)),
        KeyBinding::new("escape", CloseAssetsSearch, Some(ASSETS_PANEL_KEY_CONTEXT)),
        KeyBinding::new(
            "enter",
            ConfirmAssetsSearchEntry,
            Some(ASSETS_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "shift-enter",
            PreviousAssetResult,
            Some(ASSETS_TEXT_ENTRY_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "shift-secondary-d",
            PreviousAssetResult,
            Some(ASSETS_PANEL_KEY_CONTEXT),
        ),
        KeyBinding::new(
            "shift-secondary-f",
            NextAssetResult,
            Some(ASSETS_PANEL_KEY_CONTEXT),
        ),
    ]);
}

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
    search_open: bool,
    filter_menu_open: bool,
    filter: AssetFilter,
    active_result: Option<SharedString>,
    settings_focus_handle: FocusHandle,
    close_search_focus_handle: FocusHandle,
    filter_menu_focus_handle: FocusHandle,
    panel_bounds: Option<Bounds<Pixels>>,
    filter_anchor_bounds: Option<Bounds<Pixels>>,
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
        let search_subscription =
            cx.subscribe(&search, |this, _, event: &InputEvent, cx| match event {
                InputEvent::Change => {
                    this.active_result = None;
                    this.scroll.scroll_to_item(0);
                    cx.notify();
                }
                InputEvent::PressEnter { secondary } => {
                    this.navigate_results(*secondary, cx);
                }
                InputEvent::Focus | InputEvent::Blur => {}
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
            search_open: false,
            filter_menu_open: false,
            filter: AssetFilter::All,
            active_result: None,
            settings_focus_handle: cx.focus_handle(),
            close_search_focus_handle: cx.focus_handle(),
            filter_menu_focus_handle: cx.focus_handle(),
            panel_bounds: None,
            filter_anchor_bounds: None,
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
        if self
            .active_result
            .as_ref()
            .is_some_and(|id| !self.data.assets.iter().any(|asset| &asset.id == id))
        {
            self.active_result = None;
        }
        cx.notify();
    }

    pub fn view_data(&self) -> &AssetsViewData {
        &self.data
    }

    pub fn is_search_open(&self) -> bool {
        self.search_open
    }

    /// Opens the local Assets find surface and moves focus to its query.
    pub fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_open = true;
        self.filter_menu_open = false;
        self.search.update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    fn close_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.filter_menu_open {
            self.filter_menu_open = false;
            self.settings_focus_handle.focus(window, cx);
        } else if self.search_open {
            self.search_open = false;
            self.active_result = None;
            self.focus_handle.focus(window, cx);
        } else {
            return;
        }
        cx.notify();
    }

    fn toggle_filter_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.search_open {
            return;
        }
        self.filter_menu_open = !self.filter_menu_open;
        if self.filter_menu_open {
            let focus = self.filter_menu_focus_handle.clone();
            window.defer(cx, move |window, cx| focus.focus(window, cx));
        } else {
            self.settings_focus_handle.focus(window, cx);
        }
        cx.notify();
    }

    fn set_filter(&mut self, filter: AssetFilter, window: &mut Window, cx: &mut Context<Self>) {
        self.filter = filter;
        self.filter_menu_open = false;
        self.active_result = None;
        self.scroll.scroll_to_item(0);
        self.search.update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    fn navigate_results(&mut self, previous: bool, cx: &mut Context<Self>) {
        let ids = self
            .visible_assets(cx)
            .into_iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        if ids.is_empty() {
            self.active_result = None;
            return;
        }
        let index = self
            .active_result
            .as_ref()
            .and_then(|id| ids.iter().position(|candidate| candidate == id));
        let next = match (index, previous) {
            (None, _) => 0,
            (Some(0), true) => ids.len() - 1,
            (Some(index), true) => index - 1,
            (Some(index), false) => (index + 1) % ids.len(),
        };
        self.active_result = Some(ids[next].clone());
        self.scroll.scroll_to_item(next);
        cx.notify();
    }

    fn on_find(&mut self, _: &FindInAssets, window: &mut Window, cx: &mut Context<Self>) {
        self.open_search(window, cx);
    }

    fn on_close_search(
        &mut self,
        _: &CloseAssetsSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.search_open {
            cx.stop_propagation();
            self.close_search(window, cx);
        }
    }

    fn on_confirm_search_entry(
        &mut self,
        _: &ConfirmAssetsSearchEntry,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // InputState emits PressEnter first; consume the propagated key so a
        // single-line query cannot receive a literal newline.
        cx.stop_propagation();
    }

    fn on_previous_result(
        &mut self,
        _: &PreviousAssetResult,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.search_open {
            self.navigate_results(true, cx);
            cx.stop_propagation();
        }
    }

    fn on_next_result(&mut self, _: &NextAssetResult, _: &mut Window, cx: &mut Context<Self>) {
        if self.search_open {
            self.navigate_results(false, cx);
            cx.stop_propagation();
        }
    }

    fn visible_assets(&self, cx: &App) -> Vec<&AssetRow> {
        let query = if self.search_open {
            self.search.read(cx).value().to_lowercase()
        } else {
            String::new()
        };
        let query = query.trim();
        self.data
            .assets
            .iter()
            .filter(|asset| {
                (!self.search_open || self.filter.accepts(asset.kind))
                    && (query.is_empty()
                        || asset.label.to_lowercase().contains(query)
                        || asset.kind.label().to_lowercase().contains(query)
                        || asset.detail.to_lowercase().contains(query))
            })
            .collect()
    }

    fn render_search_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let palette = sidebar_style(cx);
        let settings_focus = self
            .settings_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        let close_focus = self
            .close_search_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        h_flex()
            .debug_selector(|| "assets-search-toolbar".to_owned())
            .w_full()
            .min_w_0()
            .gap_1()
            .p_3()
            .border_b_1()
            .border_color(palette.border)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .key_context(ASSETS_TEXT_ENTRY_KEY_CONTEXT)
                    .child(
                        Input::new(&self.search)
                            .small()
                            .cleanable(true)
                            .prefix(Icon::new(IconName::Search).small()),
                    ),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{}-filters-control", self.id)))
                    .relative()
                    .flex_none()
                    .debug_selector(|| "assets-filter-trigger".to_owned())
                    .key_context(CONTROL_KEY_CONTEXT)
                    .track_focus(&settings_focus)
                    .focus(|style| style.bg(palette.hover).rounded(px(5.)))
                    .on_activate(cx.listener(|this, _, window, cx| {
                        cx.stop_propagation();
                        this.toggle_filter_menu(window, cx);
                    }))
                    .child(
                        ui_button(SharedString::from(format!("{}-filters", self.id)))
                            .ghost()
                            .small()
                            .compact()
                            .tab_stop(false)
                            .icon(IconName::Settings2)
                            .tooltip("Asset types"),
                    )
                    .child(track_bounds(cx.entity(), |this, bounds| {
                        this.filter_anchor_bounds = Some(bounds);
                    })),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{}-close-control", self.id)))
                    .flex_none()
                    .debug_selector(|| "assets-close-search".to_owned())
                    .key_context(CONTROL_KEY_CONTEXT)
                    .track_focus(&close_focus)
                    .focus(|style| style.bg(palette.hover).rounded(px(5.)))
                    .on_activate(cx.listener(|this, _, window, cx| {
                        cx.stop_propagation();
                        this.close_search(window, cx);
                    }))
                    .child(
                        ui_button(SharedString::from(format!("{}-close-search", self.id)))
                            .ghost()
                            .small()
                            .compact()
                            .tab_stop(false)
                            .icon(IconName::Close)
                            .tooltip("Close search"),
                    ),
            )
            .into_any_element()
    }

    fn render_results_header(&self, count: usize, cx: &mut Context<Self>) -> AnyElement {
        let palette = sidebar_style(cx);
        let selected = self.active_result.as_ref().and_then(|id| {
            self.visible_assets(cx)
                .iter()
                .position(|asset| &asset.id == id)
        });
        let label = match selected {
            Some(index) => format!("{} of {} assets", index + 1, count),
            None if count == 1 => "1 asset".to_owned(),
            None => format!("{count} assets"),
        };
        h_flex()
            .debug_selector(|| "assets-results-header".to_owned())
            .h(px(48.))
            .w_full()
            .min_w_0()
            .px_3()
            .gap_2()
            .border_b_1()
            .border_color(palette.border)
            .child(
                div()
                    .debug_selector(|| "assets-result-count".to_owned())
                    .flex_none()
                    .text_size(crate::atoms::sidebar_text_size())
                    .child(label),
            )
            .child(div().flex_1())
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_size(crate::atoms::sidebar_text_size())
                    .text_color(palette.muted_text)
                    .child(self.filter.label()),
            )
            .child(
                ui_button(SharedString::from(format!("{}-previous", self.id)))
                    .ghost()
                    .xsmall()
                    .compact()
                    .icon(IconName::ChevronUp)
                    .disabled(count == 0)
                    .tooltip("Previous asset")
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.navigate_results(true, cx);
                    }))
                    .debug_selector(|| "assets-previous-result".to_owned()),
            )
            .child(
                ui_button(SharedString::from(format!("{}-next", self.id)))
                    .ghost()
                    .xsmall()
                    .compact()
                    .icon(IconName::ChevronDown)
                    .disabled(count == 0)
                    .tooltip("Next asset")
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.navigate_results(false, cx);
                    }))
                    .debug_selector(|| "assets-next-result".to_owned()),
            )
            .into_any_element()
    }

    fn render_filter_menu(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(anchor_bounds)) =
            (self.panel_bounds, self.filter_anchor_bounds)
        else {
            return div().into_any_element();
        };
        let viewport = window.viewport_size();
        let estimated = sidebar_menu_height(AssetFilter::ALL.len(), 0, cx);
        let clamped = clamp_menu_origin(
            anchor_bounds.bottom_left() + point(px(0.), px(4.)),
            viewport,
            size(px(FILTER_MENU_WIDTH), estimated.min(viewport.height)),
        );
        let origin = clamped - panel_bounds.origin;
        let selected = self.filter;
        deferred(
            sidebar_menu_surface(
                SharedString::from(format!("{}-type-menu", self.id)),
                origin,
                px(FILTER_MENU_WIDTH),
                viewport.height - clamped.y,
                cx,
            )
            .debug_selector(|| "assets-filter-menu".to_owned())
            .children(AssetFilter::ALL.into_iter().map(|option| {
                let count = self
                    .data
                    .assets
                    .iter()
                    .filter(|asset| option.accepts(asset.kind))
                    .count();
                sidebar_menu_item(
                    SharedString::from(format!("{}-type-{}", self.id, option.label())),
                    px(SIDEBAR_MENU_ITEM_HEIGHT),
                    true,
                    cx,
                )
                .debug_selector(move || format!("assets-filter-{}", option.label()))
                .when(option == AssetFilter::All, |row| {
                    row.track_focus(&self.filter_menu_focus_handle)
                })
                .on_activate(cx.listener(move |this, _, window, cx| {
                    this.set_filter(option, window, cx);
                }))
                .child(div().w(px(14.)).when(option == selected, |slot| {
                    slot.child(Icon::new(IconName::Check).xsmall())
                }))
                .child(div().flex_1().child(option.label()))
                .child(
                    div()
                        .text_color(sidebar_style(cx).muted_text)
                        .child(count.to_string()),
                )
            })),
        )
        .with_priority(2)
        .into_any_element()
    }

    fn render_asset(&self, asset: &AssetRow, cx: &mut Context<Self>) -> AnyElement {
        let palette = sidebar_style(cx);
        let selected = self.search_open && self.active_result.as_ref() == Some(&asset.id);
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
        let row_id = asset.id.clone();
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
            .id(SharedString::from(format!("{}-row-{}", self.id, asset.id)))
            .debug_selector(|| format!("asset-row-{}", asset.id))
            .w_full()
            .min_w_0()
            .gap_2()
            .px_2()
            .py_2()
            .items_center()
            .border_1()
            .border_color(if selected {
                palette.selected_border
            } else {
                palette.border.opacity(0.)
            })
            .when(selected, |row| row.bg(palette.selected))
            .hover(|row| row.bg(palette.hover))
            .when(self.search_open, |row| {
                row.key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .cursor_pointer()
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.active_result = Some(row_id.clone());
                        cx.notify();
                    }))
            })
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
                            cx.stop_propagation();
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = sidebar_style(cx);
        let visible = self.visible_assets(cx);
        let count = visible.len();
        v_flex()
            .id(self.id.clone())
            .debug_selector(|| "assets-panel".to_owned())
            .key_context(ASSETS_PANEL_KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_find))
            .on_action(cx.listener(Self::on_close_search))
            .on_action(cx.listener(Self::on_confirm_search_entry))
            .on_action(cx.listener(Self::on_previous_result))
            .on_action(cx.listener(Self::on_next_result))
            .relative()
            .size_full()
            .min_h_0()
            .min_w_0()
            .bg(palette.background)
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.panel_bounds = Some(bounds);
            }))
            .when(self.search_open, |panel| {
                panel.child(self.render_search_toolbar(cx))
            })
            .child(
                v_flex().w_full().flex_none().gap_2().p_2().child(
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
            .when(self.search_open, |panel| {
                panel.child(self.render_results_header(count, cx))
            })
            .when(!self.search_open, |panel| {
                panel.child(div().w_full().h_px().bg(palette.border))
            })
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
            .when(self.filter_menu_open, |panel| {
                panel.child(self.render_filter_menu(window, cx))
            })
    }
}

#[cfg(test)]
mod tests;
