//! Mock Fanta host for the reusable settings window.

use super::billing::{BillingNamedState, seed_billing, update_usage_range};
use super::knobs::{self, KnobOption};
use crate::*;
use fanta_gpui::billing::{BillingAction, BillingTab};
use fanta_gpui::settings::{
    SettingsAccountSummary, SettingsAction, SettingsChoice, SettingsControl, SettingsGroup,
    SettingsItem, SettingsMcpAction, SettingsMcpConnection, SettingsMcpDraft, SettingsMcpScope,
    SettingsMcpServerView, SettingsMcpStatus, SettingsPage, SettingsPageData, SettingsScreen,
    SettingsUsageSummary, SettingsViewData,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SettingsNamedState {
    #[default]
    Ready,
    NoMcpServers,
    McpError,
    SignedOut,
}

impl SettingsNamedState {
    const ALL: [Self; 4] = [
        Self::Ready,
        Self::NoMcpServers,
        Self::McpError,
        Self::SignedOut,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Ready => "Connected workspace",
            Self::NoMcpServers => "No MCP servers",
            Self::McpError => "MCP needs attention",
            Self::SignedOut => "Signed out",
        }
    }

    fn fixture(self) -> SettingsViewData {
        let mut data = seed_settings();
        match self {
            Self::Ready => {}
            Self::NoMcpServers => {
                data.selected_page = SettingsPage::McpTools;
                data.mcp_servers.clear();
            }
            Self::McpError => {
                data.selected_page = SettingsPage::McpTools;
                if let Some(server) = data.mcp_servers.first_mut() {
                    server.status = SettingsMcpStatus::Error;
                    server.status_detail = Some("The local process exited unexpectedly".into());
                    server.tool_count = None;
                }
            }
            Self::SignedOut => {
                data.selected_page = SettingsPage::Account;
                data.account = None;
                data.usage = None;
                data.billing = None;
            }
        }
        data
    }
}

fn fixture_with_billing_state(state: BillingNamedState) -> SettingsViewData {
    let mut data = SettingsNamedState::Ready.fixture();
    data.selected_page = SettingsPage::Billing;
    let billing = state.fixture();
    if let Some(account) = data.account.as_mut() {
        account.plan = billing.subscription.plan_name.clone();
        account.workspace = billing.workspace.name.clone();
    }
    if matches!(
        state,
        BillingNamedState::DashboardFree | BillingNamedState::Empty
    ) && let Some(usage) = data.usage.as_mut()
    {
        usage.available = format!("{} credits", billing.balance.available_label).into();
        usage.used = "0 credits".into();
    }
    data.billing = Some(billing);
    data
}

fn toggle(
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    value: bool,
) -> SettingsItem {
    SettingsItem::new(id, title, detail, SettingsControl::Toggle(value))
}

fn choice(
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    selected: &'static str,
    options: &[(&'static str, &'static str)],
) -> SettingsItem {
    SettingsItem::new(
        id,
        title,
        detail,
        SettingsControl::Choice {
            selected: selected.into(),
            options: options
                .iter()
                .map(|(value, label)| SettingsChoice::new(*value, *label))
                .collect(),
        },
    )
}

fn action(
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    label: &'static str,
) -> SettingsItem {
    SettingsItem::new(
        id,
        title,
        detail,
        SettingsControl::Action {
            label: label.into(),
        },
    )
}

pub(crate) fn seed_settings() -> SettingsViewData {
    use SettingsPage::*;

    SettingsViewData {
        selected_page: General,
        scope: "User".into(),
        settings_file_available: true,
        pages: vec![
            SettingsPageData::new(
                General,
                vec![SettingsGroup::new(
                    "Startup & files",
                    vec![
                        choice(
                            "startup_view",
                            "On new window",
                            "Choose what appears when a new Fanta window opens.",
                            "home",
                            &[
                                ("home", "Home"),
                                ("last", "Last project"),
                                ("blank", "Blank canvas"),
                            ],
                        ),
                        toggle(
                            "autosave",
                            "Autosave",
                            "Save edited files when focus moves away.",
                            true,
                        ),
                        toggle(
                            "confirm_quit",
                            "Confirm before quitting",
                            "Ask before closing Fanta with unfinished work.",
                            true,
                        ),
                        toggle(
                            "native_dialogs",
                            "Use system file dialogs",
                            "Use macOS Open and Save panels.",
                            true,
                        ),
                    ],
                )],
            ),
            SettingsPageData::new(
                Appearance,
                vec![SettingsGroup::new(
                    "Interface",
                    vec![
                        choice(
                            "theme",
                            "Color theme",
                            "Follow the system or choose a theme for Fanta.",
                            "system",
                            &[("system", "System"), ("dark", "Dark"), ("light", "Light")],
                        ),
                        choice(
                            "density",
                            "Interface density",
                            "Control the spacing of toolbars, lists, and panels.",
                            "comfortable",
                            &[("comfortable", "Comfortable"), ("compact", "Compact")],
                        ),
                        choice(
                            "ui_scale",
                            "Interface scale",
                            "Adjust text and controls throughout the app.",
                            "100",
                            &[
                                ("90", "90%"),
                                ("100", "100%"),
                                ("110", "110%"),
                                ("125", "125%"),
                            ],
                        ),
                        toggle(
                            "reduced_motion",
                            "Reduce motion",
                            "Minimize decorative interface animation.",
                            false,
                        ),
                    ],
                )],
            ),
            SettingsPageData::new(
                Canvas,
                vec![
                    SettingsGroup::new(
                        "Canvas behavior",
                        vec![
                            toggle(
                                "snap_objects",
                                "Snap to objects",
                                "Align objects to nearby edges and centers while moving.",
                                true,
                            ),
                            toggle(
                                "pixel_grid",
                                "Snap to pixel grid",
                                "Keep vector bounds aligned to device pixels.",
                                false,
                            ),
                            toggle(
                                "show_rulers",
                                "Show rulers",
                                "Display rulers around the active canvas.",
                                false,
                            ),
                        ],
                    ),
                    SettingsGroup::new(
                        "Panels",
                        vec![
                            choice(
                                "sidebar_side",
                                "Inspector position",
                                "Place the properties inspector on either edge.",
                                "right",
                                &[("right", "Right"), ("left", "Left")],
                            ),
                            toggle(
                                "restore_panels",
                                "Restore panel layout",
                                "Reopen the panels that were visible in the last session.",
                                true,
                            ),
                        ],
                    ),
                ],
            ),
            SettingsPageData::new(
                Editor,
                vec![
                    SettingsGroup::new(
                        "Source editor",
                        vec![
                            choice(
                                "editor_font_size",
                                "Font size",
                                "Size of code and structured text in editor tabs.",
                                "13",
                                &[
                                    ("12", "12 pt"),
                                    ("13", "13 pt"),
                                    ("14", "14 pt"),
                                    ("16", "16 pt"),
                                ],
                            ),
                            toggle(
                                "line_numbers",
                                "Line numbers",
                                "Show line numbers in source editing views.",
                                true,
                            ),
                            toggle(
                                "word_wrap",
                                "Word wrap",
                                "Wrap long lines to fit the editor width.",
                                false,
                            ),
                        ],
                    ),
                    SettingsGroup::new(
                        "Keyboard",
                        vec![action(
                            "keymap",
                            "Keyboard shortcuts",
                            "Review and customize commands for design and code work.",
                            "Open keymap",
                        )],
                    ),
                ],
            ),
            SettingsPageData::new(
                AiModels,
                vec![
                    SettingsGroup::new(
                        "Model selection",
                        vec![
                            choice(
                                "default_model",
                                "Default assistant model",
                                "Used when a new AI conversation starts.",
                                "auto",
                                &[("auto", "Fanta Auto"), ("claude", "Claude"), ("gpt", "GPT")],
                            ),
                            toggle(
                                "model_fallback",
                                "Automatic fallback",
                                "Use another available model if your selection is unavailable.",
                                true,
                            ),
                            SettingsItem::new(
                                "provider_key",
                                "Bring your own API key",
                                "Connect a provider key for models billed by that provider.",
                                SettingsControl::Credential {
                                    configured: false,
                                    action_label: "Configure".into(),
                                },
                            ),
                        ],
                    ),
                    SettingsGroup::new(
                        "Agent permissions",
                        vec![
                            toggle(
                                "confirm_tool_calls",
                                "Confirm tool use",
                                "Review actions before an agent changes files or calls connected tools.",
                                true,
                            ),
                            toggle(
                                "allow_project_context",
                                "Use project context",
                                "Allow the assistant to inspect the active project when prompted.",
                                true,
                            ),
                        ],
                    ),
                ],
            ),
            SettingsPageData::new(
                McpTools,
                vec![SettingsGroup::new(
                    "Tool access",
                    vec![
                        toggle(
                            "mcp_enabled",
                            "Enable MCP tools",
                            "Make trusted MCP servers available to the assistant.",
                            true,
                        ),
                        toggle(
                            "mcp_confirm_write",
                            "Confirm write actions",
                            "Review tool calls that can change files or remote data.",
                            true,
                        ),
                    ],
                )],
            ),
            SettingsPageData::new(
                Account,
                vec![SettingsGroup::new(
                    "Workspace",
                    vec![
                        action(
                            "workspace_details",
                            "Workspace settings",
                            "Manage your team, members, and workspace preferences.",
                            "Manage",
                        ),
                        action(
                            "api_keys",
                            "API keys",
                            "Create scoped keys for Fanta API access.",
                            "Open keys",
                        ),
                    ],
                )],
            ),
            SettingsPageData::new(
                Privacy,
                vec![SettingsGroup::new(
                    "Data & security",
                    vec![
                        toggle(
                            "usage_analytics",
                            "Share anonymous diagnostics",
                            "Help improve stability with crash and performance reports.",
                            false,
                        ),
                        toggle(
                            "keep_generation_history",
                            "Keep generation history",
                            "Show recent AI creations in your workspace gallery.",
                            true,
                        ),
                        action(
                            "manage_account",
                            "Account security",
                            "Review sign-in and account access in your Fanta account.",
                            "Manage account",
                        ),
                    ],
                )],
            ),
        ],
        mcp_servers: vec![
            SettingsMcpServerView {
                id: "example-local".into(),
                name: "Example local server".into(),
                description: "Sample workspace tool connection for this Storybook preview.".into(),
                connection: SettingsMcpConnection::Local {
                    command: "/path/to/mcp-server".into(),
                    args: vec!["--workspace".into()],
                },
                status: SettingsMcpStatus::Running,
                enabled: true,
                scope: SettingsMcpScope::Workspace,
                tool_count: Some(12),
                status_detail: Some("Connected · 12 tools".into()),
            },
            SettingsMcpServerView {
                id: "fanta-assets".into(),
                name: "Fanta Assets".into(),
                description: "Search approved workspace assets and media.".into(),
                connection: SettingsMcpConnection::Http {
                    url: "https://mcp.example.com/fanta".into(),
                },
                status: SettingsMcpStatus::AuthenticationRequired,
                enabled: true,
                scope: SettingsMcpScope::Workspace,
                tool_count: None,
                status_detail: Some("Authentication required".into()),
            },
        ],
        account: Some(SettingsAccountSummary {
            display_name: "Alex Morgan".into(),
            email: "alex@example.com".into(),
            plan: "Pro".into(),
            workspace: "Fanta Studio".into(),
        }),
        usage: Some(SettingsUsageSummary {
            available: "2,340 credits".into(),
            used: "1,160 credits".into(),
            period: "Last 30 days".into(),
        }),
        billing: Some(seed_billing()),
        notice: None,
    }
}

pub(crate) struct SettingsStory {
    pub(crate) screen: Entity<SettingsScreen>,
    pub(crate) view_data: SettingsViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: SettingsNamedState,
    pub(crate) billing_state: BillingNamedState,
    next_mcp_id: u64,
}

impl SettingsStory {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let named_state = SettingsNamedState::default();
        let view_data = named_state.fixture();
        let screen =
            cx.new(|cx| SettingsScreen::new("storybook-settings", view_data.clone(), window, cx));
        Self {
            screen,
            view_data,
            last_action: "Ready — search settings, change a preference, or configure an MCP server"
                .into(),
            named_state,
            billing_state: BillingNamedState::default(),
            next_mcp_id: 1,
        }
    }

    fn apply_named_state(&mut self, state: SettingsNamedState, cx: &mut Context<Storybook>) {
        self.named_state = state;
        self.billing_state = BillingNamedState::default();
        self.view_data = state.fixture();
        self.next_mcp_id = 1;
        let data = self.view_data.clone();
        self.screen
            .update(cx, |screen, cx| screen.set_view_data(data, cx));
        self.last_action = format!("Story state: {}", state.label()).into();
        cx.notify();
    }

    fn apply_billing_named_state(&mut self, state: BillingNamedState, cx: &mut Context<Storybook>) {
        self.named_state = SettingsNamedState::Ready;
        self.billing_state = state;
        self.view_data = fixture_with_billing_state(state);
        let data = self.view_data.clone();
        self.screen
            .update(cx, |screen, cx| screen.set_view_data(data, cx));
        self.last_action = format!("Billing state: {}", state.label()).into();
        cx.notify();
    }

    fn apply(&mut self, action: &SettingsAction, cx: &mut Context<Storybook>) {
        self.last_action = match action {
            SettingsAction::McpSaveRequested { .. } => "McpSaveRequested".into(),
            _ => format!("{action:?}").into(),
        };
        self.view_data.notice = None;
        match action {
            SettingsAction::PageSelected(page) => self.view_data.selected_page = *page,
            SettingsAction::ToggleRequested { id, value } => {
                self.for_item_mut(id, |item| item.control = SettingsControl::Toggle(*value));
            }
            SettingsAction::ChoiceRequested { id, value } => {
                self.for_item_mut(id, |item| {
                    if let SettingsControl::Choice { selected, .. } = &mut item.control {
                        *selected = value.clone();
                    }
                });
            }
            SettingsAction::ActionRequested { id } => {
                self.view_data.notice = Some(format!("{id} requested from the host").into());
            }
            SettingsAction::McpSaveRequested { server_id, draft } => {
                self.save_mcp(server_id.as_ref(), draft);
                self.screen
                    .update(cx, |screen, cx| screen.dismiss_mcp_form(cx));
            }
            SettingsAction::McpActionRequested { server_id, action } => {
                self.update_mcp(server_id, *action);
            }
            SettingsAction::WorkspaceRequested => {
                self.view_data.notice = Some("Workspace management requested from the host".into());
            }
            SettingsAction::ApiKeysRequested => {
                self.view_data.notice = Some("API key management requested from the host".into());
            }
            SettingsAction::SignInRequested => {
                self.view_data.notice = Some("Sign-in requested from the host".into());
            }
            SettingsAction::BillingActionRequested(billing_action) => {
                self.apply_billing_action(billing_action);
            }
        }
        self.screen.update(cx, |screen, cx| {
            screen.set_view_data(self.view_data.clone(), cx)
        });
        cx.notify();
    }

    fn apply_billing_action(&mut self, action: &BillingAction) {
        match action {
            BillingAction::TabSelected(tab) => {
                self.view_data.selected_page = match tab {
                    BillingTab::Overview => SettingsPage::Billing,
                    BillingTab::Usage => SettingsPage::Usage,
                    BillingTab::Plans => SettingsPage::Plans,
                    BillingTab::History => SettingsPage::Activity,
                };
            }
            BillingAction::UsageRangeSelected(range) => {
                if let Some(billing) = self.view_data.billing.as_mut() {
                    update_usage_range(billing, *range);
                }
            }
            BillingAction::RefreshRequested => {
                if let Some(billing) = self.view_data.billing.as_mut() {
                    billing.balance.refresh_label = "Updated just now".into();
                }
            }
            BillingAction::PlanRequested { .. }
            | BillingAction::AddCreditsRequested
            | BillingAction::ManageSubscriptionRequested
            | BillingAction::RestorePurchasesRequested
            | BillingAction::RedeemCodeRequested
            | BillingAction::SyncPurchasesRequested
            | BillingAction::ReceiptRequested { .. }
            | BillingAction::ExportUsageRequested => {}
        }
    }

    fn for_item_mut(&mut self, id: &SharedString, f: impl FnOnce(&mut SettingsItem)) {
        if let Some(item) = self
            .view_data
            .pages
            .iter_mut()
            .flat_map(|page| &mut page.groups)
            .flat_map(|group| &mut group.items)
            .find(|item| &item.id == id)
        {
            f(item);
        }
    }

    fn save_mcp(&mut self, server_id: Option<&SharedString>, draft: &SettingsMcpDraft) {
        let (status, status_detail, tool_count) = match &draft.connection {
            SettingsMcpConnection::Local { .. } => (
                SettingsMcpStatus::Running,
                Some("Connected · 4 tools".into()),
                Some(4),
            ),
            SettingsMcpConnection::Http { .. } => (
                SettingsMcpStatus::AuthenticationRequired,
                Some("Authentication required".into()),
                None,
            ),
        };
        if let Some(server) = server_id.and_then(|id| {
            self.view_data
                .mcp_servers
                .iter_mut()
                .find(|server| &server.id == id)
        }) {
            server.name = draft.name.clone();
            server.connection = draft.connection.clone();
            server.enabled = draft.enabled;
            server.scope = draft.scope;
            server.status = status;
            server.status_detail = status_detail;
            server.tool_count = tool_count;
        } else {
            let id = self.next_mcp_id;
            self.next_mcp_id += 1;
            self.view_data.mcp_servers.push(SettingsMcpServerView {
                id: format!("storybook-server-{id}").into(),
                name: draft.name.clone(),
                description: "Custom MCP server".into(),
                connection: draft.connection.clone(),
                status,
                enabled: draft.enabled,
                scope: draft.scope,
                tool_count,
                status_detail,
            });
        }
    }

    fn update_mcp(&mut self, server_id: &SharedString, action: SettingsMcpAction) {
        if action == SettingsMcpAction::Remove {
            self.view_data
                .mcp_servers
                .retain(|server| &server.id != server_id);
            return;
        }
        if let Some(server) = self
            .view_data
            .mcp_servers
            .iter_mut()
            .find(|server| &server.id == server_id)
        {
            match action {
                SettingsMcpAction::ToggleEnabled(enabled) => server.enabled = enabled,
                SettingsMcpAction::Authenticate | SettingsMcpAction::Retry => {
                    server.status = SettingsMcpStatus::Running;
                    server.status_detail = Some("Connected · 4 tools".into());
                    server.tool_count = Some(4);
                }
                SettingsMcpAction::Remove => {}
            }
        }
    }
}

impl Storybook {
    pub(crate) fn render_settings_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "settings-story-knobs",
            vec![
                knobs::enum_knob_row(
                    "settings-knob-state",
                    "SETTINGS STATE",
                    SettingsNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                    self.settings_story.named_state,
                    |this, state, _, cx| this.settings_story.apply_named_state(state, cx),
                    cx,
                ),
                knobs::enum_knob_row(
                    "settings-billing-knob-state",
                    "BILLING STATE",
                    BillingNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                    self.settings_story.billing_state,
                    |this, state, _, cx| this.settings_story.apply_billing_named_state(state, cx),
                    cx,
                ),
            ],
            cx,
        )
    }

    pub(crate) fn handle_settings_action(
        &mut self,
        action: &SettingsAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_story.apply(action, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_states_exercise_empty_error_and_signed_out_surfaces() {
        let empty = SettingsNamedState::NoMcpServers.fixture();
        assert_eq!(empty.selected_page, SettingsPage::McpTools);
        assert!(empty.mcp_servers.is_empty());

        let error = SettingsNamedState::McpError.fixture();
        assert!(
            error
                .mcp_servers
                .iter()
                .any(|server| server.status == SettingsMcpStatus::Error)
        );

        let signed_out = SettingsNamedState::SignedOut.fixture();
        assert_eq!(signed_out.selected_page, SettingsPage::Account);
        assert!(signed_out.account.is_none());
        assert!(signed_out.usage.is_none());
        assert!(signed_out.billing.is_none());
    }

    #[test]
    fn dashboard_free_billing_state_keeps_account_and_balance_in_sync() {
        let data = fixture_with_billing_state(BillingNamedState::DashboardFree);
        assert_eq!(data.selected_page, SettingsPage::Billing);
        let account = data.account.expect("signed-in dashboard fixture");
        assert_eq!(account.plan, "Free");
        assert_eq!(account.workspace, "Personal workspace");
        let usage = data.usage.expect("dashboard balance summary");
        assert_eq!(usage.available, "500 credits");
        assert_eq!(usage.used, "0 credits");
        assert_eq!(
            data.billing
                .expect("billing fixture")
                .balance
                .available_label,
            "500"
        );
    }
}
