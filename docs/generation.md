# AI generation screens

`fanta_gpui::generation::GenerationScreen` presents four creation screens:
Images, Video, Audio, and SVG vectors. The host supplies `GenerationViewData`
and handles typed `GenerationAction` intents. The library renders models,
model-specific choices, prompt templates, job state, output previews, and a
gallery. Its searchable model selector shows host-supplied provider marks and
per-output credit prices. Selecting a creation opens a full-screen preview with
navigation, details, and a host-owned media player when one is supplied. It does not authenticate, upload files, run models, poll jobs, save
results, or mutate a Fanta document. Storybook supplies an in-memory mock host.

The visual system is the active Zed GPUI theme, including its darkest editor
canvas token, `gpui-component` primitives,
semantic tokens, and shared Fanta controls. External screen references can
inform layout and feature grouping but do not determine colors or component
styling.

## Controlled boundary

`GenerationKind` covers only Image, Video, Audio, and SVG. Each
`GenerationModel` has a `GenerationRecipe`: text-to-image, text-to-video,
image-to-video, speech, music, prompt-to-SVG, image-to-SVG, or vectorization.
The host controls `selected_recipe`; the operation selector emits
`RecipeSelected` and then the model menu shows only models for that operation.
The host builds models from its authenticated catalog and supplies
`GenerationOptionGroup` entries with `GenerationChoice` values only for
controls the chosen backend model supports. A choice key/value is opaque to
the component; the host translates it to a validated request field. Secondary
controls can be marked `advanced`. The host also supplies a provider label,
decoded `GenerationModelLogo`, and live `credits_per_output`. Models billed by
usage can supply a variable-price hint instead. The bundled Storybook values
illustrate current seed-catalog prices; production must read the authenticated
catalog so prices reflect organization overrides and later changes.
The screen opens model-specific advanced settings when a model is selected.
The model picker calls out how many advanced controls are available, and the
composer keeps its first controls in view at a normal desktop window height.
Choice groups with more than three values use a searchable dropdown instead
of a long row of chips; the host still owns the selected value and receives
the same `OptionSelected` intent.

`GenerationTemplate` is a host-provided prompt preset. Its optional `model_id`
lets the host switch to the intended recipe when it handles `TemplateSelected`.
The host can supply a decoded preview image. If it does not, the component
uses bundled sample artwork for image, video, audio, and SVG templates; the
sample artwork never substitutes for a generated output preview.
Selecting it fills a draft; generating requires a separate `GenerateRequested` intent carrying a
`GenerationSubmission`. `GenerationSource` is an opaque source ID and optional
host-decoded preview. The source is required only for image-to-video,
image-to-SVG, and vectorization recipes. The component emits source selection
and clearing intents; the host owns file selection and upload.
Image-to-video presents start and end frame slots. The end slot is enabled only
when the selected model declares `supports_end_frame` and the host has a
matching backend input mapping. An enabled end slot submits `end_frame_id`;
models without that capability never send it. Gateway Kling 3 and selected
Veo/MiniMax variants, plus some Replicate models, can declare two-frame
generation. The source picker and backend still enforce format and size limits.

For speech models that support voice references, the host supplies an optional
`GenerationVoiceReference` and an explicit consent flag. The screen requests
selection/removal and consent changes as typed intents. A submission with a
voice reference requires consent; the host still verifies permission and
encodes the audio sample before sending it to the backend.

The host provides `GenerationOutput` cards and a selected output through
`GenerationViewData`. Each card has status, kind, model, optional prompt text,
metadata, and an optional decoded still preview. Raster and SVG thumbnails
are decoded by the host. For audio and video, the host can mount an owned
`playback_view` directly in the lightbox. The Storybook host supplies native
AVFoundation MP4 playback with a seek bar and WAV transport for its local
fixtures; production resolves fresh authenticated media and owns playback
life cycle. `PlayRequested` and `PreviewClosed` let the host start and stop the
session; `DownloadRequested` remains a host intent. A missing preview remains a valid state. Gallery
selection, refresh, and pagination are also intents; the host echoes the
authoritative gallery and `has_more` flag. The backend's history response does
not include original prompt text, so the host must omit that field or obtain
it from its own saved request history.

The component may retain only presentation state such as focus, draft text,
scroll position, and open menus. It does not consider an emitted intent to be
an accepted job. The host keeps tokens, idempotency keys, job IDs, prices, and
source references; it applies accepted actions and supplies fresh view data.
The four screens contain no Design or Masks mode, masking, inpaint, outpaint,
video editing, or post-production actions.

## Backend adapter

These routes and shapes were checked against `fanta-backend` and the existing
`fanta-edit` generation workspace. They belong in the production host adapter,
not in this component crate.

| Host task | API | Contract |
| --- | --- | --- |
| Discover models | `GET /v1/models` | Authenticated `{ models: [{ id, kind, credits_per_output, capabilities?, ... }] }`. Use the enabled, priced rows for the active organization. Branded aliases resolve to an engine at runtime. |
| Submit | `POST /v1/generations` | Body `{ model, prompt?, width?, height?, seed?, negative?, input? }`, with a stable `Idempotency-Key` for retries. Optional `kind` must match the model's catalog kind. |
| Poll | `GET /v1/generations/{id}` | A 202 response may report `queued`, `warming`, or `processing`. Terminal responses report `succeeded` or `failed` with `output[]`, model, seed, billing, and possibly an error. Persist the accepted ID and original request for recovery. |
| Upload source | `POST /v1/assets/uploads`, signed `PUT`, `POST /v1/assets/uploads/{id}/complete` | Register MIME, byte count, SHA-256, and optional file name; upload bytes directly to the signed URL; complete before using `input.source = { asset_id }`. Uploads need object storage. |
| Gallery | `GET /v1/generations?kind=&member=&limit=&before=` | Paginated history returns `generations[]` and `next_before`, including statuses and output media with fresh signed URLs or inline SVG. Use the authenticated member ID for `member` if the gallery is labeled “Your creations”; omitting it returns organization-wide history. This is the primary gallery feed. |
| Asset library | `GET /v1/assets?kind=output&q=&tag=&before=&limit=` | Optional saved-media view. Some inline SVG and GPU-uploaded video outputs do not create asset rows, so this cannot replace generation history. |

The backend also resolves `input.source = { generation_id }`, `{ url }`, or
inline `{ data }` for compatible kinds. Source uploads accept PNG, JPEG, WebP,
SVG, WAV, MP3, and MP4 among their allowed formats, with a 100 MB cap. When
object storage is unavailable the upload route returns 503; the host should
show source selection as unavailable or supply another supported source path.

Generation prompts are limited to 1,200 JavaScript string units; the component
counts UTF-16 units so the visible counter matches that limit. JSON seeds are
limited to JavaScript's maximum safe integer. The backend clamps video
to 81 frames by default and can return 503 when video generation is disabled.
Model pricing, availability, organization overrides, and fallback routing may
change independently of the UI. Refresh the live catalog and expose service
errors through `GenerationViewData`; do not treat the seed catalog as a promise
of deployed capacity.

## Verified creation models and controls

Controls require an explicit catalog capability, not an inference from the
broad modality. The Storybook has a wide mock catalog to exercise long model
lists, rich settings, creator marks, and Gateway-first ordering. It includes
representative Gateway and Replicate entries, but the authenticated live model
list remains authoritative. Disabled and unpriced backend rows do not appear
in the production selector.

| Screen | Storybook model families | Examples of host-declared controls |
| --- | --- | --- |
| Image | Fanta image aliases, Recraft V4.1, Grok Imagine, Seedream, FLUX, and Replicate fallbacks. | Canvas size and aspect ratio; negative prompt, seed, steps, guidance, and enhancement only when advertised by the selected model. |
| Video | Fanta video aliases, Wan 3, Kling 3, Veo 3.1, MiniMax H3, Grok Imagine Video, plus Replicate LTX, Wan, and Seedance. | Duration, resolution, aspect ratio, sound, quality mode, frame rate, camera motion, and prompt expansion according to the model. Image-to-video always needs a start frame; only models with `supports_end_frame` expose an end frame. |
| Audio | Fanta speech/music aliases, Grok TTS, OpenAI TTS, MiniMax Speech, and Eleven Music. | Voice, output format, pace, emotion, music length, and vocals according to the model. Reference voice and consent appear only where the host declares support. |
| SVG vectors | Fanta SVG/vectorization, Claude prompt-to-SVG, QuiverAI Arrow, and Recraft SVG. | Prompt-to-SVG, image-to-SVG, and vectorization are distinct recipes. Recraft can declare aspect ratio; image-source recipes require a ready source. |

Edit, segment, track, upscale, video-tool, stems, transcription, audio-cleanup,
and Comfy workflow models are outside these screens. The backend's Comfy
`template_id` flow is unrelated to the prompt presets shown here. The host
maps every choice to a validated request field and re-quotes models whose
price varies with duration, resolution, sound, or quality mode.

Fanta Edit already implements Claude prompt-to-SVG through `/v1/messages` with
a constrained SVG system instruction, a 1,200-character prompt cap, an
8,192-token ceiling, and SVG validation before it accepts editable artwork.
A production host can adapt this route for `GenerationRecipe::PromptSvg`, then
persist its result in host history if it should appear in the gallery. The
component only displays the supplied state and emits a submission intent.

## Current integration status

The four screens and the Storybook mock host are implemented in `fanta-gpui`.
Fanta Edit pins a Git revision of this library and owns model discovery,
submission, polling, media playback/download, and the paginated gallery.
Storybook's previews are playable local fixtures, not generated results.
Publishing a new catalog also requires backend rows to be enabled and priced;
a library update alone does not expose them in the native app.

## Integration checks

- Populate models from the live catalog, including missing or disabled rows,
  and verify that controls change with the selected model.
- Exercise submission acceptance, idempotent recovery, polling, failure, and
  video-disabled responses in the host adapter.
- Supply `url`, `data_url`, and inline `svg` outputs; decode appropriate still
  previews and resolve playable/downloadable media through the host.
- Require a ready image source for image-to-video and both image-to-SVG paths;
  use the separate chat route for prompt-to-SVG.
- Keep Storybook network-free. Its mock reducer handles intents and provides
  catalog, progress, template, output, and gallery fixtures.

Source locations at audit time: `fanta-backend/scripts/seed-catalog.ts`,
`app/v1/models/route.ts`, `app/v1/generations/route.ts`,
`app/v1/generations/[id]/route.ts`, `app/v1/assets/route.ts`,
`gpu/common/fanta_gpu_common/schemas.py`, and the modality workers. Fanta
Edit's existing adapter is in
`fanta-edit/crates/fig_viewer/src/generation_workspace.rs`.
