# Editor toolbar

`EditorToolbar` is a reusable, controlled GPUI dock. It emits typed intents;
it does not own a document or implement painting, cropping, animation, or
command execution. Fanta Edit integration is a separate adapter change.

## Layout

Design uses one row. The leading dropdown selects Design, Motion, Draw or
Dev; tools follow, then Actions and optional host chrome. Zoom and Fit to view live in the independent `ZoomBar` component. Scale,
Resources, Agent and the default sidebar toggles have been removed. Legacy
Scale/Resources/Plugin/Widget enum values are excluded from catalogs and
menus, even when supplied to `set_commands`.

Motion, Draw and Dev may add one contextual row above the main row. No mode
has a third row. At narrow widths tool/settings strips scroll horizontally;
Actions and the mode dropdown stay visible. Context settings use compact icons and values with descriptive tooltips. Zoom commands remain available in Actions and via Shift+1 / Shift+2 / Shift+0.

The host positions the content-sized dock, usually at canvas bottom center.
Tool/settings menus align to the nearest dock edge, four pixels above its top.
Actions centers over the dock with the same four-pixel gap; its results viewport
shrinks to fit above the toolbar in short windows. Popups clamp to the window edges.
Draw settings use the compact body typography for labels, values and hints.
Numeric fields reserve room for their maximum supported values. Draw preset
and Motion style dropdowns keep a fixed width, with long names truncated and
available in tooltips. Motion time also keeps a fixed slot, so accepted value
changes never resize the dock or move its controls.
Wheel events on menus and the dock stop before
reaching the canvas, even when a scroll view reaches its boundary.

## Modes

- Design: Move/Hand/Path selection, Frame/Section/Slice, shapes and media,
  Pen/Pencil/Edit path/Text on path, Text, comments/annotations/measurement.
- Motion: design creation tools plus motion path and time comments. The upper
  row contains play/pause, loop, current time, auto key, keyframe, the
  host-supplied animation-style dropdown, timeline and time comments.
- Draw: Hand, a Brush/Pencil/Eraser dropdown, a selection dropdown containing
  Rectangle/Ellipse/Lasso/Polygonal lasso/Magic wand, Crop, a
  Path selection/Edit path/Pen dropdown, and color sampling.
- Dev: inspection, annotation, measurement, comments, readiness, color sampling
  and Actions.

## Drawing contract

Hosts call `set_draw_options(DrawToolbarOptions, cx)` to supply accepted values.
The active tool determines the upper row:

| Tool | Settings/actions |
| --- | --- |
| Brush, Pencil, Eraser | Host brush-tip catalog, size (1–5000 px), hardness, opacity, flow, smoothing (0–100%), pen pressure |
| Rectangle, ellipse, lasso, polygonal lasso | New/add/subtract/intersect selection, feather (0–1000 px), anti-alias, invert, deselect |
| Magic wand | Selection settings plus tolerance (0–255) and contiguous pixels |
| Crop | Host ratio catalog, delete-cropped-pixels preference, cancel, apply |
| Path selection, Edit path, Pen | Close path, join paths, simplify |

Numeric chips open a precise text input and the shared Slider. Size uses a
nonlinear slider so smaller brushes remain easy to select. Enter or releasing
a slider emits `DrawOptionsChangeRequested { options }`. Escape/outside click
abandons uncommitted drafts. The host echoes accepted options; the toolbar
never updates accepted settings on its own. Selecting a tip returns its host
label; the host resolves that tip's brush definition and any preset defaults.

`DrawActionInvoked { action }` reports crop and selection/path operations.
The host validates those operations against its document and selection. The
Storybook echoes options and logs operation requests without simulating an
editing engine.

## Commands and keyboard

Actions has one fuzzy-search list with unique Lucide icons and shortcuts.
`set_commands` controls its order/subset. Assets, plugins and widgets have no
filter tabs or searchable entries. All retained commands emit
`CommandInvoked`; none opens an internal Agent UI.

- Enter/Space activate controls; arrows navigate menus; Escape dismisses.
- Command/Ctrl+K or Command/Ctrl+/ opens Actions.
- B/E/M/W select Brush/Eraser/Rectangle selection/Magic wand.
- In Draw, L/Shift+L select Lasso/Polygonal lasso, Shift+M selects Ellipse
  selection, and C selects Crop. Other modes retain their design bindings.
- Focused inputs retain text editing/deletion instead of invoking canvas tools.

## Storybook

Run `cargo run -p fanta-gpui-storybook` for the complete gallery.
`FANTA_STORYBOOK_GALLERY_STORY=toolbar` opens the gallery on this story.
Mode/overlay/zoom knobs exercise the controlled host adapter. Launch values
`FANTA_TOOLBAR_MODE=design|motion|draw|dev` and
`FANTA_TOOLBAR_OVERLAY=actions` are supported for visual QA.

Tests cover the mode dropdown, one/two-row layout, narrow widths, centered
Actions, scroll isolation, input editing, flyout keyboard navigation, controlled
Draw settings, crop intents, and tool/command icon coverage.

## ZoomBar

`fanta_gpui::zoom_bar::{ZoomBar, ZoomBarAction}` is independent of `EditorToolbar`.
Construct it with `ZoomBar::new(id, percent, cx)` and mount it wherever the host
needs canvas navigation, such as a sidebar. It is not mounted in the toolbar
preview. The separate ZoomBar story demonstrates standalone use.

Minus/plus step a zoom ladder; the percentage menu exposes Fit to view, Fit to
selection, 100% and 50%. The dedicated fit icon emits `FitToViewRequested`.
Hosts accept a `ZoomChangeRequested` by calling `set_percent`. Fit operations
remain host responsibilities. Menus align to the bar edge and open below a
bar near the top, or above when there is space; pointer and wheel events stay
inside the bar/menu. Keyboard arrows, Home/End, Enter/Space and Escape work.
