"""由维基文库《周易》经文（vendor/folk/wikisource/周易/<卦名>.wikitext，原样）抽出卦名、上下卦、卦辞、六爻辞、用九用六，
写成 rust/core/src/folk/zhouyi_text.rs（常量表，编译期嵌入；不经 LFS）。
用法（仓库根）：python3 scripts/folk/gen_zhouyi_text.py
"""
import re, glob, os

TRI = {'乾': 0b111, '兌': 0b011, '離': 0b101, '震': 0b001, '巽': 0b110, '坎': 0b010, '艮': 0b100, '坤': 0b000}
# 三爻卦以初爻为最低位：乾 111，兌 初二阳三阴 → 011，離 101，震 初阳 → 001，巽 初阴二三阳 → 110，坎 010，艮 上阳 → 100，坤 000
ALIAS = {'兑': '兌', '离': '離'}


def clean(s):
    s = re.sub(r'-\{([^}]*)\}-', r'\1', s)
    s = re.sub(r'<[^>]+>', '', s)
    s = s.replace("'''", '')
    return s.strip()


rows = []
for p in sorted(glob.glob('vendor/folk/wikisource/周易/*.wikitext')):
    name = os.path.basename(p)[:-9]
    if name in ('目錄', '繫辭上'):
        continue
    t = open(p, encoding='utf-8').read()
    m = re.search(r'第(.+?)卦', t)
    num = m.group(1)
    m = re.search(r'([乾兌兑離离震巽坎艮坤])下([乾兌兑離离震巽坎艮坤])上', clean(t))
    lo, up = (ALIAS.get(x, x) for x in m.groups())
    # 逐行取经文（蓝字）；有的页（如艮）span 未闭合，故按行而不按标签对
    blue = [clean(l.split('<span style="color:blue">', 1)[1]) for l in t.splitlines() if '<span style="color:blue">' in l]
    blue = [b for b in blue if b and not b.startswith('易經')]
    gua = blue[0]
    gua = re.sub(r'^' + re.escape(name) + r'：', '', gua)
    yao = [b for b in blue[1:] if re.match(r'^(初|上|六|九|用)', b)]
    yong = None
    six = []
    for y in yao:
        if y.startswith('用九') or y.startswith('用六'):
            yong = y
        else:
            six.append(y)
    assert len(six) == 6, (name, six)
    rows.append((name, TRI[lo], TRI[up], gua, six, yong))

CN = '一二三四五六七八九十'
def cn2int(s):
    # 第X卦（一至六十四）
    v = 0
    if '十' in s:
        a, b = s.split('十')
        v = (CN.index(a) + 1 if a else 1) * 10 + (CN.index(b) + 1 if b else 0)
    else:
        v = CN.index(s) + 1
    return v

out = ['//! 《周易》经文（卦辞、爻辞、用九用六）。由 scripts/folk/gen_zhouyi_text.py 自维基文库录文',
       '//! （vendor/folk/wikisource/周易/，原样）生成，勿手改。卦以六位二进制表示，初爻为最低位、阳为 1。',
       '',
       '/// (通行本卦序 1—64, 卦名, 卦画, 卦辞, 六爻辞（初至上）, 用九/用六)',
       'pub const GUA: [(u8, &str, u8, &str, [&str; 6], Option<&str>); 64] = [']
seq = {}
for p in glob.glob('vendor/folk/wikisource/周易/*.wikitext'):
    name = os.path.basename(p)[:-9]
    m = re.search(r'第(.+?)卦', open(p, encoding='utf-8').read())
    if m and name not in ('目錄', '繫辭上'):
        seq[name] = cn2int(m.group(1))
rows.sort(key=lambda r: seq[r[0]])
assert [seq[r[0]] for r in rows] == list(range(1, 65))
q = lambda s: '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'
for name, lo, up, gua, six, yong in rows:
    bits = lo | (up << 3)
    out.append(f'    ({seq[name]}, {q(name)}, 0b{bits:06b}, {q(gua)}, [{", ".join(q(x) for x in six)}], {"Some(" + q(yong) + ")" if yong else "None"}),')
out.append('];')
open('rust/core/src/folk/zhouyi_text.rs', 'w', encoding='utf-8').write('\n'.join(out) + '\n')
print(len(rows), '卦')
