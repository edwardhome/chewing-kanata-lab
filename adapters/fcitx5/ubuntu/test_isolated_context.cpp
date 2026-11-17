// Synthetic isolated libchewing smoke test; no desktop session or personal data.
#include <chewing.h>
#include <dlfcn.h>
#include <cstdlib>
#include <filesystem>
#include <iostream>
#include <stdexcept>
namespace fcitx { ChewingContext *getChewingContext(); }
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    setenv("HOME", argv[2], 1);
    const auto data = std::filesystem::path(argv[2]) / "data";
    setenv("XDG_DATA_HOME", data.c_str(), 1);
    void *lib = dlopen(argv[1], RTLD_NOW);
    if (!lib) { std::cerr << dlerror(); return 3; }
    auto *ctx = fcitx::getChewingContext();
    if (!ctx) return 5;
    chewing_handle_Default(ctx, 'f');
    if (!chewing_bopomofo_Check(ctx)) return 6;
    chewing_delete(ctx);
    if (!std::filesystem::exists(data / "ai-chewing/userphrase.sqlite")) return 7;
    if (std::filesystem::exists(data / "chewing") || std::filesystem::exists(std::filesystem::path(argv[2]) / ".chewing")) return 8;
    std::cout << "PASS: module loads, native F creates Bopomofo, dictionary remains isolated\n";
}
