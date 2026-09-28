//! Host-owned settings data and typed requests emitted by the settings screen.

use gpui::SharedString;

use crate::screens::billing::{BillingAction, BillingTab, BillingViewData};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SettingsPage {
    #[default]
    General,
    Appearance,
    Canvas,
    Editor,
    AiModels,
    McpTools,
    Account,
    Privacy,
    Billing,
    Usage,
    Plans,
    Activity,
}

impl SettingsPage {
    pub const ALL: [Self; 12] = [
        Self::General,
        Self::Appearance,
        Self::Canvas,
        Self::Editor,
        Self::Privacy,
        Self::AiModels,
        Self::McpTools,
        Self::Account,
        Self::Billing,
        Self::Usage,
        Self::Plans,
        Self::Activity,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Canvas => "Canvas & layout",
            Self::Editor => "Code editor",
            Self::AiModels => "AI & models",
            Self::McpTools => "MCP & tools",
            Self::Account => "Account & workspace",
            Self::Privacy => "Privacy & data",
            Self::Billing => "Credits & billing",
            Self::Usage => "AI usage",
            Self::Plans => "Plans",
            Self::Activity => "Activity",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::General => "Startup, files, and everyday behavior",
            Self::Appearance => "Theme, type, density, and icons",
            Self::Canvas => "Design canvas, panels, and layout",
            Self::Editor => "Source editing and keyboard preferences",
            Self::AiModels => "Models, providers, and agent behavior",
            Self::McpTools => "Connected tools and server permissions",
            Self::Account => "Profile, workspace, billing, and usage",
            Self::Privacy => "Data controls and account security",
            Self::Billing => "Your credit balance and subscription",
            Self::Usage => "AI usage and credit consumption",
            Self::Plans => "Plan details and available options",
            Self::Activity => "Purchases, credit grants, and receipts",
        }
    }

    pub const fn billing_tab(self) -> Option<BillingTab> {
        match self {
            Self::Billing => Some(BillingTab::Overview),
            Self::Usage => Some(BillingTab::Usage),
            Self::Plans => Some(BillingTab::Plans),
            Self::Activity => Some(BillingTab::History),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsChoice {
    pub value: SharedString,
    pub label: SharedString,
}

impl SettingsChoice {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SettingsTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingsControl {
    Toggle(bool),
    Choice {
        selected: SharedString,
        options: Vec<SettingsChoice>,
    },
    Action {
        label: SharedString,
    },
    Status {
        label: SharedString,
        tone: SettingsTone,
    },
    /// A credential is represented only by configuration state. Secret values
    /// never enter this view model or the emitted intent.
    Credential {
        configured: bool,
        action_label: SharedString,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsItem {
    pub id: SharedString,
    pub title: SharedString,
    pub description: SharedString,
    pub control: SettingsControl,
}

impl SettingsItem {
    pub fn new(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        control: SettingsControl,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            control,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsGroup {
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub items: Vec<SettingsItem>,
}

impl SettingsGroup {
    pub fn new(title: impl Into<SharedString>, items: Vec<SettingsItem>) -> Self {
        Self {
            title: title.into(),
            description: None,
            items,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsPageData {
    pub page: SettingsPage,
    pub description: SharedString,
    pub groups: Vec<SettingsGroup>,
}

impl SettingsPageData {
    pub fn new(page: SettingsPage, groups: Vec<SettingsGroup>) -> Self {
        Self {
            page,
            description: page.description().into(),
            groups,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsAccountSummary {
    pub display_name: SharedString,
    pub email: SharedString,
    pub plan: SharedString,
    pub workspace: SharedString,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsUsageSummary {
    pub available: SharedString,
    pub used: SharedString,
    pub period: SharedString,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsMcpStatus {
    Running,
    Stopped,
    Connecting,
    AuthenticationRequired,
    Error,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SettingsMcpScope {
    #[default]
    Workspace,
    Global,
}

impl SettingsMcpScope {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Workspace => "Workspace",
            Self::Global => "All workspaces",
        }
    }
}

/// Executable arguments and URL query strings can contain sensitive data.
/// Hosts should supply redacted display values, validate emitted drafts, and
/// store credentials separately. The form rejects URL userinfo but cannot
/// determine whether arbitrary arguments or query parameters contain secrets.
#[derive(Clone, Debug, PartialEq)]
pub enum SettingsMcpConnection {
    Local {
        command: SharedString,
        args: Vec<SharedString>,
    },
    Http {
        url: SharedString,
    },
}

impl SettingsMcpConnection {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Local { .. } => "Local process",
            Self::Http { .. } => "Remote HTTP",
        }
    }

    pub fn endpoint(&self) -> &SharedString {
        match self {
            Self::Local { command, .. } => command,
            Self::Http { url } => url,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsMcpServerView {
    pub id: SharedString,
    pub name: SharedString,
    pub description: SharedString,
    pub connection: SettingsMcpConnection,
    pub status: SettingsMcpStatus,
    pub enabled: bool,
    pub scope: SettingsMcpScope,
    pub tool_count: Option<u32>,
    pub status_detail: Option<SharedString>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsMcpDraft {
    pub name: SharedString,
    pub connection: SettingsMcpConnection,
    pub enabled: bool,
    pub scope: SettingsMcpScope,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsViewData {
    pub selected_page: SettingsPage,
    /// The scope shown at the top of the content pane, e.g. "User settings".
    pub scope: SharedString,
    /// Whether the host can open an editable settings.json file.
    pub settings_file_available: bool,
    pub pages: Vec<SettingsPageData>,
    pub mcp_servers: Vec<SettingsMcpServerView>,
    pub account: Option<SettingsAccountSummary>,
    pub usage: Option<SettingsUsageSummary>,
    /// Workspace billing snapshot for the four in-window billing destinations.
    pub billing: Option<BillingViewData>,
    pub notice: Option<SharedString>,
}

impl Default for SettingsViewData {
    fn default() -> Self {
        Self {
            selected_page: SettingsPage::General,
            scope: "User settings".into(),
            settings_file_available: false,
            pages: Vec::new(),
            mcp_servers: Vec::new(),
            account: None,
            usage: None,
            billing: None,
            notice: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsMcpAction {
    Remove,
    Authenticate,
    Retry,
    ToggleEnabled(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingsAction {
    PageSelected(SettingsPage),
    ToggleRequested {
        id: SharedString,
        value: bool,
    },
    ChoiceRequested {
        id: SharedString,
        value: SharedString,
    },
    ActionRequested {
        id: SharedString,
    },
    BillingActionRequested(BillingAction),
    SignInRequested,
    WorkspaceRequested,
    ApiKeysRequested,
    McpSaveRequested {
        server_id: Option<SharedString>,
        draft: SettingsMcpDraft,
    },
    McpActionRequested {
        server_id: SharedString,
        action: SettingsMcpAction,
    },
}
