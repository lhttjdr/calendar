"""《后汉书》日月食记录：逐条解析纪年、月、日干支、朔晦，按实历朔日表换算成儒略日数（JDN），写出
data/phenomena/houhanshu_solar_eclipses.csv 与 houhanshu_lunar_eclipses.csv，供 rust/core 的
examples/historical_eclipses.rs 检验。

- 日食：《续汉书·五行志六》（今本《后汉书》志第十八）「日蝕」一节，原文总结「日蚀七十二：朔三十二，晦三十七，月二日三」。
  刘昭注（{{*|…}}，含《古今注》所载日食）不收。
- 月食：东汉无系统月食记录。收《五行志六》「月蚀非其月」两条（有日干支），以及《律历志中》「论月食」中可定月份的五事
  （只记月份者以该月望前后为搜索中心，见 lunar 表 `guess_jdn`）。

日期只由历法（vendor 实历朔日表，东汉为四分历）换算，不用天象。

用法（工作目录为仓库根）：python3 scripts/calendar_data/qinhan_eval/houhanshu_eclipse_dates.py <缓存目录>
缓存目录下缺少 卷92.txt、卷108.txt 时自动从维基文库下载（`後漢書/卷92`、`後漢書/卷108`，action=raw）。
"""
import csv, re, sys, urllib.parse, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import gaiyuan_date as g

ROOT = HERE.parents[2]
OUT_SOLAR = ROOT / 'data/phenomena/houhanshu_solar_eclipses.csv'
OUT_LUNAR = ROOT / 'data/phenomena/houhanshu_lunar_eclipses.csv'
CACHE = Path(sys.argv[1] if len(sys.argv) > 1 else 'houhanshu_cache'); CACHE.mkdir(parents=True, exist_ok=True)
for v in ('92', '108'):
    f = CACHE / f'卷{v}.txt'
    if not f.exists():
        url = 'https://zh.wikisource.org/w/index.php?title=' + urllib.parse.quote(f'後漢書/卷{v}') + '&action=raw'
        req = urllib.request.Request(url, headers={'User-Agent': 'lunar-calendar-research/0.1'})
        f.write_bytes(urllib.request.urlopen(req, timeout=60).read())

CN = {'元': 1, '一': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9, '十': 10}
def cnnum(s):
    if s == '十': return 10
    if s.startswith('十'): return 10 + CN[s[1]]
    if '十' in s: a, b = s.split('十'); return CN[a] * 10 + (CN[b] if b else 0)
    return CN[s]

# 元年公历年（太初以后正月岁首，纪年年即正月所在公历年）；维基文库本繁简混排，两种字形都收
ERA = {'建武': 25, '中元': 56, '永平': 58, '建初': 76, '元和': 84, '章和': 87, '永元': 89, '元興': 105, '元兴': 105,
       '延平': 106, '永初': 107, '元初': 114, '永寧': 120, '永宁': 120, '建光': 121, '延光': 122, '永建': 126,
       '陽嘉': 132, '阳嘉': 132, '永和': 136, '漢安': 142, '汉安': 142, '建康': 144, '永嘉': 145, '本初': 146,
       '建和': 147, '和平': 150, '元嘉': 151, '永興': 153, '永兴': 153, '永壽': 155, '永寿': 155, '延熹': 158,
       '永康': 167, '建寧': 168, '建宁': 168, '熹平': 172, '光和': 178, '中平': 184, '初平': 190, '興平': 194,
       '兴平': 194, '建安': 196}
EMP = '光武帝|明帝|章帝|和帝|殇帝|殤帝|安帝|顺帝|順帝|冲帝|質帝|质帝|桓帝|灵帝|靈帝|献帝|獻帝'
GZ_RE = '[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳已午未申酉戌亥甲]'  # 「已」「丙甲」为原文字误，下文校正/标注
NUM = '[元一二三四五六七八九十]+'
REC = re.compile(
    rf'(?P<emp>{EMP})?(?P<era>{"|".join(ERA)})?(?:(?P<yr>{NUM})年)?(?:[春夏秋冬])?'
    rf'(?P<mon>闰|閏|正|十二|十一|十|[二三四五六七八九])月(?:(?P<d>[二三])日)?(?P<gz>{GZ_RE})(?P<pos>朔|晦)?，日有[蚀蝕食]之')
LODGE = re.compile(r'^[^。]*?在([^\s，。、]{1,2}?)(初|[一二三四五六七八九十]+度)')

def strip_notes(t):
    """去刘昭注 {{*|…}}（可内嵌模板）、<ref>，展开 {{YL|…}}。"""
    out, i, depth = [], 0, 0
    while i < len(t):
        if t.startswith('{{*|', i): depth += 1; i += 4; continue
        if depth and t.startswith('{{', i): depth += 1; i += 2; continue
        if depth and t.startswith('}}', i): depth -= 1; i += 2; continue
        if not depth: out.append(t[i])
        i += 1
    t = re.sub(r'<ref>.*?</ref>', '', ''.join(out), flags=re.S)
    return re.sub(r'\{\{YL\|([^}|]*)[^}]*\}\}', r'\1', t)

def leap_month(year):
    for ly, lm, lp, _ in g._months():
        if ly == year and lp: return lm
    raise KeyError(f'{year} 年无闰月')

def resolve(year, month, leap, gz, pos, day):
    """返回 (jdn, lunar_day, note)。干支有误时按朔晦/日序定日并注明。"""
    start, end = g.month_span(year, month, leap)
    note = []
    expect = {'朔': start, '晦': end - 1}.get(pos) if pos else (start + day - 1 if day else None)
    if gz in g.GANZHI:
        off = (g.GANZHI.index(gz) - (start + 49) % 60) % 60
        if start + off < end:
            jdn = start + off
            if expect is not None and jdn != expect:
                note.append(f'历日为该月第{off + 1}日（月{end - start}日），与「{pos or f"{day}日"}」不合')
        else:
            jdn = expect if expect is not None else min((start + off - 60, start + off), key=lambda x: abs(x - (start + end) // 2))
            note.append(f'{gz}不在该月（朔{g.ganzhi_of_jdn(start)}），' + ('按朔晦/日序定日' if expect is not None else '取最近同名干支日'))
    else:
        jdn = expect
        note.append(f'「{gz}」非干支（字误），按朔晦定日，该日为{g.ganzhi_of_jdn(jdn)}')
    return jdn, jdn - start + 1, '；'.join(note)

# ---- 日食 ----
s108 = (CACHE / '卷108.txt').read_text(encoding='utf-8')
sec = strip_notes(s108[s108.index('==日蝕=='):s108.index('==日抱')])
summary = sec[sec.index('凡汉中兴'):].split('\n')[0]
rows = []
emp = era = None; yr = None
for m in REC.finditer(sec):
    if m['emp']: emp = m['emp']
    if m['era']: era = m['era']
    if m['yr']: yr = cnnum(m['yr'])
    year = ERA[era] + yr - 1
    gz = m['gz'].replace('已', '巳')
    leap = m['mon'] in ('闰', '閏')
    month = leap_month(year) if leap else (1 if m['mon'] == '正' else cnnum(m['mon']))
    day = cnnum(m['d']) if m['d'] else None
    pos = m['pos'] or (f'{day}日' if day else '')
    jdn, lday, note = resolve(year, month, leap, gz, m['pos'], day)
    if m['gz'] != gz: note = '；'.join(x for x in (f'原文「{m["gz"]}」，「已」为「巳」之讹', note) if x)
    tail = sec[m.end():m.end() + 80]
    extent = next((k for k in ('几尽', '幾盡', '既') if tail.startswith('，' + k)), '')
    lm = LODGE.match(tail)
    y, mo, d = g.from_jdn(jdn)
    rows.append(dict(emp=emp or '', era=era, year_in_era=yr, year=year, month=month, leap=int(leap), ganzhi=m['gz'],
                     position=pos, extent=extent, lodge=(lm.group(1) + lm.group(2)) if lm else '',
                     reported_by=next((x for x in re.findall(r'史官不[见見覺觉]，([^。，]+?)以[闻聞]|([^。，]{2,8})以[闻聞]', tail.split('日有')[0]) for x in x if x), ''),
                     calendar='实历朔日表', jdn=jdn, julian=f'{y}-{mo:02d}-{d:02d}', lunar_day=lday, note=note,
                     text=(m.group(0) + tail).split('。')[0] + '。'))
cnt = {p: sum(r['position'] == p for r in rows) for p in ('朔', '晦')}
other = len(rows) - cnt['朔'] - cnt['晦']
print(f'日食 {len(rows)} 条（朔 {cnt["朔"]}，晦 {cnt["晦"]}，其他 {other}）；原文：{summary}')
assert len(rows) == 72, len(rows)

cols = ['emp', 'era', 'year_in_era', 'year', 'month', 'leap', 'ganzhi', 'position', 'extent', 'lodge', 'reported_by',
        'calendar', 'jdn', 'julian', 'lunar_day', 'note', 'text']
with open(OUT_SOLAR, 'w', encoding='utf-8', newline='') as f:
    w = csv.DictWriter(f, fieldnames=cols); w.writeheader(); w.writerows(rows)
for r in rows:
    if r['note']: print(' ', r['era'], r['year_in_era'], r['month'], r['ganzhi'], r['position'], r['note'])

# ---- 月食 ----
# (纪年年, 月, 闰, 日干支或日序, 出处, 原文, 检验要点)
LUNAR = [
    (157, 12, False, '壬戌', '五行志六', '永寿三年十二月壬戌，月蚀非其月。', '该日是否月食'),
    (165, 1, False, '辛巳', '五行志六', '延熹八年正月辛巳，月蚀非其月。', '该日是否月食'),
    (89, 7, True, '', '律历志中·论月食', '永元元年，天以七月後閏食，術以八月。', '闰七月望有月食、八月望无'),
    (89, 8, False, '', '律历志中·论月食', '同上（术以八月）', '八月望无月食'),
    (90, 1, False, 16, '律历志中·论月食', '其二年正月十二日，蒙公乘宗紺上書言：「今月十六日月當食，而曆以二月。」至期如紺言。', '正月十六日月食'),
    (146, 12, False, '', '律历志中·论月食', '至本初元年，天以十二月食，曆以後年正月，於是始差。', '十二月望有月食'),
    (147, 1, False, '', '律历志中·论月食', '同上（曆以後年正月）', '建和元年正月望无月食'),
    (175, 12, False, '', '律历志中·论月食', '其四年，紺孫誠上書言：「……今年十二月當食，而官曆以後年正月。」到期如言。', '熹平四年十二月望有月食'),
    (176, 1, False, '', '律历志中·论月食', '同上（官曆以後年正月）', '熹平五年正月望无月食'),
    (179, 3, False, '', '律历志中·论月食', '光和二年歲在己未，三月、五月皆陰……誠以四月……「去年三月不食，當以四月。」', '三、四、五月望何者有月食'),
    (179, 4, False, '', '律历志中·论月食', '同上', '同上'),
    (179, 5, False, '', '律历志中·论月食', '同上', '同上'),
]
s92 = (CACHE / '卷92.txt').read_text(encoding='utf-8')
assert '天以七月後閏食' in s92 and '今月十六日月當食' in s92 and '天以十二月食' in s92 and '今年十二月當食' in s92
lrows = []
for year, month, leap, day, src, text, check in LUNAR:
    start, end = g.month_span(year, month, leap)
    if isinstance(day, str) and day:
        jdn, lday, note = resolve(year, month, leap, day, None, None)
        guess = jdn
    elif isinstance(day, int):
        jdn, lday, note = start + day - 1, day, ''
        guess = jdn
    else:
        jdn, lday, note = '', '', '只记月份'
        guess = start + 14  # 该月望前后（日序 15）
    y, mo, d = g.from_jdn(guess)
    lrows.append(dict(year=year, month=month, leap=int(leap), day=day, source=src, jdn=jdn, lunar_day=lday,
                      guess_jdn=guess, guess_julian=f'{y}-{mo:02d}-{d:02d}', check=check, note=note, text=text))
with open(OUT_LUNAR, 'w', encoding='utf-8', newline='') as f:
    w = csv.DictWriter(f, fieldnames=list(lrows[0])); w.writeheader(); w.writerows(lrows)
print(f'月食 {len(lrows)} 条 → {OUT_LUNAR.relative_to(ROOT)}')
