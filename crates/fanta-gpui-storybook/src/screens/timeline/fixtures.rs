use super::*;
fn property(prefix: &str, name: &str, values: &[(u32, &str)]) -> TimelineProperty {
    TimelineProperty {
        id: prefix.to_owned().into(),
        name: name.to_owned().into(),
        value: values.last().unwrap().1.to_owned().into(),
        keyframes: values
            .iter()
            .enumerate()
            .map(|(i, (time, value))| TimelineKeyframe {
                id: format!("{prefix}-{i}").into(),
                time_ms: *time,
                value: (*value).to_owned().into(),
                easing: TimelineEasing::EaseOut,
            })
            .collect(),
    }
}
pub(super) fn fixture(state: TimelineNamedState) -> TimelineViewData {
    if state == TimelineNamedState::Empty {
        return TimelineViewData::default();
    }
    let mut logo = TimelineTrack::new("logo", "Fanta mark");
    logo.selected = true;
    logo.properties = vec![
        property(
            "logo-y",
            "Position Y",
            &[(0, "24"), (600, "0"), (1800, "0")],
        ),
        property("logo-opacity", "Opacity", &[(0, "0%"), (450, "100%")]),
        property("logo-scale", "Scale", &[(0, "80%"), (750, "100%")]),
    ];
    let mut headline = TimelineTrack::new("headline", "Create something great");
    headline.properties = vec![
        property("headline-y", "Position Y", &[(250, "16"), (850, "0")]),
        property("headline-opacity", "Opacity", &[(250, "0%"), (700, "100%")]),
    ];
    let mut button = TimelineTrack::new("button", "Get started");
    button.properties = vec![property(
        "button-scale",
        "Scale",
        &[(600, "90%"), (1100, "100%")],
    )];
    button.clips = vec![TimelineClip {
        id: "button-pulse".into(),
        name: "Pulse".into(),
        start_ms: 1400,
        end_ms: 2400,
        easing: TimelineEasing::Gentle,
    }];
    let mut data = TimelineViewData {
        duration_ms: 3000,
        current_time_ms: 900,
        tracks: vec![logo, headline, button],
        selected_keyframes: vec!["logo-y-1".into()],
        comments: vec![TimelineComment {
            id: "intro-review".into(),
            time_ms: 1200,
            label: "Review the entrance timing".into(),
        }],
        presets: [
            "Fade in", "Fade out", "Slide up", "Scale in", "Pulse", "Rotate",
        ]
        .into_iter()
        .map(|name| TimelinePreset {
            id: name.to_lowercase().replace(' ', "-").into(),
            name: name.into(),
        })
        .collect(),
        read_only: state == TimelineNamedState::ReadOnly,
        height: 400,
        ..Default::default()
    };
    if state == TimelineNamedState::ManyLayers {
        for i in 0..30 {
            let mut t = TimelineTrack::new(format!("item-{i}"), format!("Card {:02}", i + 1));
            t.properties = vec![property(
                &format!("item-{i}-opacity"),
                "Opacity",
                &[(i * 40, "0%"), (i * 40 + 500, "100%")],
            )];
            data.tracks.push(t);
        }
    }
    data
}
