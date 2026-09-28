# Settings and billing UI contract

The `settings` story is one native GPUI preview. Run
`cargo run -p fanta-gpui-storybook`, then open **Settings** in the Screens
group. Its vertical sidebar includes Credits & billing, AI usage, Plans, and
Activity in the same window. The story uses in-memory sample data and does not
read or change a Fanta account.

The Storybook knobs expose connected, dashboard Free, empty, error, signed-out,
loading, and purchase-unavailable examples for visual review.

## Settings

Mount `SettingsScreen` with `SettingsViewData`. The sidebar and searchable
preference rows are populated from `pages`; the account summary, usage summary,
MCP connections, and billing snapshot come from separate fields. The reusable
screen owns transient search, focus, expanded navigation, active section, and
MCP form state. `SettingsAction` gives the host the setting ID and proposed
value, page selection, billing request, or MCP operation. After validation and
persistence, echo the accepted snapshot with `set_view_data`. Set
`settings_file_available` only when the host can open Fanta's advanced settings
file; the window then shows one top-right action for it.

The MCP form supports local executables with literal arguments and remote HTTP
URLs, workspace or global scope, and enabled state. The host must validate
permissions and connection details, start or stop the server, store dedicated
authentication secrets outside the view model, and supply connection status
and tool counts. Executable arguments and URL query strings can still contain
sensitive data if entered by a user; hosts should validate, store, and redact
them accordingly. `McpSaveRequested` contains the connection draft;
`McpActionRequested` carries toggle, authenticate, retry, or remove intents.

`PageSelected` changes the right pane while keeping the sidebar mounted.
`WorkspaceRequested` and `ApiKeysRequested` open their corresponding account
destinations supplied by the host.

## Credits and billing

The Settings screen embeds `BillingScreen` with a `BillingViewData` snapshot
for the active workspace. Supply formatted credit amounts, localized prices,
current plan, checkout availability, 7/30/90-day usage points, model and
API-key breakdowns, transactions, and receipt availability. Settings maps its
vertical Billing, Usage, Plans, and Activity destinations to the selected
billing tab. `UsageRangeSelected` requests a new snapshot.

The UI emits `PlanRequested`, `AddCreditsRequested`,
`ManageSubscriptionRequested`, `RestorePurchasesRequested`,
`RedeemCodeRequested`, `SyncPurchasesRequested`, `ReceiptRequested`,
`ExportUsageRequested`, and `RefreshRequested` through
`SettingsAction::BillingActionRequested`. The Fanta Edit host will
resolve these through its account and purchase services and then echo updated
data. If a flow is unavailable, set its `BillingActionState` to disabled and
provide the reason. Use `BillingLoadState` for loading and recoverable errors.
The UI does not claim a successful purchase or change the displayed balance
until the host confirms it.

## Dashboard correspondence

The current web dashboard has separate Usage, Billing, Organization, and API
keys destinations. Settings brings credit balance, subscription, usage,
plans, and receipts into one native workflow. The Storybook fixtures model
those surfaces without coupling the component crate to dashboard APIs. MCP
configuration is a native Settings surface for the later Fanta Edit host
integration.

The balance fixture represents a shared workspace pool. Plan grants and credit
packs add to the balance; completed usage reduces it. The 7/30/90-day controls
change the usage period, not the balance's lifetime.

## Zed UI basis

The window adapts Zed's settings layout, tree navigation, and section-row
structure from [`settings_ui` at commit `8b08b0607b`](https://github.com/zed-industries/zed/blob/8b08b0607b/crates/settings_ui/src/settings_ui.rs)
and its `components/section_items.rs`. It uses the Zed `ui::TreeViewItem`
primitive already in this repository. Zed's complete settings crate depends on
editor, project, agent, and client services, so this screen retains Fanta's
host-supplied view data and typed intents.
