# Project asset browser

The File Inspector stacks Pages, Layers, and a collapsible Assets section.
`FileInspectorSidebar::set_assets_panel` mounts the Assets section. The child
`AssetsPanel` receives `AssetsViewData` and emits `AssetsPanelAction`; it does
not mutate a project. The host may call `expand_assets` after an import, and
the user can expand or collapse the section without hiding Layers.

The Assets disclosure header has its own Find control. Click it, or press
`⌘F`/`Ctrl+F` while Assets has focus, to open the same compact search pattern
used by Pages: a focused cleanable query, type settings in a popup, a result
count, and previous/next controls. Search matches each asset's semantic name,
type, and host-supplied detail, and never queries page or layer content.
Enter moves to the next matching asset; Shift+Enter moves to the previous one.
Escape dismisses the type menu first, then closes search. The active result is
highlighted and scrolled into view. Closing search returns to the full asset
list; the chosen type and query remain available for the next Find session.

The host lists its persisted project assets, including media that cannot be
placed in the current editor space. Each `AssetRow` has an opaque ID, semantic
name, type, optional thumbnail and detail, and `can_place`. Set
`disabled_reason` for unavailable rows. `AssetKind::Other` keeps fonts and
unknown project blobs visible while disabling canvas placement.

The host supplies all page targets and `selected_page_id`. Choosing a page
emits `TargetPageSelected`, which the host echoes through `set_view_data`.
Pressing Place emits the exact asset ID and page ID; the host validates both
again before ingesting or placing a node. Images, SVG, video, and audio can be
placed on a Design page. Video and audio render as static posters or waveforms
there; playback is available only in Motion or Prototype, where the host owns
their playback sessions. The panel itself never decodes video/audio, starts
playback, or changes a document.

The Storybook file-inspector story has representative image, vector, video,
audio, and non-placeable resource rows. It is a mock adapter with no project
or backend access.
