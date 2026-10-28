#!/usr/bin/env bash
# 一次性生成 VSOP87、ELP-MPP02、章动 tab5.3a、VSOP87 地球改正的 .bin，供前端零解析加载。
# 用法（在项目根目录 calendar/ 下）：./scripts/ephemeris/gen_ephemeris_bin.sh [vendor 目录]
# 默认 vendor 目录为 ./vendor；生成：vendor/vsop87/VSOP87B.ear.bin、vendor/elpmpp02/*.bin、vendor/IAU2000/tab5.3a.bin、data/fit/vsop87-{de406,de441}-earth.bin（由已提交的 data/fit/vsop87-{de406,de441}-planets.txt）

set -e
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VENDOR_DIR="${1:-$REPO_ROOT/vendor}"
RUST_DIR="$REPO_ROOT/rust"
CARGO_FEATURES="--no-default-features --features twofloat"

cd "$RUST_DIR"

echo "=== VSOP87 ==="
cargo run -p lunar-core --example vsop87_to_bin $CARGO_FEATURES -- \
  "$VENDOR_DIR/vsop87/VSOP87B.ear" \
  "$VENDOR_DIR/vsop87/VSOP87B.ear.bin"

echo ""
echo "=== ELP-MPP02 ==="
cargo run -p lunar-core --example elpmpp02_to_bin $CARGO_FEATURES -- \
  "$VENDOR_DIR/elpmpp02"

echo ""
echo "=== 章动 tab5.3a ==="
# base 为项目根，loader 读取 vendor/IAU2000/tab5.3a.txt，输出 vendor/IAU2000/tab5.3a.bin
cargo run -p lunar-core --example tab53a_to_bin $CARGO_FEATURES -- \
  "$REPO_ROOT"

echo ""
echo "=== VSOP87 地球改正（DE406 / DE441 版）==="
# 改正系数由 scripts/ephemeris/vsop87_de_planet_fit.py 本地拟合后提交（需 DE406/DE441 BSP），此处只转二进制
cargo run -p lunar-core --example patch_to_bin $CARGO_FEATURES

echo ""
echo "完成。可将 $VENDOR_DIR 与 $REPO_ROOT/data/fit 下的 .bin 拷贝到 web/public/data（npm run copy-data）。"
