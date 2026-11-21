"""《资治通鉴目录》（司马光，朔闰据刘羲叟《长历》）元和二年至长庆元年（807—821）朔闰栏转录、展开与检验。

目录每岁只列「前月为小月」之月的朔，未列之月 = 上月朔 + 30 日；闰月列「閏N」而不列干支者同此。
展开以张培瑜表 806 年十二月朔 JDN 起算，逐月取 29 或 30 日中干支相合者。
输出 data/calendar/tongjian_mulu_807_821.csv（逐月：目录朔、张表朔、本项目正元推步朔、本纪明书朔）。
用法（仓库根）：python3 scripts/calendar_data/tongjian_mulu_guanxiang.py
"""
import csv, re, subprocess

GZ = [a + b for a, b in zip('甲乙丙丁戊己庚辛壬癸' * 6, '子丑寅卯辰巳午未申酉戌亥' * 5)]
def gz(j): return GZ[(j + 49) % 60]
NUM = {'正': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9, '十': 10, '十一': 11, '十二': 12}

# 维基共享资源 SSID-12318604《資治通鑑目錄》卷22-24 PDF 第 52—69 页（0 起算），2026-11-21 看图转录
RAW = {
    807: ('正己丑四戊午六丁巳七丙戌九乙酉十一甲申朔', 52),
    808: ('正癸未五壬午七辛巳九庚辰十己酉十二戊申朔', 53),
    809: ('二丁未四丙子六乙亥八甲戌十癸酉十二壬申朔閏三', 54),
    810: ('二辛未四庚午七己亥九戊戌十二丁卯朔', 56),
    811: ('正丙申三乙未五甲午七癸巳十壬戌閏十二辛卯朔', 57),
    812: ('二庚寅三己未五戊午六丁亥九丙辰朔', 58),
    813: ('正乙卯三甲寅四癸未五壬子六辛巳九庚戌朔', 59),
    814: ('正己酉四戊寅五丁未六丙子八乙亥九甲戌十二癸卯朔閏八', 60),
    815: ('三壬申五辛未六庚子八己亥十戊戌朔', 61),
    816: ('正丁卯四丙申六乙未八甲午九癸亥十一壬戌朔', 62),
    817: ('正辛酉四庚寅六己未七戊子九丁亥十一丙戌朔閏五', 63),
    818: ('正乙酉三甲申六癸丑八壬子十辛亥十二庚戌朔', 65),
    819: ('二己酉四戊申六丁未九丙子十一乙亥朔', 67),
    820: ('正甲戌二癸酉四壬申六辛未八庚午十一己亥朔閏正', 68),
    821: ('正戊戌三丁酉五丙申六乙丑八甲子十二癸亥朔', 69),
}

def parse(s):
    """→ ({(月, 闰): 干支}, 闰月)"""
    s = s.replace('朔', '')
    leap = None
    m = re.search(r'閏(十[一二]|[正二三四五六七八九十])$', s)
    if m: leap = NUM[m.group(1)]; s = s[:m.start()]
    out = {}
    for mm in re.finditer(r'(閏)?(十[一二]|[正二三四五六七八九十])(..)', s):
        k = NUM[mm.group(2)]
        if mm.group(1): leap = k; out[(k, True)] = mm.group(3)
        else: out[(k, False)] = mm.group(3)
    return out, leap

def jdn_julian(y, m, d):
    a = (14 - m) // 12; yy = y + 4800 - a; mm = m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083

def main():
    # 张表
    zt = {}
    for d in csv.DictReader(open('vendor/calendar/chinese_calendar/实历朔日表.csv', encoding='utf-8')):
        if 800 <= int(d['lunar_year']) <= 825:
            zt[(int(d['lunar_year']), int(d['lunar_month']), d['is_leap'] == '1')] = jdn_julian(int(d['gregorian_year']), int(d['gregorian_month']), int(d['gregorian_day']))
    prev = zt[(806, 12, False)]
    seq = []
    for y in range(807, 822):
        lst, leap = parse(RAW[y][0])
        order = []
        for m in range(1, 13):
            order.append((m, False))
            if m == leap: order.append((m, True))
        for k in order:
            g = lst.get(k)
            j = prev + 30
            if g is not None:
                j = next(prev + d for d in (29, 30) if gz(prev + d) == g)
            seq.append((y, k[0], k[1], j)); prev = j
    # 本纪
    bl = [l for l in open('data/calendar/benji_shuo_check.csv', encoding='utf-8') if not l.startswith('#')]
    bj = {}
    for r in csv.DictReader(bl):
        if 'tangshu_benji' in r['书'] and 807 <= int(r['年']) <= 821 and r['类'] != '其他':
            bj.setdefault((int(r['年']), r['月']), []).append((r['书'], r['本纪朔'], int(r['本纪朔 JDN'])))
    # 正元推步
    zy = {}
    out = subprocess.run(['cargo', 'run', '-q', '--release', '--manifest-path', 'rust/Cargo.toml', '-p', 'lunar-core', '--no-default-features',
                          '--features', 'real-f64', '--example', 'zhengyuan_guanxiang_dump'], capture_output=True, text=True).stdout
    for l in out.split():
        y, m, lp, j = l.split(','); zy[(int(y), int(m), lp == '1')] = int(j)
    w = csv.writer(open('data/calendar/tongjian_mulu_807_821.csv', 'w', encoding='utf-8', newline=''))
    w.writerow(['# 《资治通鉴目录》（朔闰据刘羲叟《长历》）元和二年至长庆元年朔闰栏转录与展开：维基共享资源 SSID-12318604《資治通鑑目錄》卷22-24 PDF（2026-11-21 看图转录）；'
                '目录每岁只列前月为小月之月，未列者 = 上月 + 30 日。对照张培瑜表、本项目正元历推步（代推观象）与新旧唐书本纪明书朔。由 scripts/calendar_data/tongjian_mulu_guanxiang.py 生成'])
    w.writerow(['年', '月', '闰', '目录原文（该岁）', 'PDF页', '目录朔', '张表朔', '正元推步朔', '本纪明书朔', '目录合本纪', '张表合本纪', '正元合本纪'])
    score = {'目录': 0, '张表': 0, '正元': 0, 'n': 0}
    for (y, m, lp, j) in seq:
        zj = zt.get((y, m, lp)); pj = zy.get((y, m, lp))
        b = bj.get((y, str(m) if not lp else '闰' + str(m)), [])
        bs = ' '.join(f'{x[1]}（{x[0].split("_")[0]}）' for x in b)
        ok = lambda v: ('' if not b else ('1' if all(v == x[2] for x in b) else '0'))
        for x in b:
            score['n'] += 1; score['目录'] += j == x[2]; score['张表'] += zj == x[2]; score['正元'] += pj == x[2]
        w.writerow([y, m, '闰' if lp else '', RAW[y][0] if (m == 1 and not lp) else '', RAW[y][1] + 1 if (m == 1 and not lp) else '',
                    gz(j), gz(zj) if zj else '', gz(pj) if pj else '', bs, ok(j), ok(zj), ok(pj)])
    diff = [(y, ('闰' if lp else '') + str(m), gz(j), gz(zt[(y, m, lp)])) for (y, m, lp, j) in seq if zt.get((y, m, lp)) != j]
    print('本纪明书朔', score, '\n目录与张表不同：', diff)

if __name__ == '__main__':
    main()
