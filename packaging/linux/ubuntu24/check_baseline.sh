#!/usr/bin/env bash
# Headless baseline only. Never installs or restarts an input method.
set -euo pipefail
source /etc/os-release
[[ "$ID" == ubuntu && "$VERSION_ID" == 24.04 ]] || { echo 'Requires Ubuntu 24.04'; exit 1; }
[[ "$(dpkg --print-architecture)" == amd64 ]] || { echo 'Requires AMD64'; exit 1; }
root=$(cd "$(dirname "$0")/../../.." && pwd)
output=$(mktemp -d)
trap 'rm -rf "$output"' EXIT
printf 'Ubuntu=%s Architecture=%s\n' "$VERSION_ID" "$(dpkg --print-architecture)"
pkg-config --modversion chewing Fcitx5Core Fcitx5Config Fcitx5Utils
cc -std=c99 -Wall -Wextra -Werror -I"$root/policy-core/include" \
  "$root/policy-core/src/policy_core.c" "$root/policy-core/tests/test_policy.c" -o "$output/policy-tests"
"$output/policy-tests"
cc -std=c99 -Wall -Wextra -Werror -fPIC -I"$root/policy-core/include" \
  -c "$root/policy-core/src/policy_core.c" -o "$output/policy.o"
c++ -std=c++11 -Wall -Wextra -Werror -I"$root/policy-core/include" -I"$root/adapters/fcitx5/include" \
  "$root/adapters/fcitx5/src/fcitx5_policy_bridge.cpp" \
  "$root/adapters/fcitx5/tests/test_fcitx5_adapter.cpp" "$output/policy.o" -o "$output/adapter-tests"
"$output/adapter-tests"
echo 'Baseline checks finished. Full Fcitx5 engine and 1.3.0 feature parity are NOT verified.'
