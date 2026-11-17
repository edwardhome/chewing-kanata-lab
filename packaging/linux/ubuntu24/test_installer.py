from pathlib import Path
import os,tempfile,subprocess,sys
p=Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(prefix='ckai-installer-test-') as d:
 root=Path(d);env=dict(os.environ,HOME=d,XDG_DATA_HOME=str(root/'data'),XDG_CONFIG_HOME=str(root/'config'))
 def run(mode):return subprocess.run(['python3',str(p),mode],env=env,capture_output=True,text=True)
 assert run('install').returncode==0
 a=root/'data/fcitx5/addon/ai-chewing.conf';assert 'Library='+str(root/'data/ai-chewing/previews') in a.read_text()
 assert run('install').returncode!=0
 private=root/'data/ai-chewing/userphrase.sqlite';private.write_bytes(b'synthetic sentinel')
 assert run('remove').returncode==0;assert not a.exists();assert private.read_bytes()==b'synthetic sentinel'
 a.write_text('unmanaged');assert run('remove').returncode!=0;assert a.read_text()=='unmanaged'
 print('PASS: isolated install, absolute loader path, duplicate protection, removal preserves dictionary, unmanaged registration protected')
