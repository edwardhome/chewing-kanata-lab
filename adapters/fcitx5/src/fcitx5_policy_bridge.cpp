#include "fcitx5_policy_bridge.h"
#include <vector>

namespace chewing_kanata {

Fcitx5PolicyBridge::Fcitx5PolicyBridge(policy_mode_t mode) {
    context_ = policy_context_create(mode);
}

void Fcitx5PolicyBridge::setMode(policy_mode_t mode) {
    context_.mode = mode;
}

policy_mode_t Fcitx5PolicyBridge::mode() const {
    return context_.mode;
}

void Fcitx5PolicyBridge::setPreeditActive(bool active) {
    context_.has_active_preedit = active;
}

void Fcitx5PolicyBridge::setSecureField(bool secure) {
    context_.is_secure_field = secure;
}

void Fcitx5PolicyBridge::setEditorMode(editor_mode_t mode) {
    context_.editor_mode = mode;
}

bool Fcitx5PolicyBridge::shouldBypassKey(char asciiKey) const {
    return policy_should_bypass_key(asciiKey, &context_);
}

std::string Fcitx5PolicyBridge::transformCommit(const std::string &rawCommit) const {
    if (rawCommit.empty()) {
        return "";
    }
    std::vector<char> buf(rawCommit.size() * 2 + 16);
    size_t written = policy_transform_commit(rawCommit.c_str(), &context_, buf.data(), buf.size());
    if (written > 0) {
        return std::string(buf.data(), written);
    }
    return rawCommit;
}

} // namespace chewing_kanata
