#!/usr/bin/env bash
# 使用 uv 创建的 .venv 设置 PYO3 环境并执行 lunar-core 测试（含 VSOP87/ELPMPP02 vs DE406 Python 对比）。
# 用法：从仓库根目录执行 scripts/run_tests_with_venv.sh
# 依赖：uv 已创建 .venv，且已安装 jplephem（uv pip install jplephem）

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENV_PYTHON="${REPO_ROOT}/.venv/bin/python"
RUST_DIR="${REPO_ROOT}/rust"

if [[ ! -x "$VENV_PYTHON" ]]; then
  echo "error: .venv not found or Python missing. Create with: uv venv .venv" >&2
  exit 1
fi

export PYO3_PYTHON="$VENV_PYTHON"
export PYTHONHOME="$("$VENV_PYTHON" -c "import sys; print(sys.base_prefix)")"
export PYTHONPATH="$("$VENV_PYTHON" -c "import site; print(site.getsitepackages()[0])")"
LIBDIR="$("$VENV_PYTHON" -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')")"
if [[ -n "$LIBDIR" ]]; then
  export LD_LIBRARY_PATH="${LIBDIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi

cd "$RUST_DIR"
cargo test -p lunar-core "$@"
