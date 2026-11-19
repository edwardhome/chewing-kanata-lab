#!/usr/bin/env python3
"""Explicit user-local install/remove; never switches or restarts Fcitx5."""
from pathlib import Path
import hashlib
import os
import platform
import re
import shutil
import sys

VERSION = '1.3.1-dev.ubuntu.1'
REQUIRED_FILES = {
    'libai-chewing.so', 'ai-chewing.conf', 'install-preview.py',
    'UPSTREAM-COPYING', 'corresponding-source.tar.gz', 'README.txt',
}


def exists(path):
    return path.exists() or path.is_symlink()


def verify_bundle(src):
    seen = set()
    for line in (src / 'SHA256SUMS').read_text().splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  (.+)', line)
        if not match:
            raise SystemExit('Invalid checksum manifest.')
        digest, name = match.groups()
        if name not in REQUIRED_FILES or name in seen:
            raise SystemExit('Unexpected or duplicate manifest file.')
        seen.add(name)
        path = src / name
        if path.is_symlink() or not path.is_file():
            raise SystemExit(f'Missing or linked bundle file: {name}')
        if hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise SystemExit(f'Checksum mismatch: {name}')
    if seen != REQUIRED_FILES:
        raise SystemExit('Incomplete checksum manifest.')


if platform.system() != 'Linux' or platform.machine() not in ('x86_64', 'amd64'):
    raise SystemExit('This preview requires Linux AMD64.')
if os.geteuid() == 0:
    raise SystemExit('Run as your desktop user, without sudo.')
mode = sys.argv[1] if len(sys.argv) == 2 else ''
if mode not in ('install', 'remove'):
    raise SystemExit('Usage: python3 install-preview.py install|remove')
home = Path.home()
data = Path(os.environ.get('XDG_DATA_HOME', str(home / '.local/share')))
if not data.is_absolute():
    raise SystemExit('XDG_DATA_HOME must be absolute.')
if '\n' in str(data) or '\r' in str(data):
    raise SystemExit('Unsupported data path.')
base = data / 'ai-chewing/previews' / VERSION
addon = data / 'fcitx5/addon/ai-chewing.conf'
entry = data / 'fcitx5/inputmethod/ai-chewing.conf'
marker = f'# Managed by AI-Chewing Ubuntu preview {VERSION}\n'

# Validate every target before deleting any registration. Never follow links.
if mode == 'remove':
    for path in (addon, entry):
        if exists(path) and (path.is_symlink() or not path.is_file()
                             or not path.read_text().startswith(marker)):
            raise SystemExit(f'Refusing to remove unmanaged file: {path}')
    for path in (addon, entry):
        if path.exists():
            path.unlink()
    print('Preview registration removed. Dictionary, settings and binaries retained. Restart Fcitx5 yourself when ready.')
    raise SystemExit(0)

src = Path(__file__).resolve().parent
verify_bundle(src)
for path in (addon, entry):
    if exists(path):
        raise SystemExit(f'Existing registration preserved: {path}. Remove the old preview registration with its original installer first.')
library = base / 'libai-chewing.so'
if base.is_symlink() or library.is_symlink():
    raise SystemExit('Linked preview destination preserved.')
if exists(base):
    if not base.is_dir() or not library.is_file() or library.read_bytes() != (src / library.name).read_bytes():
        raise SystemExit('Existing different preview preserved.')
else:
    base.mkdir(parents=True)
    shutil.copy2(src / library.name, library)
addon.parent.mkdir(parents=True, exist_ok=True)
entry.parent.mkdir(parents=True, exist_ok=True)
addon_text = marker + f'[Addon]\nName=AI Chewing Ubuntu Preview\nCategory=InputMethod\nType=SharedLibrary\nOnDemand=True\nConfigurable=True\nLibrary={base}/libai-chewing\n\n[Addon/OptionalDependencies]\n0=chttrans\n'
created = []
try:
    # Exclusive creation protects files that appeared after the preflight.
    for path, content in ((addon, addon_text), (entry, marker + (src / 'ai-chewing.conf').read_text())):
        with path.open('x') as stream:
            created.append(path)
            stream.write(content)
except OSError:
    for path in created:
        path.unlink()
    raise
print('Registered AI Chewing Ubuntu preview. No input method was started or restarted.')
print('When ready, restart your Fcitx5 session and add AI Chewing (Ubuntu preview) using Fcitx5 configuration.')
print('Foundation preview: F layer, app rules and Ollama integration are not included yet.')
