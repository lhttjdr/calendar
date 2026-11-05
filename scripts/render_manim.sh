#!/usr/bin/env bash
# 渲染 scripts/manim/ 下的文档动画，产物复制到 site/public/media/<topic>/。
# 系统依赖、渲染约定见 scripts/manim/README.md。
#
# 用法（留在仓库根执行，与 AGENTS.md 中其它 scripts/ 调用方式一致）：
#   uv run --project scripts --only-group manim-docs scripts/render_manim.sh
#   uv run --project scripts --only-group manim-docs scripts/render_manim.sh celestial_sphere/precession          # 只渲染一个（可开多个终端并行渲染不同场景）
#   MANIM_QUALITY=-ql uv run --project scripts --only-group manim-docs scripts/render_manim.sh                  # 低清快速预览
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

MEDIA_DIR="scripts/manim/_build"
OUT_ROOT="site/public/media"
# 正式产物 1080p30（演示动画 30 帧足够，渲染量是 60 帧的一半）；MANIM_QUALITY=-ql 为低清快速预览
read -r -a QUALITY <<<"${MANIM_QUALITY:--r 1920,1080 --fps 30}"

# 新增动画时在此登记一行："<场景文件相对 scripts/manim/，不含 .py> <场景类名> <site/public/media/ 下的子路径，不含扩展名>"
SCENES=(
  "celestial_sphere/sphere_basics SphereBasics celestial-sphere/sphere-basics"
  "celestial_sphere/precession Precession celestial-sphere/precession"
  "celestial_sphere/nutation Nutation celestial-sphere/nutation"
  "celestial_sphere/epochs_equinoxes EpochsEquinoxes celestial-sphere/epochs-equinoxes"
  "celestial_sphere/rotation_sequences RotationSequences celestial-sphere/rotation-sequences"
  "celestial_sphere/quaternions Quaternions celestial-sphere/quaternions"
)

want="${1:-}"

for entry in "${SCENES[@]}"; do
  read -r rel_module scene_class out_name <<<"$entry"
  if [[ -n "$want" && "$want" != "$rel_module" ]]; then
    continue
  fi

  script_file="scripts/manim/${rel_module}.py"
  echo "==> 渲染 $script_file :: $scene_class"
  # 每个场景独立 media_dir：可并行渲染多个场景，LaTeX 缓存不会互相覆盖
  scene_media="$MEDIA_DIR/$scene_class"
  manim "${QUALITY[@]}" --media_dir "$scene_media" --disable_caching "$script_file" "$scene_class"

  rendered_mp4="$(find "$scene_media/videos" -type f -name "${scene_class}.mp4" | sort | tail -n1)"
  if [[ -z "$rendered_mp4" ]]; then
    echo "未找到渲染产物：$scene_class（检查上面 manim 的输出）" >&2
    exit 1
  fi

  dest_dir="$OUT_ROOT/$(dirname "$out_name")"
  mkdir -p "$dest_dir"
  dest_mp4="$OUT_ROOT/${out_name}.mp4"
  cp "$rendered_mp4" "$dest_mp4"

  dest_poster="$OUT_ROOT/${out_name}.jpg"
  if command -v ffmpeg >/dev/null 2>&1; then
    # 封面取末帧：动画逐步加元素，结尾画面最完整（取第 1 秒常是空屏）
    ffmpeg -y -loglevel error -sseof -0.2 -i "$dest_mp4" -frames:v 1 -q:v 3 "$dest_poster"
  else
    echo "警告：未找到 ffmpeg，跳过封面帧 $dest_poster" >&2
  fi

  echo "    -> $dest_mp4"
done

echo "完成。请检查 $OUT_ROOT/ 下的产物（.mp4 + .jpg）并一并提交。"
