#!/usr/bin/env bash
# 罗盘图隶书字体子集化：取 web/src/Wheels.tsx 全部汉字与排盘诸模块（奇门、六壬、太乙、七政、玄空）字符串字面量之字，
# 自 vendor/fonts/wangfonts/wt021.ttf（王漢宗中隸書繁，GPL v2）裁出 web/public/fonts/lishu.woff2。
# 用法（仓库根）：scripts/dev/subset_lishu.sh ；盘面新增文字后重跑。缺字（字库无、多为简体与卦符）浏览器自动回退缺省字体。
set -euo pipefail
cd "$(dirname "$0")/../.."
chars=$(mktemp)
python3 - > "$chars" <<'PY'
import re, glob
fs = ['web/src/Wheels.tsx', 'web/src/ZeRiPage.tsx'] + glob.glob('rust/core/src/folk/qimen/*.rs') + [f'rust/core/src/folk/{m}.rs' for m in ('liuren', 'taiyi', 'qizheng', 'xuankong', 'nianshen')]
cs = set()
for f in fs:
    t = open(f, encoding='utf-8').read()
    if f.endswith('.rs'):
        for m in re.finditer(r'"([^"\\]{1,6})"', t):
            cs.update(m.group(1))
    else:
        cs.update(t)
print(''.join(sorted(c for c in cs if '㐀' <= c <= '鿿' or c in '〇▲·')))
PY
mkdir -p web/public/fonts
uv run -q --no-project --with fonttools --with brotli pyftsubset vendor/fonts/wangfonts/wt021.ttf \
  --text-file="$chars" --flavor=woff2 --output-file=web/public/fonts/lishu.woff2 --layout-features='*' 2>&1 | grep -v "mort NOT subset" || true
cp vendor/fonts/wangfonts/COPYING web/public/fonts/LISHU-COPYING.txt
rm -f "$chars"
ls -la web/public/fonts/lishu.woff2
