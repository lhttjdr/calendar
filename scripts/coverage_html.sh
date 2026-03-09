#!/usr/bin/env bash
# 使用 cargo-llvm-cov 生成覆盖率：人类可读的 HTML 报告（默认），可选在浏览器中打开。
# 用法：从仓库根目录执行 scripts/coverage_html.sh [--open]
# 依赖：cargo install cargo-llvm-cov，且 rustup component add llvm-tools-preview
#       若需跑全量测试（含 Python 对比）：先 uv venv .venv 且 uv pip install jplephem，本脚本会复用 .venv 环境。

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUST_DIR="${REPO_ROOT}/rust"
VENV_PYTHON="${REPO_ROOT}/.venv/bin/python"

OPEN_BROWSER=""
for arg in "$@"; do
  if [[ "$arg" == "--open" ]]; then
    OPEN_BROWSER=1
  fi
done

# 若有 .venv 则设置 PYO3 环境，与 run_tests_with_venv.sh 一致
if [[ -x "$VENV_PYTHON" ]]; then
  export PYO3_PYTHON="$VENV_PYTHON"
  export PYTHONHOME="$("$VENV_PYTHON" -c "import sys; print(sys.base_prefix)")"
  export PYTHONPATH="$("$VENV_PYTHON" -c "import site; print(site.getsitepackages()[0])")"
  LIBDIR="$("$VENV_PYTHON" -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')")"
  if [[ -n "$LIBDIR" ]]; then
    export LD_LIBRARY_PATH="${LIBDIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  fi
  echo "Using .venv for PYO3 (full test set including Python comparison)."
else
  echo "No .venv found; running without Python tests (use: uv venv .venv && uv pip install jplephem for full coverage)."
fi

cd "$RUST_DIR"

# HTML 报告生成到 target/llvm-cov/html，人类可读
if [[ -n "$OPEN_BROWSER" ]]; then
  cargo llvm-cov --html --open -p lunar-core
else
  cargo llvm-cov --html -p lunar-core
  echo "HTML report: $RUST_DIR/target/llvm-cov/html/index.html"
fi
