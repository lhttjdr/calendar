"""时宪书实物月表（data/calendar/shixianshu_months.csv）与推步、张表逐月比较 → data/calendar/shixianshu_check.csv。
推步：1645—1725 新法（shixian_xinfa_check.csv 用时）、1726—1741 考成（shixian_kaocheng_check.csv 用时）、1742—1913 后编（shixian_houbian_check.csv 用时）；
1666—1668 大统不在本表（无实物）。中气交时（京师本）与后编用时推算比较分钟差（scripts/calendar_data/shixian_houbian.py）。"""
import csv, math, sys
sys.path.insert(0, 'scripts/calendar_data')
G = "甲乙丙丁戊己庚辛壬癸"; Z = "子丑寅卯辰巳午未申酉戌亥"
GZ = [G[i % 10] + Z[i % 12] for i in range(60)]


def jdn(s):
    y, m, d = map(int, s.split('-'))
    a = (14 - m) // 12; yy = y + 4800 - a; mm = m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045


def load_reck(path, col):
    out = {}
    for l in open(path, encoding='utf-8'):
        if l.startswith('#'):
            continue
        f = l.strip().split(',')
        if not f[0].isdigit():
            continue
        sy, m, leap = int(f[0]), int(f[1]), int(f[2])
        if path.endswith('xinfa_check.csv'):
            # 冬至岁,农历年,月,闰,用时朔日,平时朔日,...
            ly, m, leap, d = int(f[1]), int(f[2]), int(f[3]), f[4]
        else:
            ly = sy if m <= 10 else sy - 1
            d = f[col]
        out[(ly, m, leap)] = jdn(d)
    return out


R = {}
R.update(load_reck('data/calendar/shixian_xinfa_check.csv', 4))
for k, v in load_reck('data/calendar/shixian_kaocheng_check.csv', 3).items():
    if 1726 <= k[0] <= 1741:
        R[k] = v
for k, v in load_reck('data/calendar/shixian_houbian_check.csv', 3).items():
    if k[0] >= 1742:
        R[k] = v
ZH = {}
for r in csv.DictReader(open('vendor/calendar/chinese_calendar/实历朔日表.csv', encoding='utf-8')):
    if int(r['gregorian_year']) < 1600:
        continue
    ZH[(int(r['lunar_year']), int(r['lunar_month']), int(r['is_leap']))] = jdn(f"{r['gregorian_year']}-{r['gregorian_month']}-{r['gregorian_day']}")

NUM = {"初": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10}


def cn_num(s):
    if s in ("", "初"):
        return 0
    if s.startswith("十"):
        return 10 + (NUM[s[1]] if len(s) > 1 else 0)
    if "十" in s:
        a, b = s.split("十")
        return NUM[a] * 10 + (NUM[b] if b else 0)
    return NUM[s]


def parse_time(s):
    """「亥初一刻六分」→ 自本日子正起之分钟（夜子初为 23 时）；含 □ 者返回 None"""
    if not s or "□" in s:
        return None
    s = s.replace("夜", "")
    zhi = Z.index(s[0]); half = s[1]
    rest = s[2:]
    ke = rest.split("刻")[0]; fen = rest.split("刻")[1].replace("分", "")
    h = zhi * 2 + (0 if half == "正" else -1)
    if h < 0:
        h = 23
    return h * 60 + cn_num(ke) * 15 + cn_num(fen)


QI = ["冬至", "小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋", "处暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪"]
_qcache = {}


def qi_minutes(ly, name):
    """后编用时推算：农历年 ly 之中气 name 的 (日干支, 自子正起分钟)"""
    import shixian_houbian as hb
    k = QI.index(name)
    sy = ly + 1 if k in (0, 2) else ly           # 冬至、大寒在农历年末，属次冬至岁
    key = (sy,)
    if key not in _qcache:
        _qcache[key] = hb.jieqi_times(sy)
    t = _qcache[key][k]
    u = t + hb.shi_cha(t)
    j = hb.jdn_of(math.floor(u))
    return GZ[(j + 49) % 60], round((u % 1) * 1440)


HS = {}
for r in csv.DictReader(l for l in open('data/calendar/shixianshu_heshuo.csv', encoding='utf-8') if not l.startswith('#')):
    HS[(int(r['年']), int(r['月']), int(r['闰']))] = r['合朔时刻']


def shuo_minutes(jd):
    """后编用时合朔：落在 JDN jd 当日之合朔，返回自子正起分钟"""
    import shixian_houbian as hb
    d0 = hb.daynum_of(jd)
    for t in hb.shuo_times(d0 - 2, d0 + 2):
        u = t + hb.shi_cha(t)
        if math.floor(u) == d0:
            return round((u % 1) * 1440)
    return None


rows = []
for r in csv.DictReader(l for l in open('data/calendar/shixianshu_months.csv', encoding='utf-8') if not l.startswith('#')):
    y, m, leap = int(r['年']), int(r['月']), int(r['闰'])
    rj = R.get((y, m, leap)); zj = ZH.get((y, m, leap))
    g = lambda j: GZ[(j + 49) % 60] if j else ""
    shu = r['朔干支']
    note = r['备注']
    qd = ""
    if y >= 1742 and r['中气'] and "朝鲜" not in note and "存疑" not in note:
        bm = parse_time(r['中气交时'])
        if bm is not None:
            gz, cm = qi_minutes(y, r['中气'])
            if gz == r['中气日干支']:
                qd = str(bm - cm)
            else:
                qd = f"日异（书{r['中气日干支']}，推{gz}）"
    sd = ""
    hs = HS.get((y, m, leap))
    if hs and y >= 1742 and rj and shu == g(rj):
        bm, cm = parse_time(hs), shuo_minutes(rj)
        if bm is not None and cm is not None:
            sd = str(bm - cm)
    rows.append([y, m, leap, shu, g(rj), g(zj), int(shu == g(rj)), int(shu == g(zj)), qd, sd, r['来源'], note])

with open('data/calendar/shixianshu_check.csv', 'w', encoding='utf-8', newline='') as fh:
    fh.write("# 时宪书实物月表与推步、张表逐月比较（scripts/calendar_data/shixianshu_check.py 生成）。推步：1645—1725 新法、1726—1741 考成、1742—1913 后编，皆用时定日。"
             "中气分差 = 书载交时 − 后编用时推算（分；只算京师本、1742 年以后、非存疑）；合朔分差 = 书载合朔时刻（shixianshu_heshuo.csv）− 后编用时合朔（分）。\n")
    w = csv.writer(fh)
    w.writerow(["年", "月", "闰", "实物朔", "推步朔", "张表朔", "合推步", "合张表", "中气分差", "合朔分差", "来源", "备注"])
    w.writerows(rows)
n = len(rows)
print(n, "月；合推步", sum(r[6] for r in rows), "合张表", sum(r[7] for r in rows))
for r in rows:
    if not (r[6] and r[7]):
        print("不合", r)
d = [int(r[8]) for r in rows if r[8].lstrip('-').isdigit()]
from collections import Counter
print("中气分差分布", sorted(Counter(d).items()))
print("合朔分差分布", sorted(Counter(int(r[9]) for r in rows if r[9]).items()))
