//! Independent mock host for the freely placeable ZoomBar.
use crate::*;
use fanta_gpui::zoom_bar::{ZoomBar, ZoomBarAction};
pub(crate) struct ZoomBarScreen {
    pub(crate) bar: Entity<ZoomBar>,
    pub(crate) last_action: SharedString,
    _subscription: Subscription,
}
impl ZoomBarScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let bar = cx.new(|cx| ZoomBar::new("storybook-zoombar", 100, cx));
        let subscription = cx.subscribe(&bar, |story, bar, action: &ZoomBarAction, cx| {
            let percent = match action {
                ZoomBarAction::ZoomChangeRequested { percent } => *percent,
                ZoomBarAction::FitToViewRequested => 75,
                ZoomBarAction::FitToSelectionRequested => 150,
            };
            bar.update(cx, |bar, cx| bar.set_percent(percent, cx));
            story.zoom_bar_screen.last_action =
                format!("Host accepted {action:?}; zoom {percent}%").into();
            cx.notify();
        });
        Self {
            bar,
            last_action: "ZoomBar can be placed independently of the editing tools".into(),
            _subscription: subscription,
        }
    }
}
impl Storybook {
    pub(crate) fn render_zoom_bar_story(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .size_full()
            .relative()
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundTertiary.resolve(cx))
            .child(
                div()
                    .absolute()
                    .top_4()
                    .right_4()
                    .child(self.zoom_bar_screen.bar.clone()),
            )
            .into_any_element()
    }
}
