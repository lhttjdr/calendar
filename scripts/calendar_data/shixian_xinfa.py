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
- 月离（照表，《新法算书》卷三十二「月離表用法」）：「法於太隂恒年日時三表求各行數就本數并之」——恒年、日、时三表数字未转录，
  以《清史稿》所载戊辰元太阴日行、根数线性计算（表即依此数造，差在表的取整内）。「以太隂自行為引數於前均表求均數……依本號加減於
  太隂平行及自行得太隂實平行及實自行」：第一均数照卷三十三「太陰自行加減表」（data/calendar/xinfa_yueli_tables/zixing_part1.csv，
  每 10′ 一格，零分「用中比例」）查，顺数○至五宫减、逆数六至十一宫加；存疑四格用同行相邻两格之中数。
  时刻：「先以所設之用時變為平時……太陽日差……反用之……太隂亦有日差表亦用以均其時其加減則用本號」——太阳用平时 = 用时 − 日差，
  太阴用时 = 用时 + 月离日差（卷三十三「月離日差表」，引数太阳所躔宫度，zixing_part2.csv，顺加逆减）。
  定朔：「以減太隂實平行……若無餘分即日月同度是為定朔……朔望時太隂止用一均數，故白道上所得實平行度即為視經度」；
  白道实平行再以卷三十三「黄白二道同升表」（引数月距正交，顺减逆加，极大 7′）改为黄道经度——用法第十条「今細推黄道上正經度
  有本法，其本表曰黄白道同升表」，表说「用以密推黄道上之月離經度」；朔望「交行無均數」，正交用平行。
  逐日用时子正求日月相距，两日中比例得定朔用时。二三均数表（卷三十四、三十五）只在朔望以外用，算例三自检用之。
- 节气：日躔实行每 15° 一气，平时加日差为用时定日；置闰：无中气之月为闰。

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

# ---------------------------------------------------------------- 太阴常数（清史稿卷四十五）
# 日行取《新法算书》卷三十二恒年、日、时三表实际所用之数（表由根数 + 日行 × 日数造成，以此复现到 1 微内；
# 《清史稿》所载 13°10′35″1‴、13°3′53″56‴、3′10″、6′41″ 为截尾之数）。表的末位取整未模拟（影响定朔不过数秒）。
MOON_DAILY = D(13, 10, 35, 1, 22)
MOON_ZIXING = D(13, 3, 53, 56, 20)       # 自行（引数）日行
JIAO_DAILY = D(0, 3, 10, 38)
BEI_DAILY = D(0, 6, 41, 5)               # 月孛（最高）日行
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


_CF = {}


def _chunfen_of(i):
    """戊辰元起第 i 年之春分（日躔实行 90°）平时"""
    if i not in _CF:
        _CF[i] = _chunfen_scan(ROOT_JDN + 98 + round(i * SUI) + 8)
    return _CF[i]


def chunfen_near(t):
    """t 之前最近春分的平时"""
    i = math.floor((t - ROOT_JDN - 88) / SUI)
    while _chunfen_of(i) > t:
        i -= 1
    while _chunfen_of(i + 1) <= t:
        i += 1
    return _chunfen_of(i)


def _chunfen_scan(t):
    d = math.floor(t)
    for _ in range(400):
        a = signed(sun(d)[2] - 90)
        b = signed(sun(d + 1)[2] - 90)
        if a < 0 <= b:
            return d - a / (b - a)
        d -= 1
    raise RuntimeError


# ---------------------------------------------------------------- 月离表
import csv as _csv
from pathlib import Path as _Path
_TAB = _Path(__file__).resolve().parents[2] / "data/calendar/xinfa_yueli_tables"


def _rows(name):
    return [r for r in _csv.DictReader(l for l in open(_TAB / name, encoding="utf-8") if not l.startswith("#"))]


def _load_zixing():
    """太阴自行加减表：自行 0°—180° 每 10′ 一格（秒），存疑格以同行相邻两格之中数代"""
    v = {}
    doubt = []
    for r in _rows("zixing_part1.csv"):
        k = int(r["自行_宫"]) * 1800 + int(r["自行_度"]) * 60 + int(r["自行_分"])
        val = int(r["度"]) * 3600 + int(r["分"]) * 60 + int(r["秒"])
        v[k] = val
        if r["存疑"] == "1":
            doubt.append(k)
    for k in doubt:
        v[k] = round((v[k - 10] + v[k + 10]) / 2)
    return [v[k] for k in range(0, 10801, 10)], doubt


ZIXING_TAB, ZIXING_DOUBT = _load_zixing()


def _load_richa():
    """月离日差表：引数太阳所躔宫度，顺数（星纪、玄枵、娵訾；鹑首、鹑火、鹑尾）加，逆数（实沈、大梁、降娄；析木、大火、寿星）减。
    返回 0°—90°（自冬至或夏至起）的秒数（时）"""
    order = {"星纪/鹑首": 0, "玄枵/鹑火": 1, "娵訾/鹑尾": 2}
    v = {}
    for r in _rows("zixing_part2.csv"):
        if r["表"] != "月离日差表":
            continue
        x = order[r["自行_宫"]] * 30 + int(r["自行_度"])
        v[x] = int(r["分"]) * 60 + int(r["秒"])
    return [v[x] for x in range(91)]


RICHA_TAB = _load_richa()


def _load_tongsheng():
    """黄白二道同升表（卷三十三）：引数月距正交 0°—90°（○一二宫顺数、九十十一宫逆数；六七八、三四五宫同），返回秒"""
    order = {"○/六": 0, "一/七": 1, "二/八": 2}
    v = {}
    for r in _rows("zixing_part2.csv"):
        if r["表"] != "黄白二道同升表":
            continue
        v[order[r["自行_宫"]] * 30 + int(r["自行_度"])] = int(r["分"]) * 60 + int(r["秒"])
    return [v[x] for x in range(91)]


TONGSHENG_TAB = _load_tongsheng()


def interp(tab, x, step):
    """中比例：tab[i] 为引数 i·step 之值"""
    i = min(int(x // step), len(tab) - 2)
    f = x / step - i
    return tab[i] + (tab[i + 1] - tab[i]) * f


def moon_jun1(a):
    """第一均数（度）：自行 a；顺数○至五宫减，逆数六至十一宫加"""
    x = fold(a) * 60                       # 分
    val = interp(ZIXING_TAB, x, 10) / 3600
    return -val if norm(a) <= 180 else val


def moon_richa(lam):
    """月离日差（日）：lam 为太阳实行（中法，自冬至起）。冬至至春分、夏至至秋分为顺数，加；余为逆数，减"""
    x = norm(lam)
    q, r = divmod(x, 90)
    shun = int(q) % 2 == 0
    arg = r if shun else 90 - r
    sec = interp(RICHA_TAB, arg, 1)
    return (sec if shun else -sec) / 86400


def tongsheng(u):
    """黄白二道同升差（度）：u 为月距正交（白道经度 − 正交）。卷三十三表说「月距正交宫度以為引數……依本號以加以減于白道所得經度
    得黄道所求經度」，表注「顺减逆加」：○一二宫、六七八宫（顺数）减，三四五宫、九十十一宫（逆数）加，极大 7′0″（月距交 45°）"""
    x = norm(u)
    q, r = divmod(x, 90)
    shun = int(q) % 2 == 0
    arg = r if shun else 90 - r
    sec = interp(TONGSHENG_TAB, arg, 1)
    return (-sec if shun else sec) / 3600


def zhengjiao(t):
    """t（太阴所用之时）之正交平行经度（中法）：「於本恒年表查本年正交行度去減本日時正交行度」，朔望「交行無均數」"""
    return norm(ROOT_JIAO - JIAO_DAILY * (t - ROOT_JDN))


def moon_huang(L, t):
    """白道实平行 L（太阴所用之时 t）改为黄道经度：加减同升差"""
    return norm(L + tongsheng(L - zhengjiao(t)))


def moon_mean(t):
    """t：太阴所用之时（日）：(平行, 自行)"""
    n = t - ROOT_JDN
    return norm(ROOT_MOON + MOON_DAILY * n), norm(ROOT_ZIXING + MOON_ZIXING * n)


def moon_shi(t_yong):
    """用时 t_yong 之太阴实平行（度）与实自行：太阴用时加月离日差（本号）"""
    lam = sun(t_yong - ri_cha(t_yong))[2]
    tm = t_yong + moon_richa(lam)
    L, a = moon_mean(tm)
    j = moon_jun1(a)
    return norm(L + j), norm(a + j), tm


def sun_yong(t_yong):
    """用时 t_yong 之太阳实行：平时 = 用时 − 日差"""
    return sun(t_yong - ri_cha(t_yong))[2]


def _load_ersan():
    """二三均数表：(月距日度 1—90, 实自行度) → 秒（加正减负）。卷三十四（er_san_jun_part1/2，月距日 1—30）、卷三十五（31—90）。
    「直行……月距日或距日之衝」；逆数（月距日 90°—179°）以 f(180−D, 360−M) = −f(D, M) 读。存疑与残缺格以同行左右两格中数代。"""
    v, bad = {}, set()
    for name in ("er_san_jun_part1.csv", "er_san_jun_part2.csv", "er_san_jun_zongshu_juan35.csv"):
        for r in _rows(name):
            d = int(r["月距日_宫"]) * 30 + int(r["月距日_度"])
            m = int(r["实自行_宫"]) * 30 + int(r["实自行_度"])
            sign = -1 if ("減" in r["号"] or "减" in r["号"]) else 1
            try:
                val = int(r.get("度") or 0) * 3600 + int(r["分"] or 0) * 60 + (int(r["秒"]) if r.get("秒") else 0)
            except ValueError:
                val = None
            if (d, m) in v and (r["存疑"] == "1" or val is None):
                continue
            v[(d, m)] = None if val is None else sign * val
            if r["存疑"] == "1" or val is None:
                bad.add((d, m))
            else:
                bad.discard((d, m))
    for d, m in sorted(bad):
        cols = sorted(mm for (dd, mm) in v if dd == d)
        i = cols.index(m)
        nb = [v[(d, cols[j])] for j in (i - 1, i + 1) if 0 <= j < len(cols) and (d, cols[j]) not in bad]
        if not nb:   # 同行左右亦存疑（如一行皆疑）：取同列上下两行之中数
            nb = [v[(dd, m)] for dd in (d - 1, d + 1) if (dd, m) in v and (dd, m) not in bad and v[(dd, m)] is not None]
        if nb:
            v[(d, m)] = round(sum(nb) / len(nb))
    return v, sorted(bad)


ERSAN, ERSAN_DOUBT = _load_ersan()


def _ersan_row(d, m):
    """月距日整度 d（0—90）一行，实自行 m 按该行所列之距中比例"""
    if d == 0:
        return 0.0
    cols = sorted(mm for (dd, mm) in ERSAN if dd == d)
    m = norm(m)
    lo = max(c for c in cols if c <= m)
    hi = min((c for c in cols if c > m), default=360)
    vhi = ERSAN[(d, hi % 360)] if hi % 360 in cols or hi == 360 else ERSAN[(d, hi)]
    return ERSAN[(d, lo)] + (vhi - ERSAN[(d, lo)]) * (m - lo) / (hi - lo)


def ersan_jun(dist, m):
    """二三均数（度）：dist 月距日（实平行 − 太阳实行），m 实自行。月距日以半周为周（「距日之衝」同）"""
    x = norm(dist) % 180
    if x <= 90:
        d0 = int(x)
        f = x - d0
        a = _ersan_row(d0, m)
        b = _ersan_row(min(d0 + 1, 90), m)
        return (a + (b - a) * f) / 3600
    return -ersan_jun(180 - x, 360 - m)


# ---------------------------------------------------------------- 节气、定朔、排月
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
    j = ROOT_JDN + round((y - 1628) * SUI)
    return cross_sun(0, j - 8)


def shuo_times(d0, d1, yongshi=True):
    """[d0, d1) 内定朔时刻：用时子正逐日求日月相距（太阴黄道经度 − 太阳实行），两日中比例；yongshi=False 时一律作平时。
    太阴黄道经度 = 白道实平行（平行加第一均）加减黄白二道同升差（卷三十三）：卷三十二用法第五条「朔望時太隂止用一均數，故白道上
    所得實平行度即為視經度」只说朔望不用二三均，第十条「月離厯指諸論皆於白道取太隂經度不於黄道者其差甚微故也，今細推黄道上正經度
    有本法，其本表曰黄白道同升表」，卷三十三「二道同升表者用以密推黄道上之月離經度也」。同升差极大 7′，折合定朔约 14 分钟；
    不加则冬至岁 1645—1725 与张表不合 7 月、《清实录》九月合 2，加之则不合 3 月（皆距子正 2.3 分钟内）、实录合 6，
    对现代真合朔之 σ 由 13.1 分降为 7.8 分（见 doc/calendar/calendar-compendium.md 第十一节）。"""
    def el(d):
        if yongshi:
            L, _, tm = moon_shi(d)
            return signed(moon_huang(L, tm) - sun_yong(d))
        L, a = moon_mean(d)
        return signed(moon_huang(norm(L + moon_jun1(a)), d) - sun(d)[2])
    out = []
    e_d = el(d0)
    for d in range(d0, d1):
        e_n = el(d + 1)
        if e_d < 0 <= e_n:
            out.append(d + (-e_d) / (e_n - e_d))
        e_d = e_n
    return out


# 置闰之界：顺治初沿崇祯新法「專以合朔為主」，以中气与合朔时刻比较；其后改为以日比较（中气所在日在月内即属本月）。
# 本纪顺治二年（1645）「閏六月」唯时刻法得之；顺治五年（1648）閏四月、十年閏六月及康熙诸闰（1662、1680、1697、1699、1716）
# 唯以日法得之；1646、1647 两法所得相同，改制之年在顺治三至五年间无从细分，取冬至岁 1647 为末。
SHIKE_RUN_LAST = 1647


def year_months(y, yongshi=True):
    """冬至岁 y：(朔日 JDN, 月序, 闰否)。无中气之月为闰。中气属月：冬至岁 ≤ SHIKE_RUN_LAST 以合朔时刻为界——卷八李天经
    崇祯十五年题本「新法置閏專以合朔為主，若中氣適在合朔時刻前者，是中氣尚屬前月之晦則無閏；若在合朔日時後者，則前月當有閏
    而無疑也」，卷九十七「月有閏者，太陽躔一宫之時與月會合二次以成者也」；此后以日为界——《清史稿·时宪志一》康熙八年南怀仁
    「雨水為正月中氣，是月二十九日值雨水，即為康熙九年正月，不應置閏」。中气、合朔皆用时；月首以合朔用时所在日。"""
    dz0, dz1 = dongzhi(y), dongzhi(y + 1)
    zh = [cross_sun(30 * k, math.floor(dz0) + int(30.4 * k) - 3) for k in range(12)] + [dz1]
    zt = [t + ri_cha(t) if yongshi else t for t in zh]          # 中气时刻（用时）
    sh = shuo_times(math.floor(dz0) - 32, math.floor(dz1) + 32, yongshi)   # 合朔时刻（用时）
    shike = y <= SHIKE_RUN_LAST
    # 以时刻为界则比较时刻，以日为界则比较所在日（含天正之定：冬至所在之月）
    key_s = sh if shike else [math.floor(t) for t in sh]
    key_z = zt if shike else [math.floor(z) for z in zt]
    i0 = max(i for i, t in enumerate(key_s) if t <= key_z[0])
    i1 = max(i for i, t in enumerate(key_s) if t <= key_z[12])
    ks = key_s[i0:i1 + 1]
    ms = [math.floor(t) for t in sh[i0:i1 + 1]]
    n = i1 - i0
    has = [any(ks[i] <= z < ks[i + 1] for z in key_z) for i in range(n)]
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

    # ------------------------------------------------ 卷三十二月离假如三例
    for label, j, h, mi, want in (("第一假如（崇祯四年十月十五日乙卯夜望，食甚丙辰用时 6:51）", 2317083, 6, 51, "日差 24 分加；太阴日差 9′56″ 减；望"),
                                  ("第二假如（崇祯五年三月癸丑夜望，用时 20:14）", 2317260, 20, 14, "日差 11 分加；太阴日差 10 分减；月经大火 14°27′"),
                                  ("第三假如（崇祯四年十月十二日壬子夜，癸丑用时 6:03）", 2317080, 6, 3, "太阴日差 6 分减；月距冬至 100°07′")):
        T = j + (h + mi / 60) / 24
        L, a, _ = moon_shi(T)
        sl = sun_yong(T)
        e = ersan_jun(L - sl, a)
        v = norm(L + e)
        print(label, f"日差 {ri_cha(T) * 1440:+.2f}分 太阴日差 {moon_richa(sl) * 1440:+.2f}分 实平行 {int(L // 30)}宫{L % 30:.3f}° "
              f"日月相距 {norm(L - sl):.3f}° 加二三均实经 {int(v)}°{(v % 1) * 60:.2f}′ | 原文 {want}")

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
        fh.write("# 清初时宪历「西洋新法」照表复原（scripts/calendar_data/shixian_xinfa.py；日躔照《新法算书》卷二十五、二十六，"
                 "月离第一均照卷三十三太阴自行加减表、月离日差表，朔望止用一均，白道经度以黄白二道同升表改黄道）逐月朔日与张表比较：冬至岁 1645—1729。"
                 "用时 = 日用平时加日差、月用用时加月离日差，中气加日差定日；平时 = 一律平时。"
                 "大统 = 1 者为康熙五至八年复用大统之年，不以新法论。1726 年起行《历象考成》，只作对照。\n")
        w = csv.writer(fh)
        w.writerow(["冬至岁", "农历年", "月", "闰", "用时朔日", "平时朔日", "用时合张表", "平时合张表", "大统"])
        w.writerows(rows)
    xf = [r for r in rows if not r[8] and 1645 <= r[1] <= 1725]
    print("新法期", len(xf), "月：用时合张表", sum(r[6] for r in xf), "，平时合", sum(r[7] for r in xf))
    print("用时不合：", [(r[1], ("闰" if r[3] else "") + str(r[2]), r[4]) for r in xf if not r[6]])
    dt = [r for r in rows if r[8]]
    print("康熙五至八年", len(dt), "月：用时合张表", sum(r[6] for r in dt))

