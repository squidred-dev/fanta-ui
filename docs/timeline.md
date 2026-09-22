# Motion timeline

`Timeline` is a controlled GPUI organism. It consumes `TimelineViewData` and
emits `TimelineAction`; the application owns animation evaluation and edits.
The Storybook Timeline entry provides Animated, Empty, Many layers and Read
only scenes and applies requests to mock data.

## Layout and navigation

The transport occupies a separate header. The ruler, track bars, clips,
property rows, playhead and keyframe hit targets share a single time-to-pixel
mapping. Layer labels and lanes share one vertical scroll viewport. A shared
horizontal pan offset keeps the ruler and tracks synchronized. Narrow headers
scroll horizontally; value fields reserve fixed widths.

Scrub the ruler, enter a current time, or use the previous/next keyframe
controls. Switch seconds/milliseconds. The zoom slider, pinch, and Command/Ctrl-wheel
zoom the timeline; horizontal wheel or Shift-wheel pans it. Fit resets pan
and fits the animation duration. All wheel events stop at the timeline or its
open menu instead of reaching the editor canvas.

## Controlled editing

- Play/pause, Once/Loop/Ping-pong, auto-keyframe and snapping emit requests.
- Current time and duration fields commit with Enter. Drag the end handle to
  request a duration, and the top edge to request a panel height.
- Select a layer to select its keys. Expand/collapse layers, toggle visibility
  or lock them. The host echoes the resulting state.
- Click keys, Shift-click to extend selection, or drag a marquee. Double-click
  a key to seek. Drag selected keys together, preserving their spacing and
  clamping to the animation bounds. Escape cancels a drag.
- Arrow keys nudge by 1 ms; Shift-arrows use 10 ms. Delete removes the selected
  keys, Command/Ctrl-D duplicates, and Command/Ctrl-A selects keys.
- Drag layer bars and preset clips to move them. Drag their edges to retime.
  The application determines how a layer timing request transforms its data.
- Click an interpolation segment, a preset clip, or Easing to select Bézier/hold/spring
  presets. Custom Bézier coordinates and spring bounce values are validated
  before an easing request is emitted.
- Apply host-provided animation presets to selected layers. Comment controls
  emit timestamped requests; existing comment markers emit open requests.
- Read-only mode allows inspection, selection, zoom and playback, while
  mutation controls and drag commits are disabled. Locked tracks reject edits.

The mock host refuses duration changes that would truncate its last key or
clip. Production hosts should apply their own document rules. Timeline ids
are opaque strings and have no engine dependency. Legacy `looping` and
`LoopChangeRequested` remain adapter vocabulary; the UI uses `TimelinePlayback`.
The legacy Agent event has no built-in button.

## Reference

The control and track interactions follow the
[Figma Motion timeline guide](https://help.figma.com/hc/en-us/articles/41405906446999-Use-the-Figma-Motion-timeline).
Selection and retiming follow the
[keyframe guide](https://help.figma.com/hc/en-us/articles/41307938657559-Add-select-and-delete-keyframes).
Easing choices follow the
[easing guide](https://help.figma.com/hc/en-us/articles/41414048690839-Adjust-an-animation-s-easing).
This is the component and mock-host implementation; Fanta Edit integration
and production animation evaluation are separate host work.

Layer names support double-click or F2 to rename the selected layer. Enter or blur
commits `TrackRenameRequested { track_id, name }`; Escape cancels. Blank or
unchanged names emit no request, and locked/read-only layers cannot be renamed.
The host applies the new name through fresh view data.
