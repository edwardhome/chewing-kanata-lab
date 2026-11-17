#!/usr/bin/env python3
"""Prepare a pinned, disposable upstream checkout; never patch a user checkout."""
from pathlib import Path
import subprocess,sys
p=Path(sys.argv[1]).resolve()
expected='918156ad0157da66a8fd1afff5fbcad40e38a85b'
if subprocess.check_output(['git','-C',str(p),'rev-parse','HEAD'],text=True).strip()!=expected:
    raise SystemExit('Unexpected upstream revision')
if subprocess.check_output(['git','-C',str(p),'status','--porcelain'],text=True).strip():
    raise SystemExit('Source checkout must be clean')
def replace(path,old,new):
    f=p/path;s=f.read_text()
    if old not in s:raise SystemExit(f'Upstream context changed: {path}')
    f.write_text(s.replace(old,new))
replace('src/eim.cpp','#include <cstdarg>','#include <cstdarg>\n#include <cstdlib>\n#include <filesystem>\n#include <stdexcept>')
f=p/'src/eim.cpp';s=f.read_text();a=s.index('ChewingContext *getChewingContext() {');b=s.index('\nChewingEngine::ChewingEngine',a)
s=s[:a]+'''ChewingContext *getChewingContext() {
    // Explicit database path: never fall back to upstream personal data.
    const char *xdg = std::getenv("XDG_DATA_HOME");
    const char *home = std::getenv("HOME");
    std::filesystem::path root;
    if (xdg && xdg[0] == '/') {
        root = xdg;
    } else if (home && home[0] == '/') {
        root = std::filesystem::path(home) / ".local/share";
    } else {
        throw std::runtime_error("AI Chewing requires an absolute user data directory");
    }
    const auto directory = root / "ai-chewing";
    std::filesystem::create_directories(directory);
    const auto database = directory / "userphrase.sqlite";
    auto *context = chewing_new2(nullptr, database.c_str(), nullptr, nullptr);
    if (!context) {
        throw std::runtime_error("AI Chewing could not open its isolated dictionary");
    }
    return context;
}
''' +s[b:];s=s.replace('conf/chewing.conf','conf/ai-chewing.conf');s=s.replace('    CHEWING_DEBUG() << "KeyEvent: " << keyEvent.key().toString();\n','');f.write_text(s)
replace('src/eim.h','conf/chewing.conf','conf/ai-chewing.conf')
# Translation domain remains upstream to preserve translated native settings.
(p/'src/ai-chewing.conf').write_text('[InputMethod]\nName=AI Chewing (Ubuntu preview)\nName[zh_TW]=AI新酷音（Ubuntu 測試版）\nIcon=fcitx-chewing\nLabel=酷\nLangCode=zh_TW\nAddon=ai-chewing\nConfigurable=True\n')
(p/'src/CMakeLists.txt').write_text('''add_library(ai-chewing MODULE eim.cpp)
target_link_libraries(ai-chewing Fcitx5::Core Fcitx5::Config ${CHEWING_TARGET})
target_compile_features(ai-chewing PRIVATE cxx_std_17)
target_compile_definitions(ai-chewing PRIVATE FCITX_GETTEXT_DOMAIN="fcitx5-chewing")
fcitx5_add_i18n_definition(TARGETS ai-chewing)
install(TARGETS ai-chewing DESTINATION lib/fcitx5)
install(FILES ai-chewing.conf DESTINATION share/fcitx5/inputmethod)
''')
f=p/'CMakeLists.txt';s=f.read_text();a=s.index('fcitx5_translate_desktop_file(org.');b=s.index('feature_summary',a);s=s[:a]+s[b:];f.write_text(s)
