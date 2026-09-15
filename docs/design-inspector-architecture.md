# Design inspector architecture

This document is the implementation contract for incrementally decomposing
the Design inspector. It complements the feature-level behavior documented in
[`design-panel.md`](design-panel.md); it does not redefine the panel's visual
language.

## Dependency direction

```text
host snapshot
  -> DesignPanel facade
    -> Design section projection/controller
      -> molecules::inspector
        -> atoms + gpui-component
```

`DesignPanel` remains the stable host-facing `Entity`. Its façade owns the
public constructors, compatibility setters and selectors, and the
`DesignPanelAction` event stream. `DesignPanelFactory` owns retained child
entity and subscription assembly. `DesignPanelViewDataController` owns the
canonical snapshot projection, apply operations, and host-echo reconciliation.
`set_view_data` delegates once into that controller; granular setters remain
source-compatible thin wrappers around the same controller operations, not a
second state-mutation path.
New section modules live below `organisms/design/panel/sections` and do not add
inherent `impl DesignPanel` blocks. A renderer first creates a narrow immutable
projection, then maps generic inspector events through a section controller to
the existing typed action.

The architecture test freezes the remaining legacy extension modules. Moving
a section is complete only when its allowlist entry is removed.

Browser/resource behavior, Component/Instance authoring, property editing,
numeric scrubbing, paint and effect workflows, typography, and option-field
coordination, Layout, and Layout Guides now cross explicit internal controller
extension traits. Their existing selectors and action contracts remain
facade-compatible, but those feature files do not add inherent methods to
`DesignPanel`. The one public scrub-speed compatibility query remains a thin
facade wrapper over `DesignScrubController`. Retained Grid dimensions and Frame
preset interactions still transition through the central overlay coordinator;
the Layout controller only narrows how the facade reaches that behavior.
Page inspection now follows the same boundary: `PagePanelController` constructs
the immutable projection and revalidates typed `PageEvent` values, while the
renderer receives only `PageProjection`, `PageChrome`, and `PageEventSink`.
Top-level header, section dispatch, scrolling, and host-owned surface assembly
live in `DesignPanelShellController`; the facade's `Render` implementation only
delegates to that shell.

## State ownership

| State | Owner | Lifetime |
| --- | --- | --- |
| Inspection context, node projections, property states, resources | Host via `DesignPanelViewData` | Host snapshot |
| Active editor/viewer surface and inspector preferences | Host via snapshot or compatibility setter | Cross-selection |
| Property, paint, crop, component-authoring rename/reorder, numeric/text draft, and scrub transactions | Edit controllers | One balanced edit |
| Active popover/dialog and return focus | Overlay coordinator | One overlay interaction |
| Disclosure, search, and feature-local cursor state | Section/retained child | Presentation-only |
| Retained text inputs, Draw sliders, and per-property select children | `DesignPanelRetainedChildren` | Entity lifetime |
| Scroll offset and its post-context reset | `DesignPanelShellState` | Presentation-only |

The façade declares ownership groups rather than individual widgets:
`host`, `resources`, `preferences`, `features`, `overlays`, `sections`, `edit`,
`retained`, and `shell`. `DesignPanelFactory` is the only place that assembles
retained children, and a child's subscription is stored beside the child it
observes so replacing one replaces both. An architecture test fails if a loose
retained field returns to the façade.

Production Design-panel code transitions overlays through the coordinator's
typed open, replace, toggle, and discard operations. Overlay slots are an
implementation detail of that coordinator; section controllers consume only
its visibility and payload projections.

The inspection context is authoritative for selection identity, order,
permissions, and inspected node data. The legacy `node` setter remains only as
a compatibility adapter and page fallback while existing hosts migrate.
Storybook keeps those concerns explicit in `DesignMockHostState`,
`DesignMockEditState`, and `DesignHarnessState`; its reducers own mock document
values and rollback snapshots, then echo them through one
`DesignPanelViewData` update instead of issuing repeated setter batches.

## Controlled edit contract

Every editable molecule presents `Unset`, `Mixed`, or `Uniform` host data and
an independent access value: `Editable`, `ReadOnly`, or `Disabled`. Binding and
validation metadata do not change the underlying value state.

An edit has exactly this lifecycle:

```text
Begin -> Preview* -> Commit | Cancel
```

The component may retain draft text, parse expressions, nudge, scrub, and
deduplicate previews. It never mutates a document value or changes the target
captured at `Begin`. A selection, permission, capability, or target change
must terminally cancel an invalidated transaction before clearing it.

## Component taxonomy

- Structure: section group, section, disclosure header.
- Layout: field grid, row, and shared-border field group.
- Fields: frame plus text, number, picker, slider, toggle, checkbox, color
  field, and swatch recipes.
- Actions: action group/button and segmented modes.
- Collections: selectable/reorderable row chrome with one border owner.
- Overlays: shared anchored placement and menu/popover surfaces; the retained
  owner supplies outside-click and Escape dismissal through the coordinator.
- Feedback: field help/validation, section feedback, and compact empty state.
- Metrics: one token-backed configuration for row/header geometry, label and
  icon sizing, padding, and compact/wide thresholds.

These names follow interaction behavior in Spectrum/Figma taxonomies. Styling
continues to use Fanta's active GPUI theme and Lucide icon pipeline.

## Slice acceptance checklist

Before removing a legacy section exemption:

1. Preserve its public selectors, actions, constructors, setters, and
   permission/capability matrix.
2. Keep the renderer free of an inherent `DesignPanel` implementation.
3. Cover applicable unset, mixed, uniform, bound, read-only, disabled, and
   stale-target states.
4. Verify pointer/Enter/Space parity and one terminal event per edit.
5. Verify Escape priority, outside-click dismissal, focus restoration, and
   host echoes while editing.
6. Render without border shifts at 320, 400, and 472 pixels.
7. Run the workspace check, test, clippy, and formatting gates from
   `AGENTS.md`.
