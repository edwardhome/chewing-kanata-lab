"""Exercise the installer only in copied bundles and disposable HOME directories."""
from pathlib import Path
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest


if len(sys.argv) != 2:
    raise SystemExit('Usage: python3 test_installer.py /path/to/bundle/install-preview.py')
SOURCE_INSTALLER = Path(sys.argv.pop()).resolve()
BUNDLE_FILES = (
    'libai-chewing.so', 'ai-chewing.conf', 'install-preview.py',
    'UPSTREAM-COPYING', 'corresponding-source.tar.gz', 'README.txt',
    'SHA256SUMS',
)
version_match = re.search(r"VERSION\s*=\s*['\"]([^'\"]+)['\"]", SOURCE_INSTALLER.read_text())
if not version_match:
    raise SystemExit('The installer must declare its VERSION for this package test.')
VERSION = version_match.group(1)
SENTINEL = b'synthetic dictionary: never overwrite or remove'


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ckai-installer-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bundle = self.root / 'bundle'
        self.bundle.mkdir()
        for name in BUNDLE_FILES:
            shutil.copy2(SOURCE_INSTALLER.parent / name, self.bundle / name)
        self.home = self.root / 'home'
        self.home.mkdir()
        self.data = self.home / 'data'
        self.config = self.home / 'config'
        self.env = dict(os.environ, HOME=str(self.home), XDG_DATA_HOME=str(self.data),
                        XDG_CONFIG_HOME=str(self.config))
        self.addon = self.data / 'fcitx5/addon/ai-chewing.conf'
        self.entry = self.data / 'fcitx5/inputmethod/ai-chewing.conf'
        self.base = self.data / 'ai-chewing/previews' / VERSION
        self.dictionary = self.data / 'ai-chewing/userphrase.sqlite'
        self.dictionary.parent.mkdir(parents=True)
        self.dictionary.write_bytes(SENTINEL)

    def run_installer(self, mode, success=True):
        result = subprocess.run(
            [sys.executable, str(self.bundle / 'install-preview.py'), mode],
            env=self.env, capture_output=True, text=True,
        )
        output = result.stdout + result.stderr
        if success:
            self.assertEqual(result.returncode, 0, output)
        else:
            self.assertNotEqual(result.returncode, 0, output)
        self.assertEqual(self.dictionary.read_bytes(), SENTINEL)
        return result

    def assert_no_registration(self):
        for path in (self.addon, self.entry):
            self.assertFalse(path.exists() or path.is_symlink(), str(path))

    def manifest_lines(self):
        return (self.bundle / 'SHA256SUMS').read_text().splitlines()

    def write_manifest(self, lines):
        (self.bundle / 'SHA256SUMS').write_text('\n'.join(lines) + '\n')

    def test_isolated_install(self):
        self.run_installer('install')
        self.assertTrue(self.addon.is_file())
        self.assertTrue(self.entry.is_file())
        self.assertEqual((self.base / 'libai-chewing.so').read_bytes(),
                         (self.bundle / 'libai-chewing.so').read_bytes())
        self.assertFalse((self.home / '.chewing').exists())
        self.assertFalse((self.data / 'chewing').exists())

    def test_absolute_loader_path(self):
        self.run_installer('install')
        self.assertIn('Library=' + str(self.base / 'libai-chewing') + '\n',
                      self.addon.read_text())

    def test_duplicate_registration_preserved(self):
        self.run_installer('install')
        saved = {p: p.read_bytes() for p in (self.addon, self.entry)}
        self.run_installer('install', success=False)
        for path, content in saved.items():
            self.assertEqual(path.read_bytes(), content)

    def test_remove_preserves_dictionary_settings_and_binary(self):
        self.run_installer('install')
        setting = self.config / 'fcitx5/conf/ai-chewing.conf'
        setting.parent.mkdir(parents=True)
        setting.write_text('synthetic settings\n')
        library = (self.base / 'libai-chewing.so').read_bytes()
        self.run_installer('remove')
        self.assert_no_registration()
        self.assertEqual(setting.read_text(), 'synthetic settings\n')
        self.assertEqual((self.base / 'libai-chewing.so').read_bytes(), library)

    def test_unmanaged_registration_preserved(self):
        self.addon.parent.mkdir(parents=True)
        self.addon.write_text('unmanaged\n')
        self.run_installer('remove', success=False)
        self.assertEqual(self.addon.read_text(), 'unmanaged\n')

    def test_empty_manifest_rejected(self):
        self.write_manifest([])
        self.run_installer('install', success=False)
        self.assert_no_registration()
        self.assertFalse(self.base.exists())

    def test_missing_manifest_entry_rejected(self):
        self.write_manifest(self.manifest_lines()[1:])
        self.run_installer('install', success=False)
        self.assert_no_registration()
        self.assertFalse(self.base.exists())

    def test_duplicate_manifest_entry_rejected(self):
        lines = self.manifest_lines()
        self.write_manifest(lines + [lines[0]])
        self.run_installer('install', success=False)
        self.assert_no_registration()
        self.assertFalse(self.base.exists())

    def test_tampered_payload_rejected(self):
        with (self.bundle / 'libai-chewing.so').open('ab') as stream:
            stream.write(b'tampered')
        self.run_installer('install', success=False)
        self.assert_no_registration()
        self.assertFalse(self.base.exists())

    def test_tampered_digest_rejected(self):
        lines = self.manifest_lines()
        digest, name = lines[0].split('  ', 1)
        changed = ('0' if digest[0] != '0' else '1') + digest[1:]
        self.write_manifest([changed + '  ' + name] + lines[1:])
        self.run_installer('install', success=False)
        self.assert_no_registration()
        self.assertFalse(self.base.exists())

    def test_dangling_registration_symlink_rejected(self):
        self.entry.parent.mkdir(parents=True)
        target = self.root / 'must-not-be-created.conf'
        self.entry.symlink_to(target)
        self.run_installer('install', success=False)
        self.assertTrue(self.entry.is_symlink())
        self.assertFalse(target.exists())
        self.assertFalse(self.addon.exists())
        self.assertFalse(self.base.exists())

    def test_target_library_symlink_rejected(self):
        self.base.mkdir(parents=True)
        target = self.root / 'external-library'
        target.write_bytes((self.bundle / 'libai-chewing.so').read_bytes())
        (self.base / 'libai-chewing.so').symlink_to(target)
        self.run_installer('install', success=False)
        self.assertTrue((self.base / 'libai-chewing.so').is_symlink())
        self.assertEqual(target.read_bytes(), (self.bundle / 'libai-chewing.so').read_bytes())
        self.assert_no_registration()

    def test_dangling_target_library_symlink_rejected(self):
        self.base.mkdir(parents=True)
        target = self.root / 'must-not-be-created.so'
        (self.base / 'libai-chewing.so').symlink_to(target)
        self.run_installer('install', success=False)
        self.assertTrue((self.base / 'libai-chewing.so').is_symlink())
        self.assertFalse(target.exists())
        self.assert_no_registration()

    def test_version_directory_symlink_rejected(self):
        self.base.parent.mkdir(parents=True)
        target = self.root / 'external-version'
        target.mkdir()
        (target / 'libai-chewing.so').write_bytes((self.bundle / 'libai-chewing.so').read_bytes())
        self.base.symlink_to(target, target_is_directory=True)
        self.run_installer('install', success=False)
        self.assertTrue(self.base.is_symlink())
        self.assert_no_registration()

    def test_remove_preflights_both_registrations(self):
        self.run_installer('install')
        saved_addon = self.addon.read_bytes()
        self.entry.write_text('unmanaged second registration\n')
        self.run_installer('remove', success=False)
        self.assertEqual(self.addon.read_bytes(), saved_addon)
        self.assertEqual(self.entry.read_text(), 'unmanaged second registration\n')


if __name__ == '__main__':
    unittest.main(verbosity=2)
