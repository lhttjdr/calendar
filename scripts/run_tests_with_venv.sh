#!/usr/bin/env bash
# 使用 uv 创建的 .venv 设置 PYO3 环境并执行 lunar-core 测试（含 VSOP87/ELPMPP02/DE406 Rust vs Python 对比）。
# 用法：从仓库根目录执行 scripts/run_tests_with_venv.sh
# 依赖：uv 已创建 .venv，且已安装 jplephem（uv pip install jplephem）

set -e

# 仓库根（绝对路径）；供定气/定朔 DE406 测试解析 data/月相和二十四节气的计算/TDBtimes.txt 等
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export REPO_ROOT
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

# DE406 BSP：供 de406_rust_vs_python_de406、定气/定朔 DE406 测试使用。路径相对仓库根 data/。
if [[ -f "${REPO_ROOT}/data/jpl/de406/de406.bsp" ]]; then
  export DE406_BSP="${REPO_ROOT}/data/jpl/de406/de406.bsp"
elif [[ -f "${REPO_ROOT}/data/jpl/de406.bsp" ]]; then
  export DE406_BSP="${REPO_ROOT}/data/jpl/de406.bsp"
elif [[ -f "${RUST_DIR}/../data/jpl/de406/de406.bsp" ]]; then
  export DE406_BSP="$(cd "${RUST_DIR}/.." && pwd)/data/jpl/de406/de406.bsp"
elif [[ -f "${RUST_DIR}/../data/jpl/de406.bsp" ]]; then
  export DE406_BSP="$(cd "${RUST_DIR}/.." && pwd)/data/jpl/de406.bsp"
fi

cd "$RUST_DIR"
# --nocapture 使测试内 println!（容差、实际残差、最大残差等）在终端显示
cargo test -p lunar-core -- --nocapture "$@"
