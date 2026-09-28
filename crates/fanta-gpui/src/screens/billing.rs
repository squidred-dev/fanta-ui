//! Host-controlled credits, subscription, and AI usage screen.
//!
//! Amounts, localized prices, entitlements, purchase availability, usage, and
//! receipts come from the host. This screen presents that snapshot and emits
//! requests; it never starts a purchase or changes a balance on its own.

use gpui::{
    AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, Hsla, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
    relative,
};
use gpui_component::{Theme as ComponentTheme, tooltip::Tooltip};
use ui::StyledTypography as _;

use crate::atoms::{
    CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticButtonIconAlignment,
    SemanticButtonSize, SemanticButtonVariant, TypographyExt as _, TypographyToken,
    render_lucide_icon, semantic_button,
};

/// The narrowest useful width for this full-screen billing workflow.
pub const BILLING_SCREEN_MIN_WIDTH: f32 = 480.;
pub const BILLING_SCREEN_MIN_HEIGHT: f32 = 420.;

/// The selected billing destination is controlled by the host so another
/// surface can deep-link directly to AI usage or receipts.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BillingTab {
    #[default]
    Overview,
    Usage,
    Plans,
    History,
}

impl BillingTab {
    pub const ALL: [Self; 4] = [Self::Overview, Self::Usage, Self::Plans, Self::History];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Usage => "AI usage",
            Self::Plans => "Plans",
            Self::History => "Activity",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Usage => "usage",
            Self::Plans => "plans",
            Self::History => "history",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BillingUsageRange {
    #[default]
    Days7,
    Days30,
    Days90,
}

impl BillingUsageRange {
    pub const ALL: [Self; 3] = [Self::Days7, Self::Days30, Self::Days90];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Days7 => "7 days",
            Self::Days30 => "30 days",
            Self::Days90 => "90 days",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Days7 => "7d",
            Self::Days30 => "30d",
            Self::Days90 => "90d",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BillingLoadState {
    #[default]
    Ready,
    Loading,
    Error(SharedString),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BillingProvider {
    Apple,
    Web,
    #[default]
    None,
}

/// Text and availability are supplied by the host. In particular, an
/// unavailable checkout is never silently rendered as a working purchase.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingActionState {
    pub label: SharedString,
    pub enabled: bool,
    pub disabled_reason: Option<SharedString>,
}

impl BillingActionState {
    pub fn enabled(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            disabled_reason: None,
        }
    }

    pub fn disabled(label: impl Into<SharedString>, reason: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            enabled: false,
            disabled_reason: Some(reason.into()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingWorkspace {
    pub id: SharedString,
    pub name: SharedString,
    pub role_label: SharedString,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingBalance {
    /// Host-formatted quantity, for example "2,480".
    pub available_label: SharedString,
    pub available_caption: SharedString,
    pub included_label: SharedString,
    pub purchased_label: SharedString,
    pub refresh_label: SharedString,
    pub note: SharedString,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingSubscription {
    pub plan_name: SharedString,
    pub status_label: SharedString,
    /// Localized price, including currency and tax wording when appropriate.
    pub price_label: SharedString,
    pub cadence_label: SharedString,
    pub renewal_label: SharedString,
    pub provider: BillingProvider,
    pub provider_note: SharedString,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingPlan {
    pub id: SharedString,
    pub name: SharedString,
    pub eyebrow: SharedString,
    pub description: SharedString,
    /// Host-formatted, localized price. Empty is valid for contact sales.
    pub price_label: SharedString,
    pub cadence_label: SharedString,
    pub credits_label: SharedString,
    pub features: Vec<SharedString>,
    pub highlighted: bool,
    pub current: bool,
    pub action: BillingActionState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BillingUsageKind {
    Image,
    Video,
    Audio,
    Vector,
    Chat,
    #[default]
    Other,
}

impl BillingUsageKind {
    const fn icon(self) -> LucideIcon {
        match self {
            Self::Image => LucideIcon::Image,
            Self::Video => LucideIcon::Video,
            Self::Audio => LucideIcon::AudioLines,
            Self::Vector => LucideIcon::Shapes,
            Self::Chat => LucideIcon::MessageSquare,
            Self::Other => LucideIcon::Sparkles,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BillingUsagePoint {
    pub label: SharedString,
    /// Host-formatted exact usage for tooltips and accessibility.
    pub amount_label: SharedString,
    /// Relative chart height in the range 0..=1. The host owns the scale.
    pub fraction: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BillingUsageCategory {
    pub id: SharedString,
    pub name: SharedString,
    pub detail: SharedString,
    pub amount_label: SharedString,
    /// Fraction of the selected period's usage.
    pub fraction: f32,
    pub kind: BillingUsageKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BillingUsageApiKey {
    pub id: SharedString,
    pub name: SharedString,
    pub detail: SharedString,
    pub amount_label: SharedString,
    pub fraction: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BillingUsage {
    pub range: BillingUsageRange,
    pub period_label: SharedString,
    pub total_label: SharedString,
    pub allowance_label: SharedString,
    pub reset_label: SharedString,
    /// Fraction of the plan allowance consumed during this period.
    pub allowance_fraction: f32,
    pub trend: Vec<BillingUsagePoint>,
    pub categories: Vec<BillingUsageCategory>,
    pub api_keys: Vec<BillingUsageApiKey>,
    pub note: SharedString,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingTransaction {
    pub id: SharedString,
    pub title: SharedString,
    pub detail: SharedString,
    pub date_label: SharedString,
    pub credits_label: SharedString,
    pub amount_label: SharedString,
    pub status_label: SharedString,
    pub receipt_action: Option<BillingActionState>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BillingActions {
    pub add_credits: BillingActionState,
    pub manage_subscription: BillingActionState,
    pub restore_purchases: BillingActionState,
    pub redeem_code: BillingActionState,
    pub sync_purchases: BillingActionState,
    pub export_usage: BillingActionState,
    pub refresh: BillingActionState,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BillingViewData {
    pub selected_tab: BillingTab,
    pub workspace: BillingWorkspace,
    pub load_state: BillingLoadState,
    pub balance: BillingBalance,
    pub subscription: BillingSubscription,
    pub plans: Vec<BillingPlan>,
    pub usage: BillingUsage,
    pub transactions: Vec<BillingTransaction>,
    pub actions: BillingActions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BillingAction {
    TabSelected(BillingTab),
    UsageRangeSelected(BillingUsageRange),
    PlanRequested { plan_id: SharedString },
    AddCreditsRequested,
    ManageSubscriptionRequested,
    RestorePurchasesRequested,
    RedeemCodeRequested,
    SyncPurchasesRequested,
    ReceiptRequested { transaction_id: SharedString },
    ExportUsageRequested,
    RefreshRequested,
}

#[derive(Clone, Copy)]
struct BillingColors {
    background: Hsla,
    panel: Hsla,
    surface: Hsla,
    hover: Hsla,
    selected: Hsla,
    border: Hsla,
    focus: Hsla,
    text: Hsla,
    muted: Hsla,
    accent: Hsla,
    positive: Hsla,
    danger: Hsla,
}

impl BillingColors {
    fn current(cx: &App) -> Self {
        if cx.try_global::<theme::GlobalTheme>().is_some() {
            let colors = theme::GlobalTheme::theme(cx).colors();
            Self {
                background: colors.editor_background,
                panel: colors.panel_background,
                surface: colors.surface_background,
                hover: colors.element_hover,
                selected: colors.element_selected,
                border: colors.border_variant,
                focus: colors.border_focused,
                text: colors.text,
                muted: colors.text_muted,
                accent: colors.text_accent,
                positive: theme::GlobalTheme::theme(cx).status().success,
                danger: theme::GlobalTheme::theme(cx).status().error,
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
                focus: colors.ring,
                text: colors.foreground,
                muted: colors.muted_foreground,
                accent: colors.primary,
                positive: colors.success,
                danger: colors.danger,
            }
        }
    }
}

fn fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., 1.)
    } else {
        0.
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BillingPresentation {
    Standalone,
    Embedded,
}

/// Presentation-only GPUI entity for one workspace's credits and billing.
pub struct BillingScreen {
    id: SharedString,
    data: BillingViewData,
    presentation: BillingPresentation,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    show_daily_values: bool,
}

impl EventEmitter<BillingAction> for BillingScreen {}

impl BillingScreen {
    pub fn new(id: impl Into<SharedString>, data: BillingViewData, cx: &mut Context<Self>) -> Self {
        Self::with_presentation(id, data, BillingPresentation::Standalone, cx)
    }

    /// Renders the selected billing destination inside a host content pane.
    /// The host owns the surrounding navigation, padding, background and scroll.
    /// `selected_tab` remains controlled through [`Self::set_view_data`].
    pub fn new_embedded(
        id: impl Into<SharedString>,
        data: BillingViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::with_presentation(id, data, BillingPresentation::Embedded, cx)
    }

    fn with_presentation(
        id: impl Into<SharedString>,
        data: BillingViewData,
        presentation: BillingPresentation,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            data,
            presentation,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            show_daily_values: false,
        }
    }

    pub fn view_data(&self) -> &BillingViewData {
        &self.data
    }

    pub fn set_view_data(&mut self, data: BillingViewData, cx: &mut Context<Self>) {
        if self.data != data {
            if self.presentation == BillingPresentation::Standalone
                && self.data.selected_tab != data.selected_tab
            {
                self.scroll_handle.set_offset(gpui::point(px(0.), px(0.)));
            }
            self.data = data;
            cx.notify();
        }
    }

    fn button(
        &self,
        suffix: impl AsRef<str>,
        action: &BillingActionState,
        variant: SemanticButtonVariant,
        icon: Option<LucideIcon>,
        intent: BillingAction,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen = cx.entity();
        let mut button = semantic_button(
            format!("{}-{}", self.id, suffix.as_ref()),
            variant,
            SemanticButtonSize::Large,
        )
        .label(action.label.clone())
        .disabled(!action.enabled);
        if let Some(icon) = icon {
            button = button.icon(icon, SemanticButtonIconAlignment::Left);
        }
        if action.enabled {
            button = button.on_activate(move |_, _, cx| {
                screen.update(cx, |_, cx| cx.emit(intent.clone()));
            });
        }
        button.into_any_element()
    }

    fn overline(text: impl Into<SharedString>, colors: BillingColors) -> AnyElement {
        div()
            .typography(TypographyToken::BodySmallStrong)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(colors.muted)
            .child(text.into())
            .into_any_element()
    }

    fn detail(text: impl Into<SharedString>, colors: BillingColors) -> AnyElement {
        div()
            .typography(TypographyToken::BodyMedium)
            .text_color(colors.muted)
            .child(text.into())
            .into_any_element()
    }

    fn heading(text: impl Into<SharedString>, colors: BillingColors) -> AnyElement {
        div()
            .typography(TypographyToken::HeadingMedium)
            .text_color(colors.text)
            .child(text.into())
            .into_any_element()
    }

    fn empty_state(
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        colors: BillingColors,
    ) -> AnyElement {
        div()
            .w_full()
            .min_h(px(170.))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(8.))
            .rounded(px(9.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .child(Self::heading(title, colors))
            .child(Self::detail(description, colors))
            .into_any_element()
    }

    fn render_tab(
        &self,
        tab: BillingTab,
        colors: BillingColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.data.selected_tab == tab;
        let screen = cx.entity();
        let selector = format!("billing-tab-{}", tab.key());
        div()
            .id(SharedString::from(format!("{}-tab-{}", self.id, tab.key())))
            .debug_selector(move || selector.clone())
            .role(Role::Tab)
            .aria_label(tab.label())
            .aria_selected(selected)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(40.))
            .px(px(13.))
            .flex()
            .items_center()
            .rounded(px(6.))
            .border_1()
            .border_color(if selected {
                colors.focus
            } else {
                colors.border.opacity(0.)
            })
            .bg(if selected {
                colors.selected
            } else {
                colors.panel
            })
            .text_color(if selected { colors.text } else { colors.muted })
            .typography(TypographyToken::BodyLargeStrong)
            .cursor_pointer()
            .hover(move |style| style.bg(colors.hover))
            .focus(move |style| style.border_color(colors.focus))
            .child(tab.label())
            .on_activate(move |_, _, cx| {
                screen.update(cx, |_, cx| cx.emit(BillingAction::TabSelected(tab)));
            })
            .into_any_element()
    }

    fn render_header(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        let workspace = &self.data.workspace;
        div()
            .w_full()
            .border_b_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .child(
                div()
                    .max_w(px(1180.))
                    .w_full()
                    .mx_auto()
                    .px(px(26.))
                    .pt(px(22.))
                    .flex()
                    .flex_col()
                    .gap(px(18.))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(5.))
                                    .child(
                                        div()
                                            .typography(TypographyToken::HeadingLarge)
                                            .text_color(colors.text)
                                            .child("Credits & billing"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(7.))
                                            .child(render_lucide_icon(
                                                LucideIcon::Sparkles,
                                                colors.accent,
                                                13.,
                                            ))
                                            .child(Self::detail(
                                                format!(
                                                    "{} · {}",
                                                    workspace.name, workspace.role_label
                                                ),
                                                colors,
                                            )),
                                    ),
                            )
                            .child(self.button(
                                "refresh",
                                &self.data.actions.refresh,
                                SemanticButtonVariant::Secondary,
                                Some(LucideIcon::RefreshCw),
                                BillingAction::RefreshRequested,
                                cx,
                            )),
                    )
                    .child(
                        div().flex().flex_wrap().gap(px(6.)).pb(px(12.)).children(
                            BillingTab::ALL
                                .into_iter()
                                .map(|tab| self.render_tab(tab, colors, cx)),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn render_status(&self, colors: BillingColors, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (title, detail, color) = match &self.data.load_state {
            BillingLoadState::Ready => return None,
            BillingLoadState::Loading => (
                "Updating billing data",
                "Your last available figures remain visible while we sync.",
                colors.accent,
            ),
            BillingLoadState::Error(message) => (
                "Billing data needs attention",
                message.as_ref(),
                colors.danger,
            ),
        };
        Some(
            div()
                .w_full()
                .min_w_0()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(12.))
                .p(px(14.))
                .rounded(px(8.))
                .border_1()
                .border_color(color.opacity(0.45))
                .bg(color.opacity(0.08))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .child(Self::heading(title, colors))
                        .child(Self::detail(detail.to_owned(), colors)),
                )
                .child(self.button(
                    "status-refresh",
                    &self.data.actions.refresh,
                    SemanticButtonVariant::Secondary,
                    Some(LucideIcon::RefreshCw),
                    BillingAction::RefreshRequested,
                    cx,
                ))
                .into_any_element(),
        )
    }

    fn render_balance(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        let balance = &self.data.balance;
        let mut footer = div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(10.));
        if !balance.refresh_label.is_empty() {
            footer = footer.child(Self::detail(balance.refresh_label.clone(), colors));
        }
        footer = footer.child(self.button(
            "add-credits",
            &self.data.actions.add_credits,
            SemanticButtonVariant::Primary,
            Some(LucideIcon::Plus),
            BillingAction::AddCreditsRequested,
            cx,
        ));

        div()
            .min_w(px(305.))
            .max_w_full()
            .flex_1()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(17.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(Self::overline("WORKSPACE BALANCE", colors))
                    .child(render_lucide_icon(LucideIcon::Sparkles, colors.accent, 18.)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(3.))
                    .child(
                        div()
                            .typography(TypographyToken::Display)
                            .text_color(colors.text)
                            .child(balance.available_label.clone()),
                    )
                    .child(Self::detail(balance.available_caption.clone(), colors)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(24.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(Self::overline("INCLUDED", colors))
                            .child(Self::heading(balance.included_label.clone(), colors)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(Self::overline("PURCHASED", colors))
                            .child(Self::heading(balance.purchased_label.clone(), colors)),
                    ),
            )
            .child(
                div()
                    .border_t_1()
                    .border_color(colors.border)
                    .pt(px(15.))
                    .child(footer),
            )
            .when(!balance.note.is_empty(), |card| {
                card.child(Self::detail(balance.note.clone(), colors))
            })
            .when_some(
                self.data.actions.add_credits.disabled_reason.clone(),
                |card, reason| card.child(Self::detail(reason, colors)),
            )
            .into_any_element()
    }

    fn render_subscription(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        let subscription = &self.data.subscription;
        div()
            .min_w(px(305.))
            .flex_1()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(Self::overline("CURRENT PLAN", colors))
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .rounded(px(99.))
                            .bg(colors.selected)
                            .typography(TypographyToken::BodySmallStrong)
                            .text_color(colors.text)
                            .child(subscription.status_label.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .typography(TypographyToken::HeadingLarge)
                            .text_color(colors.text)
                            .child(subscription.plan_name.clone()),
                    )
                    .when(!subscription.price_label.is_empty(), |group| {
                        group.child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(5.))
                                .child(Self::heading(subscription.price_label.clone(), colors))
                                .child(Self::detail(subscription.cadence_label.clone(), colors)),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(7.))
                    .when(!subscription.renewal_label.is_empty(), |group| {
                        group.child(Self::detail(subscription.renewal_label.clone(), colors))
                    })
                    .when(!subscription.provider_note.is_empty(), |group| {
                        group.child(Self::detail(subscription.provider_note.clone(), colors))
                    }),
            )
            .child(
                div()
                    .border_t_1()
                    .border_color(colors.border)
                    .pt(px(15.))
                    .child(self.button(
                        "manage-subscription",
                        &self.data.actions.manage_subscription,
                        SemanticButtonVariant::Secondary,
                        Some(LucideIcon::ExternalLink),
                        BillingAction::ManageSubscriptionRequested,
                        cx,
                    )),
            )
            .when_some(
                self.data
                    .actions
                    .manage_subscription
                    .disabled_reason
                    .clone(),
                |card, reason| card.child(Self::detail(reason, colors)),
            )
            .into_any_element()
    }

    fn render_subscription_row(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        let subscription = &self.data.subscription;
        div()
            .id(SharedString::from(format!("{}-plan-row", self.id)))
            .debug_selector(|| "billing-embedded-plan-row".to_owned())
            .role(Role::Group)
            .aria_label("Current plan")
            .w_full()
            .min_w_0()
            .py(px(17.))
            .border_t_1()
            .border_b_1()
            .border_color(colors.border)
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(16.))
            .child(
                div()
                    .flex_1()
                    .min_w(px(220.))
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(Self::overline("CURRENT PLAN", colors))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(9.))
                            .child(Self::heading(subscription.plan_name.clone(), colors))
                            .when(!subscription.status_label.is_empty(), |title| {
                                title.child(
                                    div()
                                        .px(px(7.))
                                        .py(px(3.))
                                        .rounded(px(99.))
                                        .bg(colors.selected)
                                        .typography(TypographyToken::BodySmallStrong)
                                        .text_color(colors.text)
                                        .child(subscription.status_label.clone()),
                                )
                            }),
                    )
                    .when(!subscription.price_label.is_empty(), |detail| {
                        let price = if subscription.cadence_label.is_empty() {
                            subscription.price_label.clone()
                        } else {
                            format!(
                                "{} {}",
                                subscription.price_label, subscription.cadence_label
                            )
                            .into()
                        };
                        detail.child(Self::detail(price, colors))
                    })
                    .when(!subscription.renewal_label.is_empty(), |detail| {
                        detail.child(Self::detail(subscription.renewal_label.clone(), colors))
                    })
                    .when(!subscription.provider_note.is_empty(), |detail| {
                        detail.child(Self::detail(subscription.provider_note.clone(), colors))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(self.button(
                        "manage-subscription",
                        &self.data.actions.manage_subscription,
                        SemanticButtonVariant::Secondary,
                        Some(LucideIcon::ExternalLink),
                        BillingAction::ManageSubscriptionRequested,
                        cx,
                    ))
                    .when_some(
                        self.data
                            .actions
                            .manage_subscription
                            .disabled_reason
                            .clone(),
                        |actions, reason| actions.child(Self::detail(reason, colors)),
                    ),
            )
            .into_any_element()
    }

    fn render_allowance_rail(&self, colors: BillingColors) -> AnyElement {
        div()
            .w_full()
            .min_w_0()
            .h(px(7.))
            .rounded(px(99.))
            .bg(colors.selected)
            .child(
                div()
                    .w(relative(fraction(self.data.usage.allowance_fraction)))
                    .h_full()
                    .rounded(px(99.))
                    .bg(colors.accent),
            )
            .into_any_element()
    }

    fn render_range_control(
        &self,
        range: BillingUsageRange,
        colors: BillingColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.data.usage.range == range;
        let screen = cx.entity();
        let selector = format!("billing-range-{}", range.key());
        div()
            .id(SharedString::from(format!(
                "{}-range-{}",
                self.id,
                range.key()
            )))
            .debug_selector(move || selector.clone())
            .role(Role::Button)
            .aria_label(range.label())
            .aria_selected(selected)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(30.))
            .px(px(10.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(5.))
            .border_1()
            .border_color(if selected {
                colors.focus
            } else {
                colors.border.opacity(0.)
            })
            .bg(if selected {
                colors.selected
            } else {
                colors.panel
            })
            .text_color(if selected { colors.text } else { colors.muted })
            .typography(TypographyToken::BodyMediumStrong)
            .cursor_pointer()
            .hover(move |style| style.bg(colors.hover))
            .focus(move |style| style.border_color(colors.focus))
            .child(range.label())
            .on_activate(move |_, _, cx| {
                screen.update(cx, |_, cx| {
                    cx.emit(BillingAction::UsageRangeSelected(range))
                });
            })
            .into_any_element()
    }

    fn render_trend(&self, colors: BillingColors) -> AnyElement {
        if self.data.usage.trend.is_empty() {
            return Self::empty_state(
                "Usage will appear here",
                "Activity is shown after your first AI request in this period.",
                colors,
            );
        }
        let trend_len = self.data.usage.trend.len();
        let chart_width = trend_len as f32 * 26. + 48.;
        let label_step = if trend_len > 60 {
            10
        } else if trend_len > 14 {
            5
        } else {
            1
        };
        div()
            .id(SharedString::from(format!("{}-trend-clip", self.id)))
            .debug_selector(|| "billing-trend-clip".to_owned())
            .w_full()
            .min_w_0()
            .h(px(164.))
            .overflow_hidden()
            .child(
                div()
                    .id(SharedString::from(format!("{}-trend-scroll", self.id)))
                    .debug_selector(|| "billing-trend-scroll".to_owned())
                    .w_full()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .overflow_x_scroll()
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-trend-track", self.id)))
                            .debug_selector(|| "billing-trend-track".to_owned())
                            .w_full()
                            .min_w(px(chart_width))
                            .flex_none()
                            .h(px(155.))
                            .px(px(24.))
                            .flex()
                            .items_end()
                            .justify_between()
                            .gap(px(6.))
                            .children(self.data.usage.trend.iter().enumerate().map(
                                |(index, point)| {
                                    let height = (fraction(point.fraction) * 116.).max(5.);
                                    let tooltip = if point.amount_label.is_empty() {
                                        point.label.to_string()
                                    } else {
                                        format!("{} · {}", point.label, point.amount_label)
                                    };
                                    div()
                                        .id(SharedString::from(format!(
                                            "{}-usage-point-{index}",
                                            self.id
                                        )))
                                        .debug_selector(move || {
                                            format!("billing-trend-bar-{index}")
                                        })
                                        .tooltip(move |window, cx| {
                                            Tooltip::new(tooltip.clone()).build(window, cx)
                                        })
                                        .w(px(18.))
                                        .flex_none()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap(px(8.))
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(height))
                                                .rounded(px(4.))
                                                .bg(colors.accent.opacity(0.78)),
                                        )
                                        .child(
                                            div()
                                                .typography(TypographyToken::BodySmall)
                                                .text_color(colors.muted)
                                                .child(
                                                    if index % label_step == 0
                                                        || index + 1 == trend_len
                                                    {
                                                        point.label.clone()
                                                    } else {
                                                        " ".into()
                                                    },
                                                ),
                                        )
                                },
                            )),
                    ),
            )
            .into_any_element()
    }

    fn render_daily_values(&self, colors: BillingColors) -> AnyElement {
        div()
            .id(SharedString::from(format!(
                "{}-daily-values-scroll",
                self.id
            )))
            .debug_selector(|| "billing-daily-values-table".to_owned())
            .role(Role::Table)
            .aria_label(format!("Daily AI usage, {}", self.data.usage.period_label))
            .w_full()
            .min_w_0()
            .h(px(220.))
            .overflow_y_scroll()
            .rounded(px(7.))
            .border_1()
            .border_color(colors.border)
            .child(
                div()
                    .id(SharedString::from(format!(
                        "{}-daily-values-header",
                        self.id
                    )))
                    .role(Role::Row)
                    .px(px(12.))
                    .py(px(8.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(colors.surface)
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "{}-daily-values-day-header",
                                self.id
                            )))
                            .role(Role::ColumnHeader)
                            .child(Self::overline("DAY", colors)),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "{}-daily-values-credits-header",
                                self.id
                            )))
                            .role(Role::ColumnHeader)
                            .child(Self::overline("CREDITS USED", colors)),
                    ),
            )
            .children(
                self.data
                    .usage
                    .trend
                    .iter()
                    .enumerate()
                    .map(|(index, point)| {
                        div()
                            .id(SharedString::from(format!(
                                "{}-daily-values-row-{index}",
                                self.id
                            )))
                            .role(Role::Row)
                            .px(px(12.))
                            .py(px(8.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .border_t_1()
                            .border_color(colors.border)
                            .child(
                                div()
                                    .id(SharedString::from(format!(
                                        "{}-daily-values-day-{index}",
                                        self.id
                                    )))
                                    .role(Role::Cell)
                                    .child(Self::detail(point.label.clone(), colors)),
                            )
                            .child(
                                div()
                                    .id(SharedString::from(format!(
                                        "{}-daily-values-amount-{index}",
                                        self.id
                                    )))
                                    .role(Role::Cell)
                                    .typography(TypographyToken::BodyMediumStrong)
                                    .text_color(colors.text)
                                    .child(point.amount_label.clone()),
                            )
                    }),
            )
            .into_any_element()
    }

    fn render_usage_summary(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        let usage = &self.data.usage;
        div()
            .id(SharedString::from(format!("{}-usage-card", self.id)))
            .debug_selector(|| "billing-usage-card".to_owned())
            .w_full()
            .min_w_0()
            .max_w_full()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(19.))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-usage-title", self.id)))
                            .debug_selector(|| "billing-usage-title".to_owned())
                            .w_full()
                            .min_w(px(180.))
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(Self::overline("AI USAGE", colors))
                            .child(Self::heading(usage.period_label.clone(), colors)),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-usage-controls", self.id)))
                            .debug_selector(|| "billing-usage-controls".to_owned())
                            .w_full()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(3.))
                                    .rounded(px(6.))
                                    .border_1()
                                    .border_color(colors.border)
                                    .p(px(2.))
                                    .children(
                                        BillingUsageRange::ALL.into_iter().map(|range| {
                                            self.render_range_control(range, colors, cx)
                                        }),
                                    ),
                            )
                            .child(self.button(
                                "export-usage",
                                &self.data.actions.export_usage,
                                SemanticButtonVariant::Secondary,
                                Some(LucideIcon::Download),
                                BillingAction::ExportUsageRequested,
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{}-usage-total", self.id)))
                    .debug_selector(|| "billing-usage-total".to_owned())
                    .w_full()
                    .min_w(px(180.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .typography(TypographyToken::HeadingLarge)
                            .text_color(colors.text)
                            .child(usage.total_label.clone()),
                    )
                    .child(Self::detail(usage.allowance_label.clone(), colors))
                    .when(!usage.reset_label.is_empty(), |summary| {
                        summary.child(Self::detail(usage.reset_label.clone(), colors))
                    }),
            )
            .child(self.render_allowance_rail(colors))
            .child(self.render_trend(colors))
            .when(usage.trend.len() > 14, |card| {
                card.child(
                    div()
                        .id(SharedString::from(format!("{}-chart-scroll-hint", self.id)))
                        .debug_selector(|| "billing-chart-scroll-hint".to_owned())
                        .typography(TypographyToken::BodySmall)
                        .text_color(colors.muted)
                        .child("Scroll the chart horizontally to see every day."),
                )
            })
            .when(!usage.trend.is_empty(), |card| {
                card.child({
                    let screen = cx.entity();
                    semantic_button(
                        format!("{}-daily-values-toggle", self.id),
                        SemanticButtonVariant::Link,
                        SemanticButtonSize::Default,
                    )
                    .label(if self.show_daily_values {
                        "Hide daily values"
                    } else {
                        "Show daily values"
                    })
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |this, cx| {
                            this.show_daily_values = !this.show_daily_values;
                            cx.notify();
                        });
                    })
                })
            })
            .when(self.show_daily_values && !usage.trend.is_empty(), |card| {
                card.child(self.render_daily_values(colors))
            })
            .when(!usage.note.is_empty(), |card| {
                card.child(Self::detail(usage.note.clone(), colors))
            })
            .when_some(
                self.data.actions.export_usage.disabled_reason.clone(),
                |card, reason| card.child(Self::detail(reason, colors)),
            )
            .into_any_element()
    }

    fn render_category_row(category: &BillingUsageCategory, colors: BillingColors) -> AnyElement {
        div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(13.))
            .py(px(11.))
            .border_b_1()
            .border_color(colors.border)
            .child(
                div()
                    .w(px(34.))
                    .h(px(34.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(7.))
                    .bg(colors.accent.opacity(0.1))
                    .child(render_lucide_icon(category.kind.icon(), colors.accent, 16.)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(3.))
                    .child(Self::heading(category.name.clone(), colors))
                    .child(Self::detail(category.detail.clone(), colors)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap(px(4.))
                    .child(
                        div()
                            .typography(TypographyToken::BodyLargeStrong)
                            .text_color(colors.text)
                            .child(category.amount_label.clone()),
                    )
                    .child(
                        div()
                            .w(px(70.))
                            .h(px(4.))
                            .rounded(px(99.))
                            .bg(colors.selected)
                            .child(
                                div()
                                    .w(relative(fraction(category.fraction)))
                                    .h_full()
                                    .rounded(px(99.))
                                    .bg(colors.accent),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_breakdown(&self, colors: BillingColors) -> AnyElement {
        let rows = if self.data.usage.categories.is_empty() {
            Self::detail("No model usage in this period.", colors)
        } else {
            div()
                .children(
                    self.data
                        .usage
                        .categories
                        .iter()
                        .map(|category| Self::render_category_row(category, colors)),
                )
                .into_any_element()
        };
        div()
            .id(SharedString::from(format!("{}-model-card", self.id)))
            .debug_selector(|| "billing-model-card".to_owned())
            .w_full()
            .min_w_0()
            .max_w_full()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(Self::overline("BY MODEL", colors))
            .child(Self::heading("Model usage", colors))
            .child(rows)
            .into_any_element()
    }

    fn render_api_keys(&self, colors: BillingColors) -> AnyElement {
        let rows = if self.data.usage.api_keys.is_empty() {
            Self::detail("No API key usage in this period.", colors)
        } else {
            div()
                .children(self.data.usage.api_keys.iter().map(|key| {
                    div()
                        .w_full()
                        .min_w_0()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(12.))
                        .py(px(11.))
                        .border_b_1()
                        .border_color(colors.border)
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(4.))
                                .child(Self::heading(key.name.clone(), colors))
                                .child(Self::detail(key.detail.clone(), colors)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_end()
                                .gap(px(5.))
                                .child(
                                    div()
                                        .typography(TypographyToken::BodyLargeStrong)
                                        .text_color(colors.text)
                                        .child(key.amount_label.clone()),
                                )
                                .child(
                                    div()
                                        .w(px(70.))
                                        .h(px(4.))
                                        .rounded(px(99.))
                                        .bg(colors.selected)
                                        .child(
                                            div()
                                                .w(relative(fraction(key.fraction)))
                                                .h_full()
                                                .rounded(px(99.))
                                                .bg(colors.accent),
                                        ),
                                ),
                        )
                }))
                .into_any_element()
        };
        div()
            .id(SharedString::from(format!("{}-api-key-card", self.id)))
            .debug_selector(|| "billing-api-key-card".to_owned())
            .w_full()
            .min_w_0()
            .max_w_full()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(Self::overline("BY API KEY", colors))
            .child(Self::heading("Application usage", colors))
            .child(rows)
            .into_any_element()
    }

    fn render_plan_card(
        &self,
        plan: &BillingPlan,
        colors: BillingColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let plan_id = plan.id.clone();
        div()
            .id(SharedString::from(format!(
                "{}-plan-card-{}",
                self.id, plan.id
            )))
            .debug_selector({
                let id = plan.id.clone();
                move || format!("billing-plan-card-{id}")
            })
            .min_w(px(245.))
            .max_w_full()
            .flex_1()
            .p(px(20.))
            .rounded(px(10.))
            .border_1()
            .border_color(if plan.highlighted {
                colors.accent
            } else {
                colors.border
            })
            .bg(if plan.highlighted {
                colors.accent.opacity(0.055)
            } else {
                colors.panel
            })
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(Self::overline(plan.eyebrow.clone(), colors))
                    .when(plan.current, |row| {
                        row.child(
                            div()
                                .px(px(8.))
                                .py(px(4.))
                                .rounded(px(99.))
                                .bg(colors.accent.opacity(0.12))
                                .typography(TypographyToken::BodySmallStrong)
                                .text_color(colors.accent)
                                .child("Current"),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .typography(TypographyToken::HeadingLarge)
                            .text_color(colors.text)
                            .child(plan.name.clone()),
                    )
                    .child(Self::detail(plan.description.clone(), colors)),
            )
            .when(!plan.price_label.is_empty(), |card| {
                card.child(
                    div()
                        .debug_selector({
                            let id = plan.id.clone();
                            move || format!("billing-plan-price-{id}")
                        })
                        .flex()
                        .items_baseline()
                        .gap(px(5.))
                        .child(Self::heading(plan.price_label.clone(), colors))
                        .child(Self::detail(plan.cadence_label.clone(), colors)),
                )
            })
            .child(
                div()
                    .py(px(9.))
                    .border_t_1()
                    .border_b_1()
                    .border_color(colors.border)
                    .child(
                        div()
                            .typography(TypographyToken::BodyLargeStrong)
                            .text_color(colors.accent)
                            .child(plan.credits_label.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(9.))
                    .children(plan.features.iter().map(|feature| {
                        div()
                            .flex()
                            .items_start()
                            .gap(px(9.))
                            .child(render_lucide_icon(LucideIcon::Check, colors.positive, 14.))
                            .child(Self::detail(feature.clone(), colors))
                    })),
            )
            .child(div().pt(px(5.)).child(self.button(
                format!("plan-{}", plan.id),
                &plan.action,
                if plan.highlighted {
                    SemanticButtonVariant::Primary
                } else {
                    SemanticButtonVariant::Secondary
                },
                None,
                BillingAction::PlanRequested { plan_id },
                cx,
            )))
            .when_some(plan.action.disabled_reason.clone(), |card, reason| {
                card.child(Self::detail(reason, colors))
            })
            .into_any_element()
    }

    fn render_plans(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        if self.data.plans.is_empty() {
            return Self::empty_state(
                "Plans are unavailable",
                "Your workspace's eligible plans will appear when billing data is available.",
                colors,
            );
        }
        div()
            .id(SharedString::from(format!("{}-plans-content", self.id)))
            .debug_selector(|| "billing-plans-content".to_owned())
            .w_full()
            .min_w_0()
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(Self::heading("A plan for the way you create", colors))
                    .child(Self::detail(
                        "Available choices and purchase routes are determined by your workspace and account.",
                        colors,
                    )),
            )
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_wrap()
                    .gap(px(14.))
                    .children(self.data.plans.iter().map(|plan| {
                        self.render_plan_card(plan, colors, cx)
                    })),
            )
            .into_any_element()
    }

    fn render_transaction_row(
        &self,
        transaction: &BillingTransaction,
        colors: BillingColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut trailing = div().flex().flex_col().items_end().gap(px(5.));
        if !transaction.credits_label.is_empty() {
            trailing = trailing.child(
                div()
                    .typography(TypographyToken::BodyLargeStrong)
                    .text_color(colors.text)
                    .child(transaction.credits_label.clone()),
            );
        }
        if !transaction.amount_label.is_empty() {
            trailing = trailing.child(Self::detail(transaction.amount_label.clone(), colors));
        }
        if let Some(action) = &transaction.receipt_action {
            trailing = trailing.child(self.button(
                format!("receipt-{}", transaction.id),
                action,
                SemanticButtonVariant::Link,
                None,
                BillingAction::ReceiptRequested {
                    transaction_id: transaction.id.clone(),
                },
                cx,
            ));
            if let Some(reason) = &action.disabled_reason {
                trailing = trailing.child(Self::detail(reason.clone(), colors));
            }
        }
        div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .py(px(14.))
            .border_b_1()
            .border_color(colors.border)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(220.))
                    .max_w_full()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .w(px(34.))
                            .h(px(34.))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(7.))
                            .bg(colors.surface)
                            .child(render_lucide_icon(LucideIcon::Sparkles, colors.muted, 15.)),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(Self::heading(transaction.title.clone(), colors))
                            .child(Self::detail(
                                format!(
                                    "{} · {} · {}",
                                    transaction.date_label,
                                    transaction.detail,
                                    transaction.status_label
                                ),
                                colors,
                            )),
                    ),
            )
            .child(trailing)
            .into_any_element()
    }

    fn render_history(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        if self.data.transactions.is_empty() {
            return Self::empty_state(
                "No billing activity yet",
                "Credit grants, purchases, and receipts will appear here.",
                colors,
            );
        }
        div()
            .id(SharedString::from(format!("{}-history-card", self.id)))
            .debug_selector(|| "billing-history-card".to_owned())
            .w_full()
            .min_w_0()
            .max_w_full()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(Self::overline("ACTIVITY", colors))
            .child(Self::heading("Credits & payments", colors))
            .children(
                self.data
                    .transactions
                    .iter()
                    .map(|transaction| self.render_transaction_row(transaction, colors, cx)),
            )
            .into_any_element()
    }

    fn render_apple_actions(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .p(px(22.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(Self::overline("APP STORE", colors))
            .child(Self::heading("Apple purchases", colors))
            .child(Self::detail(
                "Manage your subscription through Apple. If a purchase is missing, restore or sync it with this workspace.",
                colors,
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(9.))
                    .child(self.button(
                        "restore-purchases",
                        &self.data.actions.restore_purchases,
                        SemanticButtonVariant::Secondary,
                        Some(LucideIcon::RefreshCw),
                        BillingAction::RestorePurchasesRequested,
                        cx,
                    ))
                    .child(self.button(
                        "sync-purchases",
                        &self.data.actions.sync_purchases,
                        SemanticButtonVariant::Secondary,
                        None,
                        BillingAction::SyncPurchasesRequested,
                        cx,
                    ))
                    .child(self.button(
                        "redeem-code",
                        &self.data.actions.redeem_code,
                        SemanticButtonVariant::Secondary,
                        None,
                        BillingAction::RedeemCodeRequested,
                        cx,
                    )),
            )
            .when_some(
                self.data.actions.restore_purchases.disabled_reason.clone(),
                |card, reason| card.child(Self::detail(reason, colors)),
            )
            .when_some(
                self.data.actions.sync_purchases.disabled_reason.clone(),
                |card, reason| card.child(Self::detail(reason, colors)),
            )
            .when_some(
                self.data.actions.redeem_code.disabled_reason.clone(),
                |card, reason| card.child(Self::detail(reason, colors)),
            )
            .into_any_element()
    }

    fn render_apple_actions_row(
        &self,
        colors: BillingColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(SharedString::from(format!("{}-apple-actions-row", self.id)))
            .debug_selector(|| "billing-embedded-apple-actions".to_owned())
            .role(Role::Group)
            .aria_label("Apple purchases")
            .w_full()
            .min_w_0()
            .py(px(14.))
            .border_b_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(Self::overline("APPLE PURCHASES", colors))
            .child(Self::detail(
                "If an Apple purchase is missing, restore or sync it with this workspace.",
                colors,
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(self.button(
                        "restore-purchases",
                        &self.data.actions.restore_purchases,
                        SemanticButtonVariant::Secondary,
                        Some(LucideIcon::RefreshCw),
                        BillingAction::RestorePurchasesRequested,
                        cx,
                    ))
                    .child(self.button(
                        "sync-purchases",
                        &self.data.actions.sync_purchases,
                        SemanticButtonVariant::Secondary,
                        None,
                        BillingAction::SyncPurchasesRequested,
                        cx,
                    ))
                    .child(self.button(
                        "redeem-code",
                        &self.data.actions.redeem_code,
                        SemanticButtonVariant::Secondary,
                        None,
                        BillingAction::RedeemCodeRequested,
                        cx,
                    )),
            )
            .when_some(
                self.data.actions.restore_purchases.disabled_reason.clone(),
                |row, reason| row.child(Self::detail(reason, colors)),
            )
            .when_some(
                self.data.actions.sync_purchases.disabled_reason.clone(),
                |row, reason| row.child(Self::detail(reason, colors)),
            )
            .when_some(
                self.data.actions.redeem_code.disabled_reason.clone(),
                |row, reason| row.child(Self::detail(reason, colors)),
            )
            .into_any_element()
    }

    fn render_overview(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(16.))
                    .child(self.render_balance(colors, cx))
                    .child(self.render_subscription(colors, cx)),
            )
            .child(self.render_usage_summary(colors, cx))
            .when(
                self.data.subscription.provider == BillingProvider::Apple,
                |body| body.child(self.render_apple_actions(colors, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(11.))
                    .child(Self::heading("Recent activity", colors))
                    .when(self.data.transactions.is_empty(), |section| {
                        section.child(Self::detail(
                            "Credit grants, purchases, and receipts will appear here.",
                            colors,
                        ))
                    })
                    .children(
                        self.data.transactions.iter().take(3).map(|transaction| {
                            self.render_transaction_row(transaction, colors, cx)
                        }),
                    ),
            )
            .into_any_element()
    }

    fn render_embedded_overview(
        &self,
        colors: BillingColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(18.))
            .child(
                div()
                    .id(SharedString::from(format!("{}-balance-hero", self.id)))
                    .debug_selector(|| "billing-embedded-balance-hero".to_owned())
                    .w_full()
                    .flex()
                    .child(self.render_balance(colors, cx)),
            )
            .child(self.render_subscription_row(colors, cx))
            .when(
                self.data.subscription.provider == BillingProvider::Apple,
                |body| body.child(self.render_apple_actions_row(colors, cx)),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{}-recent-activity", self.id)))
                    .debug_selector(|| "billing-embedded-recent-activity".to_owned())
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(Self::overline("RECENT ACTIVITY", colors))
                    .when(self.data.transactions.is_empty(), |section| {
                        section.child(Self::detail(
                            "Credit grants, purchases, and receipts will appear here.",
                            colors,
                        ))
                    })
                    .children(
                        self.data.transactions.iter().take(3).map(|transaction| {
                            self.render_transaction_row(transaction, colors, cx)
                        }),
                    ),
            )
            .into_any_element()
    }

    fn render_content(&self, colors: BillingColors, cx: &mut Context<Self>) -> AnyElement {
        match self.data.selected_tab {
            BillingTab::Overview if self.presentation == BillingPresentation::Embedded => {
                self.render_embedded_overview(colors, cx)
            }
            BillingTab::Overview => self.render_overview(colors, cx),
            BillingTab::Usage => div()
                .w_full()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(self.render_usage_summary(colors, cx))
                .child(self.render_breakdown(colors))
                .child(self.render_api_keys(colors))
                .into_any_element(),
            BillingTab::Plans => self.render_plans(colors, cx),
            BillingTab::History => self.render_history(colors, cx),
        }
    }
}

impl Focusable for BillingScreen {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BillingScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = BillingColors::current(cx);
        let root = div()
            .id(self.id.clone())
            .debug_selector(|| "billing-screen".to_owned())
            .key_context("FantaBillingScreen")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .text_color(colors.text);
        let root = match self.presentation {
            BillingPresentation::Standalone => root
                .size_full()
                .min_w(px(BILLING_SCREEN_MIN_WIDTH))
                .min_h(px(BILLING_SCREEN_MIN_HEIGHT))
                .bg(colors.background)
                .child(self.render_header(colors, cx))
                .child(
                    div()
                        .id(SharedString::from(format!("{}-content", self.id)))
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll_handle)
                        .child(
                            div()
                                .max_w(px(1180.))
                                .w_full()
                                .mx_auto()
                                .px(px(26.))
                                .py(px(24.))
                                .flex()
                                .flex_col()
                                .gap(px(16.))
                                .when_some(self.render_status(colors, cx), |body, status| {
                                    body.child(status)
                                })
                                .child(self.render_content(colors, cx)),
                        ),
                ),
            BillingPresentation::Embedded => root.w_full().min_w_0().max_w_full().min_h_0().child(
                div()
                    .id(SharedString::from(format!("{}-embedded-content", self.id)))
                    .debug_selector(|| "billing-embedded-content".to_owned())
                    .w_full()
                    .min_w_0()
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .when_some(self.render_status(colors, cx), |body, status| {
                        body.child(status)
                    })
                    .child(self.render_content(colors, cx)),
            ),
        };
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
    use crate::screens::settings::{SettingsPage, SettingsScreen, SettingsViewData};
    use crate::test_support::mount_component;
    use gpui::{Modifiers, TestAppContext, size};

    #[test]
    fn fraction_is_safe_for_untrusted_usage_data() {
        assert_eq!(fraction(f32::NAN), 0.);
        assert_eq!(fraction(-0.5), 0.);
        assert_eq!(fraction(1.5), 1.);
    }

    #[gpui::test]
    fn navigation_and_usage_range_wait_for_host_echo(cx: &mut TestAppContext) {
        let data = BillingViewData {
            selected_tab: BillingTab::Overview,
            usage: BillingUsage {
                range: BillingUsageRange::Days7,
                ..Default::default()
            },
            actions: BillingActions {
                refresh: BillingActionState::enabled("Refresh"),
                export_usage: BillingActionState::enabled("Export CSV"),
                ..Default::default()
            },
            ..Default::default()
        };
        let (host, actions, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new("test-billing", data, cx)
        });
        cx.simulate_resize(size(px(900.), px(700.)));
        cx.run_until_parked();

        let tab = cx.debug_bounds("billing-tab-usage").unwrap();
        cx.simulate_click(tab.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[BillingAction::TabSelected(BillingTab::Usage)]
        );
        let screen = cx.read(|app| host.read(app).component.clone());
        assert_eq!(
            cx.read(|app| screen.read(app).view_data().selected_tab),
            BillingTab::Overview,
            "the host must echo navigation before the screen changes tabs"
        );

        let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
        echoed.selected_tab = BillingTab::Usage;
        cx.read(|app| {
            screen
                .read(app)
                .scroll_handle
                .set_offset(gpui::point(px(0.), px(-200.)))
        });
        cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
        assert_eq!(
            cx.read(|app| screen.read(app).scroll_handle.offset()),
            gpui::point(px(0.), px(0.)),
            "switching tabs should show the new destination from its top"
        );
        actions.borrow_mut().clear();
        let range = cx.debug_bounds("billing-range-30d").unwrap();
        cx.simulate_click(range.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[BillingAction::UsageRangeSelected(BillingUsageRange::Days30)]
        );
        assert_eq!(
            cx.read(|app| screen.read(app).view_data().usage.range),
            BillingUsageRange::Days7,
            "the host must echo a new usage snapshot before the range changes"
        );
    }

    #[gpui::test]
    fn purchase_button_obeys_host_availability(cx: &mut TestAppContext) {
        let data = BillingViewData {
            actions: BillingActions {
                add_credits: BillingActionState::disabled(
                    "Add credits",
                    "Purchases are unavailable for this workspace",
                ),
                refresh: BillingActionState::enabled("Refresh"),
                ..Default::default()
            },
            ..Default::default()
        };
        let (host, actions, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new("test-billing", data, cx)
        });
        cx.simulate_resize(size(px(900.), px(700.)));
        cx.run_until_parked();
        let button = cx.debug_bounds("test-billing-add-credits").unwrap();
        cx.simulate_click(button.center(), Modifiers::none());
        cx.run_until_parked();
        assert!(actions.borrow().is_empty());

        let screen = cx.read(|app| host.read(app).component.clone());
        let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
        echoed.actions.add_credits = BillingActionState::enabled("Add credits");
        cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
        let button = cx.debug_bounds("test-billing-add-credits").unwrap();
        cx.simulate_click(button.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[BillingAction::AddCreditsRequested]
        );
    }

    #[gpui::test]
    fn exact_daily_values_can_be_revealed_with_keyboard(cx: &mut TestAppContext) {
        let data = BillingViewData {
            selected_tab: BillingTab::Usage,
            usage: BillingUsage {
                trend: vec![BillingUsagePoint {
                    label: "Sep 27".into(),
                    amount_label: "42 credits".into(),
                    fraction: 0.7,
                }],
                ..Default::default()
            },
            actions: BillingActions {
                export_usage: BillingActionState::enabled("Export CSV"),
                ..Default::default()
            },
            ..Default::default()
        };
        let (host, actions, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new("test-billing", data, cx)
        });
        cx.simulate_resize(size(px(1100.), px(900.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("billing-daily-values-table").is_none());
        let toggle = cx.debug_bounds("test-billing-daily-values-toggle").unwrap();
        cx.simulate_click(toggle.center(), Modifiers::none());
        cx.run_until_parked();
        assert!(cx.debug_bounds("billing-daily-values-table").is_some());

        let screen = cx.read(|app| host.read(app).component.clone());
        assert!(cx.read(|app| screen.read(app).show_daily_values));
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        assert!(cx.debug_bounds("billing-daily-values-table").is_none());
        assert!(!cx.read(|app| screen.read(app).show_daily_values));
        assert!(
            actions.borrow().is_empty(),
            "local chart disclosure emits no host action"
        );
    }

    #[gpui::test]
    fn contact_sales_plan_has_no_empty_price_row(cx: &mut TestAppContext) {
        let data = BillingViewData {
            selected_tab: BillingTab::Plans,
            plans: vec![BillingPlan {
                id: "team".into(),
                name: "Team".into(),
                description: "Talk to our team about your workspace".into(),
                price_label: "".into(),
                action: BillingActionState::enabled("Contact sales"),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (host, _, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new("test-billing", data, cx)
        });
        cx.simulate_resize(size(px(1100.), px(900.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("billing-plan-price-team").is_none());

        let screen = cx.read(|app| host.read(app).component.clone());
        let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
        echoed.plans[0].price_label = "Custom".into();
        cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
        assert!(cx.debug_bounds("billing-plan-price-team").is_some());
    }

    #[gpui::test]
    fn embedded_billing_uses_host_navigation_and_content_pane_width(cx: &mut TestAppContext) {
        let data = BillingViewData {
            selected_tab: BillingTab::Usage,
            plans: vec![BillingPlan {
                id: "team".into(),
                name: "Team".into(),
                action: BillingActionState::enabled("Contact sales"),
                ..Default::default()
            }],
            actions: BillingActions {
                export_usage: BillingActionState::enabled("Export CSV"),
                ..Default::default()
            },
            ..Default::default()
        };
        let (host, actions, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new_embedded("test-billing", data, cx)
        });
        cx.simulate_resize(size(px(420.), px(700.)));
        cx.run_until_parked();

        assert!(cx.debug_bounds("billing-embedded-content").is_some());
        assert!(cx.debug_bounds("billing-tab-usage").is_none());
        assert!(cx.debug_bounds("test-billing-refresh").is_none());
        assert!(cx.debug_bounds("test-billing-export-usage").is_some());
        assert!(
            cx.debug_bounds("billing-screen").unwrap().size.width < px(BILLING_SCREEN_MIN_WIDTH),
            "embedded content must not impose the standalone window minimum",
        );

        let screen = cx.read(|app| host.read(app).component.clone());
        let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
        echoed.selected_tab = BillingTab::Plans;
        cx.update(|_, app| screen.update(app, |screen, cx| screen.set_view_data(echoed, cx)));
        assert!(cx.debug_bounds("test-billing-plan-team").is_some());
        assert!(cx.debug_bounds("billing-tab-plans").is_none());
        assert!(actions.borrow().is_empty());
    }

    #[gpui::test]
    fn embedded_overview_keeps_balance_and_purchase_tools_without_usage_card(
        cx: &mut TestAppContext,
    ) {
        let data = BillingViewData {
            selected_tab: BillingTab::Overview,
            subscription: BillingSubscription {
                plan_name: "Pro".into(),
                status_label: "Active".into(),
                provider: BillingProvider::Apple,
                ..Default::default()
            },
            actions: BillingActions {
                add_credits: BillingActionState::enabled("Add credits"),
                manage_subscription: BillingActionState::enabled("Manage with Apple"),
                restore_purchases: BillingActionState::enabled("Restore purchases"),
                sync_purchases: BillingActionState::enabled("Sync purchases"),
                redeem_code: BillingActionState::enabled("Redeem offer code"),
                export_usage: BillingActionState::enabled("Export CSV"),
                ..Default::default()
            },
            ..Default::default()
        };
        let (_, _, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new_embedded("test-billing", data, cx)
        });
        cx.simulate_resize(size(px(560.), px(850.)));
        cx.run_until_parked();

        assert!(cx.debug_bounds("billing-embedded-balance-hero").is_some());
        assert!(cx.debug_bounds("billing-embedded-plan-row").is_some());
        assert!(cx.debug_bounds("billing-embedded-apple-actions").is_some());
        assert!(
            cx.debug_bounds("billing-embedded-recent-activity")
                .is_some()
        );
        assert!(cx.debug_bounds("test-billing-add-credits").is_some());
        assert!(
            cx.debug_bounds("test-billing-manage-subscription")
                .is_some()
        );
        assert!(cx.debug_bounds("test-billing-export-usage").is_none());
    }

    #[gpui::test]
    fn embedded_sections_fit_a_narrow_settings_content_pane(cx: &mut TestAppContext) {
        let data = BillingViewData {
            selected_tab: BillingTab::Overview,
            balance: BillingBalance {
                available_label: "2,340".into(),
                available_caption: "Credits available".into(),
                included_label: "1,840 monthly".into(),
                purchased_label: "500 purchased".into(),
                ..Default::default()
            },
            subscription: BillingSubscription {
                plan_name: "Pro".into(),
                status_label: "Active".into(),
                provider: BillingProvider::Apple,
                ..Default::default()
            },
            usage: BillingUsage {
                range: BillingUsageRange::Days90,
                period_label: "Last 90 days".into(),
                total_label: "3,420 credits used".into(),
                allowance_label: "9,000 included credits across three cycles".into(),
                trend: (0..90)
                    .map(|day| BillingUsagePoint {
                        label: format!("Sep {day}").into(),
                        amount_label: "40 credits".into(),
                        fraction: 0.4,
                    })
                    .collect(),
                categories: vec![BillingUsageCategory {
                    id: "image".into(),
                    name: "Fanta Image".into(),
                    detail: "Image · 146 requests".into(),
                    amount_label: "2,420 credits".into(),
                    fraction: 0.7,
                    kind: BillingUsageKind::Image,
                }],
                api_keys: vec![BillingUsageApiKey {
                    id: "studio-api".into(),
                    name: "Studio automation".into(),
                    detail: "API key ·•••• 9E2A".into(),
                    amount_label: "3,420 credits".into(),
                    fraction: 1.,
                }],
                ..Default::default()
            },
            plans: ["free", "pro", "team"]
                .into_iter()
                .map(|id| BillingPlan {
                    id: id.into(),
                    name: id.into(),
                    price_label: "$44.99".into(),
                    credits_label: "3,000 credits".into(),
                    action: BillingActionState::enabled("Select plan"),
                    ..Default::default()
                })
                .collect(),
            transactions: vec![BillingTransaction {
                id: "receipt-1".into(),
                title: "Credit pack".into(),
                detail: "One-time workspace credit purchase".into(),
                date_label: "Sep 12, 2026".into(),
                credits_label: "+500 credits".into(),
                amount_label: "$8.99".into(),
                status_label: "Completed".into(),
                receipt_action: Some(BillingActionState::enabled("Receipt")),
            }],
            actions: BillingActions {
                add_credits: BillingActionState::enabled("Buy 500 credits · $8.99"),
                manage_subscription: BillingActionState::enabled("Manage with Apple"),
                restore_purchases: BillingActionState::enabled("Restore purchases"),
                redeem_code: BillingActionState::enabled("Redeem offer code"),
                sync_purchases: BillingActionState::enabled("Sync purchases"),
                export_usage: BillingActionState::enabled("Export CSV"),
                ..Default::default()
            },
            ..Default::default()
        };
        let (host, _, cx) = mount_component(cx, move |_, cx| {
            BillingScreen::new_embedded("test-billing", data, cx)
        });
        let screen = cx.read(|app| host.read(app).component.clone());
        for width in [668., 500., 420.] {
            cx.simulate_resize(size(px(width), px(640.)));
            cx.run_until_parked();
            for (tab, selectors) in [
                (
                    BillingTab::Overview,
                    &[
                        "billing-embedded-balance-hero",
                        "billing-embedded-plan-row",
                        "billing-embedded-recent-activity",
                        "test-billing-add-credits",
                        "test-billing-manage-subscription",
                        "test-billing-restore-purchases",
                        "test-billing-sync-purchases",
                        "test-billing-redeem-code",
                    ][..],
                ),
                (
                    BillingTab::Usage,
                    &[
                        "billing-usage-card",
                        "billing-trend-scroll",
                        "billing-model-card",
                        "billing-api-key-card",
                        "billing-range-7d",
                        "billing-range-30d",
                        "billing-range-90d",
                        "test-billing-export-usage",
                        "test-billing-daily-values-toggle",
                    ][..],
                ),
                (
                    BillingTab::Plans,
                    &[
                        "billing-plans-content",
                        "billing-plan-card-free",
                        "billing-plan-card-pro",
                        "billing-plan-card-team",
                        "test-billing-plan-free",
                        "test-billing-plan-pro",
                        "test-billing-plan-team",
                    ][..],
                ),
                (
                    BillingTab::History,
                    &["billing-history-card", "test-billing-receipt-receipt-1"][..],
                ),
            ] {
                let mut echoed = cx.read(|app| screen.read(app).view_data().clone());
                echoed.selected_tab = tab;
                cx.update(|_, app| {
                    screen.update(app, |screen, cx| screen.set_view_data(echoed, cx))
                });
                cx.run_until_parked();
                let pane = cx.debug_bounds("billing-embedded-content").unwrap();
                for selector in selectors {
                    let bounds = cx.debug_bounds(selector).unwrap();
                    assert!(
                        bounds.left() >= pane.left() - px(1.)
                            && bounds.right() <= pane.right() + px(1.),
                        "{tab:?} {selector} is outside {width}px pane: {bounds:?} vs {pane:?}"
                    );
                }
            }
        }
    }

    #[gpui::test]
    fn usage_inner_layout_keeps_readable_width_inside_settings(cx: &mut TestAppContext) {
        let billing = BillingViewData {
            selected_tab: BillingTab::Usage,
            usage: BillingUsage {
                range: BillingUsageRange::Days30,
                period_label: "Last 30 days".into(),
                total_label: "1,160 credits used".into(),
                allowance_label: "3,000 included credits".into(),
                reset_label: "Resets October 1".into(),
                allowance_fraction: 0.39,
                trend: (0..30)
                    .map(|day| BillingUsagePoint {
                        label: format!("Sep {day}").into(),
                        amount_label: "40 credits".into(),
                        fraction: 0.4,
                    })
                    .collect(),
                ..Default::default()
            },
            actions: BillingActions {
                export_usage: BillingActionState::enabled("Export CSV"),
                ..Default::default()
            },
            ..Default::default()
        };
        let data = SettingsViewData {
            selected_page: SettingsPage::Usage,
            billing: Some(billing),
            ..Default::default()
        };
        let (_, _, cx) = mount_component(cx, move |window, cx| {
            SettingsScreen::new("test-settings", data, window, cx)
        });
        cx.simulate_resize(size(px(900.), px(640.)));
        cx.run_until_parked();

        let pane = cx.debug_bounds("settings-content-scroll").unwrap();
        let card = cx.debug_bounds("billing-usage-card").unwrap();
        let title = cx.debug_bounds("billing-usage-title").unwrap();
        let controls = cx.debug_bounds("billing-usage-controls").unwrap();
        let total = cx.debug_bounds("billing-usage-total").unwrap();
        let scroll = cx.debug_bounds("billing-trend-scroll").unwrap();
        let track = cx.debug_bounds("billing-trend-track").unwrap();
        let bar = cx.debug_bounds("billing-trend-bar-0").unwrap();
        assert!(card.left() >= pane.left() && card.right() <= pane.right());
        assert!(title.size.width >= px(180.) && title.size.height < px(100.));
        assert!(total.size.width >= px(180.) && total.size.height < px(120.));
        assert!(controls.top() > title.bottom());
        assert!(total.top() > controls.bottom());
        assert!(controls.left() >= card.left() && controls.right() <= card.right());
        assert!(scroll.left() >= card.left() && scroll.right() <= card.right());
        assert!(scroll.size.width >= px(300.));
        assert!(track.size.width >= px(780.));
        assert!(bar.size.width >= px(16.));
        assert!(cx.debug_bounds("billing-chart-scroll-hint").is_some());
    }
}
