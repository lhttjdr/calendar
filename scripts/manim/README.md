# 文档动画（Manim）

`doc/` 下的天文文档配图/动画，用 [Manim Community](https://github.com/ManimCommunity/manim) 生成。渲染是**本地一次性步骤**：产物（`.mp4` + 封面 `.jpg`）直接提交到 `doc/public/media/`，随 VitePress 静态构建一起部署；CI（`.github/workflows/pages.yml`）不装 Manim、不现场渲染。

## 系统依赖（仅本地渲染需要，CI 不需要）

- `ffmpeg`
- `cairo` + `pango`（`pycairo`/`manimpango` 的构建/运行依赖；Debian/Ubuntu 下装 `pkgconf libcairo2-dev libpango1.0-dev`）
- 一款支持简体中文的字体，如 **Noto Sans CJK SC**（`_style.py` 里 `CJK_FONT` 的候选字体族之一即可，装其中一种）

Debian/Ubuntu 一次装齐：

```bash
sudo apt install -y ffmpeg pkgconf libcairo2-dev libpango1.0-dev fonts-noto-cjk
```

Python 侧走 `scripts/` 既有的 uv 工程，用独立 dependency-group（见 `scripts/pyproject.toml` 的 `manim-docs`），不装进主依赖：

```bash
uv sync --project scripts --only-group manim-docs
```

## 渲染

留在仓库根（脚本内部路径假定 cwd 为仓库根，与 AGENTS.md 中其它 `scripts/` 调用约定一致）：

```bash
# 全部登记的动画，1080p60
uv run --project scripts --only-group manim-docs scripts/render_manim.sh

# 只渲染一个（路径为 scripts/manim/ 下场景文件相对路径，不含 .py）
uv run --project scripts --only-group manim-docs scripts/render_manim.sh reference_frames/timeline_fk_icrs

# 低清快速预览（-ql，迭代画面时用，不要提交这个质量的产物）
MANIM_QUALITY=-ql uv run --project scripts --only-group manim-docs scripts/render_manim.sh
```

产物写到 `doc/public/media/<topic>/<slug>.mp4`（+ 同名 `.jpg` 封面帧），文档里用根绝对路径引用：

```html
<video controls loop muted playsinline preload="metadata" poster="/media/<topic>/<slug>.jpg" style="width:100%;border-radius:8px">
  <source src="/media/<topic>/<slug>.mp4" type="video/mp4" />
</video>
```

`video`/`source` 的 `poster`/`src` 会被 VitePress（经 Vue 模板编译）当作资源引用处理：写根绝对路径 `/media/...` 即可，构建时会自动加上站点 base 前缀（本仓库文档站部署在子路径 `/<repo>/docs/`，见 `doc/.vitepress/config.mts` 的 `DOCS_BASE`）；**不要**用 `%BASE_URL%` 占位符——那是给纯文本/属性之外场景用的，写在这两个属性里会被当成字面模块路径去 `import`，构建直接报 `Rollup failed to resolve import` 失败。另外，**引用的文件必须已经存在于 `doc/public/media/` 下**：Vite 在构建时会尝试解析这个资源引用，文件不存在会导致 `doc` 的构建（连带 CI 的 Pages 部署）失败，所以渲染、复制产物必须在把 `<video>` 片段加进 `.md` 之前（或至少同一次提交里）完成。

Manim 渲染的中间产物（`scripts/manim/_build/`）已在 `.gitignore` 中排除，不要提交；只提交 `doc/public/media/` 下复制出来的最终 `.mp4`/`.jpg`。

## 新增一个动画

1. 在对应主题子目录（如 `scripts/manim/reference_frames/`）新建场景文件，复用 `scripts/manim/_style.py` 的配色与中文 `Text` 辅助函数（`label`/`caption`），保持跨动画视觉一致：
   - `COLOR_FIXED_FRAME`：惯性架/固定参照（ICRS、黄道等）
   - `COLOR_MEAN` / `COLOR_TRUE`：「平」/「真」语义对（岁差 vs 章动、平赤道 vs 真赤道等），全仓库动画统一用这一对颜色表达同一语义
   - `COLOR_HIGHLIGHT`：强调/连接关系（如某个改正矩阵、某条箭头）
   - 背景固定用 `BACKGROUND`（深空蓝），不要用纯黑/纯白——video 是内嵌在文档页面里的固定背景，需要在浅色/深色两种站点主题下都不违和
   - 中文一律用 `Text`（Pango），不要用 `Tex`/`MathTex`（LaTeX），避免多引入一套 texlive 系统依赖；数学符号用 unicode（如 `≈`、上标可用全角字符或直接写文字）
   - 场景文件用 `sys.path.insert(0, ...)` 显式把 `scripts/manim/` 加入 `sys.path` 再 `from _style import ...`（manim CLI 只把*场景文件自身所在目录*加入 `sys.path`，子目录里的场景拿不到上一级的 `_style.py`），已有场景文件顶部是现成写法，照抄即可。
2. 在 `scripts/render_manim.sh` 的 `SCENES` 数组里登记一行（场景文件相对路径、类名、输出子路径）。
3. 渲染、检查 `doc/public/media/` 下的产物，在对应 `doc/*.md` 里用上面的 `<video>` 片段引用，然后一并提交 `.mp4`/`.jpg` 与 `.md` 改动。

## 已有场景

| 场景 | 对应文档 | 内容 |
|------|----------|------|
| `reference_frames/timeline_fk_icrs.py` | `doc/astronomy/reference-frames.md` §天球参考的发展历程 | 各国为政 → FK4(B1950.0) → FK5(J2000.0) → ICRS/ICRF 时间线，标出 Frame Bias B |
| `reference_frames/precession_nutation_equator.py` | `doc/astronomy/reference-frames.md` §动力学坐标、平赤道与真赤道 | 平赤道绕黄极的岁差进动，叠加章动得到真赤道（周期夸张加速，定性示意） |
