"""琉球推步手册（《琉球王國漢文文獻集成》第 23 册《推朔望法》《求節氣》《四行立成》、第 24 册三种立成）常数、根数与立成抽格，
与本项目清代三法复原逐项比较 → doc/calendar/ryukyu-tuibu-checks.md。

三法：
- 考成：《历象考成》康熙甲子元（scripts/calendar_data/shixian_kaocheng.py，Rust rust/core/src/calendar/shixian_kaocheng.rs）；
- 后编：《历象考成后编》雍正癸卯元（shixian_houbian.py，Rust shixian.rs）；
- 新法：汤若望西洋新法天聪戊辰元（shixian_xinfa.py，Rust shixian_xinfa.rs）。
三家 Python 与 Rust 常数相同（Rust 由 Python 移植），此处调 Python 实现。

比较法：
- 「每月」「十二朔」「十三朔」诸策 = 各家日行 × 朔策；朔策先用各家自己的（日行之差所定），再用书载 29 日 12 时 44 分 3 秒各算一次，
  书数取到秒，故 |差| ≤ 0.5″ 为合（书末位舍入）、≤ 1″ 为「末位差一」。
- 根数（1740 年正月首朔时刻之太阳引数、太阴引数、太阳经度、交周）：在书载首朔时刻（乙丑起甲子日序 39 日 02:03:48，即 1740-01-29）
  算各家平行，相减。
- 气应、閏應、首朔：以各家天正冬至与平朔（平太阳、平太阴同经）算，京师子正起日。
- 首朔立成：以书的递推（加十二或十三朔实，满六十去之）检书内各年是否自洽，并以閏應与閏限判闰，与张表（实历朔日表）该年有无闰月对照。
- 立成：时分立成以各家日行乘时；均度立成以各家日躔、月离均数函数；黄赤道差以各家黄赤大距求升度时差。

用法（仓库根）：uv run --project scripts python -I scripts/calendar_data/ryukyu_tuibu_check.py
"""
import csv
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import shixian_houbian as hb  # noqa: E402
import shixian_kaocheng as kc  # noqa: E402
import shixian_xinfa as xf  # noqa: E402

GZ = kc.GZ
SRC_C = ROOT / "data/calendar/ryukyu_tuibu_changshu.csv"
SRC_L = ROOT / "data/calendar/ryukyu_tuibu_licheng.csv"
OUT = ROOT / "doc/calendar/ryukyu-tuibu-checks.md"
BOOK_SHUOCE = 29 + (12 + 44 / 60 + 3 / 3600) / 24
J1740 = 2356610                      # 1740-01-29 癸卯（书首朔 39 日）
T1740 = J1740 + (2 + 3 / 60 + 48 / 3600) / 24


def rows(p):
    return list(csv.DictReader(l for l in open(p, encoding="utf-8") if not l.startswith("#")))


# ---------------------------------------------------------------- 三家日行（角秒／日）与平行
RATES = {
    "考成": dict(sun=kc.SUN_DAILY * 3600, moon=kc.MOON_DAILY * 3600, gao=kc.BEI_DAILY * 3600,
               jiao=kc.JIAO_DAILY * 3600, bei=kc.ZUIBEI_DAY * 3600, sui=kc.ZHOU_SUI),
    "后编": dict(sun=hb.SUN_DAILY * 3600, moon=hb.MOON_DAILY * 3600, gao=hb.GAO_DAILY * 3600,
               jiao=hb.JIAO_DAILY * 3600, bei=hb.ZUIBEI_DAY * 3600, sui=hb.ZHOU_SUI),
    "新法": dict(sun=xf.SUN_DAILY * 3600, moon=xf.MOON_DAILY * 3600, gao=xf.BEI_DAILY * 3600,
               jiao=xf.JIAO_DAILY * 3600, bei=xf.ZUIGAO_YEAR * 3600 / xf.SUI, sui=xf.SUI),
}
for r in RATES.values():
    r["shuo"] = 1296000 / (r["moon"] - r["sun"])
    # 新法太阴引数用自行日行（与月孛日行定义的引数日行相差在微以下）
RATES["新法"]["yin"] = xf.MOON_ZIXING * 3600
RATES["考成"]["yin"] = RATES["考成"]["moon"] - RATES["考成"]["gao"]
RATES["后编"]["yin"] = RATES["后编"]["moon"] - RATES["后编"]["gao"]


def mean_at(name, t):
    """t（JDN + 日内，京师子正起）时各家平行（度）：太阳平行、太阳引数、太阴平行、太阴引数、交周"""
    if name == "考成":
        tt = kc.daynum_of(math.floor(t)) + (t % 1)
        n = tt - (math.floor(kc.dongzhi(kc.YUAN_YEAR)[0]) + 1)
        sp, syin, _, _ = kc.sun(tt)
        mo = kc.MOON_YING + kc.MOON_DAILY * n
        return sp, syin, mo, mo - (kc.BEI_YING + kc.BEI_DAILY * n), mo - (kc.JIAO_YING - kc.JIAO_DAILY * n)
    if name == "后编":
        tt = hb.daynum_of(math.floor(t)) + (t % 1)
        dz0, _ = hb.dongzhi(hb.YUAN_YEAR)
        n = tt - (math.floor(dz0) + 1)
        sp, syin, _, _, _ = hb.sun(tt, hb.sui_of(tt))
        mo = hb.MOON_YING + hb.MOON_DAILY * n
        return sp, syin, mo, mo - (hb.GAO_YING + hb.GAO_DAILY * n), mo - (hb.JIAO_YING - hb.JIAO_DAILY * n)
    n = t - xf.ROOT_JDN
    sp = xf.ROOT_L + xf.SUN_DAILY * n
    chong = xf.ROOT_GAO + xf.ZUIGAO_YEAR * n / xf.SUI
    ml, mz = xf.moon_mean(t)
    return sp, sp - chong, ml, mz, ml - (xf.ROOT_JIAO - xf.JIAO_DAILY * n) + 180   # 新法交周自中交起，加半周与考成同起点


def mean_conj(name, t0):
    """t0 附近平朔（平太阳、平太阴同经）"""
    t = t0
    for _ in range(30):
        sp, _, mo, _, _ = mean_at(name, t)
        d = (mo - sp + 180) % 360 - 180
        t -= d / 12.19
    return t


def dongzhi_t(name, y):
    """冬至岁 y 之天正冬至（平气，JDN + 日内）"""
    if name == "考成":
        d, _ = kc.dongzhi(y)
        return kc.jdn_of(math.floor(d)) + d % 1
    if name == "后编":
        d, _ = hb.dongzhi(y)
        return hb.jdn_of(math.floor(d)) + d % 1
    return xf.dongzhi(y)


def cyc(t):
    """JDN + 日内 → 甲子起日序 + 日内（0—60）"""
    j = math.floor(t)
    return (j + 49) % 60 + (t - j)


def dms(x, unit="度"):
    s = "-" if x < 0 else ""
    x = abs(x)
    if unit == "日":
        d = int(x)
        h = (x - d) * 24
        m = (h - int(h)) * 60
        return f"{s}{d}日{int(h):02d}:{int(m):02d}:{(m - int(m)) * 60:05.2f}"
    g = int(x // 30)
    d = x - g * 30
    m = (d - int(d)) * 60
    return f"{s}{g}宫{int(d)}°{int(m):02d}′{(m - int(m)) * 60:05.2f}″"


def arcsec_diff(a, b):
    return ((a - b + 180) % 360 - 180) * 3600


# ---------------------------------------------------------------- 常数
def changshu():
    C = {r["条目"]: r for r in rows(SRC_C)}
    out = []

    def add(item, book, vals, unit, note=""):
        cells = []
        for nm in ("考成", "后编", "新法"):
            v = vals.get(nm)
            if v is None:
                cells += ["", ""]
                continue
            if unit == "日":
                diff = (book - v) * 86400
                cells += [dms(v, "日"), f"{diff:+.1f}秒"]
            else:
                diff = arcsec_diff(book, v)
                cells += [dms(v), f"{diff:+.2f}″"]
        out.append([item, dms(book, unit) if unit else str(book)] + cells + [note])

    def per(key, n, shuo):
        return {nm: (RATES[nm][key] * n * (RATES[nm]["shuo"] if shuo is None else shuo) / 3600) % 360 for nm in RATES}

    rates_items = [("太陽引數月策", "sun_yin", 1), ("十二朔太陽引數", "sun_yin", 12), ("十三朔太陽引數", "sun_yin", 13),
                   ("太陰引數月策", "yin", 1), ("十二朔太陰引數", "yin", 12), ("十三朔太陰引數", "yin", 13),
                   ("太陽經度月策", "sun", 1), ("十二朔太陽經度", "sun", 12), ("十三朔太陽經度", "sun", 13),
                   ("朔交策", "jiaozhou", 1), ("十二朔交周", "jiaozhou", 12), ("十三朔交周", "jiaozhou", 13)]
    for r in RATES.values():
        r["sun_yin"] = r["sun"] - r["bei"]
        r["jiaozhou"] = r["moon"] + r["jiao"]
    for item, key, n in rates_items:
        b = float(C[item]["值"])
        add(item + "（各家朔策）", b, per(key, n, None), "度")
        add(item + "（书朔策）", b, per(key, n, BOOK_SHUOCE), "度")
    add("朔策", float(C["朔策"]["值"]), {nm: RATES[nm]["shuo"] for nm in RATES}, "日")
    add("十二朔實", float(C["十二朔實"]["值"]), {nm: 12 * RATES[nm]["shuo"] for nm in RATES}, "日")
    add("十三朔", float(C["十三朔"]["值"]), {nm: 13 * RATES[nm]["shuo"] for nm in RATES}, "日")
    add("歲實", float(C["歲實"]["值"]), {nm: RATES[nm]["sui"] for nm in RATES}, "日")
    add("閏餘", float(C["閏餘"]["值"]), {nm: RATES[nm]["sui"] - 12 * RATES[nm]["shuo"] for nm in RATES}, "日")
    # 根数：1740 年
    dz = {nm: dongzhi_t(nm, 1740) for nm in RATES}
    add("庚申氣應（天正冬至，甲子起日）", float(C["庚申氣應"]["值"]), {nm: cyc(dz[nm]) for nm in RATES}, "日")
    sh = {nm: mean_conj(nm, T1740) for nm in RATES}
    add("庚申首朔應（正月平朔，甲子起日）", float(C["庚申首朔應"]["值"]), {nm: cyc(sh[nm]) for nm in RATES}, "日")
    tz = {nm: mean_conj(nm, T1740 - 59.06) for nm in RATES}
    add("庚申閏應（冬至距天正经朔）", float(C["庚申閏應"]["值"]), {nm: dz[nm] - tz[nm] for nm in RATES}, "日")
    roots = [("太陽經度根", 0), ("太陽引數根", 1), ("太陰引數根", 3), ("首朔交應", 4)]
    for item, k in roots:
        b = float(C[item]["值"])
        add(item + "（书首朔时刻）", b, {nm: mean_at(nm, T1740)[k] % 360 for nm in RATES}, "度")
        add(item + "（各家平朔时刻）", b, {nm: mean_at(nm, sh[nm])[k] % 360 for nm in RATES}, "度")
    # 求節氣：平行加法
    for item, n in (("平行十四日", 14), ("平行十五日", 15), ("平行十六日", 16)):
        add(item, float(C[item]["值"]), {nm: RATES[nm]["sun"] * n / 3600 for nm in RATES}, "度")
    return out, C


# ---------------------------------------------------------------- 首朔立成
def shoushuo():
    """书递推：首朔(y+1) = 首朔(y) + (13 朔 若 y 有闰 否则 12 朔实)，閏應(y+1) = 閏應(y) + 閏餘（满朔策去之）。
    书术文以閏應 ≥ 閏限为有闰，但书中 1756、1757 两年实按当年颁历之闰（1756 闰九月）递推，故递推用张表闰月，
    「按閏限」列另给术文规则之判。以书载 1740 根起算，逐年与书内所录各年比较。"""
    L = [r for r in rows(SRC_L) if r["表"] == "首朔立成"]
    book = {int(r["引数"]): r for r in L}
    run_ying = 21 + (4 + 39 / 60 + 27 / 3600) / 24
    run_yu = 10 + (21 + 0 / 60 + 8 / 3600) / 24
    run_xian = 18 + (15 + 43 / 60 + 55 / 3600) / 24
    shuo = BOOK_SHUOCE
    s12 = 354 + (8 + 48 / 60 + 38 / 3600) / 24
    s13 = 383 + (21 + 32 / 60 + 41 / 3600) / 24
    first = 39 + (2 + 3 / 60 + 48 / 3600) / 24
    zh_leap = {}
    for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        if r["is_leap"] == "1":
            zh_leap[int(r["lunar_year"])] = int(r["lunar_month"])
    out = []
    y, ying, fs = 1740, run_ying, first
    while y <= 1771:
        leap = ying >= run_xian
        if y in book:
            r = book[y]
            bv = float(r["值"])
            d = ((bv - fs + 30) % 60 - 30) * 86400
            out.append([str(y), GZ[(y - 4) % 60], r["原文"], dms(fs, "日"), f"{d:+.0f}秒",
                        "閏" if leap else "", (f"閏{zh_leap[y]}月" if y in zh_leap else ""),
                        "合" if (leap == (y in zh_leap)) else "不合", r["备注"]])
        fs = (fs + (s13 if y in zh_leap else s12)) % 60
        ying = ying + run_yu
        if ying >= shuo:
            ying -= shuo
        y += 1
    return out


# ---------------------------------------------------------------- 立成
def licheng():
    L = [r for r in rows(SRC_L) if r["表"] != "首朔立成"]
    out = []
    for r in L:
        tab, arg = r["表"], r["引数"]
        b = float(r["值"]) if r["值"] else None
        calc = {}
        if tab.startswith("四行立成"):
            hours = float(arg[:-1]) if arg.endswith("時") else float(arg[:-1]) / 60
            for nm in RATES:
                rate = (RATES[nm]["moon"] - RATES[nm]["sun"]) if "月距日" in tab else RATES[nm]["sun"]
                calc[nm] = rate * hours / 24 / 60          # 值以角分计
        elif tab in LICHENG_FUNCS:
            calc = LICHENG_FUNCS[tab](arg)
        cells = []
        for nm in ("考成", "后编", "新法"):
            v = calc.get(nm)
            if v is None or b is None:
                cells += ["", ""]
                continue
            cells += [f"{v:.5f}", f"{(b - v) * UNIT_SEC.get(tab.split('·')[0], 60):+.3f}"]
        out.append([r["册"], tab, arg, r["原文"], "" if b is None else f"{b:.5f}"] + cells + [f"PDF {r['PDF页']}／书页 {r['书页']}", r["备注"]])
    return out


UNIT_SEC = {"四行立成": 60, "太陽均度立成": 3600, "太陰均度立成": 3600, "黃赤道差加減時分立成": 60}
# 四行立成值以角分计 → 差 ×60 为角秒；均度立成值以度计 → ×3600 为角秒；黄赤道差值以时分计 → ×60 为时秒


def gongdu(arg):
    """「3宫15°」或「3宫15°30′」→ 度"""
    g, rest = arg.split("宫")
    d, _, m = rest.partition("°")
    return int(g) * 30 + float(d or 0) + (float(m.rstrip("′")) / 60 if m else 0)


def sun_jun_all(arg):
    """太阳均数绝对值（度），引数自最卑（考成、后编）／最高冲（新法）起，两者同指近地点"""
    a = gongdu(arg)
    return {"考成": abs(kc.sun_jun(a)), "后编": abs(hb.sun_jun(a)),
            "新法": abs(xf.sun_jun_from_gao(180 - xf.fold(a)))}


def moon_jun_all(arg):
    """太阴（初）均数绝对值（度），引数自最高起。考成：本轮均轮法初均（单引数）；新法：卷三十三太阴自行加减表第一均数；
    后编初均随本时两心差变，无单引数之表，取两心差中数 550505（最高本轮半径）一算作参考。"""
    a = gongdu(arg)
    m = a % 360
    h = kc.MOON_JUNLUN
    kcv = math.degrees(math.atan2(3 * h * abs(math.sin(math.radians(m))), kc.R + h * math.cos(math.radians(m))))
    e = hb.GAO_BENLUN
    mm = hb.fold(m)
    c = 180 - mm
    beta, _ = hb.tri_opp(1e7, e, c)
    q_ang, _ = hb.tri_opp(1e7, e, c + beta)
    py = 180 - (c + beta) - q_ang
    hbv = mm - hb.atan2(math.sqrt(1 - (e / 1e7) ** 2) * hb.sin(py), hb.cos(py))
    return {"考成": kcv, "后编": abs(hbv), "新法": abs(xf.moon_jun1(a))}


def shengdu_all(arg):
    """黄赤升度差变时（时分）：黄道宫度 λ（自冬至起）与其赤经之差 ×4 分／度，取绝对值"""
    lam = (gongdu(arg) + 270) % 360
    out = {}
    for nm, eps in (("考成", kc.HUANG_CHI), ("后编", hb.HUANG_CHI), ("新法", xf.HUANG_CHI)):
        ra = math.degrees(math.atan2(math.cos(math.radians(eps)) * math.sin(math.radians(lam)), math.cos(math.radians(lam)))) % 360
        out[nm] = abs((lam - ra + 180) % 360 - 180) * 4
    return out


LICHENG_FUNCS = {"太陽均度立成": sun_jun_all, "太陰均度立成": moon_jun_all, "黃赤道差加減時分立成": shengdu_all}


QI = ["冬至", "小寒", "大寒", "立春", "雨水", "驚蟄", "春分", "清明", "穀雨", "立夏", "小滿", "芒種", "夏至", "小暑", "大暑", "立秋",
      "處暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪"]
CNUM = {"一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10, "〇": 0}


def cn(s):
    s = s.replace("〇", "")
    if not s:
        return 0
    if "十" in s:
        a, _, b = s.partition("十")
        return (CNUM[a] if a else 1) * 10 + (CNUM[b] if b else 0)
    return CNUM[s]


def xinfa_qi_times(y0=2356572.0):
    """新法定气（平时）：1739 年冬至起 25 气"""
    out, t = [], y0
    for k in range(25):
        for _ in range(60):
            d = (xf.sun(t)[2] - 15 * k + 180) % 360 - 180
            t -= d / 0.9856
        out.append(t)
        t += 15.2
    return out


def jieqi_extra(C):
    """求節氣：距春分差立成（新法「日差」自春分起算）与日率（定气相距）"""
    import re
    txt = C["距春分差立成"]["原文"].split()
    book = []
    for item in txt:
        m = re.match(r"(..)(加|減|不加不減)(.*)分?", item)
        name, op, num = m.group(1), m.group(2), m.group(3).rstrip("分")
        book.append((name, 0 if op == "不加不減" else (cn(num) if op == "加" else -cn(num))))
    xq = xinfa_qi_times()
    kq, hq = kc.jieqi_times(1740), hb.jieqi_times(1740)
    x = [xf.ri_cha(t) * 1440 for t in xq[:24]]
    k = [kc.shi_cha(t) * 1440 for t in kq[:24]]
    h = [hb.shi_cha(t) * 1440 for t in hq[:24]]
    out1 = []
    for i, (name, v) in enumerate(book):
        out1.append([name, f"{v:+d}", f"{x[i]:+.1f}", f"{v - x[i]:+.1f}", f"{k[i] - k[6]:+.1f}", f"{v - (k[i] - k[6]):+.1f}",
                     f"{h[i] - h[6]:+.1f}", f"{v - (h[i] - h[6]):+.1f}"])
    rl = C["日率"]["原文"].split()
    out2 = []
    for i, item in enumerate(rl):
        m = re.match(r"(.+?)(?:至|距)(.+?)(十[四五])日(.+?)時(..)$", item)
        d = cn(m.group(3))
        hh = cn(m.group(4))
        tail = m.group(5)
        book_d = d + hh / 24 + (CNUM[tail[0]] * 10 + CNUM[tail[1]]) / 1440
        def fmt(v):
            dd = int(v)
            r = (v - dd) * 24
            return f"{dd}日{int(r):02d}時{(r - int(r)) * 60:04.1f}分"
        xs = xq[i + 1] - xq[i]
        ks = kq[i + 1] - kq[i]
        hs = hq[i + 1] - hq[i]
        out2.append([item, fmt(book_d), fmt(xs), f"{(book_d - xs) * 1440:+.0f}", fmt(ks), f"{(book_d - ks) * 1440:+.0f}", fmt(hs),
                     f"{(book_d - hs) * 1440:+.0f}"])
    # 壬戌年天正定冬至
    ex = []
    for nm, mod in (("考成", kc), ("后编", hb)):
        t = mod.jieqi_times(1742)[0]
        u = t + mod.shi_cha(t)
        j = mod.jdn_of(math.floor(u))
        ex.append(f"{nm} {GZ[(j + 49) % 60]}（{(j + 49) % 60} 日）{(u % 1) * 24:.2f} 时")
    return out1, out2, ex


def main():
    cs, C = changshu()
    q1, q2, ex = jieqi_extra(C)
    ss = shoushuo()
    lc = licheng()
    md = ["# 琉球推步手册与清代三法（自动生成）", "",
          "> 由 `scripts/calendar_data/ryukyu_tuibu_check.py` 生成，勿手改。录文 `data/calendar/ryukyu_tuibu_changshu.csv`、"
          "`data/calendar/ryukyu_tuibu_licheng.csv`；书目、结论与限制见 [ryukyu-sources.md](ryukyu-sources.md)「推步手册」节。", "",
          "## 常数与根数", "",
          "「差」= 书 − 各家（度量取角秒，日量取秒）。书载到秒，|差| ≤ 0.5 为合，≤ 1 为末位差一。", "",
          "| 条目 | 书 | 考成 | 差 | 后编 | 差 | 新法 | 差 | 注 |", "|---|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(r) + " |" for r in cs]
    md += ["", "## 求節氣：距春分差立成", "",
           "书「置實氣時分，加減距春分差立成分，即得定氣也」：各气所加减之分（时间）。三家以 1740 年各气之时差（均数时差 + 升度时差）"
           "减春分之值比较（新法「日差」本即「從春分起算」）；差 = 书 − 各家（分）。", "",
           "| 气 | 书 | 新法 | 差 | 考成 | 差 | 后编 | 差 |", "|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(r) + " |" for r in q1]
    md += ["", "## 求節氣：日率（相邻定气相距）", "",
           "书「累加各節日率而得各節平紀日」，以 1740 年三家定气（平时）相距比较；书末两位按「分」读，差 = 书 − 各家（分）。"
           "差多在 ±60 分以内而无一家系统相合，书末两位或非分，读数亦待核。", "",
           "| 书原文 | 书 | 新法 | 差 | 考成 | 差 | 后编 | 差 |", "|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(r) + " |" for r in q2]
    md += ["", "书例「壬戌年天正定冬至十二日〇時」（乾隆七年，冬至在 1741 年 12 月），用时京师推算：" + "；".join(ex) + "。", ""]
    md += ["", "## 首朔立成（逐年閏應、正月首朔）", "",
           "书递推：首朔逐年加十二朔实（354 日 08:48:38）或十三朔（383 日 21:32:41），满六十去之；閏應逐年加閏餘（10 日 21:00:08），"
           "满朔策去之；閏應 ≥ 閏限（18 日 15:43:55）者其年有闰。「递推」列由书载 1740 年根起算，「差」= 书 − 递推。"
           "「张表闰」为该农历年张表闰月；递推之闰取张表（书 1756 年即如此，见注）。", "",
           "| 年 | 干支 | 书 | 递推首朔 | 差 | 按閏限 | 张表闰 | 闰合否 | 注 |", "|---|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(r) + " |" for r in ss]
    md += ["", "## 立成抽格", "",
           ("「差」= 书 − 各家；四行立成以角秒计，均度立成以角秒计，黄赤道差以时秒计。均度立成之引数自最卑（新法自最高冲）、月最高起，"
            "值取均数绝对值；后编月离初均随本时两心差变，无单引数之表，后编列取两心差 550505 一算，只作参考。黄赤道差以宫度为距冬至之黄经，"
            "值取升度差（黄经 − 赤经）变时之绝对值。度位、分位书常省，按上下文补，见录文备注。"), "",
           "| 册 | 表 | 引数 | 原文 | 书值 | 考成 | 差 | 后编 | 差 | 新法 | 差 | 页 | 注 |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(r) + " |" for r in lc]
    OUT.write_text("\n".join(md) + "\n", encoding="utf-8")
    for r in cs:
        print(r[:8])
    for r in ss:
        print(r[:8])
    for r in lc:
        print(r[2:11])


if __name__ == "__main__":
    main()
