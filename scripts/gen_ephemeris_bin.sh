#!/usr/bin/env bash
# 一次性生成 VSOP87、ELP-MPP02、章动 tab5.3a、拟合表（若有 .txt）的 .bin，供前端零解析加载。
# 用法（在项目根目录 calendar/ 下）：./scripts/gen_ephemeris_bin.sh [数据目录]
# 默认数据目录为 ./data；生成：data/vsop87/VSOP87B.ear.bin、data/elpmpp02/*.bin、data/IAU2000/tab5.3a.bin、data/fit/vsop87-de406-icrs.bin（当 data/fit/vsop87-de406-icrs.txt 存在时）

set -e
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATA_DIR="${1:-$REPO_ROOT/data}"
RUST_DIR="$REPO_ROOT/rust"
CARGO_FEATURES="--no-default-features --features twofloat"

cd "$RUST_DIR"

echo "=== VSOP87 ==="
cargo run -p lunar-core --example vsop87_to_bin $CARGO_FEATURES -- \
  "$DATA_DIR/vsop87/VSOP87B.ear" \
  "$DATA_DIR/vsop87/VSOP87B.ear.bin"

echo ""
echo "=== ELP-MPP02 ==="
cargo run -p lunar-core --example elpmpp02_to_bin $CARGO_FEATURES -- \
  "$DATA_DIR/elpmpp02"

echo ""
echo "=== 章动 tab5.3a ==="
# base 为项目根，loader 读取 data/IAU2000/tab5.3a.txt，输出 data/IAU2000/tab5.3a.bin
cargo run -p lunar-core --example tab53a_to_bin $CARGO_FEATURES -- \
  "$REPO_ROOT"

echo ""
echo "=== 拟合表 vsop87-de406-icrs ==="
if [ -f "$REPO_ROOT/data/fit/vsop87-de406-icrs.txt" ]; then
  cargo run -p lunar-core --example patch_to_bin $CARGO_FEATURES
else
  echo "跳过（无 data/fit/vsop87-de406-icrs.txt）"
fi

echo ""
echo "完成。可将 $DATA_DIR 下 .bin 拷贝到 web/public/data（npm run copy-data）。"
