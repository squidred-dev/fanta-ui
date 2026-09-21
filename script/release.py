#!/usr/bin/env python3
"""Audit or publish the coordinated crates.io release, dependencies first."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import tomllib
from urllib.error import HTTPError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
USER_AGENT = 'fanta-gpui-release/0.1 (https://github.com/squidred-dev/fanta-ui)'


def cargo(*args, capture=False):
    return subprocess.run(['cargo', *args], cwd=ROOT, check=True, text=True,
                          stdout=subprocess.PIPE if capture else None).stdout


def packages():
    metadata = json.loads(cargo('metadata', '--no-deps', '--format-version', '1', capture=True))
    return {p['name']: p for p in metadata['packages'] if p['publish'] != []}


def ordered(packages):
    pending = dict(packages)
    result = []
    while pending:
        ready = sorted(name for name, p in pending.items()
                       if not any(d['name'] in pending for d in p['dependencies']))
        if not ready:
            raise RuntimeError(f'Publication dependency cycle: {sorted(pending)}')
        for name in ready:
            result.append(pending.pop(name))
    return result


def audit(packages):
    root_manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    if root_manifest.get('patch'):
        raise RuntimeError('Published workspace must not rely on root patches')
    version = root_manifest['workspace']['package']['version']
    for p in packages.values():
        if p['version'] != version or not p['license'] or not p['description'] or not p['repository']:
            raise RuntimeError(f"Incomplete release metadata: {p['name']}")
        for dep in p['dependencies']:
            if dep['name'].startswith('fanta-') and not dep['name'].startswith('fanta-gpui'):
                raise RuntimeError(f"Engine dependency in UI workspace: {p['name']} -> {dep['name']}")
            if dep['source'] and dep['source'].startswith('git+'):
                raise RuntimeError(f"Git dependency: {p['name']} -> {dep['name']}")
            if dep.get('path'):
                Path(dep['path']).resolve().relative_to(ROOT)
                if dep['name'] not in packages or dep['req'] != '=' + version:
                    raise RuntimeError(f"Unpublishable local dependency: {p['name']} -> {dep['name']}")
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    for p in lock['package']:
        if p.get('source', '').startswith('git+'):
            raise RuntimeError(f"Git source in lockfile: {p['name']}")
        if p['name'] == 'gpui':
            raise RuntimeError('Upstream GPUI creates incompatible duplicate types')
    return ordered(packages)


def registry(name):
    request = Request(f'https://crates.io/api/v1/crates/{name}', headers={'User-Agent': USER_AGENT})
    try:
        with urlopen(request, timeout=30) as response:
            return json.load(response)
    except HTTPError as error:
        if error.code == 404:
            return None
        raise


def index_record(name, version):
    prefix = name[:2] + '/' + name[2:4]
    request = Request(f'https://index.crates.io/{prefix}/{name}', headers={'User-Agent': USER_AGENT})
    try:
        with urlopen(request, timeout=30) as response:
            return next((record for line in response if line.strip()
                         if (record := json.loads(line))['vers'] == version), None)
    except HTTPError as error:
        if error.code == 404:
            return False
        raise


def publish(name):
    command = ['cargo', 'publish', '-p', name, '--locked']
    for attempt in range(4):
        result = subprocess.run(command, cwd=ROOT, text=True,
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        print(result.stdout, end='', flush=True)
        if result.returncode == 0:
            return
        if '429 Too Many Requests' not in result.stdout or attempt == 3:
            raise subprocess.CalledProcessError(result.returncode, command)
        # New-crate tokens replenish every ten minutes. A 429 rejects the upload,
        # so retry the same immutable version only after that interval has elapsed.
        print(f'{name}: crates.io rate limit; retrying in 601 seconds.', flush=True)
        time.sleep(601)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release-tag', help='Require a v<workspace version> release tag')
    parser.add_argument('--publish', action='store_true', help='Upload verified packages to crates.io')
    parser.add_argument('--resume', action='store_true', help='Skip published versions only when their archive checksum matches')
    parser.add_argument('--registry-check', action='store_true', help='Check registry names and versions')
    parser.add_argument('--package-lists', action='store_true', help='Audit each Cargo package file list')
    args = parser.parse_args()
    release = audit(packages())
    if args.release_tag:
        expected = 'v' + release[0]['version']
        if args.release_tag != expected:
            raise RuntimeError(f'Release tag {args.release_tag!r} must be {expected!r}')
    for p in release:
        print(f"{p['name']} {p['version']}", flush=True)
        if args.registry_check:
            data = registry(p['name'])
            print('  available' if data is None else '  exists: verify publisher ownership')
        if args.package_lists:
            files = cargo('package', '-p', p['name'], '--list', '--allow-dirty', capture=True).splitlines()
            if not any(Path(f).name.startswith('LICENSE') for f in files):
                raise RuntimeError(f"Missing packaged license: {p['name']}")
    if not args.publish:
        return
    # A release must correspond to a reviewable commit, including all imported files.
    changes = subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()
    if changes:
        raise RuntimeError('Commit the release sources before publishing:\n' + changes)
    for p in release:
        existing = index_record(p['name'], p['version'])
        if existing:
            if not args.resume:
                raise RuntimeError(f"{p['name']} {p['version']} exists; use --resume to verify its archive")
            cargo('package', '-p', p['name'], '--locked')
            archive = ROOT / 'target/package' / f"{p['name']}-{p['version']}.crate"
            if hashlib.sha256(archive.read_bytes()).hexdigest() != existing['cksum']:
                raise RuntimeError(f"Published archive differs from local source: {p['name']}")
            continue
        cargo('publish', '-p', p['name'], '--locked', '--dry-run')
        publish(p['name'])
        deadline = time.monotonic() + 180
        while not index_record(p['name'], p['version']):
            if time.monotonic() > deadline:
                raise RuntimeError(f"Registry index timeout after publishing {p['name']}")
            time.sleep(5)


if __name__ == '__main__':
    main()
