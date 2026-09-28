# Remaining UI extraction assessment

Snapshot: 2026-09-21. Read-only architectural assessment of the current editor
workspace; no additional crates have been moved by this assessment.

## Counts

| Scope | Crates |
| --- | ---: |
| Editor workspace before extraction | 179 |
| Existing editor workspace members extracted | 28 |
| Editor workspace after the current extraction | 151 |
| Independent UI workspace, publishable packages | 33 |
| Independent UI workspace, unpublished development hosts | 3 |
| Primarily UI crates still in the editor | 40 |
| Additional mixed or UI-adjacent crates in this assessment | 10 |
| Remaining editor crates directly depending on GPUI in normal/build dependencies | 100 |

These are workspace package counts, not the number of Cargo dependencies downloaded
from crates.io. The 33 published packages include 28 former editor members plus
five additional fork packages brought in as dependency closure. The physical old
source directories remain until registry integration passes; deleting those
already-excluded copies will not reduce the active member count again.

The 40/10 categorization is an architectural assessment, not a Cargo property.
The first three groups below make up the 40 primarily UI crates. Incidental
prompts inside services (for example auto_update) do not make those entire
services UI libraries. A GPUI dependency alone is not evidence of presentation:
GPUI also supplies application state, entities, tasks, and events.

## Dependency evidence

Normal/build dependencies across all declared platforms are included; development
dependencies and external packages are excluded from these editor-local closure
counts. Each count includes the starting crate itself.

| Starting crate | Remaining editor crates in its transitive dependency closure |
| --- | ---: |
| ui | 7 |
| ui_input | 8 |
| file_icons | 3 |
| assets | 1 |
| theme_settings | 23 |
| markdown | 39 |
| picker | 71 |
| git_ui | 89 |
| fanta_ui | 73 |
| fig_viewer | 122 |

The nine foundational crates form a closed normal/build dependency group relative
to the remaining editor workspace. Their other dependencies are registry packages
or the already-extracted GPUI/support packages. Publication still needs its own
metadata, resource, macro, dev-dependency-cycle, licensing, and consumer checks.

## 9 foundational UI crates

Best next extraction; maintain aliases and existing behavior.

| Crate | Direct dependencies within the remaining editor workspace |
| --- | --- |
| `component` | `theme` |
| `file_icons` | `theme` |
| `icons` | None |
| `menu` | None |
| `syntax_theme` | None |
| `theme` | `syntax_theme` |
| `ui` | `component`, `icons`, `menu`, `theme`, `ui_macros` |
| `ui_input` | `component`, `ui` |
| `ui_macros` | None |

## 22 presentation feature crates

Extract presentation behind host data and typed intents; these are not dependency-free packages.

| Crate | Direct dependencies within the remaining editor workspace |
| --- | --- |
| `breadcrumbs` | `ui`, `workspace` |
| `command_palette` | `client`, `command_palette_hooks`, `db`, `fuzzy_nucleo`, `menu`, `picker`, `settings`, `telemetry`, `theme`, `ui`, `workspace`, `zed_actions` |
| `go_to_line` | `editor`, `language`, `menu`, `multi_buffer`, `settings`, `text`, `theme`, `ui`, `workspace` |
| `image_viewer` | `db`, `editor`, `file_icons`, `language`, `project`, `settings`, `theme_settings`, `ui`, `workspace` |
| `markdown` | `language`, `mermaid_render`, `settings`, `theme`, `theme_settings`, `ui` |
| `markdown_preview` | `db`, `editor`, `language`, `markdown`, `project`, `settings`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `notifications` | `channel`, `client`, `component`, `rpc`, `ui`, `workspace`, `zed_actions` |
| `outline` | `editor`, `fuzzy_nucleo`, `language`, `picker`, `settings`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `outline_panel` | `db`, `editor`, `file_icons`, `fuzzy`, `language`, `menu`, `outline`, `project`, `search`, `settings`, `theme`, `theme_settings`, `ui`, `workspace`, `worktree`, `zed_actions` |
| `panel` | `ui`, `workspace` |
| `picker` | `db`, `language`, `menu`, `project`, `settings`, `theme`, `theme_settings`, `ui`, `ui_input`, `workspace`, `zed_actions` |
| `picker_preview` | `editor`, `language`, `multi_buffer`, `picker`, `project`, `rope`, `settings`, `ui` |
| `platform_title_bar` | `project`, `settings`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `project_panel` | `client`, `command_palette_hooks`, `editor`, `feature_flags`, `file_icons`, `fs`, `git`, `git_ui`, `language`, `markdown_preview`, `menu`, `notifications`, `project`, `search`, `settings`, `telemetry`, `theme`, `theme_settings`, `ui`, `workspace`, `worktree`, `zed_actions` |
| `recent_projects` | `askpass`, `editor`, `fs`, `fuzzy_nucleo`, `menu`, `open_path_prompt`, `paths`, `picker`, `project`, `remote`, `remote_connection`, `settings`, `task`, `telemetry`, `ui`, `ui_input`, `workspace`, `zed_actions` |
| `remote_connection` | `askpass`, `auto_update`, `markdown`, `menu`, `release_channel`, `remote`, `settings`, `theme_settings`, `ui`, `ui_input`, `workspace` |
| `search` | `db`, `editor`, `file_icons`, `fs`, `language`, `menu`, `multi_buffer`, `picker`, `picker_preview`, `project`, `settings`, `text`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `sidebar` | `acp_thread`, `action_log`, `agent`, `agent_settings`, `agent_ui`, `editor`, `feature_flags`, `fs`, `git`, `git_ui`, `language_model`, `menu`, `notifications`, `platform_title_bar`, `project`, `recent_projects`, `remote`, `remote_connection`, `settings`, `telemetry`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `terminal_view` | `breadcrumbs`, `db`, `editor`, `language`, `menu`, `project`, `settings`, `task`, `terminal`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `theme_selector` | `fs`, `fuzzy`, `picker`, `settings`, `telemetry`, `theme`, `theme_settings`, `ui`, `workspace`, `zed_actions` |
| `title_bar` | `agent_settings`, `auto_update`, `client`, `cloud_api_types`, `command_palette_hooks`, `db`, `fs`, `git_ui`, `notifications`, `platform_title_bar`, `project`, `recent_projects`, `remote`, `settings`, `telemetry`, `theme`, `ui`, `workspace`, `zed_actions` |
| `ui_prompt` | `markdown`, `menu`, `settings`, `theme`, `theme_settings`, `ui`, `workspace` |

## 9 application surfaces

Split presentation from host orchestration; do not relocate these crates wholesale.

| Crate | Direct dependencies within the remaining editor workspace |
| --- | --- |
| `agent_ui` | `acp_thread`, `action_log`, `agent`, `agent_servers`, `agent_settings`, `agent_skills`, `ai_onboarding`, `audio`, `buffer_diff`, `client`, `cloud_api_types`, `command_palette_hooks`, `component`, `context_server`, `db`, `editor`, `eval_utils`, `extension`, `feature_flags`, `file_icons`, `fs`, `fuzzy`, `git`, `git_ui`, `html_to_markdown`, `language`, `language_model`, `language_models`, `lsp`, `markdown`, `menu`, `multi_buffer`, `notifications`, `paths`, `picker`, `platform_title_bar`, `project`, `prompt_store`, `proto`, `release_channel`, `remote`, `remote_connection`, `rope`, `sandbox`, `search`, `settings`, `streaming_diff`, `task`, `telemetry`, `terminal`, `terminal_view`, `text`, `theme`, `theme_settings`, `ui`, `ui_input`, `watch`, `workspace`, `zed_actions` |
| `ai_onboarding` | `client`, `cloud_api_types`, `component`, `language_model`, `telemetry`, `ui`, `zed_actions` |
| `editor` | `assets`, `breadcrumbs`, `buffer_diff`, `client`, `clock`, `dap`, `db`, `edit_prediction_types`, `feature_flags`, `file_icons`, `fs`, `fuzzy`, `git`, `language`, `lsp`, `markdown`, `menu`, `multi_buffer`, `project`, `rope`, `rpc`, `settings`, `snippet`, `task`, `telemetry`, `text`, `theme`, `theme_settings`, `ui`, `ui_input`, `vim_mode_setting`, `workspace`, `zed_actions` |
| `fanta_ui` | `assets`, `picker`, `settings`, `theme`, `theme_settings`, `ui` |
| `fig_viewer` | `agent_ui`, `client`, `context_server`, `db`, `design_surface`, `editor`, `fanta-canvas`, `fanta-doc`, `fanta-fig-interop`, `fanta-format`, `fanta-present`, `fanta-render`, `fanta-text`, `fanta-tools`, `fanta_ui`, `file_icons`, `fs`, `language`, `paths`, `project`, `settings`, `theme`, `ui`, `ui_input`, `workspace`, `worktree`, `zed_actions` |
| `git_ui` | `agent_settings`, `askpass`, `buffer_diff`, `client`, `component`, `db`, `editor`, `file_icons`, `fs`, `fuzzy`, `fuzzy_nucleo`, `git`, `language`, `language_model`, `markdown`, `menu`, `multi_buffer`, `notifications`, `panel`, `picker`, `project`, `prompt_store`, `proto`, `release_channel`, `remote`, `remote_connection`, `search`, `settings`, `task`, `telemetry`, `terminal`, `theme`, `theme_settings`, `time_format`, `ui`, `ui_input`, `watch`, `workspace`, `zed_actions` |
| `vim` | `command_palette`, `command_palette_hooks`, `db`, `editor`, `fuzzy`, `language`, `menu`, `multi_buffer`, `picker`, `project`, `search`, `settings`, `task`, `text`, `theme`, `theme_settings`, `ui`, `vim_mode_setting`, `workspace`, `zed_actions` |
| `workspace` | `agent_settings`, `client`, `clock`, `component`, `db`, `fs`, `language`, `markdown`, `menu`, `node_runtime`, `project`, `remote`, `session`, `settings`, `sqlez`, `task`, `telemetry`, `theme`, `theme_settings`, `ui`, `ui_input`, `zed_actions` |
| `zed` | `acp_tools`, `agent`, `agent_settings`, `agent_ui`, `askpass`, `assets`, `audio`, `auto_update`, `breadcrumbs`, `channel`, `cli`, `client`, `command_palette`, `command_palette_hooks`, `component`, `crashes`, `db`, `editor`, `etw_tracing`, `fanta_languages`, `feature_flags`, `fig_viewer`, `fs`, `git`, `git_hosting_providers`, `git_ui`, `image_viewer`, `input_latency_ui`, `install_cli`, `language`, `language_model`, `language_models`, `markdown`, `menu`, `migrator`, `node_runtime`, `notifications`, `paths`, `project`, `prompt_store`, `proto`, `recent_projects`, `release_channel`, `remote`, `rope`, `sandbox`, `search`, `session`, `settings`, `sidebar`, `snippet_provider`, `system_specs`, `task`, `telemetry`, `telemetry_events`, `terminal_view`, `theme`, `theme_selector`, `theme_settings`, `time_format`, `title_bar`, `ui`, `ui_prompt`, `vim_mode_setting`, `watch`, `web_search`, `web_search_providers`, `windows_resources`, `workspace`, `zed_actions`, `zed_env_vars`, `zlog_settings` |

## 7 resources, settings, and host bridges

Review individually; several should remain host-owned.

| Crate | Direct dependencies within the remaining editor workspace |
| --- | --- |
| `assets` | None |
| `command_palette_hooks` | `workspace` |
| `design_surface` | None |
| `input_latency_ui` | `telemetry` |
| `install_cli` | `client`, `release_channel`, `workspace` |
| `open_path_prompt` | `file_icons`, `fuzzy`, `picker`, `project`, `settings`, `ui`, `workspace` |
| `theme_settings` | `settings`, `theme` |

## 3 services with embedded presentation

Extract individual views if useful; retain service implementations in the editor.

| Crate | Direct dependencies within the remaining editor workspace |
| --- | --- |
| `acp_tools` | `agent_servers`, `agent_ui`, `language`, `markdown`, `project`, `settings`, `theme_settings`, `ui`, `workspace` |
| `agent` | `acp_thread`, `action_log`, `agent_servers`, `agent_settings`, `agent_skills`, `client`, `cloud_api_types`, `cloud_llm_client`, `context_server`, `db`, `design_surface`, `feature_flags`, `fs`, `git`, `html_to_markdown`, `http_proxy`, `language`, `language_model`, `language_models`, `paths`, `project`, `prompt_store`, `release_channel`, `sandbox`, `settings`, `shell_command_parser`, `sqlez`, `streaming_diff`, `task`, `telemetry`, `text`, `ui`, `watch`, `web_search`, `zed_env_vars` |
| `language_models` | `anthropic`, `client`, `credentials_provider`, `fs`, `language_model`, `menu`, `open_ai`, `settings`, `ui`, `ui_input` |

## Assessment and suggested sequence

1. **Move the nine foundations together:** component, file_icons, icons, menu,
   syntax_theme, theme, ui, ui_input, and ui_macros. This would leave 142 editor
   workspace crates, assuming no compensating new host packages. The UI repository
   would have 42 publishable packages if each retained a separate identity.
2. **Separate reusable resources and provide the theme host interface.** The theme
   crate already defines ThemeSettingsProvider and accepts a host implementation;
   theme_settings owns the settings integration and can stay in the editor.
   Storybook can supply an independent mock provider. The syntax_theme test loads
   a theme fixture outside its crate, which must become package-local. assets
   embeds the editor's root assets directory, including sounds and prompts as
   well as fonts/icons/themes, so do not move that package blindly.
3. **Extract Fanta presentation and selected generic feature views.** Retain the
   agreed fig_viewer::gpui_adapters boundary and the document/canvas engines.
   For picker, notifications, title bars, project lists, markdown previews, and
   similar views, replace direct host calls with view data, providers, and intents
   before moving the presentation. Prioritize views actually used in Fanta.
4. **Treat Git, agent, editor, terminal, and workspace screens as separate projects.**
   git_ui can become portable presentation with Git state supplied by a host and
   Git actions emitted back to it. Moving it unchanged would drag in project,
   editor, workspace, search, remote, database, and service code. The same issue
   applies to agent_ui and much of fig_viewer. design_surface is an agent/editor
   contract, not a reusable widget; retain its host ownership under the current
   architecture. zed is the executable and remains an application host.

There are currently two component/theme stacks: the Zed-derived ui/theme stack
and the gpui-component stack used by fanta-gpui. Both use GPUI, but co-location
alone does not unify their theme APIs or component behavior. An initial extraction
can preserve ui/theme as a compatibility layer. Later work should explicitly
choose a shared token/theme adapter and migrate call sites gradually rather than
introducing a third design system.

A final count after extracting all presentation cannot be predicted by subtracting
40 or 50 from 151. Many mixed crates must remain as host adapters or services,
while their views move or consolidate into existing component packages. The useful
target is that reusable presentation depends on host contracts and data, with no
reverse dependency on fanta-edit or its document engines.
