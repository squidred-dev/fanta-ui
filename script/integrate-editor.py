#!/usr/bin/env python3
"""Switch an editor checkout to this release, preserving unrelated TOML edits.

Requires tomlkit (python3 -m pip install tomlkit). Local mode is temporary;
registry mode is the final published configuration. Never removes source files.
"""
import argparse
import json
from pathlib import Path
import tomlkit

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('editor', type=Path)
parser.add_argument('--mode', choices=['local', 'git', 'registry'], required=True)
parser.add_argument('--revision', help='Full published extraction commit for git mode')
args = parser.parse_args()
if args.mode == 'git' and not args.revision:
    parser.error('--revision is required for git mode')
root = Path(__file__).resolve().parents[1]
rows = json.loads((root / 'docs/extraction/packages.json').read_text())
rows.append({'original_name': 'fanta-gpui', 'package': 'fanta-gpui', 'source_path': 'crates/fanta-gpui'})
manifest = args.editor.resolve() / 'Cargo.toml'
doc = tomlkit.parse(manifest.read_text())
workspace = doc['workspace']
dependencies = workspace['dependencies']
renames = {row['original_name']: row['package'] for row in rows}
removed = {row['source_path'] for row in rows if 'upstream' not in row}
for key, values in [('members', [member for member in workspace['members'] if member not in removed]),
                    ('exclude', sorted(set(workspace.get('exclude', [])) | removed))]:
    array = tomlkit.array().multiline(True)
    for value in values:
        array.append(value)
    workspace[key] = array
for row in rows:
    name = row['original_name']
    if name not in dependencies:
        continue
    old = dependencies[name]
    entry = tomlkit.inline_table()
    if hasattr(old, 'items'):
        for key, value in old.items():
            if key not in ['path', 'git', 'rev', 'branch', 'tag', 'version', 'package']:
                entry[key] = value
    entry['package'] = row['package']
    entry['version'] = '=0.1.0'
    if args.mode == 'local':
        entry['path'] = str(root / row['source_path'])
    elif args.mode == 'git':
        entry['git'] = 'https://github.com/squidred-dev/fanta-ui'
        entry['rev'] = args.revision
    dependencies[name] = entry
# These registry packages expose types through the imported libraries.
for name in ['reqwest', 'scap', 'font-kit', 'xim']:
    if name in dependencies and hasattr(dependencies[name], 'get'):
        for key in ['git', 'rev', 'branch', 'tag']:
            dependencies[name].pop(key, None)
patches = doc.get('patch', {}).get('crates-io', {})
for name in ['gpui', 'gpui-component', 'async-process']:
    patches.pop(name, None)
for profile in doc.get('profile', {}).values():
    table = profile.get('package', {})
    for name, renamed in renames.items():
        if name != renamed and name in table:
            table[renamed] = table.pop(name)
manifest.write_text(tomlkit.dumps(doc))
config = manifest.parent / '.cargo/config.toml'
if config.exists():
    text = config.read_text().replace('cargo run -p perf ', 'cargo run -p fanta-gpui-perf ')
    text = text.replace('\"-p\", \"perf\"', '\"-p\", \"fanta-gpui-perf\"')
    config.write_text(text)
workflow = manifest.parent / '.github/workflows/check.yml'
if workflow.exists():
    text = workflow.read_text().replace('-p media ', '-p fanta-gpui-media ')
    text = text.replace('-p gpui_macos ', '-p fanta-gpui-gpui-macos ')
    workflow.write_text(text)
print(f'Updated {manifest} ({args.mode}); old sources retained until release validation.')
