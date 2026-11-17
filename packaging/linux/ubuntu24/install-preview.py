#!/usr/bin/env python3
"""Explicit user-local install/remove; never switches or restarts Fcitx5."""
from pathlib import Path
import os,platform,shutil,sys,hashlib
if platform.system()!='Linux' or platform.machine() not in ('x86_64','amd64'):
    raise SystemExit('This preview requires Linux AMD64.')
if os.geteuid()==0:raise SystemExit('Run as your desktop user, without sudo.')
mode=sys.argv[1] if len(sys.argv)==2 else ''
if mode not in ('install','remove'):raise SystemExit('Usage: python3 install-preview.py install|remove')
home=Path.home();data=Path(os.environ.get('XDG_DATA_HOME',str(home/'.local/share')))
if not data.is_absolute():raise SystemExit('XDG_DATA_HOME must be absolute.')
if '\n' in str(data) or '\r' in str(data):raise SystemExit('Unsupported data path.')
base=data/'ai-chewing/previews/1.3.0-dev.ubuntu.1'
addon=data/'fcitx5/addon/ai-chewing.conf';entry=data/'fcitx5/inputmethod/ai-chewing.conf'
marker='# Managed by AI-Chewing Ubuntu preview 1.3.0-dev.ubuntu.1\n'
if mode=='remove':
    for p in (addon,entry):
        if p.exists():
            if p.is_symlink() or not p.read_text().startswith(marker):raise SystemExit(f'Refusing to remove unmanaged file: {p}')
            p.unlink()
    print('Preview registration removed. Existing dictionary, settings and binaries retained. Restart Fcitx5 yourself when ready.')
    raise SystemExit(0)
src=Path(__file__).resolve().parent
for line in (src/'SHA256SUMS').read_text().splitlines():
    digest,name=line.split('  ',1);p=src/name
    if Path(name).is_absolute() or '..' in Path(name).parts:raise SystemExit('Invalid manifest path')
    if hashlib.sha256(p.read_bytes()).hexdigest()!=digest:raise SystemExit(f'Checksum mismatch: {name}')
for p in (addon,entry):
    if p.exists():raise SystemExit(f'Existing registration preserved: {p}. Remove this preview registration first if reinstalling.')
if base.exists():
    if (base/'libai-chewing.so').read_bytes()!=(src/'libai-chewing.so').read_bytes():raise SystemExit('Existing different preview preserved.')
else:
    base.mkdir(parents=True)
    shutil.copy2(src/'libai-chewing.so',base/'libai-chewing.so')
addon.parent.mkdir(parents=True,exist_ok=True);entry.parent.mkdir(parents=True,exist_ok=True)
addon.write_text(marker+f'[Addon]\nName=AI Chewing Ubuntu Preview\nCategory=InputMethod\nType=SharedLibrary\nOnDemand=True\nConfigurable=True\nLibrary={base}/libai-chewing\n\n[Addon/OptionalDependencies]\n0=chttrans\n')
entry.write_text(marker+(src/'ai-chewing.conf').read_text())
print('Registered AI Chewing Ubuntu preview. No input method was started or restarted.')
print('When ready, restart your Fcitx5 session and add AI Chewing (Ubuntu preview) using Fcitx5 configuration.')
print('This is a native-engine foundation preview; F layer, app rules and Ollama integration are not included yet.')
