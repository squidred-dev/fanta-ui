//! A Zed-derived settings window surface for Fanta hosts.
//!
//! The navigation, header and section/row layout are adapted from Zed's
//! `crates/settings_ui/src/settings_ui.rs` and `components/section_items.rs`
//! at commit `8b08b0607b` (GPL-3.0-or-later). Fanta keeps the host-controlled
//! data and typed-intent boundary and substitutes its own applicable pages.
//!
//! The host supplies every accepted setting, account value and MCP server.
//! This screen owns only search, navigation continuity and the unsaved MCP
//! form. It never writes a settings file, purchases a plan or starts a server.

mod model;
pub use model::*;

use std::collections::HashSet;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, Hsla, InteractiveElement as _, IntoElement, ParentElement as _, Render, Role,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Disableable as _, Icon, IconName, Selectable as _, Sizable as _, Theme as ComponentTheme,
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::{DropdownMenu as _, PopupMenuItem},
    switch::Switch,
    v_flex,
};
use ui::{StyledTypography as _, TreeViewItem, prelude::Toggleable as _};
use url::Url;

use crate::atoms::{ButtonControlExt as _, CONTROL_KEY_CONTEXT, ControlExt as _, ui_button};
use crate::screens::billing::{BillingAction, BillingScreen, BillingViewData};

pub const SETTINGS_SCREEN_MIN_WIDTH: f32 = 900.;
pub const SETTINGS_SCREEN_MIN_HEIGHT: f32 = 520.;

#[derive(Clone, Copy)]
struct Palette {
    background: Hsla,
    panel: Hsla,
    surface: Hsla,
    hover: Hsla,
    selected: Hsla,
    border: Hsla,
    text: Hsla,
    muted: Hsla,
    accent: Hsla,
    success: Hsla,
    warning: Hsla,
    danger: Hsla,
}

impl Palette {
    fn current(cx: &App) -> Self {
        if cx.try_global::<theme::GlobalTheme>().is_some() {
            let active_theme = theme::GlobalTheme::theme(cx);
            let colors = active_theme.colors();
            let status = active_theme.status();
            Self {
                background: colors.editor_background,
                panel: colors.panel_background,
                surface: colors.surface_background,
                hover: colors.element_hover,
                selected: colors.element_selected,
                border: colors.border_variant,
                text: colors.text,
                muted: colors.text_muted,
                accent: colors.text_accent,
                success: status.success,
                warning: status.warning,
                danger: status.error,
            }
        } else {
            let colors = ComponentTheme::global(cx);
            Self {
                background: colors.background,
                panel: colors.sidebar,
                surface: colors.secondary,
                hover: colors.secondary_hover,
                selected: colors.list_active,
                border: colors.border,
                text: colors.foreground,
                muted: colors.muted_foreground,
                accent: colors.primary,
                success: colors.success,
                warning: colors.warning,
                danger: colors.danger,
            }
        }
    }

    fn tone(self, tone: SettingsTone) -> Hsla {
        match tone {
            SettingsTone::Neutral => self.muted,
            SettingsTone::Accent => self.accent,
            SettingsTone::Success => self.success,
            SettingsTone::Warning => self.warning,
            SettingsTone::Danger => self.danger,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FormTransport {
    Local,
    Http,
}

#[derive(Clone, Debug)]
struct McpForm {
    server_id: Option<SharedString>,
    transport: FormTransport,
    enabled: bool,
    scope: SettingsMcpScope,
    error: Option<SharedString>,
    submission_pending: bool,
}

fn mcp_save_action(
    form: &McpForm,
    name: &str,
    endpoint: &str,
    args: &str,
) -> Result<SettingsAction, SharedString> {
    let name = name.trim();
    let endpoint = endpoint.trim();
    if name.is_empty() {
        return Err("Enter a server name.".into());
    }
    if endpoint.is_empty() {
        return Err(match form.transport {
            FormTransport::Local => "Enter a command to start the server.",
            FormTransport::Http => "Enter the server URL.",
        }
        .into());
    }
    if form.transport == FormTransport::Http {
        let url = Url::parse(endpoint).map_err(|_| "Enter a valid server URL.")?;
        if !matches!(url.scheme(), "https" | "http") || url.host_str().is_none() {
            return Err("Enter a valid HTTP or HTTPS server URL.".into());
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("Remove credentials from the URL and authenticate after saving.".into());
        }
    }
    let connection = match form.transport {
        FormTransport::Local => SettingsMcpConnection::Local {
            command: endpoint.to_owned().into(),
            args: args
                .lines()
                .map(str::trim)
                .filter(|arg| !arg.is_empty())
                .map(Into::into)
                .collect(),
        },
        FormTransport::Http => SettingsMcpConnection::Http {
            url: endpoint.to_owned().into(),
        },
    };
    Ok(SettingsAction::McpSaveRequested {
        server_id: form.server_id.clone(),
        draft: SettingsMcpDraft {
            name: name.to_owned().into(),
            connection,
            enabled: form.enabled,
            scope: form.scope,
        },
    })
}

pub struct SettingsScreen {
    id: SharedString,
    data: SettingsViewData,
    billing_screen: Entity<BillingScreen>,
    search: Entity<InputState>,
    mcp_name: Entity<InputState>,
    mcp_endpoint: Entity<InputState>,
    mcp_args: Entity<InputState>,
    mcp_form: Option<McpForm>,
    confirm_remove: Option<SharedString>,
    expanded_pages: HashSet<SettingsPage>,
    active_group: Option<SharedString>,
    pending_group: Option<(SettingsPage, SharedString)>,
    focus_handle: FocusHandle,
    sidebar_scroll: ScrollHandle,
    content_scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SettingsAction> for SettingsScreen {}

impl SettingsScreen {
    fn billing_snapshot(data: &SettingsViewData, page: SettingsPage) -> BillingViewData {
        let mut billing = data.billing.clone().unwrap_or_default();
        if let Some(tab) = page.billing_tab() {
            billing.selected_tab = tab;
        }
        billing
    }

    fn sync_billing_page(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        let billing = Self::billing_snapshot(&self.data, page);
        self.billing_screen
            .update(cx, |screen, cx| screen.set_view_data(billing, cx));
    }

    pub fn new(
        id: impl Into<SharedString>,
        data: SettingsViewData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search settings…")
                .clean_on_escape()
        });
        let mcp_name = cx.new(|cx| InputState::new(window, cx).placeholder("Server name"));
        let mcp_endpoint = cx.new(|cx| InputState::new(window, cx).placeholder("Command"));
        let mcp_args = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder("One argument per line")
        });
        let billing = Self::billing_snapshot(&data, data.selected_page);
        let billing_screen =
            cx.new(|cx| BillingScreen::new_embedded("settings-billing-content", billing, cx));
        let subscriptions = vec![
            cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let page = this.effective_page(&this.search_query(cx));
                    this.sync_billing_page(page, cx);
                    cx.notify();
                }
            }),
            cx.subscribe(&mcp_name, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Some(form) = this.mcp_form.as_mut() {
                        form.error = None;
                    }
                    cx.notify();
                }
            }),
            cx.subscribe(&mcp_endpoint, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Some(form) = this.mcp_form.as_mut() {
                        form.error = None;
                    }
                    cx.notify();
                }
            }),
            cx.subscribe(&mcp_args, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Some(form) = this.mcp_form.as_mut() {
                        form.error = None;
                    }
                    cx.notify();
                }
            }),
            cx.subscribe(&billing_screen, |_, _, event: &BillingAction, cx| {
                cx.emit(SettingsAction::BillingActionRequested(event.clone()));
            }),
        ];
        Self {
            id: id.into(),
            data,
            billing_screen,
            search,
            mcp_name,
            mcp_endpoint,
            mcp_args,
            mcp_form: None,
            confirm_remove: None,
            expanded_pages: HashSet::new(),
            active_group: None,
            pending_group: None,
            focus_handle: cx.focus_handle(),
            sidebar_scroll: ScrollHandle::new(),
            content_scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn set_view_data(&mut self, data: SettingsViewData, cx: &mut Context<Self>) {
        if self.data.selected_page != data.selected_page {
            self.content_scroll.set_offset(gpui::point(px(0.), px(0.)));
        }
        self.data = data;
        if let Some((page, group)) = self.pending_group.clone()
            && self.data.selected_page == page
        {
            self.active_group = Some(group.clone());
            self.scroll_to_group(page, &group);
            self.pending_group = None;
        }
        let page = self.effective_page(&self.search_query(cx));
        self.sync_billing_page(page, cx);
        if self
            .confirm_remove
            .as_ref()
            .is_some_and(|id| !self.data.mcp_servers.iter().any(|server| &server.id == id))
        {
            self.confirm_remove = None;
        }
        cx.notify();
    }

    pub fn view_data(&self) -> &SettingsViewData {
        &self.data
    }

    /// Complete a previously requested save after the host validates and stores it.
    /// A rejected save keeps the user's draft available for correction.
    pub fn finish_mcp_save(&mut self, result: Result<(), SharedString>, cx: &mut Context<Self>) {
        match result {
            Ok(()) => self.mcp_form = None,
            Err(error) => {
                if let Some(form) = self.mcp_form.as_mut() {
                    form.error = Some(error);
                    form.submission_pending = false;
                }
            }
        }
        cx.notify();
    }

    /// Dismiss the form after the host has accepted and persisted its draft.
    pub fn dismiss_mcp_form(&mut self, cx: &mut Context<Self>) {
        self.finish_mcp_save(Ok(()), cx);
    }

    fn search_query(&self, cx: &App) -> String {
        self.search
            .read(cx)
            .text()
            .to_string()
            .trim()
            .to_lowercase()
    }

    fn item_matches(item: &SettingsItem, query: &str) -> bool {
        query.is_empty()
            || item.title.to_lowercase().contains(query)
            || item.description.to_lowercase().contains(query)
            || item.id.to_lowercase().contains(query)
    }

    fn group_matches(group: &SettingsGroup, query: &str) -> bool {
        query.is_empty()
            || group.title.to_lowercase().contains(query)
            || group
                .items
                .iter()
                .any(|item| Self::item_matches(item, query))
    }

    fn page_matches(&self, page: SettingsPage, query: &str) -> bool {
        if query.is_empty() || page.label().to_lowercase().contains(query) {
            return true;
        }
        if page == SettingsPage::Account
            && ["api keys", "workspace", "profile"]
                .iter()
                .any(|term| term.contains(query))
        {
            return true;
        }
        if page == SettingsPage::Billing
            && ["balance", "subscription", "credits", "invoices"]
                .iter()
                .any(|term| term.contains(query))
        {
            return true;
        }
        if page == SettingsPage::Usage
            && ["requests", "models", "spend", "usage"]
                .iter()
                .any(|term| term.contains(query))
        {
            return true;
        }
        if page == SettingsPage::Activity
            && ["receipts", "purchases", "transactions"]
                .iter()
                .any(|term| term.contains(query))
        {
            return true;
        }
        if page == SettingsPage::McpTools
            && self.data.mcp_servers.iter().any(|server| {
                server.name.to_lowercase().contains(query)
                    || server.description.to_lowercase().contains(query)
                    || server.connection.endpoint().to_lowercase().contains(query)
            })
        {
            return true;
        }
        self.data.pages.iter().any(|data| {
            data.page == page
                && data
                    .groups
                    .iter()
                    .any(|group| Self::group_matches(group, query))
        })
    }

    fn effective_page(&self, query: &str) -> SettingsPage {
        if self.page_matches(self.data.selected_page, query) {
            self.data.selected_page
        } else {
            SettingsPage::ALL
                .into_iter()
                .find(|page| self.page_matches(*page, query))
                .unwrap_or(self.data.selected_page)
        }
    }

    fn page_data(&self, page: SettingsPage) -> Option<&SettingsPageData> {
        self.data.pages.iter().find(|data| data.page == page)
    }

    fn select_page(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        self.active_group = None;
        self.pending_group = None;
        self.content_scroll.set_offset(gpui::point(px(0.), px(0.)));
        cx.emit(SettingsAction::PageSelected(page));
        cx.notify();
    }

    fn toggle_page_expanded(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        if !self.expanded_pages.insert(page) {
            self.expanded_pages.remove(&page);
        }
        cx.notify();
    }

    fn scroll_to_group(&self, page: SettingsPage, group: &SharedString) {
        let Some(data) = self.page_data(page) else {
            return;
        };
        let preceding_groups = data
            .groups
            .iter()
            .take_while(|candidate| &candidate.title != group)
            .filter(|candidate| {
                candidate.items.iter().any(|item| {
                    !self.data.settings_file_available || item.id.as_ref() != "settings_json"
                })
            })
            .count();
        let base = 2
            + usize::from(self.data.notice.is_some())
            + usize::from(page == SettingsPage::Account)
            + usize::from(page == SettingsPage::McpTools);
        self.content_scroll
            .scroll_to_top_of_item(base + preceding_groups);
    }

    fn open_group(&mut self, page: SettingsPage, group: SharedString, cx: &mut Context<Self>) {
        self.active_group = Some(group.clone());
        if self.data.selected_page == page {
            self.scroll_to_group(page, &group);
        } else {
            self.pending_group = Some((page, group));
            cx.emit(SettingsAction::PageSelected(page));
        }
        cx.notify();
    }

    fn open_form(
        &mut self,
        server_id: Option<SharedString>,
        transport: FormTransport,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing = server_id
            .as_ref()
            .and_then(|id| self.data.mcp_servers.iter().find(|server| &server.id == id));
        let name = existing.map_or_else(String::new, |server| server.name.to_string());
        let endpoint = existing.map_or_else(String::new, |server| {
            server.connection.endpoint().to_string()
        });
        let args = existing.map_or_else(String::new, |server| match &server.connection {
            SettingsMcpConnection::Local { args, .. } => args
                .iter()
                .map(SharedString::as_ref)
                .collect::<Vec<&str>>()
                .join("\n"),
            SettingsMcpConnection::Http { .. } => String::new(),
        });
        let transport = existing.map_or(transport, |server| match server.connection {
            SettingsMcpConnection::Local { .. } => FormTransport::Local,
            SettingsMcpConnection::Http { .. } => FormTransport::Http,
        });
        self.mcp_name
            .update(cx, |input, cx| input.set_value(name, window, cx));
        self.mcp_endpoint
            .update(cx, |input, cx| input.set_value(endpoint, window, cx));
        self.mcp_endpoint.update(cx, |input, cx| {
            input.set_placeholder(
                if transport == FormTransport::Local {
                    "Executable path or name"
                } else {
                    "https://example.com/mcp"
                },
                window,
                cx,
            )
        });
        self.mcp_args
            .update(cx, |input, cx| input.set_value(args, window, cx));
        self.mcp_form = Some(McpForm {
            server_id,
            transport,
            enabled: existing.is_none_or(|server| server.enabled),
            scope: existing.map_or(SettingsMcpScope::Workspace, |server| server.scope),
            error: None,
            submission_pending: false,
        });
        self.confirm_remove = None;
        self.mcp_name.read(cx).focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    fn save_form(&mut self, cx: &mut Context<Self>) {
        let Some(form) = self.mcp_form.as_mut() else {
            return;
        };
        if form.submission_pending {
            return;
        }
        let name = self.mcp_name.read(cx).text().to_string().trim().to_owned();
        let endpoint = self
            .mcp_endpoint
            .read(cx)
            .text()
            .to_string()
            .trim()
            .to_owned();
        let args = self.mcp_args.read(cx).text().to_string();
        let action = match mcp_save_action(form, &name, &endpoint, &args) {
            Ok(action) => action,
            Err(error) => {
                form.error = Some(error);
                cx.notify();
                return;
            }
        };
        form.submission_pending = true;
        cx.emit(action);
        cx.notify();
    }

    fn render_sidebar(&self, palette: Palette, query: &str, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.effective_page(query);
        let zed_theme_available = cx.try_global::<theme::GlobalTheme>().is_some();
        let mut nav = v_flex()
            .id("settings-navigation")
            .debug_selector(|| "settings-navigation".to_owned())
            .role(Role::Tree)
            .aria_label("Settings navigation")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.sidebar_scroll)
            .gap(px(2.));
        let sections: [(&str, &[SettingsPage]); 3] = [
            (
                "Preferences",
                &[
                    SettingsPage::General,
                    SettingsPage::Appearance,
                    SettingsPage::Canvas,
                    SettingsPage::Editor,
                    SettingsPage::Privacy,
                ],
            ),
            (
                "AI & tools",
                &[SettingsPage::AiModels, SettingsPage::McpTools],
            ),
            (
                "Workspace",
                &[
                    SettingsPage::Account,
                    SettingsPage::Billing,
                    SettingsPage::Usage,
                    SettingsPage::Plans,
                    SettingsPage::Activity,
                ],
            ),
        ];
        for (section_label, pages) in sections {
            if !pages.iter().any(|page| self.page_matches(*page, query)) {
                continue;
            }
            nav = nav.child(
                div()
                    .pt(px(11.))
                    .pb(px(4.))
                    .px(px(5.))
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.muted)
                    .child(section_label),
            );
            for &page in pages {
                if !self.page_matches(page, query) {
                    continue;
                }
                let active = selected == page;
                let groups = self
                    .page_data(page)
                    .filter(|_| page != SettingsPage::Account || self.data.account.is_some())
                    .map(|data| {
                        data.groups
                            .iter()
                            .filter(|group| {
                                query.is_empty()
                                    || page.label().to_lowercase().contains(query)
                                    || Self::group_matches(group, query)
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let expandable = !groups.is_empty();
                let expanded =
                    expandable && (self.expanded_pages.contains(&page) || !query.is_empty());
                let screen = cx.entity();
                let root_id = SharedString::from(format!("settings-nav-{:?}", page));
                let root = if expandable && zed_theme_available {
                    let toggle_screen = screen.clone();
                    TreeViewItem::new(
                        SharedString::from(format!("settings-tree-{:?}", page)),
                        page.label(),
                    )
                    .root_item(true)
                    .expanded(expanded)
                    .toggle_state(active)
                    .tab_index(0)
                    .on_toggle(move |_, _, cx| {
                        toggle_screen.update(cx, |this, cx| this.toggle_page_expanded(page, cx));
                    })
                    .on_click(move |_, _, cx| {
                        screen.update(cx, |this, cx| this.select_page(page, cx));
                    })
                    .into_any_element()
                } else if expandable {
                    let toggle_screen = screen.clone();
                    h_flex()
                        .id(SharedString::from(format!(
                            "settings-fallback-tree-{:?}",
                            page
                        )))
                        .role(Role::TreeItem)
                        .aria_label(page.label())
                        .aria_selected(active)
                        .aria_level(1)
                        .aria_expanded(expanded)
                        .key_context(CONTROL_KEY_CONTEXT)
                        .tab_stop(true)
                        .cursor_pointer()
                        .h(px(28.))
                        .w_full()
                        .pl(px(4.))
                        .pr(px(5.))
                        .gap(px(6.))
                        .rounded(px(4.))
                        .bg(if active {
                            palette.selected
                        } else {
                            palette.panel
                        })
                        .hover(|style| style.bg(palette.hover))
                        .focus_visible(|style| style.border_1().border_color(palette.accent))
                        .on_activate(move |_, _, cx| {
                            screen.update(cx, |this, cx| this.select_page(page, cx));
                        })
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "settings-disclosure-{:?}",
                                    page
                                )))
                                .role(Role::Button)
                                .aria_label(if expanded {
                                    "Collapse section"
                                } else {
                                    "Expand section"
                                })
                                .key_context(CONTROL_KEY_CONTEXT)
                                .tab_stop(true)
                                .cursor_pointer()
                                .w(px(18.))
                                .h(px(22.))
                                .flex_none()
                                .on_activate(move |_, _, cx| {
                                    cx.stop_propagation();
                                    toggle_screen
                                        .update(cx, |this, cx| this.toggle_page_expanded(page, cx));
                                })
                                .child(
                                    Icon::new(if expanded {
                                        IconName::ChevronDown
                                    } else {
                                        IconName::ChevronRight
                                    })
                                    .xsmall()
                                    .text_color(palette.muted),
                                ),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_size(px(13.))
                                .text_color(if active { palette.text } else { palette.muted })
                                .child(page.label()),
                        )
                        .into_any_element()
                } else {
                    h_flex()
                        .id(SharedString::from(format!("settings-leaf-{:?}", page)))
                        .role(Role::TreeItem)
                        .aria_label(page.label())
                        .aria_selected(active)
                        .aria_level(1)
                        .key_context(CONTROL_KEY_CONTEXT)
                        .tab_stop(true)
                        .cursor_pointer()
                        .h(px(28.))
                        .w_full()
                        .pl(px(31.))
                        .pr(px(5.))
                        .rounded(px(4.))
                        .bg(if active {
                            palette.selected
                        } else {
                            palette.panel
                        })
                        .hover(|style| style.bg(palette.hover))
                        .focus_visible(|style| style.border_1().border_color(palette.accent))
                        .on_activate(move |_, _, cx| {
                            screen.update(cx, |this, cx| this.select_page(page, cx));
                        })
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_size(px(13.))
                                .text_color(if active { palette.text } else { palette.muted })
                                .child(page.label()),
                        )
                        .into_any_element()
                };
                nav = nav.child(
                    div()
                        .id(root_id)
                        .debug_selector(move || format!("settings-nav-{:?}", page))
                        .w_full()
                        .child(root),
                );
                if expanded {
                    for group in groups {
                        let title = group.title.clone();
                        let chosen = active && self.active_group.as_ref() == Some(&title);
                        let screen = cx.entity();
                        let section_id = format!("settings-section-{:?}-{}", page, title);
                        let child: AnyElement = if zed_theme_available {
                            TreeViewItem::new(
                                SharedString::from(format!(
                                    "settings-section-tree-{:?}-{}",
                                    page, title
                                )),
                                title.clone(),
                            )
                            .toggle_state(chosen)
                            .tab_index(0)
                            .on_click(move |_, _, cx| {
                                screen.update(cx, |this, cx| {
                                    this.open_group(page, title.clone(), cx)
                                });
                            })
                            .into_any_element()
                        } else {
                            h_flex()
                                .id(SharedString::from(format!(
                                    "settings-fallback-section-{:?}-{}",
                                    page, title
                                )))
                                .role(Role::TreeItem)
                                .aria_label(title.clone())
                                .aria_selected(chosen)
                                .aria_level(2)
                                .key_context(CONTROL_KEY_CONTEXT)
                                .tab_stop(true)
                                .cursor_pointer()
                                .h(px(28.))
                                .w_full()
                                .pl(px(31.))
                                .pr(px(5.))
                                .rounded(px(4.))
                                .bg(if chosen {
                                    palette.selected
                                } else {
                                    palette.panel
                                })
                                .hover(|style| style.bg(palette.hover))
                                .focus_visible(|style| {
                                    style.border_1().border_color(palette.accent)
                                })
                                .on_activate(move |_, _, cx| {
                                    screen.update(cx, |this, cx| {
                                        this.open_group(page, title.clone(), cx)
                                    });
                                })
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_size(px(12.))
                                        .text_color(if chosen {
                                            palette.text
                                        } else {
                                            palette.muted
                                        })
                                        .child(group.title.clone()),
                                )
                                .into_any_element()
                        };
                        nav = nav.child(
                            div()
                                .id(SharedString::from(section_id.clone()))
                                .debug_selector(move || section_id.clone())
                                .child(child),
                        );
                    }
                }
            }
        }
        let mut sidebar = v_flex()
            .id("settings-sidebar")
            .debug_selector(|| "settings-sidebar".to_owned())
            .w(px(232.))
            .h_full()
            .flex_none()
            .bg(palette.panel)
            .border_r_1()
            .border_color(palette.border)
            .p(px(12.))
            .pt(px(if cfg!(target_os = "macos") { 44. } else { 12. }))
            .gap(px(8.))
            .child(
                Input::new(&self.search)
                    .w_full()
                    .small()
                    .cleanable(true)
                    .text_size(px(12.))
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(5.))
                    .prefix(
                        Icon::new(IconName::Search)
                            .xsmall()
                            .text_color(palette.muted),
                    ),
            )
            .child(nav);
        if let Some(account) = &self.data.account {
            let screen = cx.entity();
            let account_name = account.display_name.clone();
            let workspace = account.workspace.clone();
            let plan = account.plan.clone();
            let credit_balance = self
                .data
                .usage
                .as_ref()
                .map(|usage| usage.available.clone());
            sidebar = sidebar.child(
                v_flex()
                    .id("settings-account-footer")
                    .debug_selector(|| "settings-account-footer".to_owned())
                    .role(Role::Button)
                    .aria_label(format!(
                        "{} · {} · Account settings",
                        account_name, workspace
                    ))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_stop(true)
                    .cursor_pointer()
                    .w_full()
                    .flex_none()
                    .pt(px(11.))
                    .px(px(7.))
                    .pb(px(3.))
                    .gap(px(4.))
                    .border_t_1()
                    .border_color(palette.border)
                    .hover(|style| style.bg(palette.hover))
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |this, cx| this.select_page(SettingsPage::Account, cx));
                    })
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(px(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(account_name),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(px(11.))
                            .text_color(palette.muted)
                            .child(workspace),
                    )
                    .child(
                        h_flex()
                            .min_w_0()
                            .flex_wrap()
                            .gap(px(6.))
                            .text_size(px(10.))
                            .text_color(palette.muted)
                            .child(plan)
                            .when_some(credit_balance, |row, balance| {
                                row.child("·").child(balance)
                            }),
                    ),
            );
        } else {
            let screen = cx.entity();
            sidebar = sidebar.child(
                div()
                    .w_full()
                    .flex_none()
                    .pt(px(10.))
                    .border_t_1()
                    .border_color(palette.border)
                    .child(
                        ui_button("settings-footer-sign-in")
                            .label("Sign in to Fanta")
                            .small()
                            .outline()
                            .on_activate(move |_, _, cx| {
                                screen.update(cx, |_, cx| cx.emit(SettingsAction::SignInRequested));
                            }),
                    ),
            );
        }
        sidebar.into_any_element()
    }

    fn render_item(
        &self,
        item: &SettingsItem,
        palette: Palette,
        bottom_border: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control: AnyElement = match &item.control {
            SettingsControl::Toggle(value) => {
                let id = item.id.clone();
                let screen = cx.entity();
                Switch::new(SharedString::from(format!("settings-switch-{id}")))
                    .checked(*value)
                    .on_click(move |value, _, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(SettingsAction::ToggleRequested {
                                id: id.clone(),
                                value: *value,
                            });
                        });
                    })
                    .into_any_element()
            }
            SettingsControl::Choice { selected, options } => {
                let label = options
                    .iter()
                    .find(|option| &option.value == selected)
                    .map_or_else(|| selected.clone(), |option| option.label.clone());
                let options = options.clone();
                let selected = selected.clone();
                let id = item.id.clone();
                let screen = cx.entity();
                ui_button(SharedString::from(format!("settings-choice-{id}")))
                    .label(label)
                    .dropdown_caret(true)
                    .small()
                    .compact()
                    .outline()
                    .min_w(px(145.))
                    .dropdown_menu(move |mut menu, _, _| {
                        for option in &options {
                            let option_id = id.clone();
                            let value = option.value.clone();
                            let screen = screen.clone();
                            menu = menu.item(
                                PopupMenuItem::new(option.label.clone())
                                    .checked(option.value == selected)
                                    .on_click(move |_, _, cx| {
                                        screen.update(cx, |_, cx| {
                                            cx.emit(SettingsAction::ChoiceRequested {
                                                id: option_id.clone(),
                                                value: value.clone(),
                                            });
                                        });
                                    }),
                            );
                        }
                        menu
                    })
                    .into_any_element()
            }
            SettingsControl::Action { label } => {
                let id = item.id.clone();
                let screen = cx.entity();
                ui_button(SharedString::from(format!("settings-action-{id}")))
                    .label(label.clone())
                    .small()
                    .outline()
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(SettingsAction::ActionRequested { id: id.clone() });
                        });
                    })
                    .into_any_element()
            }
            SettingsControl::Status { label, tone } => div()
                .px(px(8.))
                .py(px(4.))
                .rounded(px(4.))
                .bg(palette.surface)
                .text_size(px(11.))
                .text_color(palette.tone(*tone))
                .child(label.clone())
                .into_any_element(),
            SettingsControl::Credential {
                configured,
                action_label,
            } => {
                let id = item.id.clone();
                let screen = cx.entity();
                h_flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(if *configured {
                                palette.success
                            } else {
                                palette.muted
                            })
                            .child(if *configured { "Configured" } else { "Not set" }),
                    )
                    .child(
                        ui_button(SharedString::from(format!("settings-credential-{id}")))
                            .label(action_label.clone())
                            .small()
                            .outline()
                            .on_activate(move |_, _, cx| {
                                screen.update(cx, |_, cx| {
                                    cx.emit(SettingsAction::ActionRequested { id: id.clone() });
                                });
                            }),
                    )
                    .into_any_element()
            }
        };
        let selector = format!("settings-row-{}", item.id);
        h_flex()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .role(Role::Group)
            .aria_label(item.title.clone())
            .w_full()
            .min_w_0()
            .min_h(px(66.))
            .justify_between()
            .items_center()
            .gap(px(18.))
            .py(px(14.))
            .when(bottom_border, |row| {
                row.border_b_1().border_color(palette.border)
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(3.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(palette.text)
                            .child(item.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(18.))
                            .text_color(palette.muted)
                            .child(item.description.clone()),
                    ),
            )
            .child(div().max_w(px(300.)).flex_none().child(control))
            .into_any_element()
    }

    fn render_group(
        &self,
        group: &SettingsGroup,
        query: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !Self::group_matches(group, query) {
            return None;
        }
        let group_title_match = group.title.to_lowercase().contains(query);
        let visible = group
            .items
            .iter()
            .filter(|item| {
                (!self.data.settings_file_available || item.id.as_ref() != "settings_json")
                    && (group_title_match || Self::item_matches(item, query))
            })
            .collect::<Vec<_>>();
        if visible.is_empty() {
            return None;
        }
        let mut rows = v_flex().w_full();
        for (index, item) in visible.iter().enumerate() {
            rows = rows.child(self.render_item(item, palette, index + 1 < visible.len(), cx));
        }
        let mut section_label = div()
            .id(SharedString::from(format!(
                "settings-heading-{}",
                group.title
            )))
            .role(Role::Heading)
            .aria_level(2)
            .aria_label(group.title.clone())
            .text_size(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(palette.muted)
            .child(group.title.clone());
        if cx.try_global::<theme::GlobalTheme>().is_some() {
            section_label = section_label.font_buffer(cx);
        }
        let mut section = v_flex().w_full().gap(px(8.)).child(
            v_flex()
                .w_full()
                .gap(px(7.))
                .child(section_label)
                .child(div().h(px(1.)).w_full().bg(palette.border)),
        );
        if let Some(description) = &group.description {
            section = section.child(
                div()
                    .text_size(px(11.))
                    .text_color(palette.muted)
                    .child(description.clone()),
            );
        }
        Some(section.child(rows).into_any_element())
    }

    fn render_account(&self, palette: Palette, cx: &mut Context<Self>) -> AnyElement {
        let mut content = v_flex().w_full().min_w_0().gap(px(20.));
        if self.data.account.is_none() {
            let screen = cx.entity();
            return content
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .items_center()
                        .justify_between()
                        .gap(px(20.))
                        .p(px(20.))
                        .rounded(px(7.))
                        .border_1()
                        .border_color(palette.border)
                        .bg(palette.panel)
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap(px(7.))
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child("Sign in to Fanta"),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(palette.muted)
                                        .child("Access your workspace, AI usage, and billing."),
                                ),
                        )
                        .child(
                            ui_button("settings-sign-in")
                                .label("Sign in")
                                .small()
                                .on_activate(move |_, _, cx| {
                                    screen.update(cx, |_, cx| {
                                        cx.emit(SettingsAction::SignInRequested)
                                    });
                                }),
                        ),
                )
                .into_any_element();
        }
        if let Some(account) = &self.data.account {
            content = content.child(
                h_flex()
                    .id("settings-account-summary")
                    .debug_selector(|| "settings-account-summary".to_owned())
                    .w_full()
                    .min_w_0()
                    .items_center()
                    .justify_between()
                    .gap(px(16.))
                    .p(px(16.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(palette.border)
                    .bg(palette.panel)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(5.))
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(account.display_name.clone()),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(11.))
                                    .text_color(palette.muted)
                                    .child(account.email.clone()),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(11.))
                                    .text_color(palette.muted)
                                    .child(format!("{} · {}", account.plan, account.workspace)),
                            ),
                    )
                    .child(
                        ui_button("settings-workspace")
                            .label("Manage workspace")
                            .small()
                            .outline()
                            .on_activate({
                                let screen = cx.entity();
                                move |_, _, cx| {
                                    screen.update(cx, |_, cx| {
                                        cx.emit(SettingsAction::WorkspaceRequested)
                                    });
                                }
                            }),
                    ),
            );
        }
        let mut links = h_flex().w_full().gap(px(10.)).flex_wrap();
        for (id, heading, detail, action) in [
            (
                "billing",
                "Billing & credits",
                "Plan, purchases, invoices, and available credits",
                SettingsAction::PageSelected(SettingsPage::Billing),
            ),
            (
                "usage",
                "AI usage",
                "Activity, spend, model breakdown, and limits",
                SettingsAction::PageSelected(SettingsPage::Usage),
            ),
            (
                "api-keys",
                "API keys",
                "Create and manage scoped Fanta API keys",
                SettingsAction::ApiKeysRequested,
            ),
        ] {
            let screen = cx.entity();
            links = links.child(
                v_flex()
                    .id(SharedString::from(format!("settings-link-{id}")))
                    .role(Role::Button)
                    .aria_label(format!("Open {heading}"))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_stop(true)
                    .cursor_pointer()
                    .w(px(210.))
                    .min_h(px(112.))
                    .p(px(14.))
                    .gap(px(8.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(palette.border)
                    .bg(palette.surface)
                    .hover(|style| style.bg(palette.hover))
                    .focus_visible(|style| style.border_color(palette.accent))
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| cx.emit(action.clone()));
                    })
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(heading),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(palette.muted)
                            .child(detail),
                    )
                    .child(
                        Icon::new(IconName::ExternalLink)
                            .xsmall()
                            .text_color(palette.accent),
                    ),
            );
        }
        content = content.child(links);
        if let Some(usage) = &self.data.usage {
            content = content.child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .p(px(14.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(palette.border)
                    .child(
                        v_flex()
                            .gap(px(4.))
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(palette.muted)
                                    .child("AVAILABLE WORKSPACE CREDITS"),
                            )
                            .child(
                                div()
                                    .text_size(px(18.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(usage.available.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(palette.muted)
                                    .child(format!("{} used · {}", usage.used, usage.period)),
                            ),
                    )
                    .child(
                        ui_button("settings-usage-details")
                            .label("View usage")
                            .small()
                            .outline()
                            .on_activate({
                                let screen = cx.entity();
                                move |_, _, cx| {
                                    screen.update(cx, |_, cx| {
                                        cx.emit(SettingsAction::PageSelected(SettingsPage::Usage))
                                    });
                                }
                            }),
                    ),
            );
        }
        content.into_any_element()
    }

    fn status_label(status: SettingsMcpStatus) -> (&'static str, SettingsTone) {
        match status {
            SettingsMcpStatus::Running => ("Connected", SettingsTone::Success),
            SettingsMcpStatus::Stopped => ("Stopped", SettingsTone::Neutral),
            SettingsMcpStatus::Connecting => ("Connecting", SettingsTone::Accent),
            SettingsMcpStatus::AuthenticationRequired => {
                ("Sign in required", SettingsTone::Warning)
            }
            SettingsMcpStatus::Error => ("Needs attention", SettingsTone::Danger),
        }
    }

    fn server_status_label(server: &SettingsMcpServerView) -> (&'static str, SettingsTone) {
        if server.enabled {
            Self::status_label(server.status)
        } else {
            ("Disabled", SettingsTone::Neutral)
        }
    }

    fn render_mcp_server(
        &self,
        server: &SettingsMcpServerView,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (status_label, status_tone) = Self::server_status_label(server);
        let id = server.id.clone();
        let confirming_remove = self.confirm_remove.as_ref() == Some(&id);
        let screen = cx.entity();
        let enabled = server.enabled;
        let mut actions = h_flex().gap(px(6.)).flex_wrap().items_center();
        if server.enabled && server.status == SettingsMcpStatus::AuthenticationRequired {
            let screen = screen.clone();
            let id = id.clone();
            actions = actions.child(
                ui_button(SharedString::from(format!("mcp-auth-{id}")))
                    .label("Sign in")
                    .small()
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(SettingsAction::McpActionRequested {
                                server_id: id.clone(),
                                action: SettingsMcpAction::Authenticate,
                            })
                        });
                    }),
            );
        }
        if server.enabled && server.status == SettingsMcpStatus::Error {
            let screen = screen.clone();
            let id = id.clone();
            actions = actions.child(
                ui_button(SharedString::from(format!("mcp-retry-{id}")))
                    .label("Retry")
                    .small()
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(SettingsAction::McpActionRequested {
                                server_id: id.clone(),
                                action: SettingsMcpAction::Retry,
                            })
                        });
                    }),
            );
        }
        let edit_id = id.clone();
        let edit_screen = screen.clone();
        let toggle_id = id.clone();
        let toggle_screen = screen.clone();
        actions = actions.child(
            ui_button(SharedString::from(format!("mcp-configure-{id}")))
                .label("Configure")
                .small()
                .outline()
                .on_activate(move |_, window, cx| {
                    edit_screen.update(cx, |this, cx| {
                        this.open_form(Some(edit_id.clone()), FormTransport::Local, window, cx);
                    });
                }),
        );
        if confirming_remove {
            let cancel_screen = screen.clone();
            let remove_screen = screen.clone();
            let remove_id = id.clone();
            actions = actions
                .child(
                    ui_button(SharedString::from(format!("mcp-cancel-remove-{id}")))
                        .label("Keep")
                        .small()
                        .outline()
                        .on_activate(move |_, _, cx| {
                            cancel_screen.update(cx, |this, cx| {
                                this.confirm_remove = None;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    ui_button(SharedString::from(format!("mcp-confirm-remove-{id}")))
                        .label("Confirm remove")
                        .small()
                        .on_activate(move |_, _, cx| {
                            remove_screen.update(cx, |this, cx| {
                                this.confirm_remove = None;
                                cx.emit(SettingsAction::McpActionRequested {
                                    server_id: remove_id.clone(),
                                    action: SettingsMcpAction::Remove,
                                });
                                cx.notify();
                            });
                        }),
                );
        } else {
            let remove_screen = screen.clone();
            let remove_id = id.clone();
            actions = actions.child(
                ui_button(SharedString::from(format!("mcp-remove-{id}")))
                    .label("Remove")
                    .small()
                    .outline()
                    .on_activate(move |_, _, cx| {
                        remove_screen.update(cx, |this, cx| {
                            this.confirm_remove = Some(remove_id.clone());
                            cx.notify();
                        });
                    }),
            );
        }
        actions = actions.child(
            Switch::new(SharedString::from(format!("mcp-enabled-{id}")))
                .checked(enabled)
                .tooltip("Enable server")
                .on_click(move |value, _, cx| {
                    toggle_screen.update(cx, |_, cx| {
                        cx.emit(SettingsAction::McpActionRequested {
                            server_id: toggle_id.clone(),
                            action: SettingsMcpAction::ToggleEnabled(*value),
                        })
                    });
                }),
        );
        let selector = format!("settings-mcp-server-{id}");
        v_flex()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .w_full()
            .min_w_0()
            .gap(px(11.))
            .p(px(15.))
            .rounded(px(7.))
            .border_1()
            .border_color(palette.border)
            .bg(palette.panel)
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .justify_between()
                    .gap(px(15.))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(4.))
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(server.name.clone()),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(11.))
                                    .text_color(palette.muted)
                                    .child(server.description.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(10.))
                            .text_color(palette.tone(status_tone))
                            .child(status_label),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .flex_wrap()
                    .gap(px(10.))
                    .text_size(px(10.))
                    .text_color(palette.muted)
                    .child(server.connection.label())
                    .child("·")
                    .child(server.scope.label())
                    .when_some(server.tool_count, |row, count| {
                        row.child("·").child(format!("{count} tools"))
                    }),
            )
            .child(
                div()
                    .w_full()
                    .truncate()
                    .text_size(px(10.))
                    .text_color(palette.muted)
                    .child(server.connection.endpoint().clone()),
            )
            .when_some(
                server
                    .enabled
                    .then(|| server.status_detail.clone())
                    .flatten(),
                |card, detail| {
                    card.child(
                        div()
                            .text_size(px(10.))
                            .text_color(palette.tone(status_tone))
                            .child(detail),
                    )
                },
            )
            .when(confirming_remove, |card| {
                card.child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.warning)
                        .child("Remove this server from Fanta? This disconnects its tools."),
                )
            })
            .child(actions)
            .into_any_element()
    }

    fn render_mcp_form(&self, palette: Palette, cx: &mut Context<Self>) -> AnyElement {
        let Some(form) = self.mcp_form.as_ref() else {
            return div().into_any_element();
        };
        let screen = cx.entity();
        let title = if form.server_id.is_some() {
            "Configure server"
        } else {
            "Add MCP server"
        };
        v_flex()
            .id("settings-mcp-form")
            .debug_selector(|| "settings-mcp-form".to_owned())
            .w_full()
            .min_w_0()
            .gap(px(16.))
            .p(px(18.))
            .rounded(px(7.))
            .border_1()
            .border_color(palette.border)
            .bg(palette.panel)
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_size(px(14.)).font_weight(FontWeight::SEMIBOLD).child(title))
                    .child(ui_button("mcp-form-cancel").label("Cancel").small().outline().on_activate({
                        let screen = screen.clone();
                        move |_, _, cx| screen.update(cx, |this, cx| { this.mcp_form = None; cx.notify(); })
                    })),
            )
            .child(
                h_flex()
                    .gap(px(7.))
                    .child(ui_button("mcp-form-local").label("Local command").small().selected(form.transport == FormTransport::Local).on_activate({
                        let screen = screen.clone();
                        move |_, window, cx| screen.update(cx, |this, cx| {
                            if let Some(form) = this.mcp_form.as_mut() { form.transport = FormTransport::Local; form.error = None; }
                            this.mcp_endpoint.update(cx, |input, cx| input.set_placeholder("Executable path or name", window, cx));
                            cx.notify();
                        })
                    }))
                    .child(ui_button("mcp-form-http").label("Remote HTTP").small().selected(form.transport == FormTransport::Http).on_activate({
                        let screen = screen.clone();
                        move |_, window, cx| screen.update(cx, |this, cx| {
                            if let Some(form) = this.mcp_form.as_mut() { form.transport = FormTransport::Http; form.error = None; }
                            this.mcp_endpoint.update(cx, |input, cx| input.set_placeholder("https://example.com/mcp", window, cx));
                            cx.notify();
                        })
                    })),
            )
            .child(
                v_flex()
                    .gap(px(5.))
                    .child(div().text_size(px(11.)).child("Server name"))
                    .child(Input::new(&self.mcp_name).small().border_1().border_color(palette.border).rounded(px(5.))),
            )
            .child(
                v_flex()
                    .gap(px(5.))
                    .child(div().text_size(px(11.)).child(if form.transport == FormTransport::Local { "Executable" } else { "Server URL" }))
                    .child(Input::new(&self.mcp_endpoint).small().border_1().border_color(palette.border).rounded(px(5.)))
                    .child(div().text_size(px(10.)).text_color(palette.muted).child(if form.transport == FormTransport::Local {
                        "Enter an executable path or name. Store credentials outside this form."
                    } else {
                        "Use an HTTP endpoint. Authentication is completed separately after saving."
                    })),
            )
            .when(form.transport == FormTransport::Local, |card| {
                card.child(
                    v_flex()
                        .gap(px(5.))
                        .child(div().text_size(px(11.)).child("Arguments"))
                        .child(
                            Input::new(&self.mcp_args)
                                .small()
                                .h(px(76.))
                                .border_1()
                                .border_color(palette.border)
                                .rounded(px(5.)),
                        )
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(palette.muted)
                                .child("Enter one literal argument per line. No shell expansion is applied."),
                        ),
                )
            })
            .child(
                h_flex()
                    .w_full()
                    .flex_wrap()
                    .gap(px(20.))
                    .items_center()
                    .child(
                        h_flex()
                            .gap(px(7.))
                            .child(Switch::new("mcp-form-enabled").checked(form.enabled).on_click({
                                let screen = screen.clone();
                                move |value, _, cx| screen.update(cx, |this, cx| {
                                    if let Some(form) = this.mcp_form.as_mut() { form.enabled = *value; }
                                    cx.notify();
                                })
                            }))
                            .child(div().text_size(px(11.)).child("Enabled")),
                    )
                    .child(
                        h_flex()
                            .gap(px(6.))
                            .child(div().text_size(px(11.)).text_color(palette.muted).child("Scope"))
                            .child(ui_button("mcp-scope-workspace").label("Workspace").small().selected(form.scope == SettingsMcpScope::Workspace).on_activate({
                                let screen = screen.clone();
                                move |_, _, cx| screen.update(cx, |this, cx| {
                                    if let Some(form) = this.mcp_form.as_mut() { form.scope = SettingsMcpScope::Workspace; }
                                    cx.notify();
                                })
                            }))
                            .child(ui_button("mcp-scope-global").label("All workspaces").small().selected(form.scope == SettingsMcpScope::Global).on_activate({
                                let screen = screen.clone();
                                move |_, _, cx| screen.update(cx, |this, cx| {
                                    if let Some(form) = this.mcp_form.as_mut() { form.scope = SettingsMcpScope::Global; }
                                    cx.notify();
                                })
                            })),
                    ),
            )
            .when_some(form.error.clone(), |card, error| {
                card.child(div().text_size(px(11.)).text_color(palette.danger).child(error))
            })
            .when(form.submission_pending, |card| {
                card.child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.muted)
                        .child("Saving server…"),
                )
            })
            .child(
                h_flex()
                    .w_full()
                    .justify_end()
                    .child(ui_button("mcp-form-save").label("Save server").small().disabled(form.submission_pending).on_activate(move |_, _, cx| {
                        screen.update(cx, |this, cx| this.save_form(cx));
                    })),
            )
            .into_any_element()
    }

    fn render_mcp(&self, palette: Palette, query: &str, cx: &mut Context<Self>) -> AnyElement {
        let mut panel = v_flex().w_full().min_w_0().gap(px(16.));
        panel = panel.child(
            h_flex()
                .id("settings-mcp-header")
                .debug_selector(|| "settings-mcp-header".to_owned())
                .w_full()
                .min_w_0()
                .justify_between()
                .gap(px(16.))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap(px(4.))
                        .child(div().text_size(px(12.)).font_weight(FontWeight::SEMIBOLD).child("Configured servers"))
                        .child(div().text_size(px(11.)).text_color(palette.muted).child("Connect model tools to services you trust. Each server has its own scope and state.")),
                )
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap(px(6.))
                        .child(
                            div()
                                .id("settings-mcp-add-local")
                                .debug_selector(|| "settings-mcp-add-local".to_owned())
                                .child(ui_button("mcp-add-local").label("Add local").small().outline().on_activate({
                                    let screen = cx.entity();
                                    move |_, window, cx| screen.update(cx, |this, cx| this.open_form(None, FormTransport::Local, window, cx))
                                })),
                        )
                        .child(ui_button("mcp-add-remote").label("Add remote").small().on_activate({
                            let screen = cx.entity();
                            move |_, window, cx| screen.update(cx, |this, cx| this.open_form(None, FormTransport::Http, window, cx))
                        })),
                ),
        );
        if self.mcp_form.is_some() {
            panel = panel.child(self.render_mcp_form(palette, cx));
        }
        let mut count = 0;
        for server in &self.data.mcp_servers {
            if !query.is_empty()
                && !server.name.to_lowercase().contains(query)
                && !server.description.to_lowercase().contains(query)
                && !server.connection.label().to_lowercase().contains(query)
                && !server.connection.endpoint().to_lowercase().contains(query)
            {
                continue;
            }
            panel = panel.child(self.render_mcp_server(server, palette, cx));
            count += 1;
        }
        if count == 0 {
            panel = panel.child(
                v_flex()
                    .w_full()
                    .p(px(20.))
                    .gap(px(6.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(palette.border)
                    .child(div().text_size(px(12.)).child(if query.is_empty() { "No MCP servers yet" } else { "No matching servers" }))
                    .child(div().text_size(px(11.)).text_color(palette.muted).child(if query.is_empty() {
                        "Add a local command or remote endpoint to make its tools available to your agent."
                    } else {
                        "Try a different search term."
                    })),
            );
        }
        panel.into_any_element()
    }

    fn render_contents(&self, palette: Palette, query: &str, cx: &mut Context<Self>) -> AnyElement {
        let page = self.effective_page(query);
        let visible_query = if page.label().to_lowercase().contains(query) {
            ""
        } else {
            query
        };
        let mut contents = v_flex()
            .id("settings-content-scroll")
            .debug_selector(|| "settings-content-scroll".to_owned())
            .role(Role::Group)
            .aria_label("Settings content")
            .w_full()
            .min_w_0()
            .h_full()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.content_scroll)
            .px(px(32.))
            .pt(px(30.))
            .pb(px(22.))
            .gap(px(19.));
        let screen = cx.entity();
        let billing_page = page.billing_tab().is_some();
        let scope_label: SharedString = if billing_page {
            self.data.billing.as_ref().map_or_else(
                || "Workspace billing".into(),
                |billing| format!("{} · Workspace", billing.workspace.name).into(),
            )
        } else {
            self.data.scope.clone()
        };
        let mut utility = h_flex().w_full().items_center().justify_between().child(
            div()
                .px(px(7.))
                .py(px(3.))
                .rounded(px(4.))
                .bg(palette.accent.opacity(0.18))
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(palette.accent)
                .child(scope_label),
        );
        if !billing_page && self.data.settings_file_available {
            utility = utility.child(
                ui_button("settings-edit-json")
                    .label("Edit in settings.json")
                    .small()
                    .outline()
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(SettingsAction::ActionRequested {
                                id: "settings_json".into(),
                            });
                        });
                    }),
            );
        }
        contents = contents.child(utility);
        if let Some(notice) = &self.data.notice {
            contents = contents.child(
                div()
                    .w_full()
                    .p(px(12.))
                    .rounded(px(5.))
                    .border_1()
                    .border_color(palette.warning)
                    .bg(palette.surface)
                    .text_size(px(11.))
                    .text_color(palette.warning)
                    .child(notice.clone()),
            );
        }
        contents = contents.child(
            v_flex()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(page.label()),
                )
                .child(div().text_size(px(11.)).text_color(palette.muted).child(
                    self.page_data(page).map_or_else(
                        || page.description().into(),
                        |data| data.description.clone(),
                    ),
                )),
        );
        if billing_page {
            contents = if self.data.billing.is_some() {
                contents.child(
                    div()
                        .w_full()
                        .min_w_0()
                        .flex_1()
                        .child(self.billing_screen.clone()),
                )
            } else {
                contents.child(
                    div()
                        .w_full()
                        .p(px(18.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(palette.border)
                        .text_size(px(11.))
                        .text_color(palette.muted)
                        .child("Billing details are unavailable for this workspace."),
                )
            };
            return contents.child(div().h(px(16.))).into_any_element();
        }
        if page == SettingsPage::Account {
            contents = contents.child(self.render_account(palette, cx));
        }
        if page == SettingsPage::McpTools {
            contents = contents.child(self.render_mcp(palette, visible_query, cx));
        }
        let mut group_count = 0;
        if let Some(data) = self
            .page_data(page)
            .filter(|_| page != SettingsPage::Account || self.data.account.is_some())
        {
            for group in &data.groups {
                if let Some(section) = self.render_group(group, visible_query, palette, cx) {
                    contents = contents.child(section);
                    group_count += 1;
                }
            }
        }
        if group_count == 0 && page != SettingsPage::McpTools && page != SettingsPage::Account {
            contents = contents.child(
                div()
                    .p(px(18.))
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(6.))
                    .text_size(px(12.))
                    .text_color(palette.muted)
                    .child(if query.is_empty() {
                        "No settings are available in this section."
                    } else {
                        "No settings match this search."
                    }),
            );
        }
        contents.child(div().h(px(20.))).into_any_element()
    }
}

impl Focusable for SettingsScreen {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::current(cx);
        let query = self.search_query(cx);
        let root = h_flex()
            .id(self.id.clone())
            .key_context("FantaSettingsScreen")
            .track_focus(&self.focus_handle)
            .size_full()
            .min_w(px(SETTINGS_SCREEN_MIN_WIDTH))
            .min_h(px(SETTINGS_SCREEN_MIN_HEIGHT))
            .bg(palette.background)
            .text_color(palette.text)
            .child(self.render_sidebar(palette, &query, cx))
            .child(self.render_contents(palette, &query, cx));
        if cx.try_global::<theme::GlobalTheme>().is_some() {
            root.font_ui(cx)
        } else {
            root
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::billing::{BillingActionState, BillingActions};
    use crate::test_support::mount_component;
    use gpui::{Modifiers, TestAppContext, size};

    #[gpui::test]
    fn billing_destinations_stay_inside_settings_and_forward_intents(cx: &mut TestAppContext) {
        let data = SettingsViewData {
            billing: Some(BillingViewData {
                actions: BillingActions {
                    export_usage: BillingActionState::enabled("Export CSV"),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let (host, actions, cx) = mount_component(cx, move |window, cx| {
            SettingsScreen::new("test-settings", data, window, cx)
        });
        cx.simulate_resize(size(px(1000.), px(740.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("settings-navigation").is_some());
        assert!(cx.debug_bounds("billing-embedded-content").is_none());

        let billing = cx.debug_bounds("settings-nav-Billing").unwrap();
        cx.simulate_click(billing.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[SettingsAction::PageSelected(SettingsPage::Billing)]
        );
        let screen = cx.read(|app| host.read(app).component.clone());
        assert_eq!(
            cx.read(|app| screen.read(app).view_data().selected_page),
            SettingsPage::General
        );

        let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
        echoed.selected_page = SettingsPage::Billing;
        cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("settings-navigation").is_some());
        assert!(cx.debug_bounds("billing-embedded-content").is_some());
        assert!(cx.debug_bounds("billing-tab-usage").is_none());
        assert_eq!(
            cx.read(|app| screen
                .read(app)
                .billing_screen
                .read(app)
                .view_data()
                .selected_tab),
            crate::screens::billing::BillingTab::Overview,
        );

        actions.borrow_mut().clear();
        let usage = cx.debug_bounds("settings-nav-Usage").unwrap();
        cx.simulate_click(usage.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[SettingsAction::PageSelected(SettingsPage::Usage)]
        );
        let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
        echoed.selected_page = SettingsPage::Usage;
        cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("settings-navigation").is_some());
        assert_eq!(
            cx.read(|app| screen
                .read(app)
                .billing_screen
                .read(app)
                .view_data()
                .selected_tab),
            crate::screens::billing::BillingTab::Usage,
        );
        actions.borrow_mut().clear();
        let export = cx
            .debug_bounds("settings-billing-content-export-usage")
            .unwrap();
        cx.simulate_click(export.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[SettingsAction::BillingActionRequested(
                BillingAction::ExportUsageRequested
            )],
        );
    }

    #[gpui::test]
    fn zed_style_sidebar_and_fanta_pages_fit_reference_window(cx: &mut TestAppContext) {
        let data = SettingsViewData {
            pages: vec![SettingsPageData::new(
                SettingsPage::General,
                vec![SettingsGroup::new(
                    "Startup",
                    vec![SettingsItem::new(
                        "startup.restore",
                        "Restore last workspace",
                        "Reopen the most recent workspace when Fanta starts.",
                        SettingsControl::Toggle(true),
                    )],
                )],
            )],
            account: Some(SettingsAccountSummary {
                display_name: "A long account name that must not widen the settings pane".into(),
                email: "someone-with-a-long-address@example.com".into(),
                plan: "Pro".into(),
                workspace: "A workspace with a longer than normal name".into(),
            }),
            mcp_servers: vec![SettingsMcpServerView {
                id: "tools".into(),
                name: "A descriptive tools server with a longer name".into(),
                description: "Search and retrieval tools available to this workspace.".into(),
                connection: SettingsMcpConnection::Http {
                    url: "https://tools.example.com/a/long/path/to/a/workspace/mcp/endpoint".into(),
                },
                status: SettingsMcpStatus::Running,
                enabled: true,
                scope: SettingsMcpScope::Workspace,
                tool_count: Some(5),
                status_detail: None,
            }],
            ..Default::default()
        };
        let (host, _, cx) = mount_component(cx, move |window, cx| {
            SettingsScreen::new("test-settings-narrow", data, window, cx)
        });
        cx.simulate_resize(size(px(900.), px(640.)));
        cx.run_until_parked();
        let sidebar = cx.debug_bounds("settings-sidebar").unwrap();
        let content = cx.debug_bounds("settings-content-scroll").unwrap();
        assert!(sidebar.right() <= content.left() + px(1.));
        assert!(sidebar.bottom() <= px(641.));
        assert!(cx.debug_bounds("settings-account-footer").unwrap().bottom() <= px(641.));

        let screen = cx.read(|app| host.read(app).component.clone());
        cx.update(|_, app| {
            screen.update(app, |screen, cx| {
                screen.toggle_page_expanded(SettingsPage::General, cx)
            })
        });
        cx.run_until_parked();
        let section = cx.debug_bounds("settings-section-General-Startup").unwrap();
        cx.simulate_click(section.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            cx.read(|app| screen.read(app).active_group.clone()),
            Some("Startup".into())
        );

        for (page, selectors) in [
            (SettingsPage::General, &["settings-row-startup.restore"][..]),
            (SettingsPage::Account, &["settings-account-summary"][..]),
            (
                SettingsPage::McpTools,
                &["settings-mcp-header", "settings-mcp-server-tools"][..],
            ),
        ] {
            let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
            echoed.selected_page = page;
            cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
            cx.run_until_parked();
            let pane = cx.debug_bounds("settings-content-scroll").unwrap();
            for selector in selectors {
                let bounds = cx.debug_bounds(selector).unwrap();
                assert!(
                    bounds.left() >= pane.left() - px(1.)
                        && bounds.right() <= pane.right() + px(1.),
                    "{page:?} {selector} widened the pane: {bounds:?} vs {pane:?}"
                );
            }
        }

        let add_local = cx.debug_bounds("settings-mcp-add-local").unwrap();
        cx.simulate_click(add_local.center(), Modifiers::none());
        cx.run_until_parked();
        let pane = cx.debug_bounds("settings-content-scroll").unwrap();
        let form = cx.debug_bounds("settings-mcp-form").unwrap();
        assert!(form.left() >= pane.left() - px(1.) && form.right() <= pane.right() + px(1.));
    }

    #[test]
    fn search_matches_labels_descriptions_and_setting_ids() {
        let item = SettingsItem::new(
            "agent.tool_permissions",
            "Tool permissions",
            "Review what an agent can run in this workspace.",
            SettingsControl::Action {
                label: "Configure".into(),
            },
        );
        let group = SettingsGroup::new("Agent safety", vec![item.clone()]);
        assert!(SettingsScreen::item_matches(&item, "tool"));
        assert!(SettingsScreen::item_matches(&item, "workspace"));
        assert!(SettingsScreen::item_matches(&item, "agent.tool"));
        assert!(SettingsScreen::group_matches(&group, "safety"));
        assert!(!SettingsScreen::group_matches(&group, "typography"));
    }

    #[test]
    fn mcp_form_validates_and_emits_host_owned_configuration() {
        let form = McpForm {
            server_id: Some("search-server".into()),
            transport: FormTransport::Http,
            enabled: false,
            scope: SettingsMcpScope::Workspace,
            error: None,
            submission_pending: false,
        };
        assert!(mcp_save_action(&form, "", "https://tools.example/mcp", "").is_err());
        assert!(mcp_save_action(&form, "Search", "localhost:3000", "").is_err());
        assert!(mcp_save_action(&form, "Search", "https://", "").is_err());
        assert!(mcp_save_action(&form, "Search", "https://mcp example.com", "").is_err());
        assert!(mcp_save_action(&form, "Search", "https://%", "").is_err());
        assert!(
            mcp_save_action(&form, "Search", "https://user:secret@tools.example/mcp", "").is_err()
        );
        assert_eq!(
            mcp_save_action(&form, " Search ", " https://tools.example/mcp ", ""),
            Ok(SettingsAction::McpSaveRequested {
                server_id: Some("search-server".into()),
                draft: SettingsMcpDraft {
                    name: "Search".into(),
                    connection: SettingsMcpConnection::Http {
                        url: "https://tools.example/mcp".into(),
                    },
                    enabled: false,
                    scope: SettingsMcpScope::Workspace,
                },
            })
        );
    }

    #[test]
    fn local_mcp_arguments_remain_distinct_and_literal() {
        let form = McpForm {
            server_id: None,
            transport: FormTransport::Local,
            enabled: true,
            scope: SettingsMcpScope::Global,
            error: None,
            submission_pending: false,
        };
        assert_eq!(
            mcp_save_action(
                &form,
                "Tools",
                " /usr/local/bin/mcp ",
                " --workspace\nA project with spaces\n\n--read-only "
            ),
            Ok(SettingsAction::McpSaveRequested {
                server_id: None,
                draft: SettingsMcpDraft {
                    name: "Tools".into(),
                    connection: SettingsMcpConnection::Local {
                        command: "/usr/local/bin/mcp".into(),
                        args: vec![
                            "--workspace".into(),
                            "A project with spaces".into(),
                            "--read-only".into()
                        ],
                    },
                    enabled: true,
                    scope: SettingsMcpScope::Global,
                },
            })
        );
    }

    #[test]
    fn disabled_mcp_server_never_appears_connected() {
        let server = SettingsMcpServerView {
            id: "tools".into(),
            name: "Tools".into(),
            description: "".into(),
            connection: SettingsMcpConnection::Local {
                command: "mcp".into(),
                args: vec![],
            },
            status: SettingsMcpStatus::Running,
            enabled: false,
            scope: SettingsMcpScope::Workspace,
            tool_count: Some(4),
            status_detail: Some("Connected · 4 tools".into()),
        };
        assert_eq!(SettingsScreen::server_status_label(&server).0, "Disabled");
    }
}
