#ifndef FCITX5_POLICY_BRIDGE_H
#define FCITX5_POLICY_BRIDGE_H

#include "policy_core.h"
#include <string>

namespace chewing_kanata {

class Fcitx5PolicyBridge {
public:
    explicit Fcitx5PolicyBridge(policy_mode_t mode = POLICY_MODE_LEGACY);

    void setMode(policy_mode_t mode);
    policy_mode_t mode() const;

    void setPreeditActive(bool active);
    void setSecureField(bool secure);
    void setEditorMode(editor_mode_t mode);

    /**
     * @brief 判定是否在 keyEvent 階段直接放行按鍵給應用程式 (Bypass)
     */
    bool shouldBypassKey(char asciiKey) const;

    /**
     * @brief 對即將 Commit 的文字進行政策轉換
     */
    std::string transformCommit(const std::string &rawCommit) const;

private:
    policy_context_t context_;
};

} // namespace chewing_kanata

#endif // FCITX5_POLICY_BRIDGE_H
