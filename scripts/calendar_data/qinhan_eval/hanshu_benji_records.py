"""《汉书》本纪卷一至卷六（高帝至武帝太初元年四月）「某月某干支」纪日抽取，供秦汉初历日检验
（`rust/core/examples/zhongyuan_reckoning_eval.rs` 读入，逐条比颛顼历推步与李忠林复原历）。

年月解析同 check_hanshu.py：纪年年（建亥，十月为岁首）换为夏正年——十月至十二月属上一夏正年；后九月记作闰九月。
只抽记录，不判合否。

用法（仓库根）：python3 scripts/calendar_data/qinhan_eval/hanshu_benji_records.py [缓存目录]
缓存目录（缺省 hanshu_cache/hanshu_benji，不入库）缺 001..006.txt 时从维基文库下载《汉书》卷一至卷六 wikitext。
输出 data/calendar/hanshu_qinhan_benji.csv。
"""
import csv, re, sys, time, urllib.parse, urllib.request
from pathlib import Path

CACHE = Path(sys.argv[1] if len(sys.argv) > 1 else 'hanshu_cache/hanshu_benji')
CACHE.mkdir(parents=True, exist_ok=True)
for v in ['001', '002', '003', '004', '005', '006']:
    f = CACHE / f'{v}.txt'
    if not f.exists():
        url = 'https://zh.wikisource.org/w/index.php?title=' + urllib.parse.quote(f'漢書/卷{v}') + '&action=raw'
        req = urllib.request.Request(url, headers={'User-Agent': 'calendar-research-script/0.1'})
        f.write_bytes(urllib.request.urlopen(req, timeout=60).read())
        time.sleep(2)

GZ = ['甲乙丙丁戊己庚辛壬癸'[i % 10] + '子丑寅卯辰巳午未申酉戌亥'[i % 12] for i in range(60)]
CN = {'元': 1, '一': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9, '十': 10}
def cnnum(s):
    if s in ('元', ''): return 1
    if s == '十': return 10
    if s.startswith('十'): return 10 + CN[s[1]]
    if '十' in s:
        a, b = s.split('十'); return CN[a] * 10 + (CN[b] if b else 0)
    return CN[s]
WUDI = {'建元': -139, '元光': -133, '元朔': -127, '元狩': -121, '元鼎': -115, '元封': -109, '太初': -103}
def N_from(vol, era, n):
    """纪年年（建亥年，以其正月所在之夏正年记）"""
    if vol == '001': return -205 + n - 1
    if vol == '002': return -193 + n - 1
    if vol == '003': return -186 + n - 1
    if vol == '004': return (-162 if era == '後' else -178) + n - 1
    if vol == '005': return {'中': -148, '後': -142}.get(era, -155) + n - 1
    if vol == '006': return WUDI[era] + n - 1 if era in WUDI else None
MONTH = re.compile(r'(?:春|夏|秋|冬)?(後九|閏|正|十二|十一|十|[二三四五六七八九])月')
TOK = re.compile(MONTH.pattern + '|(' + '|'.join(GZ) + ')(朔|晦)?')
YEAR_AT_START = re.compile(r'^(建元|元光|元朔|元狩|元鼎|元封|太初)?(元|[一二三四五六七八九十]+)年')
VOLNAME = {'001': '高帝紀', '002': '惠帝紀', '003': '高后紀', '004': '文帝紀', '005': '景帝紀', '006': '武帝紀'}

rows = []
for vol in ['001', '002', '003', '004', '005', '006']:
    text = open(CACHE / f'{vol}.txt', encoding='utf-8').read()
    text = re.sub(r'\{\{YL\|([^}|]*)[^}]*\}\}', r'\1', text)
    N = month = era = None
    for line in text.splitlines():
        line = line.strip()
        h = re.match(r'^==\s*(.+?)\s*==$', line)
        if h:
            head = h.group(1); month = None
            if head in ('前元', '中元', '後元'): era = {'前元': None, '中元': '中', '後元': '後'}[head]; N = None
            elif head.startswith('漢高祖'): N = -205
            elif head.startswith('秦') or head == '早年' or '贊' in head: N = None
            else:
                m = YEAR_AT_START.match(head)
                if m: N = N_from(vol, m.group(1) or era, cnnum(m.group(2)))
            continue
        m = YEAR_AT_START.match(line)
        if m and (vol != '006' or m.group(1) or era):
            if vol == '006' and m.group(1): era = m.group(1)
            N = N_from(vol, era, cnnum(m.group(2))); month = None
        if N is None or N < -205: continue
        for t in TOK.finditer(line):
            if t.group(1):
                lab = t.group(1)
                month = {'後九': '后9', '閏': '后9', '正': '1'}.get(lab) or str(cnnum(lab)); continue
            if month is None: continue
            # 太初元年五月起属太初历，不在本表
            if N > -103 or (N == -103 and month not in ('10', '11', '12', '1', '2', '3', '4')): continue
            gz, suf = t.group(2), t.group(3) or ''
            leap = month == '后9'
            m_num = 9 if leap else int(month)
            ly = N - 1 if m_num >= 10 and not leap else N
            ctx = line[max(0, t.start() - 16):t.end() + 6].replace(',', '，')
            rows.append([VOLNAME[vol], N, ly, m_num, int(leap), gz, suf, ctx])

out = Path('data/calendar/hanshu_qinhan_benji.csv')
with open(out, 'w', encoding='utf-8', newline='') as f:
    f.write('# 《汉书》本纪高帝至太初元年四月纪日（scripts/calendar_data/qinhan_eval/hanshu_benji_records.py）。纪年年为建亥年；'
            '夏正年、月为本项目月序（十至十二月属上一夏正年，后九月记闰九月）。底本维基文库，未对校点校本。\n')
    w = csv.writer(f)
    w.writerow(['卷', '纪年年', '夏正年', '月', '闰', '干支', '朔晦', '原文'])
    w.writerows(rows)
print(len(rows), 'rows;', sum(1 for r in rows if r[6]), '朔晦')
