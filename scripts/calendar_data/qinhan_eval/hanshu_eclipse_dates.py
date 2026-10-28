"""《汉书·五行志下之下》西汉日食（所列 54 条，原文总结作五十三）：逐条解析纪年、月、日干支、朔晦，换算成儒略日数（JDN），写出
data/phenomena/hanshu_solar_eclipses.csv，供 rust/core 的日食检验（examples/historical_eclipses.rs）读取。

日期换算只用历法，不用天象：
- 太初改历以前（至元封四年）：秦汉初复原历 hzx.py（李忠林 2012，十月岁首）；
- 太初以后：vendor 实历朔日表（gaiyuan_date.month_span）。
干支不在所记之月内时，取离该月中点最近的同名干支日并在 note 里注明（多为史书月份或干支有误）。

用法（工作目录为仓库根）：python3 scripts/calendar_data/qinhan_eval/hanshu_eclipse_dates.py <缓存目录>
缓存目录下缺少 漢書_卷027.txt 时自动从维基文库下载《汉书》卷二十七 wikitext（action=raw）。
"""
import csv, re, sys, urllib.parse, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent))
import gaiyuan_date as g
import hzx

ROOT = HERE.parents[2]
OUT = ROOT / 'data/phenomena/hanshu_solar_eclipses.csv'
CACHE = Path(sys.argv[1] if len(sys.argv) > 1 else 'hanshu_cache'); CACHE.mkdir(parents=True, exist_ok=True)
SRC = CACHE / '漢書_卷027.txt'
if not SRC.exists():
    url = 'https://zh.wikisource.org/w/index.php?title=' + urllib.parse.quote('漢書/卷027') + '&action=raw'
    req = urllib.request.Request(url, headers={'User-Agent': 'lunar-calendar-research/0.1'})
    SRC.write_bytes(urllib.request.urlopen(req, timeout=60).read())

CN = {'元': 1, '一': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9, '十': 10}
def cnnum(s):
    if s == '十': return 10
    if s.startswith('十'): return 10 + CN[s[1]]
    if '十' in s: a, b = s.split('十'); return CN[a] * 10 + (CN[b] if b else 0)
    return CN[s]

# 元年的天文纪年（太初以前十月岁首，N 年 = 正月所在公历年）
REIGN = {'高帝': -205, '惠帝': -193, '高后': -186, '文帝': -178, '景帝': -155}
SUB = {('文帝', '後'): -162, ('景帝', '中'): -148, ('景帝', '後'): -142}
ERA = {'建元': -139, '元光': -133, '元朔': -127, '元狩': -121, '元鼎': -115, '元封': -109,
       '太始': -95, '征和': -91, '始元': -85, '元鳳': -79, '地節': -68, '五鳳': -56,
       '永光': -42, '建昭': -37, '建始': -31, '河平': -27, '陽朔': -23, '永始': -15, '元延': -11,
       '元壽': -1, '元始': 1}
EMPERORS = '高帝|惠帝|高后|文帝|景帝|武帝|昭帝|宣帝|元帝|成帝|哀帝|平帝'
LINE = re.compile(
    rf'^(?P<emp>{EMPERORS})?(?P<era>{"|".join(ERA)})?(?P<sub>中|後)?(?:(?P<yr>元|[一二三四五六七八九十]+)年)?'
    r'(?P<mon>正|十二|十一|十|[二三四五六七八九])月(?P<gz>[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥])'
    r'(?P<pos>朔|晦|，先晦一日)')
LODGE = re.compile(r'在([^\s，。、]{1,2}?)(初|[一二三四五六七八九十]+度)?[，。]')

text = re.sub(r'\{\{YL\|([^}|]*)[^}]*\}\}', r'\1', SRC.read_text(encoding='utf-8'))
sec = text[text.index('====漢===='):text.index('凡漢著紀十二世')]

rows = []
emp = era = sub = None; year_in = None
for line in sec.splitlines():
    line = line.strip()
    m = LINE.match(line)
    if not m:
        continue
    if m['emp']:
        emp, era, sub = m['emp'], None, None
    if m['era']:
        era, sub = m['era'], None
    if m['sub']:
        sub = m['sub']
    if m['yr']:
        year_in = cnnum(m['yr'])
    if era:
        base, era_name = ERA[era], era
    elif sub:
        base, era_name = SUB[(emp, sub)], sub + '元'
    else:
        base, era_name = REIGN[emp], ''
    n = base + year_in - 1
    month = 1 if m['mon'] == '正' else cnnum(m['mon'])
    pos = {'朔': '朔', '晦': '晦', '，先晦一日': '先晦一日'}[m['pos']]
    body = line[m.end():]
    extent = next((k for k in ('不盡如鉤', '幾盡', '既') if k in body.split('在')[0] + body[:8]), '')
    lm = LODGE.search(body)
    lodge = (lm.group(1) + (lm.group(2) or '')) if lm else ''
    rows.append(dict(emp=emp, era=era_name, year_in_era=year_in, year=n, month=month, ganzhi=m['gz'], position=pos,
                     extent=extent, lodge=lodge, text=line.split('。')[0] + '。'))

# 原文总结作「日食五十三，朔十四，晦三十六，先晦一日三」，但所列实为 54 条（晦 37 条），与总数不符一条晦；
# 此处按所列逐条处理，断言条数以防解析遗漏或多出。
cnt = {p: sum(r['position'] == p for r in rows) for p in ('朔', '晦', '先晦一日')}
assert len(rows) == 54 and cnt == {'朔': 14, '晦': 37, '先晦一日': 3}, (len(rows), cnt)

def month_span(n, month):
    if n <= -104:  # 太初以前（元封四年为止）用复原历
        labs = hzx.year(n); nxt = hzx.year(n + 1)[0][1]
        d = dict(labs); keys = [l for l, _ in labs]
        i = keys.index(str(month))
        return d[str(month)], d[keys[i + 1]] if i + 1 < len(keys) else nxt, '复原历'
    s, e = g.month_span(n, month, False)
    return s, e, '实历朔日表'

for r in rows:
    start, end, cal = month_span(r['year'], r['month'])
    target = g.GANZHI.index(r['ganzhi'])
    off = (target - (start + 49) % 60) % 60
    note = []
    if start + off >= end:
        mid = (start + end) // 2
        cands = [start + off - 60, start + off]
        jdn = min(cands, key=lambda x: abs(x - mid))
        note.append(f'{r["ganzhi"]}不在该月（朔{g.ganzhi_of_jdn(start)}），取最近同名干支日')
    else:
        jdn = start + off
        expect = {'朔': start, '晦': end - 1, '先晦一日': end - 2}[r['position']]
        if jdn != expect:
            note.append(f'历日为该月第{off + 1}日（月{end - start}日），与「{r["position"]}」不合')
    y, mo, d = g.from_jdn(jdn)
    r.update(calendar=cal, jdn=jdn, julian=f'{y}-{mo:02d}-{d:02d}', lunar_day=jdn - start + 1, note='；'.join(note))

OUT.parent.mkdir(parents=True, exist_ok=True)
cols = ['emp', 'era', 'year_in_era', 'year', 'month', 'ganzhi', 'position', 'extent', 'lodge',
        'calendar', 'jdn', 'julian', 'lunar_day', 'note', 'text']
with open(OUT, 'w', encoding='utf-8', newline='') as f:
    w = csv.DictWriter(f, fieldnames=cols); w.writeheader(); w.writerows(rows)
print(f'{len(rows)} 条 → {OUT.relative_to(ROOT)}；{cnt}')
for r in rows:
    if r['note']:
        print(r['emp'], r['era'], r['year_in_era'], r['month'], r['ganzhi'], r['position'], r['note'])
