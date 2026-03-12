#!/usr/bin/env bash
# 构建两个 WASM 包：wasm-lib（TwoFloat）、wasm-lib-f64（Real=f64）。
# 用法（在项目根目录 calendar/ 下）：./scripts/build_wasm.sh [--release]
# 输出到 rust/wasm-lib/pkg、rust/wasm-lib-f64/pkg；前端通过 optionalDependencies 引用，npm install 后生效。
# CI 部署用：./scripts/build_wasm.sh --release

set -e
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUST_DIR="$REPO_ROOT/rust"
RELEASE_ARG=""
[[ "${1:-}" == "--release" ]] && RELEASE_ARG="--release"

cd "$RUST_DIR"

echo "=== wasm-lib (TwoFloat) ==="
wasm-pack build wasm-lib --target web $RELEASE_ARG

echo ""
echo "=== wasm-lib-f64 (Real=f64) ==="
wasm-pack build wasm-lib-f64 --target web $RELEASE_ARG

echo ""
echo "Done. 在 web/ 下执行 npm install 后即可使用。"
