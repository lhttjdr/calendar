#!/usr/bin/env bash
# 渲染 scripts/manim/ 下的文档动画，产物复制到 doc/public/media/<topic>/。
# 系统依赖、渲染约定见 scripts/manim/README.md。
#
# 用法（留在仓库根执行，与 AGENTS.md 中其它 scripts/ 调用方式一致）：
#   uv run --project scripts --only-group manim-docs scripts/render_manim.sh
#   uv run --project scripts --only-group manim-docs scripts/render_manim.sh reference_frames/timeline_fk_icrs   # 只渲染一个
#   MANIM_QUALITY=-ql uv run --project scripts --only-group manim-docs scripts/render_manim.sh                  # 低清快速预览
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

MEDIA_DIR="scripts/manim/_build"
OUT_ROOT="doc/public/media"
QUALITY="${MANIM_QUALITY:--qh}" # -ql 快速预览，-qh 正式产物（1080p60）

# 新增动画时在此登记一行："<场景文件相对 scripts/manim/，不含 .py> <场景类名> <doc/public/media/ 下的子路径，不含扩展名>"
SCENES=(
  "reference_frames/timeline_fk_icrs TimelineFkIcrs reference-frames/timeline-fk-icrs"
  "reference_frames/precession_nutation_equator PrecessionNutationEquator reference-frames/precession-nutation-equator"
)

want="${1:-}"

for entry in "${SCENES[@]}"; do
  read -r rel_module scene_class out_name <<<"$entry"
  if [[ -n "$want" && "$want" != "$rel_module" ]]; then
    continue
  fi

  script_file="scripts/manim/${rel_module}.py"
  echo "==> 渲染 $script_file :: $scene_class"
  manim "$QUALITY" --media_dir "$MEDIA_DIR" --disable_caching "$script_file" "$scene_class"

  rendered_mp4="$(find "$MEDIA_DIR/videos" -type f -name "${scene_class}.mp4" | sort | tail -n1)"
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
    ffmpeg -y -loglevel error -i "$dest_mp4" -ss 00:00:01 -frames:v 1 "$dest_poster"
  else
    echo "警告：未找到 ffmpeg，跳过封面帧 $dest_poster" >&2
  fi

  echo "    -> $dest_mp4"
done

echo "完成。请检查 $OUT_ROOT/ 下的产物（.mp4 + .jpg）并一并提交。"
