#!/usr/bin/env python3
"""Build real .crate archives and test a consumer through a temporary sparse registry.

No packages are uploaded. External registry entries are proxied unchanged;
only this workspace's packages are supplied from freshly built archives.
"""
import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
from urllib.error import HTTPError
from urllib.request import Request, urlopen
import release


def index_entry(package, archive):
    dependencies = []
    for dep in package['dependencies']:
        dependencies.append({
            'name': dep['rename'] or dep['name'], 'req': dep['req'],
            'features': dep['features'], 'optional': dep['optional'],
            'default_features': dep['uses_default_features'], 'target': dep['target'],
            'kind': dep['kind'] or 'normal', 'registry': None,
            'package': dep['name'] if dep['rename'] else None,
        })
    return {'name': package['name'], 'vers': package['version'], 'deps': dependencies,
            'cksum': hashlib.sha256(archive.read_bytes()).hexdigest(),
            'features': {}, 'features2': package['features'], 'v': 2, 'yanked': False,
            'links': package['links'], 'rust_version': package['rust_version']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', action='store_true', help='Also launch the packaged macOS consumer')
    args = parser.parse_args()
    packages = release.packages()
    ordered = release.audit(packages)
    destination = release.ROOT / 'target/release-verification'
    destination.mkdir(parents=True, exist_ok=True)
    entries = {}
    archive_paths = {}
    external_cache = {}

    class Registry(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            path = self.path.split('?')[0].lstrip('/')
            if path == 'config.json':
                self.respond(json.dumps({'dl': base + '/download/{crate}/{version}'}).encode())
                return
            if path.startswith('download/'):
                _, name, version = path.split('/')
                archive = archive_paths.get((name, version))
                if archive:
                    self.respond(archive.read_bytes())
                else:
                    self.proxy(f'https://static.crates.io/crates/{name}/{name}-{version}.crate')
                return
            name = path.rsplit('/', 1)[-1]
            if name in packages:
                if name in entries:
                    self.respond(json.dumps(entries[name]).encode() + b'\n')
                else:
                    self.send_error(404)
                return
            self.proxy('https://index.crates.io/' + path)

        def respond(self, data):
            self.send_response(200)
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def proxy(self, url):
            try:
                if url not in external_cache and url.startswith('https://static.crates.io/crates/'):
                    filename = url.rsplit('/', 1)[-1]
                    cached = next((Path.home() / '.cargo/registry/cache').glob('index.crates.io-*/' + filename), None)
                    if cached:
                        external_cache[url] = cached.read_bytes()
                if url not in external_cache:
                    request = Request(url, headers={'User-Agent': release.USER_AGENT})
                    with urlopen(request, timeout=60) as response:
                        external_cache[url] = response.read()
                self.respond(external_cache[url])
            except HTTPError as error:
                self.send_error(error.code)
            except Exception as error:
                print(f'Registry proxy error for {url}: {error}', flush=True)
                self.send_error(502)

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Registry)
    base = f'http://127.0.0.1:{server.server_port}'
    config = ['--config', 'source.crates-io.replace-with="release-verification"',
              '--config', f'source.release-verification.registry="sparse+{base}/"']
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        for package in ordered:
            print(f"Packaging {package['name']}", flush=True)
            subprocess.run(['cargo', 'package', '-p', package['name'], '--allow-dirty',
                            '--no-verify', *config], cwd=release.ROOT, check=True)
            archive = release.ROOT / 'target/package' / f"{package['name']}-{package['version']}.crate"
            entries[package['name']] = index_entry(package, archive)
            archive_paths[(package['name'], package['version'])] = archive
        (destination / 'archives.json').write_text(json.dumps(entries, indent=2) + '\n')
        with tempfile.TemporaryDirectory(prefix='fanta-registry-consumer-') as temp:
            consumer = Path(temp)
            shutil.copytree(release.ROOT / 'examples/registry-smoke', consumer, dirs_exist_ok=True,
                            ignore=shutil.ignore_patterns('target', 'Cargo.lock'))
            env = {**os.environ, 'CARGO_TARGET_DIR': str(destination / 'target')}
            for command in ['check', 'test']:
                subprocess.run(['cargo', command, *config], cwd=consumer, env=env, check=True)
            metadata = json.loads(subprocess.check_output(
                ['cargo', 'metadata', '--format-version', '1', '--filter-platform',
                 subprocess.check_output(['rustc', '-vV'], text=True).split('host: ')[1].splitlines()[0],
                 *config], cwd=consumer, env=env, text=True))
            core = [p for p in metadata['packages'] if p['name'] in ['gpui', 'fanta-gpui-core']]
            assert len(core) == 1 and core[0]['name'] == 'fanta-gpui-core', core
            for package in metadata['packages']:
                if package['name'] in packages:
                    assert package['source'] is not None, package['name']
                    assert not Path(package['manifest_path']).is_relative_to(release.ROOT)
            if args.run:
                subprocess.run(['cargo', 'run', *config], cwd=consumer,
                               env={**env, 'FANTA_SMOKE_AUTO_QUIT': '1'}, check=True, timeout=300)
        print('Packaged registry consumer passed; no workspace paths or patches.', flush=True)
    finally:
        server.shutdown()


if __name__ == '__main__':
    main()
