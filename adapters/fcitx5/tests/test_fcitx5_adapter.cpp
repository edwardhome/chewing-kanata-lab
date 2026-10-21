#include "fcitx5_policy_bridge.h"
#include <iostream>
#include <cassert>

int main() {
    std::cout << "=========================================" << std::endl;
    std::cout << " Fcitx5 Adapter Bridge Unit Test Suite" << std::endl;
    std::cout << "=========================================" << std::endl;

    chewing_kanata::Fcitx5PolicyBridge bridge(POLICY_MODE_ASCII_FIRST);

    // 1. 測試 ASCII_FIRST 模式下無組字時放行語法鍵
    bridge.setPreeditActive(false);
    assert(bridge.shouldBypassKey(':') == true);
    assert(bridge.shouldBypassKey('[') == true);
    assert(bridge.shouldBypassKey(']') == true);
    assert(bridge.shouldBypassKey('`') == true);
    assert(bridge.shouldBypassKey('_') == true);
    std::cout << "  [PASS] ASCII syntax keys bypass when preedit is empty" << std::endl;

    // 2. 測試組字中時不放行以保護中文
    bridge.setPreeditActive(true);
    assert(bridge.shouldBypassKey(':') == false);
    assert(bridge.shouldBypassKey('[') == false);
    std::cout << "  [PASS] Keys are not bypassed when preedit is active" << std::endl;

    // 3. 測試 Commit 轉換
    std::string colon = "\xEF\xBC\x9A"; // 全形 ：
    std::string converted = bridge.transformCommit(colon);
    assert(converted == ":");
    std::cout << "  [PASS] Transform fullwidth colon to ascii in ASCII_FIRST" << std::endl;

    // 4. 測試 Legacy 模式回退
    bridge.setMode(POLICY_MODE_LEGACY);
    bridge.setPreeditActive(false);
    assert(bridge.shouldBypassKey(':') == false);
    std::string legacy_colon = bridge.transformCommit(colon);
    assert(legacy_colon == colon);
    std::cout << "  [PASS] Legacy mode preserves original fullwidth colon" << std::endl;

    // 5. 測試中文字詞無損保護
    std::string chinese = "測試新酷音輸入法";
    assert(bridge.transformCommit(chinese) == chinese);
    std::cout << "  [PASS] Chinese phrase remains 100% untouched" << std::endl;

    std::cout << "=========================================" << std::endl;
    std::cout << " All Fcitx5 Adapter Tests Passed!" << std::endl;
    std::cout << "=========================================" << std::endl;

    return 0;
}
