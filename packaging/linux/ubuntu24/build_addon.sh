#!/usr/bin/env bash
set -euo pipefail
source /etc/os-release
[[ "$ID" == ubuntu && "$VERSION_ID" == 24.04 && "$(dpkg --print-architecture)" == amd64 ]] || exit 1
root=$(cd "$(dirname "$0")/../../.." && pwd)
out=${1:?Provide an empty output directory}
mkdir -p "$out"
[[ -z "$(ls -A "$out")" ]] || { echo 'Output must be empty'; exit 1; }
out=$(cd "$out" && pwd)
git clone https://github.com/fcitx/fcitx5-chewing.git "$out/source"
git -C "$out/source" checkout --detach 918156ad0157da66a8fd1afff5fbcad40e38a85b
python3 "$root/adapters/fcitx5/ubuntu/prepare_source.py" "$out/source"
cmake -S "$out/source" -B "$out/build" -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr
cmake --build "$out/build" --parallel 4
DESTDIR="$out/stage" cmake --install "$out/build"
file "$out/stage/usr/lib/fcitx5/libai-chewing.so"
ldd "$out/stage/usr/lib/fcitx5/libai-chewing.so"
echo 'Native addon built. No installation, no Fcitx5 restart, no 1.3.0 feature-parity claim.'
