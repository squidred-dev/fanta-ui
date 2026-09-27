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

`GenerationTemplate` is a host-provided prompt preset. Its optional `model_id`
lets the host switch to the intended recipe when it handles `TemplateSelected`.
Selecting it fills a draft; generating requires a separate `GenerateRequested` intent carrying a
`GenerationSubmission`. `GenerationSource` is an opaque source ID and optional
host-decoded preview. The source is required only for image-to-video,
image-to-SVG, and vectorization recipes. The component emits source selection
and clearing intents; the host owns file selection and upload.
Image-to-video presents start and end frame slots. Current Wan I2V only accepts
one start image, so the end slot remains unavailable for that model. A future
model can enable the end slot with `supports_end_frame` once its backend worker
accepts and validates an explicit end image; the submission then includes
`end_frame_id`. The current backend must not be sent that field.

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

Only Qwen Image currently has a full generation-control schema in the backend
catalog. Other controls require an explicit host recipe, not an inference from
the broad modality. The live model list remains authoritative.

| Screen | Seeded aliases and engines | Supported creation controls |
| --- | --- | --- |
| Image | `fanta-image-1` → Qwen Image 2.0; `fanta-image-fast-1` → Z-Image Turbo; `flux-schnell` is a hosted fallback. | Qwen catalog sizes: `1024x1024`, `1280x768`, `768x1280`, `2048x2048`; steps 10–50, default 30; guidance 1–10, default 4; negative prompt; optional seed and prompt enhancement. Turbo has no published capability schema and has an eight-step worker default; do not apply Qwen's step range to Turbo. |
| Video | `fanta-video-1` → LTX 2.3; `fanta-video-hd-1` → Wan 2.2 text-to-video; `fanta-animate-1` → Wan 2.2 image-to-video. | Send selected width/height, `input.frames` up to 81, and `input.fps` for MP4 playback rate. The Storybook LTX and Wan presets use different frame sizes and 17/33/49/65/81 frame lengths; changing frame rate updates the displayed duration. Only Animate takes one start image. No current video worker accepts an end image or video audio toggle; video workers do not consume seed. |
| Audio | `fanta-voice-1` → Chatterbox; `fanta-voice-turbo-1` → Chatterbox Turbo; `fanta-voice-fast-1` → Kokoro; `fanta-music-1` → ACE-Step 1.5. | Top-level `prompt` becomes GPU `text`. Kokoro uses `input.speed`; Chatterbox workers ignore it. Chatterbox voice reference requires `input.voice_ref_b64` and explicit `input.consent: true`. Workers return WAV regardless of the schema's `format`. ACE-Step's validated TTS schema omits `duration_s`, so a music duration control is not reliable yet. |
| SVG vectors | `fanta-svg-1` → StarVector 8B; `fanta-vectorize-1` → FLUX Klein plus vtracer; Claude chat models for prompt-to-SVG. | StarVector requires a source image and ignores prompt guidance. Vectorize requires an image and accepts `input.mode` (`line_art`, `flat_color`, `trace`) and `input.detail` (`low`, `medium`, `high`); the host enables its optional prompt only in line-art mode. Claude uses a separate `/v1/messages` adapter and its SVG is not included in generation history. |

The Qwen catalog advertises reference images, but the current worker's singular
`reference_image` path is part of image editing and is omitted here. Edit,
segment, track, upscale, video-tool, stems, transcription, audio-cleanup, and
Comfy workflow models are also outside these screens. The backend's Comfy
`template_id` flow is unrelated to the prompt presets shown here.

Fanta Edit already implements Claude prompt-to-SVG through `/v1/messages` with
a constrained SVG system instruction, a 1,200-character prompt cap, an
8,192-token ceiling, and SVG validation before it accepts editable artwork.
A production host can adapt this route for `GenerationRecipe::PromptSvg`, then
persist its result in host history if it should appear in the gallery. The
component only displays the supplied state and emits a submission intent.

## Current integration status

The four screens and the Storybook mock host are implemented in `fanta-gpui`.
The existing `fanta-edit` workspace still uses its own Image, Video, Vector,
Design, and Masks UI and pins an earlier `fanta-gpui` Git revision. Shipping
these screens inside Fanta Edit requires a coordinated library revision and a
production host adapter for model discovery, submission, polling, media
playback/download, and the paginated gallery. Storybook's previews are playable
local fixtures, not generated results.

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
