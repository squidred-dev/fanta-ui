# Editor toolbar

`EditorToolbar` is Fanta GPUI's reusable, host-controlled implementation of
Figma's floating editor toolbar. It covers Design, Motion, and Dev modes,
split-tool flyouts, contextual secondary controls, zoom, the Actions command
bar, a contextual Agent composer, and a capsule for host-supplied chrome
controls.

The component is editor chrome only. It never imports `fanta-engine`, mutates a
document, advances an animation, runs a plugin, or calls an AI service.

## Surface and placement

`EditorToolbar` renders one intrinsic, content-sized dock. It does not render a
full-canvas transparent root and it does not choose its own viewport position.
All persistent chrome is contained by the dock:

1. a mode-specific secondary row when applicable;
2. a horizontally constrained primary-tool row;
3. a utility row containing the Design, Motion, and Dev mode tray, zoom, the
   Agent launcher, and — when the host supplies any — the trailing chrome
   capsule.

The dock coordinates at most one transient tool-group, Actions, Agent, or zoom
surface at a time.

The dock occludes the pointer. Because hosts float it over the canvas, every
mouse event on the dock — clicks, presses, hovers, and wheel — stops at the
dock surface, so pressing a tool never also reaches the canvas underneath and
wheel gestures over the dock never pan or zoom it. The dock's own overflow
rows keep scrolling because they sit above that surface, and the transient
popups block on their own surfaces. Hosts whose canvas listens for pointer
events through GPUI hitboxes need no extra guard; a host that installs raw
window-level mouse handlers must still consult its hitbox hover state.

The host owns outer positioning. `PseudoEditor` and the standalone Toolbar
story mount the dock in a canvas-relative wrapper positioned at bottom center.
A production host can choose a different bottom inset, safe-area policy, or
surrounding clipping policy without changing `EditorToolbar`; constrained
widths scroll the dock's internal rows instead of shrinking or letting controls
escape its surface. When a row actually clips content, a pointer-transparent
theme-colored gradient fades the clipped edge so off-screen tools stay
discoverable.

Transient surfaces are local to their triggers. Split-tool menus open from the
corresponding disclosure, Actions opens from the Actions control, and the zoom
menu and Agent composer open from their respective utility controls. They may
use a deferred layer so they are not clipped by the dock, but they are never
centered against a full-canvas toolbar root. Popup widths respond to the window
and snap to an 8 px viewport margin when their preferred side would overflow.

Mode accents are blue for Design, purple for Motion, and green for Dev. Mode
selection is also expressed by a raised tile, not color alone.

Tool and mode artwork is Lucide. Where gpui-component's `IconName` set has a
fitting Lucide asset the toolbar renders that SVG (Frame, Star, Resources →
`layout-dashboard`, Inspect → `search`, Ready-for-dev → `circle-check`, the
Design mode tile → `square-dashed-mouse-pointer`); those assets resolve
against the host's asset source (ARCHITECTURE §5). Every other tool and mode
is a stroke path authored on Lucide's 24-unit grid with its 2-unit
round-capped stroke — `mouse-pointer-2`, `hand`, `scaling`, `scan`, `square`,
`arrow-up-right`, `circle`, `pentagon`, `image`, `pen-tool`, `pencil`,
`spline`, `type`, `message-square`, `sticky-note`, `ruler`, `sparkles`,
`pipette`, `code-xml`, `variable`, `refresh-cw`, `circle-play`,
`clapperboard`, plus Fanta's own section, path-select, text-on-path, and
motion glyphs in the same idiom — so mixed assets and drawings read as one
set. Font-dependent Unicode glyphs are never an icon source.

## Mode layouts

### Design

Design follows Figma's grouped primary strip:

| Position | Tools |
| --- | --- |
| Move split | Move, Hand, Scale, PathSelect |
| Region split | Frame, Section, Slice |
| Shape split | Rectangle, Line, Arrow, Ellipse, Polygon, Star, Image/video |
| Creation split | Pen, Pencil, NodeEdit, TextPath |
| Standalone | Text |
| Standalone | Resources (⇧ I) |
| Feedback split | Comment, Annotation, Measurement |
| Utility | Actions |

Design has no persistent secondary strip. Selection-specific properties remain
the responsibility of the host's inspector.

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

`MotionToolbarOptions` supplies playback, looping, autokey, time, duration,
style, and the `available_animation_styles` catalog behind the style chip's
anchored menu. The component displays those values but never advances time
itself. The storybook host renders a mock timeline to demonstrate the
integration.

### Dev

Dev presents Move, color picker, measurement, annotation, comment, and Actions.
Its secondary handoff strip contains Inspect, Annotate, Measure, and
Ready-for-dev. Readiness is supplied through `DevToolbarOptions`; every other
operation is a typed intent for the host.

## Host chrome capsule

Editor-level affordances such as fit-to-view and the sidebar toggles are host
chrome, not canvas tools (§12: the host owns chrome, the toolbar owns the
dock). Rather than rendering a second bubble beside the dock, a host describes
them with `ToolbarChromeControl` records and the toolbar renders them as one
capsule at the end of the utility row, after the Agent launcher:

```rust
toolbar.set_chrome_controls(
    [
        ToolbarChromeControl::new("fit", IconName::Maximize, "Fit to view").shortcut("⇧ 1"),
        ToolbarChromeControl::new("layers", IconName::PanelLeftClose, "Hide layers")
            .active(true),
        ToolbarChromeControl::new("inspector", IconName::PanelRightOpen, "Show inspector"),
    ],
    cx,
);
```

Each record carries a stable `id`, an `IconName` (resolved against the host's
asset source), a tooltip `label`, an optional `shortcut` hint, and an `active`
flag rendered as the same raised tile the selected mode uses. Activating a
control — pointer, Enter, or Space — emits
`ToolbarAction::ChromeControlInvoked { id }`; the toolbar never flips
`active` itself. The host applies the effect and echoes the new state through
`set_chrome_controls`, which preserves order, drops later duplicate ids, and
removes the capsule when given an empty list. Chrome controls sit inside the
dock's pointer occlusion like every other control, and the capsule's width is
subtracted before the zoom cluster picks its collapse tier, so a narrow host
sheds zoom steppers rather than scrolling the row.

## Zoom cluster

The utility row's zoom cluster contains a minus stepper, the percentage
trigger, and a plus stepper. The steppers move through Figma's multiplicative
ladder (1, 2, 3, 6, 12, 25, 50, 100, 200, 400, 800, 1,600, 3,200 percent):
each press requests the nearest ladder step in its direction, clamped to the
1–3,200 range the host also enforces.

The percentage trigger opens a menu of Figma's zoom entries with shortcut
hints:

| Entry | Shortcut | Intent |
| --- | --- | --- |
| Zoom in | + | `ZoomChangeRequested` at the next ladder step |
| Zoom out | − | `ZoomChangeRequested` at the previous ladder step |
| Zoom to fit | ⇧ 1 | `CommandInvoked` with `ZoomToFit` |
| Zoom to selection | ⇧ 2 | `CommandInvoked` with `ZoomToSelection` |
| Zoom to 100% | ⇧ 0 | `ZoomChangeRequested` at 100 |
| Zoom to 50% | | `ZoomChangeRequested` at 50 |

Up/Down/Home/End move the menu highlight, Enter commits it, and Escape
dismisses. The ⇧ 1, ⇧ 2, and ⇧ 0 shortcuts also work directly from the
toolbar without opening the menu. The host stays authoritative: fit and
selection zoom levels are host knowledge, so those entries emit commands
rather than percentages.

## Actions command bar

Actions is available in all three modes and through Command/Ctrl+K or the legacy
Command/Ctrl+/ shortcut. The palette contains:

- a focused search field with an explicit clear affordance;
- All, Assets, and Plugins & widgets scopes;
- the complete filtered `ToolbarCommand` list in a bounded scroll viewport;
- pointer hover and Up/Down keyboard highlighting;
- Enter execution and Escape dismissal;
- labels, categories, descriptions, and shortcut hints.

The default catalog contains AI, edit, selection, layout, view, file,
resource, mode, collaboration, plugin, and widget commands. A host can provide
an allowed ordered subset with `set_commands`. Selecting one emits exactly one
`ToolbarAction::CommandInvoked`; command execution remains host-owned.
The palette is positioned above and adjacent to its Actions trigger rather
than at the center of the canvas.

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
The composer is anchored to the Agent launcher inside the dock.

No credentials, network client, generated operations, or request results live
inside `fanta-gpui`.

## Controlled state and typed intents

| Host-controlled read model | Component-owned presentation |
| --- | --- |
| `ToolbarMode` and `ToolbarTool` | Hover, pressed, focus, and tooltip styling |
| Canvas zoom | Open overlay: flyout, option editor, Actions, Agent, or zoom |
| `MotionToolbarOptions` values and animation-style catalog | Menu and option-editor highlight cursor |
| `DevToolbarOptions` | Actions query, scope, result cursor, and scroll |
| `AgentToolbarOptions` | Unsent Agent prompt |
| `ToolbarChromeControl` list, including each `active` flag | Input focus continuity |
| Allowed `ToolbarCommand` order/subset | Row scroll offsets and overflow-fade visibility |

The main intent families are:

- `ModeChangeRequested`;
- `ToolChangeRequested`;
- `SecondaryControlInvoked`;
- `ControlChangeRequested`;
- `CommandQueryChanged` and `CommandInvoked`;
- `AiPromptSubmitted`, Agent attachment/voice/visibility;
- `ZoomChangeRequested`;
- `ChromeControlInvoked`.

Hosts accept an intent by updating their model and calling the corresponding
setter. Rejecting an intent simply means continuing to provide the old value.

## Integration

```rust
use fanta_gpui::prelude::*;
use gpui::{div, prelude::*, px};

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

let canvas = div()
    .relative()
    .size_full()
    .child(document_canvas)
    .child(
        div()
            .absolute()
            .left_0()
            .right_0()
            .bottom(px(18.))
            .flex()
            .justify_center()
            .child(toolbar.clone()),
    );

toolbar.update(cx, |toolbar, cx| {
    toolbar.set_agent_options(
        AgentToolbarOptions::new("Checkout frame")
            .suggestions(["Explore 3 directions", "Polish this", "Animate this"]),
        cx,
    );
    toolbar.set_commands(ToolbarCommand::ALL.iter().copied(), cx);
    toolbar.set_chrome_controls(host.chrome_controls(), cx);
});

cx.subscribe(&toolbar, |host, toolbar, action: &ToolbarAction, cx| {
    host.apply_toolbar_intent(action);
    toolbar.update(cx, |toolbar, cx| {
        toolbar.set_mode(host.toolbar_mode(), cx);
        toolbar.set_active_tool(host.toolbar_tool(), cx);
        toolbar.set_zoom_percent(host.canvas_zoom(), cx);
        toolbar.set_motion_options(host.motion_toolbar_options(), cx);
        toolbar.set_dev_options(host.dev_toolbar_options(), cx);
        toolbar.set_agent_options(host.agent_toolbar_options(), cx);
        toolbar.set_chrome_controls(host.chrome_controls(), cx);
    });
});
```

The wrapper is deliberately host code. It supplies the canvas-relative
bottom-center placement while `EditorToolbar` keeps its intrinsic dimensions.

Call `fanta_gpui::init` once after `gpui_component::init` to register the
default actions and key bindings. A host that reloads or replaces its keymap
should rebuild those programmatic bindings before applying user bindings. Fanta
also provides a narrowly scoped editing fallback for Toolbar and Pages inputs
when an integrating host clears all bindings; it is not a replacement for
restoring the complete component keymap.

## Keyboard behavior

- Tab and Shift-Tab reach primary tools, split disclosures, modes, secondary
  controls, zoom, Actions scopes/results, Agent controls, the launcher, and
  the host chrome controls.
- Enter and Space activate a focused toolbar control. While a split-tool
  flyout, the zoom menu, or a chip's choice editor is open, they commit the
  highlighted row instead.
- Escape closes the current overlay and returns focus to the toolbar surface.
- Up and Down wrap through all filtered Actions results and keep the active row
  scrolled into view. The same keys, plus Home and End, move the highlight in
  split-tool flyouts, the zoom menu, and chip choice editors.
- Left and Right move the highlight in an open chip choice editor, like Up
  and Down.
- Command/Ctrl+K and Command/Ctrl+/ open Actions.
- Command/Ctrl+Enter opens Agent.
- Tool shortcuts include V, H, K, F, Shift+S, S, R, L, Shift+L, O,
  Shift+Command/Ctrl+K, P, Shift+P, T, C, Shift+T, Shift+M, and Shift+I.
- Shift+1, Shift+2, and Shift+0 request zoom to fit, zoom to selection, and
  zoom to 100%.
- Shift+D requests Dev mode.
- While the Actions or Agent text field is open, unmodified tool shortcuts use
  a separate key context and do not intercept typed text.
- Focused text fields retain deletion, selection/navigation, clipboard,
  undo/redo, Enter, and Escape behavior after a destructive host keymap reload.
  The same scoped guarantee applies to other Fanta inputs such as Pages search.

If a shortcut requests a tool unavailable in the current mode, the toolbar
emits a mode request for the first compatible mode followed by the tool
request. The host remains authoritative for both.

## Storybook and visual QA

Run:

```sh
cargo run -p fanta-gpui-storybook
```

The Toolbar story is the mock host and applies every emitted intent back
through the controlled setters; it seeds three chrome controls (fit to view
plus the two sidebar toggles, whose active state mirrors the story's mock
columns) and logs every intent. Deterministic screenshot states are available
for local QA:

```sh
FANTA_TOOLBAR_MODE=motion cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_MODE=dev cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_OVERLAY=actions cargo run -p fanta-gpui-storybook
FANTA_TOOLBAR_OVERLAY=agent cargo run -p fanta-gpui-storybook
```

Both variables seed the initial values of the Toolbar story's runtime
knobs. In the Gallery the same mode, overlay, and zoom states are
switchable live from the knob rows under the story surface, so the
environment variables stay a restart-style launch interface for
screenshot capture rather than the only way to reach a state.
