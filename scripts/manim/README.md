# 文档动画（Manim）

读者文档站（`site/`）的几何演示动画，用 [Manim Community](https://github.com/ManimCommunity/manim) 生成。渲染是**本地一次性步骤**：产物（`.mp4` + 封面 `.jpg`）直接提交到 `site/public/media/`，随 VitePress 静态构建一起部署；CI（`.github/workflows/pages.yml`）不装 Manim、不现场渲染。

**原则**：动画演示几何关系，按真实几何与真实数值绘制（岁差、历元用 IAU 2006 / Vondrák 的真实角度，章动用真实级数）；量级太小画不出的效应（章动在天球上、frame bias）才放大，且字幕必须注明「放大约 N 倍」或「示意」。每个场景文件的模块文档字符串写明哪些是真实值、哪些放大。

## 系统依赖（仅本地渲染需要，CI 不需要）

- `ffmpeg`
- `cairo` + `pango`（`pycairo`/`manimpango` 的构建/运行依赖；Debian/Ubuntu 下装 `pkgconf libcairo2-dev libpango1.0-dev`）
- 一款支持简体中文的字体，如 **Noto Sans CJK SC**（`_style.py` 里 `CJK_FONT` 的候选字体族之一即可，装其中一种）
- LaTeX 与 `dvisvgm`（画面中的数学式用 `MathTex` 排版）

Debian/Ubuntu 一次装齐：

```bash
sudo apt install -y ffmpeg pkgconf libcairo2-dev libpango1.0-dev fonts-noto-cjk \
  texlive-latex-base texlive-latex-extra texlive-fonts-recommended dvisvgm
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
uv run --project scripts --only-group manim-docs scripts/render_manim.sh celestial_sphere/precession

# 低清快速预览（-ql，迭代画面时用，不要提交这个质量的产物）
MANIM_QUALITY=-ql uv run --project scripts --only-group manim-docs scripts/render_manim.sh
```

产物写到 `site/public/media/<topic>/<slug>.mp4`，封面 `.jpg` 取末帧（动画逐步加元素，末帧最完整）。正文里用主题组件引用，`src` 不含扩展名：

```md
<Anim src="celestial-sphere/precession" caption="一句说明" />
```

组件（`site/.vitepress/theme/Anim.vue`）内部用 `withBase` 拼路径，部署在子路径 `/<repo>/docs/` 下也正确。

Manim 渲染的中间产物（`scripts/manim/_build/`）已在 `.gitignore` 中排除，不要提交；只提交 `site/public/media/` 下复制出来的最终 `.mp4`/`.jpg`。

## 新增一个动画

1. 在对应主题子目录（如 `scripts/manim/celestial_sphere/`）新建场景文件。天球类场景继承 `_sky.SkyScene`，用 `_sky` 的大圆、球面弧、坐标换算构件；配色与文字用 `_style`：
   - 坐标约定（`_sky.py` 模块文档）：黄道为 xy 面，黄极 +z，基准春分点 +x，北天极 = (0, sin ε, cos ε)。
   - 语义色：`COLOR_ECLIPTIC` 黄道，`COLOR_MEAN` 平（当日平赤道/平极），`COLOR_TRUE` 真（章动后），`COLOR_EPOCH` 另一历元（如 J2000），`COLOR_HIGHLIGHT` 强调；坐标三轴 x 红、y 绿、z 蓝。全部动画同一语义用同一颜色。
   - 背景固定用 `BACKGROUND`（深空蓝），不要用纯黑/纯白——在文档站浅色/深色两种主题下都不违和。
   - 纯中文用 `label`/`caption`（Pango `Text`）；含数学式的一律用 `rich(r"绕 $x$ 转 $-\varepsilon_A$")`：`$…$` 内交给 LaTeX（`MathTex`），其余用 Pango，`\n` 分行。不要用 Unicode 下标拼写公式（如 ζ_A、R₃(θ)）。中文不要放进 `$…$`（LaTeX 未配 ctex）。
   - 动态读数只让数字落在 `$…$` 外（`rf"$\\Delta\\psi =$ {x:+.2f}″"`），公式部分每帧不变，LaTeX 按内容缓存，不会逐帧编译。
   - 多个渲染进程不要并行共用同一个 `--media_dir`：Tex 缓存同名文件会互相覆盖，报「does not support converting .dvi」之类的假错误。
   - 屏幕固定的文字用 `self.fix(m)`、随 3D 位置但始终正对镜头的标签用 `self.face(m)`，二者只登记不显示，由随后的 `FadeIn` 引入。直接用 `add_fixed_in_frame_mobjects` 会立即显示，文字提前整块出现。
   - 设了 `frame_center` 时，`fix()` 会把屏幕固定层平移回来。因此文字一律用 `to_corner`/`to_edge` 等绝对位置排版，**不要** `next_to` 一个已经 `fix` 过的对象（会被平移两次）。
   - 场景文件用 `sys.path.insert(0, ...)` 把 `scripts/manim/` 加入 `sys.path` 再导入 `_style`/`_sky`（manim CLI 只把场景文件自身所在目录加入 `sys.path`）。
2. 在 `scripts/render_manim.sh` 的 `SCENES` 数组里登记一行（场景文件相对路径、类名、输出子路径）。
3. 渲染、检查 `site/public/media/` 下的产物，在 `site/` 对应章节用 `<Anim>` 引用，一并提交 `.mp4`/`.jpg` 与 `.md`。

## 已有场景

| 场景 | 所在章节（site/） | 内容 | 真实 / 放大 |
|------|------|------|------|
| `celestial_sphere/sphere_basics.py` | 天球与坐标 | 赤道、黄道、黄赤交角、二分二至；太阳一年的赤纬；同一颗星的黄道与赤道坐标 | 全部真实 |
| `celestial_sphere/precession.py` | 岁差 | 天极绕黄极进动（前 3000—公元 16000 年），经右枢、北极星、织女一；春分点西退；黄道自身转动 | 日月岁差真实（匀速近似）；黄道转动放大 |
| `celestial_sphere/nutation.py` | 章动 | 月球交点线逆行；真极绕平极的轨迹（Meeus 四项级数，角秒）；叠加岁差的波浪线 | 全部真实，横纵同比例 |
| `celestial_sphere/epochs_equinoxes.py` | 历元、分点与参考架 | J2000 与前 1000 年的平赤道、平春分点；角宿一两套坐标；平/真春分点；ICRS 与 J2000 | 岁差真实；Δψ、frame bias 放大并注明倍数 |
| `celestial_sphere/rotation_sequences.py` | 岁差 | 前 1000 年岁差矩阵按三次旋转、Lieske 四次旋转、Fukushima–Williams 四次旋转分步完成 | 全部真实（P03 系数），三者互差约 1″ |
| `celestial_sphere/quaternions.py` | 转动的表示 | 欧拉定理：岁差 = 绕一轴转 41.64°；元素四元数相乘；对偶四元数与螺旋运动（日心黄道架 → 地心赤道架） | 全部真实 |
