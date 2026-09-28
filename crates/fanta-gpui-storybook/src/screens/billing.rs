//! Mock billing snapshots for the Settings story. Amounts and localized prices
//! are supplied here; the reusable screen never starts a purchase.

use fanta_gpui::billing::{
    BillingActionState, BillingActions, BillingBalance, BillingLoadState, BillingPlan,
    BillingProvider, BillingSubscription, BillingTab, BillingTransaction, BillingUsage,
    BillingUsageApiKey, BillingUsageCategory, BillingUsageKind, BillingUsagePoint,
    BillingUsageRange, BillingViewData, BillingWorkspace,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum BillingNamedState {
    #[default]
    Ready,
    DashboardFree,
    Loading,
    Error,
    Empty,
    Unavailable,
    Web,
}

impl BillingNamedState {
    pub(crate) const ALL: [Self; 7] = [
        Self::Ready,
        Self::DashboardFree,
        Self::Loading,
        Self::Error,
        Self::Empty,
        Self::Unavailable,
        Self::Web,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Ready => "Apple · Pro",
            Self::DashboardFree => "Dashboard · Free",
            Self::Loading => "Loading",
            Self::Error => "Error",
            Self::Empty => "Empty",
            Self::Unavailable => "Purchases unavailable",
            Self::Web => "Web · Pro",
        }
    }

    pub(crate) fn fixture(self) -> BillingViewData {
        let mut data = seed_billing();
        match self {
            Self::Ready => {}
            Self::DashboardFree => apply_dashboard_free(&mut data),
            Self::Loading => {
                data.load_state = BillingLoadState::Loading;
                disable_purchase_actions(&mut data, "Billing is loading");
            }
            Self::Error => {
                data.load_state = BillingLoadState::Error(
                    "We couldn't load the latest billing details. Check your connection and try again."
                        .into(),
                );
                disable_purchase_actions(&mut data, "Billing data is unavailable");
            }
            Self::Empty => {
                data.balance.available_label = "0".into();
                data.balance.included_label = "0 from plan".into();
                data.balance.purchased_label = "0 purchased".into();
                data.balance.note = "Credits will appear here after a plan or credit pack is added to this workspace.".into();
                data.subscription = BillingSubscription {
                    plan_name: "No plan".into(),
                    status_label: "Inactive".into(),
                    provider: BillingProvider::None,
                    provider_note: "Choose a plan to start using Fanta AI.".into(),
                    ..Default::default()
                };
                data.plans.clear();
                data.usage.total_label = "0 credits used".into();
                data.usage.allowance_label = "No included credits".into();
                data.usage.reset_label = "".into();
                data.usage.allowance_fraction = 0.;
                data.usage.trend.clear();
                data.usage.categories.clear();
                data.usage.api_keys.clear();
                data.transactions.clear();
                disable_purchase_actions(&mut data, "No plan is available for this workspace");
            }
            Self::Unavailable => disable_purchase_actions(
                &mut data,
                "Purchases aren't available for this workspace right now",
            ),
            Self::Web => {
                data.subscription.provider = BillingProvider::Web;
                data.subscription.price_label = "".into();
                data.subscription.cadence_label = "".into();
                data.subscription.renewal_label = "".into();
                data.subscription.provider_note =
                    "Billed on the web. Manage your subscription in the Fanta account portal."
                        .into();
                data.actions.manage_subscription = BillingActionState::enabled("Manage on web");
                data.actions.restore_purchases = BillingActionState::default();
                data.actions.redeem_code = BillingActionState::default();
                data.actions.sync_purchases = BillingActionState::default();
                data.actions.add_credits = BillingActionState::disabled(
                    "Add credits on web",
                    "Open the account portal to buy credits",
                );
                for plan in &mut data.plans {
                    if plan.id == "pro" {
                        plan.price_label = "".into();
                        plan.cadence_label = "".into();
                    }
                }
                for transaction in &mut data.transactions {
                    transaction.amount_label = "".into();
                }
            }
        }
        data
    }
}

fn apply_dashboard_free(data: &mut BillingViewData) {
    data.workspace.id = "dashboard-free".into();
    data.workspace.name = "Personal workspace".into();
    data.balance.available_label = "500".into();
    data.balance.included_label = "500 granted once".into();
    data.balance.purchased_label = "0 purchased".into();
    data.balance.note =
        "Grants and purchases share one workspace balance; completed usage and refunds reduce it."
            .into();
    data.subscription = BillingSubscription {
        plan_name: "Free".into(),
        status_label: "Current plan".into(),
        provider: BillingProvider::None,
        provider_note: "No paid subscription is attached to this workspace.".into(),
        ..Default::default()
    };
    for plan in &mut data.plans {
        plan.current = plan.id == "free";
        plan.highlighted = plan.current;
        plan.price_label = "".into();
        plan.cadence_label = "".into();
        plan.eyebrow = if plan.current {
            "Current plan".into()
        } else {
            "Available plan".into()
        };
        plan.action = if plan.current {
            BillingActionState::disabled("Current plan", "This workspace is already on Free")
        } else {
            BillingActionState::disabled("Unavailable", "Checkout is currently unavailable")
        };
    }
    data.usage.total_label = "0 credits used".into();
    data.usage.allowance_label = "500 credits granted once".into();
    data.usage.reset_label = "".into();
    data.usage.allowance_fraction = 0.;
    data.usage.trend.clear();
    data.usage.categories.clear();
    data.usage.api_keys.clear();
    data.usage.note = "No usage in this period. Requests appear after they are processed.".into();
    data.transactions.clear();
    disable_purchase_actions(data, "Checkout is currently unavailable");
    data.actions.manage_subscription = BillingActionState::disabled(
        "No subscription",
        "No paid subscription is attached to this workspace",
    );
}

fn disable_purchase_actions(data: &mut BillingViewData, reason: &'static str) {
    data.actions.add_credits = BillingActionState::disabled("Add credits", reason);
    data.actions.manage_subscription = BillingActionState::disabled("Manage subscription", reason);
    data.actions.restore_purchases = BillingActionState::disabled("Restore purchases", reason);
    data.actions.redeem_code = BillingActionState::disabled("Redeem offer code", reason);
    data.actions.sync_purchases = BillingActionState::disabled("Sync purchases", reason);
    for plan in &mut data.plans {
        if plan.action.enabled {
            plan.action = BillingActionState::disabled(plan.action.label.clone(), reason);
        }
    }
}

fn allocate_credits(total: u32, fractions: &[f32]) -> Vec<u32> {
    if fractions.is_empty() {
        return Vec::new();
    }
    let weights = fractions
        .iter()
        .map(|fraction| {
            if fraction.is_finite() && *fraction > 0. {
                *fraction
            } else {
                0.
            }
        })
        .collect::<Vec<_>>();
    let weight_sum: f32 = weights.iter().sum();
    if weight_sum <= 0. {
        let mut amounts = vec![0; fractions.len()];
        amounts[0] = total;
        return amounts;
    }
    let exact = weights
        .iter()
        .map(|weight| total as f32 * *weight / weight_sum)
        .collect::<Vec<_>>();
    let mut amounts = exact
        .iter()
        .map(|value| value.floor() as u32)
        .collect::<Vec<_>>();
    let mut remainders = exact
        .iter()
        .enumerate()
        .map(|(index, value)| (index, *value - value.floor()))
        .collect::<Vec<_>>();
    remainders.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    for (index, _) in remainders
        .into_iter()
        .take(total.saturating_sub(amounts.iter().sum()) as usize)
    {
        amounts[index] += 1;
    }
    amounts
}

fn plan(
    id: &'static str,
    name: &'static str,
    eyebrow: &'static str,
    description: &'static str,
    pricing: (&'static str, &'static str, &'static str),
    features: &[&'static str],
    presentation: (bool, bool, BillingActionState),
) -> BillingPlan {
    let (price, cadence, credits) = pricing;
    let (highlighted, current, action) = presentation;
    BillingPlan {
        id: id.into(),
        name: name.into(),
        eyebrow: eyebrow.into(),
        description: description.into(),
        price_label: price.into(),
        cadence_label: cadence.into(),
        credits_label: credits.into(),
        features: features.iter().map(|feature| (*feature).into()).collect(),
        highlighted,
        current,
        action,
    }
}

fn mock_daily_trend(range: BillingUsageRange, total: u32) -> Vec<BillingUsagePoint> {
    let days = match range {
        BillingUsageRange::Days7 => 7,
        BillingUsageRange::Days30 => 30,
        BillingUsageRange::Days90 => 90,
    };
    let weights: Vec<f32> = (0..days)
        .map(|day| 0.25 + ((day * 17 + 11) % 63) as f32 / 100.)
        .collect();
    let sum: f32 = weights.iter().sum();
    let peak = weights.iter().copied().fold(0.0_f32, f32::max);
    let mut allocated = 0;
    weights
        .into_iter()
        .enumerate()
        .map(|(day, weight)| {
            let amount = if day + 1 == days {
                total - allocated
            } else {
                (total as f32 * weight / sum).floor() as u32
            };
            allocated += amount;
            let label = match range {
                BillingUsageRange::Days7 => format!("Sep {}", day + 21),
                BillingUsageRange::Days30 if day < 3 => format!("Aug {}", day + 29),
                BillingUsageRange::Days30 => format!("Sep {}", day - 2),
                BillingUsageRange::Days90 if day == 0 => "Jun 30".to_owned(),
                BillingUsageRange::Days90 if day <= 31 => format!("Jul {day}"),
                BillingUsageRange::Days90 if day <= 62 => format!("Aug {}", day - 31),
                BillingUsageRange::Days90 => format!("Sep {}", day - 62),
            };
            BillingUsagePoint {
                label: label.into(),
                amount_label: format!("{amount} credits").into(),
                fraction: weight / peak,
            }
        })
        .collect()
}

pub(crate) fn update_usage_range(data: &mut BillingViewData, range: BillingUsageRange) {
    data.usage.range = range;
    if data.usage.trend.is_empty()
        && data.usage.categories.is_empty()
        && data.usage.api_keys.is_empty()
    {
        data.usage.period_label = match range {
            BillingUsageRange::Days7 => "Last 7 days",
            BillingUsageRange::Days30 => "Last 30 days",
            BillingUsageRange::Days90 => "Last 90 days",
        }
        .into();
        data.usage.total_label = "0 credits used".into();
        data.usage.allowance_fraction = 0.;
        return;
    }
    let (period, total) = match range {
        BillingUsageRange::Days7 => ("Last 7 days", 310),
        BillingUsageRange::Days30 => ("Last 30 days", 1_160),
        BillingUsageRange::Days90 => ("Last 90 days", 3_420),
    };
    data.usage.period_label = period.into();
    data.usage.total_label = format!("{total} credits used").into();
    let allowance = if range == BillingUsageRange::Days90 {
        9_000
    } else {
        3_000
    };
    data.usage.allowance_label = if range == BillingUsageRange::Days90 {
        "9,000 credits granted by the plan in this period".into()
    } else {
        "3,000 credits granted by the plan this month".into()
    };
    data.usage.allowance_fraction = total as f32 / allowance as f32;
    data.usage.trend = mock_daily_trend(range, total);
    let model_amounts = allocate_credits(
        total,
        &data
            .usage
            .categories
            .iter()
            .map(|category| category.fraction)
            .collect::<Vec<_>>(),
    );
    for (category, amount) in data.usage.categories.iter_mut().zip(model_amounts) {
        category.amount_label = format!("{amount} credits").into();
        category.detail = format!("{:.1}% of model usage", category.fraction * 100.).into();
    }
    let key_amounts = allocate_credits(
        total,
        &data
            .usage
            .api_keys
            .iter()
            .map(|key| key.fraction)
            .collect::<Vec<_>>(),
    );
    for (key, amount) in data.usage.api_keys.iter_mut().zip(key_amounts) {
        key.amount_label = format!("{amount} credits").into();
    }
}

pub(crate) fn seed_billing() -> BillingViewData {
    BillingViewData {
        selected_tab: BillingTab::Overview,
        workspace: BillingWorkspace {
            id: "fanta-studio".into(),
            name: "Fanta Studio".into(),
            role_label: "Workspace owner".into(),
        },
        balance: BillingBalance {
            available_label: "2,340".into(),
            available_caption: "Credits available".into(),
            included_label: "1,840 from plan".into(),
            purchased_label: "500 purchased".into(),
            refresh_label: "Updated just now".into(),
            note: "Credits are shared across this workspace. Plan grants and credit packs add to the balance; completed usage reduces it.".into(),
        },
        subscription: BillingSubscription {
            plan_name: "Pro".into(),
            status_label: "Active".into(),
            price_label: "$44.99".into(),
            cadence_label: "per month".into(),
            renewal_label: "Renews October 1, 2026".into(),
            provider: BillingProvider::Apple,
            provider_note: "Billed by Apple. Manage or cancel this subscription in your Apple account.".into(),
        },
        plans: vec![
            plan(
                "free", "Free", "For exploring", "Start designing with essential tools.",
                ("", "", "500 credits granted once"), &[
                    "1 seat",
                    "API access",
                    "Design and code workspace",
                ], (false, false,
                BillingActionState::disabled("Switch plan", "Manage your current Apple subscription first")),
            ),
            plan(
                "pro", "Pro", "Current plan", "For daily design and AI creation.",
                ("$44.99", "per month", "3,000 AI credits monthly"), &[
                    "AI image, video, audio & SVG creation",
                    "Workspace generation history",
                    "MCP and connected tools",
                ], (true, true,
                BillingActionState::disabled("Current plan", "Your workspace is already on Pro")),
            ),
            plan(
                "team", "Team", "For studios", "Coordinate work and usage across your team.",
                ("", "", "6,000 credits granted each month"), &[
                    "5 seats",
                    "API access",
                    "Everything in Pro",
                ], (false, false,
                BillingActionState::disabled("Unavailable", "Checkout is not available for this plan")),
            ),
        ],
        usage: BillingUsage {
            range: BillingUsageRange::Days30,
            period_label: "Last 30 days".into(),
            total_label: "1,160 credits used".into(),
            allowance_label: "3,000 credits granted by the plan".into(),
            reset_label: "Next plan grant October 1".into(),
            allowance_fraction: 1_160. / 3_000.,
            trend: mock_daily_trend(BillingUsageRange::Days30, 1_160),
            categories: vec![
                BillingUsageCategory { id: "fanta-image-1".into(), name: "Fanta Image".into(), detail: "Image · 146 requests".into(), amount_label: "520 credits".into(), fraction: 0.448, kind: BillingUsageKind::Image },
                BillingUsageCategory { id: "fanta-video-1".into(), name: "Fanta Video".into(), detail: "Video · 18 requests".into(), amount_label: "410 credits".into(), fraction: 0.353, kind: BillingUsageKind::Video },
                BillingUsageCategory { id: "fanta-voice-1".into(), name: "Fanta Voice".into(), detail: "Speech · 34 requests".into(), amount_label: "140 credits".into(), fraction: 0.121, kind: BillingUsageKind::Audio },
                BillingUsageCategory { id: "fanta-svg-1".into(), name: "Fanta SVG".into(), detail: "SVG · 31 requests".into(), amount_label: "90 credits".into(), fraction: 0.078, kind: BillingUsageKind::Vector },
            ],
            api_keys: vec![
                BillingUsageApiKey { id: "desktop".into(), name: "Desktop client key".into(), detail: "API key ·•••• 4C10".into(), amount_label: "670 credits".into(), fraction: 0.578 },
                BillingUsageApiKey { id: "studio-api".into(), name: "Studio automation".into(), detail: "API key ·•••• 9E2A".into(), amount_label: "340 credits".into(), fraction: 0.293 },
                BillingUsageApiKey { id: "team-api".into(), name: "Team workflow".into(), detail: "API key ·•••• B7F3".into(), amount_label: "150 credits".into(), fraction: 0.129 },
            ],
            note: "Usage is measured when a request completes. Refunds and adjustments appear in activity.".into(),
        },
        transactions: vec![
            BillingTransaction {
                id: "txn-2026-09-pro".into(),
                title: "Pro monthly".into(),
                detail: "3,000 credits added to Fanta Studio".into(),
                date_label: "Sep 1, 2026".into(),
                credits_label: "+3,000 credits".into(),
                amount_label: "$44.99".into(),
                status_label: "Completed".into(),
                receipt_action: Some(BillingActionState::enabled("Receipt")),
            },
            BillingTransaction {
                id: "txn-2026-09-pack".into(),
                title: "Credit pack".into(),
                detail: "One-time 500-credit purchase".into(),
                date_label: "Sep 12, 2026".into(),
                credits_label: "+500 credits".into(),
                amount_label: "$8.99".into(),
                status_label: "Completed".into(),
                receipt_action: Some(BillingActionState::enabled("Receipt")),
            },
        ],
        actions: BillingActions {
            add_credits: BillingActionState::enabled("Buy 500 credits · $8.99"),
            manage_subscription: BillingActionState::enabled("Manage with Apple"),
            restore_purchases: BillingActionState::enabled("Restore purchases"),
            redeem_code: BillingActionState::enabled("Redeem offer code"),
            sync_purchases: BillingActionState::enabled("Sync Apple purchases"),
            export_usage: BillingActionState::enabled("Export CSV"),
            refresh: BillingActionState::enabled("Refresh"),
        },
        ..BillingViewData::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credits(label: &str) -> u32 {
        label
            .split(' ')
            .next()
            .unwrap()
            .replace(',', "")
            .parse::<u32>()
            .unwrap()
    }

    #[test]
    fn mock_usage_ranges_have_one_point_per_day_and_reconcile_with_totals() {
        for (range, days, total) in [
            (BillingUsageRange::Days7, 7, 310),
            (BillingUsageRange::Days30, 30, 1_160),
            (BillingUsageRange::Days90, 90, 3_420),
        ] {
            let trend = mock_daily_trend(range, total);
            assert_eq!(trend.len(), days);
            assert_eq!(
                trend
                    .iter()
                    .map(|point| credits(&point.amount_label))
                    .sum::<u32>(),
                total,
            );
            assert!(
                trend
                    .iter()
                    .all(|point| (0.0..=1.0).contains(&point.fraction))
            );
        }
    }

    #[test]
    fn usage_ranges_reconcile_model_and_api_key_totals() {
        let mut data = seed_billing();
        assert_eq!(data.usage.categories[0].name, "Fanta Image");
        assert_eq!(data.usage.categories[1].name, "Fanta Video");
        assert_eq!(data.usage.categories[2].name, "Fanta Voice");
        assert_eq!(data.usage.categories[3].name, "Fanta SVG");
        for (range, total) in [
            (BillingUsageRange::Days7, 310),
            (BillingUsageRange::Days30, 1_160),
            (BillingUsageRange::Days90, 3_420),
        ] {
            update_usage_range(&mut data, range);
            assert_eq!(data.usage.range, range);
            assert_eq!(credits(&data.usage.total_label), total);
            assert_eq!(
                data.usage
                    .categories
                    .iter()
                    .map(|category| credits(&category.amount_label))
                    .sum::<u32>(),
                total,
            );
            assert_eq!(
                data.usage
                    .api_keys
                    .iter()
                    .map(|key| credits(&key.amount_label))
                    .sum::<u32>(),
                total,
            );
        }
    }

    #[test]
    fn named_states_expose_real_availability_and_provider_variants() {
        let apple = BillingNamedState::Ready.fixture();
        assert_eq!(apple.subscription.provider, BillingProvider::Apple);
        assert!(apple.actions.add_credits.enabled);

        let loading = BillingNamedState::Loading.fixture();
        assert_eq!(loading.load_state, BillingLoadState::Loading);
        assert!(!loading.actions.add_credits.enabled);
        assert!(loading.actions.refresh.enabled);

        let error = BillingNamedState::Error.fixture();
        assert!(matches!(error.load_state, BillingLoadState::Error(_)));
        assert!(!error.actions.add_credits.enabled);
        assert!(error.actions.refresh.enabled);

        let empty = BillingNamedState::Empty.fixture();
        assert_eq!(empty.subscription.provider, BillingProvider::None);
        assert!(empty.usage.trend.is_empty());
        assert!(empty.plans.is_empty());
        assert!(empty.transactions.is_empty());

        let unavailable = BillingNamedState::Unavailable.fixture();
        assert!(!unavailable.actions.add_credits.enabled);
        assert!(!unavailable.actions.restore_purchases.enabled);
        assert!(unavailable.plans.iter().all(|plan| !plan.action.enabled));

        let web = BillingNamedState::Web.fixture();
        assert_eq!(web.subscription.provider, BillingProvider::Web);
        assert!(web.actions.manage_subscription.enabled);
        assert!(!web.actions.add_credits.enabled);
        assert!(web.subscription.price_label.is_empty());
        assert_eq!(
            web.plans
                .iter()
                .find(|plan| plan.id == "pro")
                .unwrap()
                .price_label,
            ""
        );
    }

    #[test]
    fn dashboard_free_matches_the_live_account_shape_across_usage_ranges() {
        let mut dashboard = BillingNamedState::DashboardFree.fixture();
        assert_eq!(dashboard.subscription.plan_name, "Free");
        assert_eq!(dashboard.subscription.provider, BillingProvider::None);
        assert_eq!(dashboard.balance.available_label, "500");
        assert_eq!(dashboard.balance.included_label, "500 granted once");
        assert!(!dashboard.actions.add_credits.enabled);
        assert!(!dashboard.actions.manage_subscription.enabled);
        assert!(dashboard.plans.iter().all(|plan| !plan.action.enabled));
        assert!(dashboard.transactions.is_empty());
        assert!(dashboard.usage.trend.is_empty());

        let free = dashboard
            .plans
            .iter()
            .find(|plan| plan.id == "free")
            .unwrap();
        assert!(free.current);
        assert_eq!(free.credits_label, "500 credits granted once");
        let team = dashboard
            .plans
            .iter()
            .find(|plan| plan.id == "team")
            .unwrap();
        assert_eq!(team.credits_label, "6,000 credits granted each month");
        assert!(team.features.contains(&"5 seats".into()));
        assert!(team.price_label.is_empty());

        update_usage_range(&mut dashboard, BillingUsageRange::Days90);
        assert_eq!(dashboard.usage.period_label, "Last 90 days");
        assert_eq!(dashboard.usage.total_label, "0 credits used");
        assert_eq!(dashboard.balance.available_label, "500");
        assert!(dashboard.usage.trend.is_empty());
    }
}
