# Editor toolbar

`EditorToolbar` is Fanta GPUI's reusable, host-controlled implementation of
Figma's floating editor toolbar. It covers Design, Draw, Motion, and Dev modes,
split-tool flyouts, contextual secondary controls, zoom, the Actions command
bar, and a contextual Agent composer.

The component is editor chrome only. It never imports `fanta-engine`, mutates a
document, advances an animation, runs a plugin, or calls an AI service.

## Surface

The desktop presentation uses the same coordinated regions as current Figma:

1. a separate zoom capsule at bottom left;
2. a centered 48 px primary capsule;
3. a nested mode tray ordered Draw, Design, Motion, Dev;
4. a mode-specific secondary capsule when applicable;
5. a separate Agent launcher at bottom right;
6. one overlay at a time for a tool group, Actions, Agent, or zoom.

Mode accents are cyan for Draw, blue for Design, purple for Motion, and green
for Dev. Mode selection is also expressed by a raised tile, not color alone.

## Mode layouts

### Design

Design follows Figma's grouped primary strip:

| Position | Tools |
| --- | --- |
| Move split | Move, Hand, Scale |
| Region split | Frame, Section, Slice |
| Shape split | Rectangle, Line, Arrow, Ellipse, Polygon, Star, Image/video |
| Creation split | Pen, Pencil |
| Standalone | Text |
| Feedback split | Comment, Annotation, Measurement |
| Utility | Actions |

Design has no persistent secondary strip. Selection-specific properties remain
the responsibility of the host's inspector.

### Draw

Draw keeps navigation and creation tools visible while adding illustration and
vector-edit flyouts:

- Move/Hand/Scale;
- Pen;
- Brush/Paint bucket;
- Pencil;
- Shape builder/Lasso/Variable width;
- Region and Shape groups;
- Text, Feedback, and Actions.

For Brush, Pencil, Paint bucket, and Variable width, the secondary strip shows
the controlled stroke color, brush style, weight, smoothing, and pressure
state. Each activation emits `ControlChangeRequested`; the host accepts it by
supplying a new `DrawToolbarOptions`.

### Motion

Motion retains the Design creation tools and adds a Motion split containing
Motion select, preview, keyframe, autokey, motion path, animation style, and
time-comment tools. Its secondary transport contains:

- play/pause and loop;
- current time and duration;
- autokey;
- add keyframe;
- animation style;
- timeline and time-comment actions.

`MotionToolbarOptions` supplies playback, looping, autokey, time, duration, and
style. The component displays those values but never advances time itself. The
storybook host renders a mock timeline to demonstrate the integration.

### Dev

Dev presents Move, color picker, measurement, annotation, comment, and Actions.
Its secondary handoff strip contains Inspect, Annotate, Measure, and
Ready-for-dev. Readiness is supplied through `DevToolbarOptions`; every other
operation is a typed intent for the host.

## Actions command bar

Actions is available in all four modes and through Command/Ctrl+K or the legacy
Command/Ctrl+/ shortcut. The palette contains:

- a focused search field;
- All, Assets, and Plugins & widgets scopes;
- the complete filtered `ToolbarCommand` list in a bounded scroll viewport;
- pointer hover and Up/Down keyboard highlighting;
- Enter execution and Escape dismissal;
- labels, categories, descriptions, and shortcut hints.

The default catalog contains AI, edit, selection, layout, view, file,
resource, mode, collaboration, plugin, and widget commands. A host can provide
an allowed ordered subset with `set_commands`. Selecting one emits exactly one
`ToolbarAction::CommandInvoked`; command execution remains host-owned.

## Contextual Agent composer

The Agent launcher opens a contextual on-canvas composer. The host supplies
`AgentToolbarOptions`:

- `context_label`, such as the selected frame;
- up to three suggestion chips;
- the mention/attachment hint.

The component owns only the unsent prompt, focus, and open presentation.
Submitting emits `AiPromptSubmitted`. The plus and empty-state voice controls
emit `AgentAttachmentRequested` and `AgentVoiceInputRequested`. Opening or
replacing the composer emits a balanced `AgentVisibilityChanged` event.

No credentials, network client, generated operations, or request results live
inside `fanta-gpui`.

## Controlled state and typed intents

| Host-controlled read model | Component-owned presentation |
| --- | --- |
| `ToolbarMode` and `ToolbarTool` | Hover, pressed, and focus styling |
| Canvas zoom | Zoom flyout visibility |
| `DrawToolbarOptions` | Open tool-group flyout |
| `MotionToolbarOptions` | Actions query, scope, result cursor, and scroll |
| `DevToolbarOptions` | Unsent Agent prompt |
| `AgentToolbarOptions` | Open overlay and tooltip presentation |
| Allowed `ToolbarCommand` order/subset | Input focus continuity |

The main intent families are:

- `ModeChangeRequested`;
- `ToolChangeRequested`;
- `SecondaryControlInvoked`;
- `ControlChangeRequested`;
- `CommandQueryChanged` and `CommandInvoked`;
- `AiPromptSubmitted`, Agent attachment/voice/visibility;
- `ZoomChangeRequested`.

Hosts accept an intent by updating their model and calling the corresponding
setter. Rejecting an intent simply means continuing to provide the old value.

## Integration

```rust
use fanta_gpui::prelude::*;

let toolbar = cx.new(|cx| {
    EditorToolbar::new(
        "editor-toolbar",
        ToolbarMode::Design,
        ToolbarTool::Move,
        100,
        window,
        cx,
    )
});

toolbar.update(cx, |toolbar, cx| {
    toolbar.set_agent_options(
        AgentToolbarOptions::new("Checkout frame")
            .suggestions(["Explore 3 directions", "Polish this", "Animate this"]),
        cx,
    );
    toolbar.set_commands(ToolbarCommand::ALL.iter().copied(), cx);
});

cx.subscribe(&toolbar, |host, toolbar, action: &ToolbarAction, cx| {
    host.apply_toolbar_intent(action);
    toolbar.update(cx, |toolbar, cx| {
        toolbar.set_mode(host.toolbar_mode(), cx);
        toolbar.set_active_tool(host.toolbar_tool(), cx);
        toolbar.set_zoom_percent(host.canvas_zoom(), cx);
        toolbar.set_draw_options(host.draw_toolbar_options(), cx);
        toolbar.set_motion_options(host.motion_toolbar_options(), cx);
        toolbar.set_dev_options(host.dev_toolbar_options(), cx);
        toolbar.set_agent_options(host.agent_toolbar_options(), cx);
    });
});
```

Call `fanta_gpui::init` once after `gpui_component::init` to register the
default actions and key bindings.

## Keyboard behavior

- Tab and Shift-Tab reach primary tools, split disclosures, modes, secondary
  controls, zoom, Actions scopes/results, Agent controls, and the launcher.
- Enter and Space activate a focused toolbar control.
- Escape closes the current overlay and returns focus to the toolbar surface.
- Up and Down wrap through all filtered Actions results and keep the active row
  scrolled into view.
- Command/Ctrl+K and Command/Ctrl+/ open Actions.
- Command/Ctrl+Enter opens Agent.
- Tool shortcuts include V, H, K, F, Shift+S, S, R, L, Shift+L, O,
  Shift+Command/Ctrl+K, P, Shift+P, T, C, Shift+T, and Shift+M.
- Shift+D requests Dev mode.
- While the Actions or Agent text field is open, unmodified tool shortcuts use
  a separate key context and do not intercept typed text.

If a shortcut requests a tool unavailable in the current mode, the toolbar
emits a mode request for the first compatible mode followed by the tool
request. The host remains authoritative for both.

## Storybook and visual QA

Run:

```sh
cargo run -p fanta-gpui-storybook
```

The Toolbar story is the mock host and applies every emitted intent back
through the controlled setters. Deterministic screenshot states are available
for local QA:

```sh
FANTA_TOOLBAR_MODE=draw cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_MODE=motion cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_MODE=dev cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_OVERLAY=actions cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_OVERLAY=agent cargo run -p fanta-gpui-storybook
```
