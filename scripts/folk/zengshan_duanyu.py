"""《增刪卜易》298 卦例之断语结构化：取各例卦图之后至下一卦图（或下一章）之间的断语，
按可机械判定之象（月破、日破、暗動、旬空、沖空、合起、合住／合絆、入墓、化進神、化退神、回頭生、回頭克、
化沖、化合、化空、化破、化墓、六沖、六合、反吟、伏吟、休囚、旺相）切出所在分句，并记分句中所指之爻（地支、六亲、世應）
与肯否（「不爲」「非」「不是」等为否）。

用法（仓库根）：python3 scripts/folk/zengshan_duanyu.py data/folk/zsby_duanyu.csv
例号与 rust/core/src/folk/liuyao.rs 测试常量 `ZSBY` 同序（与 zengshan_buyi_extract.py 取例之法相同）。
比对在 `liuyao::tests::zsby_duanyu`（以 huang_jin_marks 逐条核），结果见 doc/folk/liuyao.md。
"""
import re
import sys

D = 'vendor/folk/wikisource/增刪卜易/'
order = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10', '11', '12', '13', '14', '15', '15又', '16', '17', '18', '19', '20',
         '21', '22', '23', '24', '25', '26', '26又1', '26又2', '26又3', '26又4']
src = []
for o in order:
    t = open(D + o + '.wikitext').read()
    sec = re.search(r'section = ([^\n|}]*)', t).group(1).strip()
    src.append(('卷一·' + sec, t))
full = open(D + '全文.wikitext').read()
for m in re.finditer(r'== (增刪卜易卷之[二三四]) ==\n(.*?)(?=\n== 增刪卜易卷之|\Z)', full, re.S):
    src.append((m.group(1), m.group(2)))
Q = '父母|兄弟|妻財|官鬼|子孫'
Z = '子丑寅卯夘辰巳午未申酉戌亥'
SH = '青龍|靑龍|朱雀|勾陳|螣蛇|滕蛇|媵蛇|白虎|玄武|元武'
YL = re.compile(rf'^(?:({SH})　)?({Q})([{Z}])([金木水火土])([⚊⚋○ㄨ])(世|應)?(?:伏({Q})([{Z}])([金木水火土]))?(?:　+(?:({Q})([{Z}])([金木水火土])([⚊⚋]))?)?\s*$')
HEAD = re.compile(r'^[^\s]{2,14}章第|^(《星煞章》|增刪《黃金策千金賦》)第')

# 象：(书中写法正则, 归一名)。先长后短，已匹配处挖去，免「化進神」又算「進神」等重复
XIANG = [
    (r'回頭之?[克剋尅]', '回頭克'), (r'回頭之?生', '回頭生'),
    (r'化[為爲]?進神|化進', '化進神'), (r'化[為爲]?退神|化退', '化退神'),
    (r'化[為爲]?旬空|化空', '化空'), (r'化[為爲]?月破|化破', '化破'), (r'化[入爲為]?墓', '化墓'),
    (r'化[為爲]?回頭之?沖|化沖|變沖', '化沖'), (r'化合|變合', '化合'),
    (r'沖空', '沖空'), (r'月破', '月破'), (r'日破', '日破'), (r'暗動', '暗動'),
    (r'旬空|空亡', '旬空'), (r'合起', '合起'), (r'合住|合絆', '合絆'),
    (r'入墓|隨鬼入墓|墓於日', '入墓'),
    (r'六沖卦|卦逢六沖|爲六沖|乃六沖|是六沖|六沖', '六沖'), (r'六合卦|卦逢六合|爲六合|乃六合|是六合|六合', '六合'),
    (r'反吟', '反吟'), (r'伏吟', '伏吟'),
    (r'休囚', '休囚'), (r'旺相', '旺相'),
]
# 泛论、假设、应期之句（不专指本例卦象）：含此类字者记「泛」，余记「断」
FAN = re.compile(r'凡|若|倘|如|或|宜|待|之日|之年|之月|之時|欲|嫌|怕|最|每|不應|驗|章|術者|舊|謂之|名曰|曰|假|設|豈|何|乎|多|皆')
GUA_LEVEL = {'六沖', '六合', '反吟', '伏吟'}
NEG = re.compile(r'(不[爲為是]|非|不作|不以|不算|不論|無|未|不成)$')
QIN = [('父母', '父'), ('父', '父'), ('兄弟', '兄'), ('兄', '兄'), ('妻財', '財'), ('財', '財'), ('官鬼', '官'), ('官', '官'), ('鬼', '官'),
       ('子孫', '子孫'), ('福', '子孫')]
INTRO = re.compile(r'^(又如|如|例[：﹕]?|又|再|假如|譬如)')
SPLIT = re.compile(r'[﹐，。﹔；﹕：！？﹖、\s]')


def subjects(c):
    """分句中所指：地支（「子孫」之子除外）、六亲、世應"""
    out = []
    c2 = c.replace('子孫', '＃')
    for z in re.findall(r'[子丑寅卯夘辰巳午未申酉戌亥](?=[水火木金土爻])', c2):
        out.append(z.replace('夘', '卯'))
    for a, b in QIN:
        if a in c2:
            out.append(b)
            c2 = c2.replace(a, '')
    if '＃' in c.replace('子孫', '＃'):
        out.append('子孫')
    for s in '世應':
        if s in c:
            out.append(s)
    return sorted(set(out), key=out.index)


recs = []
for name, t in src:
    lines = [l.replace('<br>', '').strip() for l in t.split('\n')]
    chap = name
    i = 0
    while i < len(lines):
        l = lines[i]
        if HEAD.match(l) and len(l) < 24:
            chap = name + '·' + re.sub(r'﹝.*', '', l).replace('《', '').replace('》', '')
        if YL.match(l):
            j = i
            while j < len(lines) and YL.match(lines[j]):
                j += 1
            # 断语：卦图后至下一卦图或下一章（至多 12 行非空）
            k = j
            body = []
            while k < len(lines) and not YL.match(lines[k]) and not (HEAD.match(lines[k]) and len(lines[k]) < 24) and len(body) < 12:
                if INTRO.match(lines[k]) and '得' in lines[k] and re.search(r'[月日]', lines[k]):
                    break  # 下一例之起卦句
                if lines[k] and not lines[k].startswith('{{') and not lines[k].startswith('<'):
                    body.append(lines[k])
                k += 1
            recs.append((chap, ''.join(body)))
            i = j
            continue
        i += 1
assert len(recs) == 298, len(recs)

rows = []
for n, (chap, body) in enumerate(recs, 1):
    for c in SPLIT.split(body):
        if not c:
            continue
        rest = c
        for pat, nm in XIANG:
            for m in re.finditer(pat, rest):
                pre = rest[:m.start()]
                neg = bool(NEG.search(pre))
                sub = subjects(c)
                if not sub and nm not in GUA_LEVEL:
                    continue  # 无所指之泛论（如「最怕休囚」）不取
                rows.append((n, nm, '否' if neg else '是', '/'.join(sub), '泛' if FAN.search(c) else '断', c))
            rest = re.sub(pat, '〇' * 2, rest)
with open(sys.argv[1], 'w') as f:
    f.write('# 《增刪卜易》298 卦例之断语中可机械判定之象（由 scripts/folk/zengshan_duanyu.py 自维基文库录文 vendor/folk/wikisource/增刪卜易/ 生成）。\n')
    f.write('# 例＝rust/core/src/folk/liuyao.rs 测试常量 ZSBY 之序（1 起）；象＝归一之象名；书断：是＝书言有此象，否＝分句中此象前有「不爲」「非」「無」等；\n')
    f.write('# 所指＝分句中所见地支（带五行或爻字者）、六亲（父兄財官子孫）、世應；无所指者只取卦象（六沖六合反吟伏吟），余泛论不取；句＝所在分句原文。比对见 liuyao::tests::zsby_duanyu、doc/folk/liuyao.md。\n')
    f.write('# 类：断＝分句径指本例卦象；泛＝含「凡、若、如、或、宜、之日、之年、謂之……」等，多为泛论、假设、应期或他例，只作参考。\n')
    f.write('例,象,书断,所指,类,句\n')
    for r in rows:
        f.write(','.join(str(x).replace(',', '，') for x in r) + '\n')
print(len(rows), file=sys.stderr)
from collections import Counter
print(Counter(r[1] for r in rows), file=sys.stderr)
print(len({r[0] for r in rows}), 'examples', file=sys.stderr)
