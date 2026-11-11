"""清初时宪历「西洋新法」（汤若望，天聪戊辰元；顺治二年 1645 至雍正三年 1725 行用，康熙五至八年复用大统除外）推节气、合朔。

依据与读法（详见 doc 草稿 report_C.md）：
- 常数：《清史稿·时宪志一》（卷四十五）「若望之法，以天聪戊辰为元」所列：太阳一日平行 59′8″19‴49⁗36，最高一年行 45″，
  戊辰年平行距冬至 53′35″39‴，最高距冬至 5°59′59″；太阴一日平行 13°10′35″1‴、自行 13°3′53″56‴、正交行 3′10″、月孛行 6′41″，
  戊辰年平行距冬至 6 宫 1°50′54″46‴，自行 6 宫 25°32′15″34‴，正交 1 宫 0°0′14″，月孛 11 宫 6°19′。
  诸应「從天聰二年戊辰前冬至後己卯日子正為始」：戊辰年根日己卯（1627-12-23，JDN 2315666）子正。
- 日躔（原文复原）：《新法算书》卷二十五、二十六日躔表说。岁实（平行岁实）365 日 24 刻 21 分 88 秒 64 微；
  加减表按两心差 3584（半径十万）分作两段 1792，两三角形之角相并为均数（卷二十六「算加減表說」两例：
  距最高 30° 得 1°0′39″，距高冲 30° 两角并 30′46″ + 31′44″ = 1°2′30″，原文「一度二十分三十秒」之「二十」当作「〇二」）。
  引数 = 经数 + 最高冲（自最高冲起算），引数初宫至五宫均数为加。
  日差（卷二十六）：「測太陽行度以春分為本……從春分起算立日差表」，即（平行 − 赤经）变时，减去春分之值，加于平时得用时。
- 月离（代用）：《新法算书》月离表（卷三十二至三十五）维基文库只录说明，数字表不存；月离历指所给本轮 5800、均轮 2900（半径十万）
  与《历象考成》（甲子元）本轮 580000、均轮 290000 同，《清史稿·时宪志一》引允禄「曆象考成一書，其數惟黃赤大距減少二分，
  餘皆仍新法算書西人第穀之舊」。故月离之初均、二三均、交均用《清史稿·时宪志四》（卷四十八）甲子元法之作图与半径代用，
  平行与历元用若望之数。这是代用，不是原文复原。
- 节气、合朔：子正实行内插；合朔、中气化为用时定日（新法月离表「先以所設之用時變為平時」，可知历书时刻为用时）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/shixian_xinfa.py
"""
import math

D = lambda deg, m=0, s=0, t=0, q=0, w=0: deg + m / 60 + s / 3600 + t / 216000 + q / 12960000 + w / 777600000

# ---------------------------------------------------------------- 日躔（新法算书卷二十五、二十六；清史稿卷四十五）
SUI = 365.2421886 + 0.4e-8               # 365 日 24 刻 21 分 88 秒 64 微（刻、分、秒、微皆百进）
SUN_DAILY = D(0, 59, 8, 19, 49, 36)
ZUIGAO_YEAR = 45 / 3600                  # 最高一年行
ROOT_JDN = 2315666                       # 戊辰年根日己卯（1627-12-23）
ROOT_L = D(0, 53, 35, 39)                # 根日子正太阳平行距平冬至
ROOT_GAO = D(5, 59, 59)                  # 清史稿「最高距冬至五度五十九分五十九秒」：按卷二十五算例即恒年表之「最高衝」（近地点），中法自冬至起
SUN_R, SUN_A = 100000.0, 1792.0          # 本天半径、两心差之半
HUANG_CHI = D(23, 31, 30)                # 黄赤大距（新法；考成减二分为 23°29′30″）

# ---------------------------------------------------------------- 月离（清史稿卷四十五常数；卷四十八甲子元作图代用）
MOON_DAILY = D(13, 10, 35, 1)
MOON_ZIXING = D(13, 3, 53, 56)           # 自行（引数）日行
JIAO_DAILY = D(0, 3, 10)
BEI_DAILY = D(0, 6, 41)                  # 月孛（最高）日行
ROOT_MOON = 6 * 30 + D(1, 50, 54, 46)
ROOT_ZIXING = 6 * 30 + D(25, 32, 15, 34)
ROOT_JIAO = 1 * 30 + D(0, 0, 14)
ROOT_BEI = 11 * 30 + D(6, 19)
MOON_HOUR = MOON_DAILY / 24
BEN_LUN, JUN_LUN = 580000.0, 290000.0    # 甲子元：本轮、均轮半径（本天千万）
CI_LUN, CI_JUN_LUN = 217000.0, 117500.0  # 次轮、次均轮
DA_JU_ZHONG, DA_JU_BAN = D(5, 8), D(0, 9, 30)


def sin(d):
    return math.sin(math.radians(d))


def cos(d):
    return math.cos(math.radians(d))


def tan(d):
    return math.tan(math.radians(d))


def asind(x):
    return math.degrees(math.asin(x))


def atan2(y, x):
    return math.degrees(math.atan2(y, x))


def norm(d):
    return d % 360.0


def signed(d):
    return norm(d + 180) - 180


def fold(d):
    d = norm(d)
    return d if d <= 180 else 360 - d


def sun_jun_from_gao(phi):
    """加减均数（度，正值）：phi 为距最高之度（0—180）。卷二十六：丙辛 1792、辛角 = phi，丙丁 = 1792·sin、丁辛 = 1792·cos，
    乙丁 = √(半径² − 丙丁²)，乙角 = asin(丙丁/半径)，乙辛 = 乙丁 ± 丁辛；又辛戊 1792、乙辛戊角 = 180° − phi，
    丁戊 = 1792·sin、乙丁' = 乙辛 ± 1792·cos，乙角 = atan(丁戊/乙丁')；两角相并。"""
    p = SUN_A * sin(phi)
    q = SUN_A * cos(phi)
    a1 = asind(p / SUN_R)
    yi_xin = math.sqrt(SUN_R ** 2 - p * p) + q
    a2 = atan2(p, yi_xin + q)
    return a1 + a2


def sun(t):
    """t：JDN 计之时刻（日，平时，0 为该 JDN 日子正前一日之子正……即 t = JDN + 日内比例，以京师子正为日首）。
    返回 (平行 L, 均数 E, 实行 λ, 最高)：中法宫度自冬至起。"""
    n = t - ROOT_JDN
    L = norm(ROOT_L + SUN_DAILY * n)
    chong = ROOT_GAO + ZUIGAO_YEAR * n / SUI       # 最高冲
    yin = norm(L - chong)                 # 自最高冲起之平行距
    e = sun_jun_from_gao(180 - fold(yin))
    e = e if norm(yin) <= 180 else -e     # 过最高冲为加（实行在前），未至为减；卷二十五算例「引數＝經數＋最髙衝」即未至之距
    return L, e, norm(L + e), chong


def _ri_cha_raw(t):
    L, e, lam, _ = sun(t)
    lm = norm(lam + 270)                  # 现代黄经
    ra = norm(atan2(cos(HUANG_CHI) * sin(lm), cos(lm)))
    return (signed(lm - ra) - e) / 360    # 平太阳 − 视太阳（以日计）：（λ − 赤经）− 均数


def ri_cha(t):
    """日差（日）：自春分起算，加于平时得用时。取本岁春分之值为零点。"""
    return _ri_cha_raw(t) - _ri_cha_raw(chunfen_near(t))


def chunfen_near(t):
    """t 之前最近春分（日躔实行 90°，中法）的平时"""
    d = math.floor(t)
    for _ in range(400):
        a = signed(sun(d)[2] - 90)
        b = signed(sun(d + 1)[2] - 90)
        if a < 0 <= b:
            return d - a / (b - a)
        d -= 1
    raise RuntimeError


# ---------------------------------------------------------------- 月离（甲子元作图代用）
def moon(t, yongshi=True):
    """t：日序（JDN 计，子正为整数）。yongshi=True 时视 t 为用时子正（甲子元「求用時太陰平行」：以时差行减太阴平行），
    所得合朔内插时刻即用时；False 则不改（平时）。返回 (黄道实行, 太阳实行)。卷四十八原文逐句见各步注。"""
    n = t - ROOT_JDN
    _, _, lam_sun, _ = sun(t)
    ping = norm(ROOT_MOON + MOON_DAILY * n)
    zi = norm(ROOT_ZIXING + MOON_ZIXING * n)
    jiao = norm(ROOT_JIAO - JIAO_DAILY * n)
    if yongshi:
        rc = ri_cha(t)                    # 用时 − 平时（日）
        ping = norm(ping - MOON_DAILY * rc)
        zi = norm(zi - MOON_ZIXING * rc)
    # 求初實行：「以本輪半徑之半為對正角之邊，以引數為一角，求得對角之邊三因之。又求得對又一角之邊，與本天半徑相加減。
    # 復用平三角形，以三因數為小邊，加減本天半徑數為大邊……求得對小邊之角為初均數，求得對正角之邊（次輪最近點距地心之線）」
    m = zi
    half = BEN_LUN / 2
    small = 3 * half * abs(sin(m))
    big = 1e7 + half * cos(m)             # 九宫至二宫相加，三宫至八宫相减
    chu_jun = atan2(small, big)
    l0 = math.hypot(small, big)
    jian = norm(m) < 180                  # 引数初宫至五宫为减
    chu = norm(ping - chu_jun) if jian else norm(ping + chu_jun)
    # 求白道實行：次引 = 初实行 − 太阳实行
    ci = norm(chu - lam_sun)
    chord = 2 * CI_LUN * abs(sin(ci))     # 倍次引之通弦
    ju_ban = 180 - m if m < 180 else m - 180          # 引数减半周之度
    c = ci
    if c < 90:
        q = 90 - c
    elif c < 180:
        q = c - 90
    elif c < 270:
        q = 90 - (c - 180)
    else:
        q = c - 270
    guo = (90 <= c < 180) or c >= 270                  # 过象限或过三象限
    add = guo if jian else not guo
    ang = chu_jun + ju_ban + (q if add else -q)
    ang = abs(ang)
    if ang > 180:
        ang = 360 - ang
    if chord == 0 or ang in (0, 180) or c in (0, 180):
        er = 0.0
        d2 = l0 + (CI_LUN * 0)
    else:
        er = atan2(chord * sin(ang), l0 - chord * cos(ang))
        d2 = math.sqrt(l0 * l0 + chord * chord - 2 * l0 * chord * cos(ang))
        # 二均加减（卷四十八小注）：初均与均轮心距最卑之度相加为泛限
        fan = chu_jun + ju_ban
        x2 = norm(2 * ci)
        if abs(fan - 90) < 1e-12:
            same = True
        elif fan < 90:
            ding = 2 * (90 - fan)
            xx = x2 if jian else 360 - x2
            same = xx > ding
        else:
            ding = 2 * (fan - 90)
            xx = x2 if not jian else 360 - x2
            same = xx > ding
        er_add = (not jian) if same else jian
        er = er if er_add else -er
    # 三均：「以次均輪心距地為一邊，次均輪半徑為一邊，次引倍度為所夾之角……求得對次均輪半徑之角為三均數」；倍度不及半周为加
    x2 = norm(2 * ci)
    san = atan2(CI_JUN_LUN * sin(fold(x2)), d2 - CI_JUN_LUN * cos(fold(x2)))
    san = san if x2 < 180 else -san
    bai = norm(chu + er + san)
    # 求黃道實行：黄白大距中数、半较为两边，次引倍度为夹角，对角之边为黄白大距，对半较之角为交均；倍度不及半周为减
    cc = fold(x2)
    ca = cos(DA_JU_ZHONG) * cos(DA_JU_BAN) + sin(DA_JU_ZHONG) * sin(DA_JU_BAN) * cos(cc)
    da_ju = math.degrees(math.acos(ca))
    jiao_jun = asind(sin(DA_JU_BAN) * sin(cc) / sin(da_ju))
    jiao_shi = norm(jiao - jiao_jun) if x2 < 180 else norm(jiao + jiao_jun)
    jj = norm(bai - jiao_shi)
    hd = atan2(cos(da_ju) * sin(jj), cos(jj))
    return norm(bai + signed(hd - jj)), lam_sun


# ---------------------------------------------------------------- 节气、合朔、排月
def cross_sun(target, d):
    """d 日起首个日躔实行过 target 的平时"""
    for _ in range(800):
        a = signed(sun(d)[2] - target)
        b = signed(sun(d + 1)[2] - target)
        if a < 0 <= b:
            return d - a / (b - a)
        d += 1
    raise RuntimeError


def dongzhi(y):
    """冬至岁 y（天正冬至在 y−1 年十二月）的定冬至平时"""
    j = 2316030 + round((y - 1629) * SUI)
    return cross_sun(0, j - 5)


def shuo_times(d0, d1, yongshi=True):
    out = []
    e_d = signed(moon(d0, yongshi)[0] - moon(d0, yongshi)[1])
    for d in range(d0, d1):
        m1, s1 = moon(d + 1, yongshi)
        e_n = signed(m1 - s1)
        if e_d < 0 <= e_n:
            out.append(d + (-e_d) / (e_n - e_d))
        e_d = e_n
    return out


def year_months(y, yongshi=True):
    """冬至岁 y：(朔日 JDN, 月序, 闰否)。用时：中气平时加日差、合朔以用时子正内插；平时：皆不改。置闰同甲子元（无中气之月）。"""
    dz0, dz1 = dongzhi(y), dongzhi(y + 1)
    zh = [cross_sun(30 * k, math.floor(dz0) + int(30.4 * k) - 3) for k in range(12)] + [dz1]
    zday = [math.floor(t + ri_cha(t)) if yongshi else math.floor(t) for t in zh]
    sh = shuo_times(math.floor(dz0) - 32, math.floor(dz1) + 32, yongshi)
    days = [math.floor(t) for t in sh]
    i0 = max(i for i, d in enumerate(days) if d <= zday[0])
    i1 = max(i for i, d in enumerate(days) if d <= zday[12])
    ms = days[i0:i1 + 1]
    n = i1 - i0
    has = [any(ms[i] <= z < ms[i + 1] for z in zday) for i in range(n)]
    leap = next(i for i in range(1, n) if not has[i]) if n == 13 else None
    out, mo = [], 11
    for i in range(n):
        if i == leap:
            out.append((ms[i], out[-1][1], True))
            continue
        out.append((ms[i], mo, False))
        mo = mo % 12 + 1
    return out


if __name__ == "__main__":
    # ------------------------------------------------ 原文算例：加减表、冬至
    print("距最高30°均数", f"{sun_jun_from_gao(30) * 60:.3f}′（原文 60.65′）", "距高冲30°", f"{sun_jun_from_gao(150) * 60:.3f}′（原文两角并 62.50′）")
    print("引数 6°20′36″ 均数", f"{sun_jun_from_gao(180 - D(6, 20, 36, 3)) * 3600:.1f}″（原文 13′52″20‴ = 832.3″）")

    def dongzhi_after(jdn_root_next):
        d = jdn_root_next - 3
        for _ in range(10):
            a = signed(sun(d)[2] - 0)
            b = signed(sun(d + 1)[2] - 0)
            if a < 0 <= b:
                return d - a / (b - a)
            d += 1
    for label, j0, want in (("己巳天正冬至（1628-12 癸未）", 2316031, "癸未 13:12:09 平，加日差八分 未初一刻05分29秒"),
                            ("辛未天正冬至（1630-12 根日甲午）", 2316031 + 365 + 365, "甲午 子正后三刻13分20秒（含日差）")):
        t = dongzhi_after(j0)
        rc = ri_cha(t)
        G = "甲乙丙丁戊己庚辛壬癸"
        Z = "子丑寅卯辰巳午未申酉戌亥"
        def gz(j):
            g = (j + 49) % 60
            return G[g % 10] + Z[g % 12]
        def hms(x):
            x = (x % 1) * 24
            h = int(x)
            m = int((x - h) * 60)
            s = ((x - h) * 60 - m) * 60
            return f"{h:02d}:{m:02d}:{s:04.1f}"
        print(label, "平时", gz(math.floor(t)), hms(t), "日差", f"{rc * 1440:+.2f}分", "用时", gz(math.floor(t + rc)), hms(t + rc), "| 原文", want)

    # ------------------------------------------------ 逐月与张表比较 → data/calendar/shixian_xinfa_check.csv
    import csv
    from pathlib import Path
    root = Path(__file__).resolve().parents[2]

    def jdn(y, m, d):
        a = (14 - m) // 12
        yy, mm = y + 4800 - a, m + 12 * a - 3
        return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045

    def ymd(j):
        a = j + 32044
        b = (4 * a + 3) // 146097
        c = a - 146097 * b // 4
        d = (4 * c + 3) // 1461
        e = c - 1461 * d // 4
        m = (5 * e + 2) // 153
        return f"{100 * b + d - 4800 + m // 10}-{m + 3 - 12 * (m // 10):02d}-{e - (153 * m + 2) // 5 + 1:02d}"
    zh = {}
    for r in csv.DictReader(open(root / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        zh[jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))] = (int(r["lunar_month"]), r["is_leap"] == "1")
    rows = []
    for y in range(1645, 1730):
        for (ja, m, l), (jb, _, _) in zip(year_months(y, True), year_months(y, False)):
            ly = y - 1 if m >= 11 else y
            rows.append([y, ly, m, int(l), ymd(ja), ymd(jb), int(zh.get(ja) == (m, l)), int(zh.get(jb) == (m, l)), int(1666 <= ly <= 1669)])
    with open(root / "data/calendar/shixian_xinfa_check.csv", "w", encoding="utf-8", newline="") as fh:
        fh.write("# 清初时宪历「西洋新法」复原（scripts/calendar_data/shixian_xinfa.py；日躔照《新法算书》原文，月离借《历象考成》甲子元作图代用）"
                 "逐月朔日与张表比较：冬至岁 1645—1729。用时 = 合朔以用时子正内插、中气加日差（自春分起算）定日；平时 = 不改。"
                 "大统 = 1 者为康熙五至八年复用大统之年，不以新法论。1726 年起行《历象考成》，只作对照。\n")
        w = csv.writer(fh)
        w.writerow(["冬至岁", "农历年", "月", "闰", "用时朔日", "平时朔日", "用时合张表", "平时合张表", "大统"])
        w.writerows(rows)
    xf = [r for r in rows if not r[8] and 1645 <= r[1] <= 1725]
    print("新法期", len(xf), "月：用时合张表", sum(r[6] for r in xf), "，平时合", sum(r[7] for r in xf))
