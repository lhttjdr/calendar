"""《汉书·五行志下之下》春秋日食的汉人说解：刘歆改订的月日与分野、董仲舒的「宿在某」，与三统历推算逐条对照。

- 刘歆：按三统历（周正，冬至所在月为正月；无中气之月为闰）求记载之日在三统历中的月、日，与其「某月二日」等比较；
  再按《汉书·律历志》十二次度数（自牵牛初起）求三统历日躔所在之次，与其所言分野（两国）比较。
- 董仲舒：三统历日躔所在之宿与其「宿在某」比较（真太阳位置另由 rust 检验 sky_culture::tests_huangdao 给出）。
日期：data/phenomena/chunqiu_solar_eclipses.csv 的 jdn（按春秋历换算的记载之日；经文无日干支者取该月朔）。

输出 data/phenomena/chunqiu_han_commentary.csv（说解转录）与 data/phenomena/checks/chunqiu_han_commentary.csv（逐条对照）。
用法（仓库根）：python3 scripts/calendar_data/qinhan_eval/chunqiu_han_commentary.py <缓存目录>
缓存目录下缺《漢書_卷027.txt》时自动从维基文库下载（action=raw）。
"""
import csv
import re
import sys
import urllib.parse
import urllib.request
from fractions import Fraction as F
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent))
import han_eclipse_sun_by_calendar as h  # noqa: E402

CACHE = Path(sys.argv[1] if len(sys.argv) > 1 else 'hanshu_cache'); CACHE.mkdir(parents=True, exist_ok=True)
SRC = CACHE / '漢書_卷027.txt'
if not SRC.exists():
    url = 'https://zh.wikisource.org/w/index.php?title=' + urllib.parse.quote('漢書/卷027') + '&action=raw'
    req = urllib.request.Request(url, headers={'User-Agent': 'lunar-calendar-research/0.1'})
    SRC.write_bytes(urllib.request.urlopen(req, timeout=60).read())

GONG = '隱桓嚴莊閔僖文宣成襄昭定哀'
CN = {'元': 1, '一': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9, '十': 10}


def cnnum(s):
    if s in ('正',):
        return 1
    if s == '十':
        return 10
    if s.startswith('十'):
        return 10 + CN[s[1]]
    if '十' in s:
        a, b = s.split('十')
        return CN[a] * 10 + (CN[b] if b else 0)
    return CN[s]


# 十二次（《汉书·律历志》，起度自牵牛初；斗宿 26+385/1539 度）与分野（《汉书·地理志》）
CI = [('星紀', '斗', 12, '越'), ('玄枵', '女', 8, '齊'), ('諏訾', '危', 16, '衛'), ('降婁', '奎', 5, '魯'), ('大梁', '胃', 7, '趙'),
      ('實沈', '畢', 12, '晉'), ('鶉首', '井', 16, '秦'), ('鶉火', '柳', 9, '周'), ('鶉尾', '張', 18, '楚'), ('壽星', '軫', 12, '鄭'),
      ('大火', '氐', 5, '宋'), ('析木', '尾', 10, '燕')]
SIMP = str.maketrans('畢張軫', '毕张轸')
DOU = h.CALENDARS['三统'][1] - 365
TOT = h.CALENDARS['三统'][1]


def ci_of(x):
    """三统历日躔（自牵牛初起的度数）所在之次及距该次起点、终点的度数。"""
    starts = [(n, (h.lodge_start(l.translate(SIMP), DOU) + d) % TOT, st) for n, l, d, st in CI]
    starts.sort(key=lambda t: t[1])
    for i, (n, s0, st) in enumerate(starts):
        s1 = starts[(i + 1) % 12][1]
        if (x - s0) % TOT < (s1 - s0) % TOT:
            return n, st, float((x - s0) % TOT), float((s1 - x) % TOT), starts[(i + 1) % 12][2], starts[i - 1][2]


def santong_month_day(jdn):
    """三统历（周正）中 jdn 所在的月、闰与日。"""
    E, yr, mon, _ = h.CALENDARS['三统']
    k = (F(jdn) - E) // mon
    while int(E + (k + 1) * mon) <= jdn:
        k += 1
    while int(E + k * mon) > jdn:
        k -= 1
    s0, s1 = E + k * mon, E + (k + 1) * mon
    zq = yr / 12  # 中气间隔；冬至为中气 0

    def zhongqi_in(a, b):
        n = -((E - a) // zq)  # 第一个 >= a 的中气序号
        t = E + n * zq
        return n if t < b else None
    n = zhongqi_in(s0, s1)
    leap = n is None
    if leap:
        n = zhongqi_in(E + (k - 1) * mon, s0)
    month = n % 12 + 1  # 冬至所在月为正月（周正）
    return month, leap, jdn - int(s0) + 1


def main():
    t = re.sub(r'\{\{YL\|([^}|]*)[^}]*\}\}', r'\1', SRC.read_text(encoding='utf-8'))
    sec = t[t.index('====春秋===='):t.index('====漢====')]
    items = re.split(r'\n(?=[^\n]{0,8}年[^\n]{0,20}日有食之)', sec)
    with open(ROOT / 'data/phenomena/chunqiu_solar_eclipses.csv', encoding='utf-8') as f:
        base = list(csv.DictReader(line for line in f if not line.startswith('#')))
    out, gong = [], None
    for it in items:
        s = it.replace('\n', ' ').strip()
        m = re.match(r'(?:(\S)公)?([元一二三四五六七八九十]+)年「(?:[^」]*?)?([正一二三四五六七八九十]+)月', s)
        if not m or '日有食之' not in s[:40]:
            continue
        if m.group(1):
            gong = m.group(1)
        g = {'嚴': '莊'}.get(gong, gong) + '公'
        yr, mo = cnnum(m.group(2)), cnnum(m.group(3))
        rows = [r for r in base if r['emp'] == g and int(r['year_in_era']) == yr and int(r['month']) == mo]
        if not rows:
            print('未对上', s[:30]); continue
        r = rows[0]
        dong = re.search(r'董仲舒以為(?:時)?宿在([^，。、]+)', s)
        xin = re.search(r'劉歆以為(前年)?((?:正|十二|十一|十|[一二三四五六七八九])月)?(二日|朔|晦朓|晦)?，?([^，。、]{1,2})(?:、|與)([^，。、之]{1,2})之?分', s)
        out.append({'公': g, '年': yr, '月': mo, '经文': re.match(r'[^」]*」', s).group(0) if '」' in s else s[:30],
                    'jdn': r['jdn'], '董仲舒宿': dong.group(1) if dong else '',
                    '刘歆月日': ''.join(x or '' for x in xin.group(1, 2, 3)) if xin else '',
                    '刘歆分野': (xin.group(4) + '、' + xin.group(5)) if xin else ''})
    with open(ROOT / 'data/phenomena/chunqiu_han_commentary.csv', 'w', encoding='utf-8', newline='') as f:
        f.write('# 《汉书·五行志下之下》春秋日食条下董仲舒、刘歆说解（维基文库 漢書/卷027 解析）；jdn 取自 chunqiu_solar_eclipses.csv\n')
        w = csv.DictWriter(f, fieldnames=list(out[0].keys())); w.writeheader(); w.writerows(out)
    rows = []
    E, yr, mon, _ = h.CALENDARS['三统']
    CNM = {'正': 1, '十二': 12, '十一': 11, '十': 10, '一': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9}
    DONG = {'東壁': '壁', '畢': '毕', '軫': '轸'}
    n_xin = n_in = n_pair = 0
    for o in out:
        b = next(x for x in base if x['emp'] == o['公'] and x['year_in_era'] == str(o['年']) and x['month'] == str(o['月']))
        Y = int(b['year'])
        # 刘歆所给月日 → 三统历日子（周正：含冬至之月为正月，无中气为闰）
        m = re.match(r'(前年)?((?:正|十二|十一|十|[一二三四五六七八九]))?月?(二日|朔|晦朓|晦)?', o['刘歆月日'])
        day, basis = int(b['jdn']), '记载之日'
        if m and m.group(2):
            mo = CNM[m.group(2)] - (12 if m.group(1) else 0)
            sm = santong_months(Y)
            hit = [x for x in sm if x[0] == mo and not x[1]]
            if hit:
                s0, e0 = hit[0][2], hit[0][3]
                d = m.group(3) or ''
                day = s0 + 1 if d == '二日' else (e0 - 1 if d.startswith('晦') else s0)
                basis = '刘歆月日'
        x = h.sun_degree('三统', day)
        n, st, d0, d1, nxt, prv = ci_of(x)
        near = nxt if d1 < d0 else prv
        pair = set(o['刘歆分野'].split('、')) if o['刘歆分野'] else set()
        if pair:
            n_xin += 1; n_in += st in pair; n_pair += {st, near} == pair
        # 董仲舒：使三统历推算落入其宿所需的冬至点平移区间
        dong = ''
        if o['董仲舒宿']:
            l = DONG.get(o['董仲舒宿'], o['董仲舒宿'])
            xr = h.sun_degree('三统', int(b['jdn']))
            lo = float((h.lodge_start(l, DOU) - xr + TOT / 2) % TOT - TOT / 2)
            dong = f'[{lo:+.1f}, {lo + float(h.WIDTHS[l] + (DOU if l == "斗" else 0)):+.1f}]'
        rows.append([o['公'], o['年'], o['月'], o['经文'][:24], o['刘歆月日'], o['刘歆分野'], basis, f'{n}（{st}）', f'{d0:.1f}', f'{d1:.1f}', near,
                     ('是' if st in pair else '否') if pair else '', ('是' if {st, near} == pair else '否') if pair else '', o['董仲舒宿'], dong])
    with open(ROOT / 'data/phenomena/checks/chunqiu_han_commentary.csv', 'w', encoding='utf-8', newline='') as f:
        f.write('# 春秋日食汉人说解与三统历（scripts/calendar_data/qinhan_eval/chunqiu_han_commentary.py）：刘歆所给月日在三统历中之日、该日日躔所在之次与分野；'
                '董仲舒之宿所需冬至点平移区间（度，负为西移）\n')
        f.write(f'# 刘歆分野含三统日躔所在之次 {n_in}/{n_xin}，恰为所在次与较近邻次 {n_pair}/{n_xin}\n')
        w = csv.writer(f)
        w.writerow(['公', '年', '月', '经文', '刘歆月日', '刘歆分野', '取日', '三统日躔之次', '距次首°', '距次尾°', '较近邻次', '分野含所在次', '分野=所在次+近邻次', '董仲舒宿', '董宿所需冬至点平移'])
        w.writerows(rows)
    print(f'刘歆分野含三统日躔所在之次 {n_in}/{n_xin}，恰为所在次与较近邻次 {n_pair}/{n_xin}')


def santong_months(Y):
    """三统历天文年 Y 的周正各月 [(月号, 闰, 朔日 JDN, 次月朔 JDN)]：含冬至之月为正月，无中气之月为闰。"""
    E, yr, mon, _ = h.CALENDARS['三统']
    w = E + (Y - (-103)) * yr
    m = (w - E) // mon
    zq = yr / 12
    out = []
    for i in range(-14, 15):
        s, e = E + (m + i) * mon, E + (m + i + 1) * mon
        n = -((E - s) // zq)
        out.append([None, not (E + n * zq < e), int(s), int(e)])
    i0 = 14
    out[i0][0], out[i0][1] = 1, False
    num = 1
    for i in range(i0 + 1, len(out)):
        num += not out[i][1]
        out[i][0] = num
    num = 1
    for i in range(i0 - 1, -1, -1):
        num -= not out[i + 1][1]
        out[i][0] = num
    return out


if __name__ == '__main__':
    main()
