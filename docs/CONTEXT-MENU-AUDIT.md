# Layers and pages context-menu audit

Compared on 2026-09-22 in [Figma's native editor](https://www.figma.com/design/Wfiw03026lgtHyCLiChG4z).
The file contains native test specimens, not a reproduction of the menu UI.

## Node inventory

| Fanta panel kind | Figma representation | Fixture | Live menu checked |
| --- | --- | --- | --- |
| Frame | FRAME | 2:4 | Yes |
| Group | GROUP | 2:7 | Yes |
| Section | SECTION | 2:8 | Yes |
| Component | COMPONENT | 2:9 | Yes |
| Component set | COMPONENT_SET | 2:13 | Yes |
| Instance | INSTANCE | 2:10 | Yes |
| Text | TEXT | 2:14 | Yes |
| Image | RECTANGLE with IMAGE fill | 4:2 | Yes |
| Video | Shape with VIDEO fill | — | No: connected Plugin API cannot create video media |
| Rectangle | RECTANGLE | 2:15 | Yes |
| Ellipse | ELLIPSE | 2:16 | Yes |
| Polygon | POLYGON | 2:17 | Yes |
| Star | STAR | 2:18 | Yes |
| Line | LINE | 2:19 | Yes |
| Arrow | LINE with arrow caps | 2:20 | Yes |
| Vector | VECTOR | 2:21 | Yes |
| Boolean operation | BOOLEAN_OPERATION | 2:26 | Yes |
| Slice | SLICE | 2:27 | Yes |
| Mask | ELLIPSE with isMask | 2:28 | Yes |
| Pen | VECTOR (creation tool, not a distinct node type) | 2:22 | Yes |
| Pencil | VECTOR (creation tool, not a distinct node type) | 2:23 | Yes |
| Other | Host extension/fallback | — | No equivalent |

The engine's complete `NodeData` inventory is Group, Vector, Text, Bitmap,
Video, Audio, NodeGraph, Model3d, AiArtifact, Instance, Boolean, and Embed.
Group supplies frame/component presentations; Vector supplies parametric
shapes. Audio, NodeGraph, Model3d, AiArtifact and Embed currently map to Other.
Figma CANVAS maps to a page rather than a layer. DOCUMENT is the file root.

## Observations and product decisions

The shape and text menus share copy/paste, arrange, group/frame, rename,
flatten, outline stroke, mask, auto layout, component, visibility, lock and
flip actions. Image fills use the same shape menu: crop/replace are not
separate top-level entries in the observed context menu. Pen and Pencil
match Vector. Line/Arrow can be wrapped in auto layout. Mask changes the
mask command to Remove mask.

Frame offers Ungroup, conversion to section and thumbnail; Group offers
Ungroup, flatten and outline. Component/Component set expose a Main component
submenu. Instance adds reset, detach and navigation to its main component.
Section omits mask/component/flip commands. Slice omits flatten/outline/mask.
Boolean operations offer Ungroup in addition to the vector commands.

Figma services (Send to Figma Make, plugin/widget launchers, design discovery)
are not Fanta layer operations and are absent from the Fanta menu. Single-node
menus do not show Rename layers. Domain-specific entries remain typed intents,
but a host can supply an allowlist per node; empty sections are removed after
filtering. Hosts must revalidate an intent against the current document.

Fanta additionally exposes Duplicate and Delete directly. fanta-edit advertises
only the operations it handles: copy, duplicate, delete, rename, visibility,
lock, stacking order, grouping/framing/ungrouping, component creation and
instance detachment/navigation, as applicable. Locked nodes retain copy and
unlock; read-only documents retain nonmutating actions. Unsupported entries
are not advertised as successful commands.

## Pages

The observed one-page menu disables Delete. A selected middle page has Copy
link, Rename, Duplicate, Move up, Move down and Delete with separated groups.
Figma's ordering shortcuts depend on the page's position and number of pages.
The requested Fanta layout also keeps explicit Move to top/bottom actions.
All four movement commands emit a direction plus stable page ID; boundary
commands and deletion of the last page cannot emit a mutation intent.

Copy link is host-controlled because only the host knows whether a durable
route exists. fanta-edit links to an existing saved page source, resolving its
slug folder rather than assuming the node ID is the folder name. Unsaved page
sources do not offer a usable copy-link action.

## Verification scope

The native menus above were opened in Chrome using right-clicks on the Layers
rows. Geometry-only specimen states do not prove behavior for every combination
of selection, lock state, nested instance, stroke or auto-layout configuration.
Video remains explicitly unverified; it is not represented by a mislabeled
rectangle. The full inventory and capability policy cover the fallback types.
