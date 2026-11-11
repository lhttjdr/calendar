"""宋初三历：应天历（王处讷）、乾元历（吴昭素）、仪天历（史序）定朔与排月，照《宋史》卷六十八、六十九（三历同表分栏，
乾元、仪天之异在注文）复原，读法参王应伟《中国古历通解》第六章第一节（书页 510—531）。

读法要点（详见报告与 compendium）：
- 应天岁盈「二十六萬九千三百六十五」当作 730635（王应伟书页 511：5 × 730635 = 3653175 = 岁实之元法分）；元积 = 积年 × 730635，
  「以八十四萬一百六十八去元積，不盡者，半而進位，以元法收爲所盈日」即 (元积 mod 840168) × 5 / 元法。
- 「展」：表中损益率与先后积之差相补——应天日躔一气之变 ≈ 定日 × (100 − 损益准)，月离一日之变 = 1000 − 损益率（后历逐日
  分毫不差，先历据此校出讹字）。本项目读「下以損益率展」为乘其补数（日躔补至百，月离「餘進位」后补至千），此为据表闭合的解释。
- 乾元、仪天之表以逐日（逐气）损益与积闭合校讹（见 FIXES）。
- 进朔：「視朔所入辰分皆與二分相減，餘二收，用減八分之六，其朔定小餘如此；以上者進一日」——辰分读作该日晨分（卷六十九
  步晷漏），二收读作折半；差之正负两读：A 取有号（夏减冬加），B 取绝对值（皆减）。有交不进朔未实现。
- 中气用常气（恒气）；无中气之月为闰。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/song_early_dingshuo.py
"""
import csv
import math
from fractions import Fraction as Fr
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GZ = [("甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12]) for i in range(60)]


def jul_jdn(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def civil_ymd(j):
    """儒略历（1582 年前）"""
    c = j + 32082
    d = (4 * c + 3) // 1461
    e = c - 1461 * d // 4
    m = (5 * e + 2) // 153
    return d - 4800 + m // 10, m + 3 - 12 * (m // 10), e - (153 * m + 2) // 5 + 1


# 讹字校改（据表内逐日/逐气损益与积闭合；王应伟书页 529—531 亦论应天月离损益率「间接」之理）
FIXES = {
    "应天日躔定日": "后半年依对称：大暑 15.8136→15.6947、白露 15.4328→15.2899、秋分「十五」→15.1470；芒种 15.9372→15.9327（盈缩准差）",
    "应天月离": "先三日损 288→278（先四积 2574 证之）；先六日积 3126→3536；先十一日积 3300→3030；先十四日初益 331→231（积 769 须归零）",
    "乾元月离": "十二日阳差 72→692；十二、十三日损 121→211、156→256（陽差 481、225 证之）；十六日阴差 243→343；十八、十九日阴差 750→790、"
                "995→955；十九日益 146→126（二十日阴差 1081 证之）",
    "仪天月离": "疾初六日益 65→75（同迟初）；疾末一日损 356→236、五日损 726→916（积 3946→3710、1982→1066 证之）；"
                "疾末二日积 2710→3710",
}


# ---------------------------------------------------------------- 应天
class Yingtian:
    name = "应天历"
    F = 10002
    QI = Fr(5 * 730635, 24)          # 气策（分）= 3653175/24
    SHUO = Fr(5 * 59073)             # 朔实（分）
    LI_ZHONG = Fr(5 * 551201242, 10000 * 2)   # 历中（分）：离总 55120.1242 ×5 /2
    JI_BASE, Y_BASE = 4825558, 962   # 距建隆三年壬戌积年
    # 日躔：定日（日）、损益准（负为损）、先后积（先 +、后 −）
    DING = [14.5045, 14.6236, 14.7425, 14.8616, 15.0042, 15.1470, 15.2899, 15.4328, 15.5757, 15.6947, 15.8136, 15.9327]
    SUN_RATE = [-64, -69, -76, -82, -89, -97, 97, 89, 83, 78, 72, 66, -65, -72, -77, -83, -89, -97, 97, 89, 82, 75, 70, 64]
    SUN_JI = [-20, 529, 975, 1335, 1606, 1771, 1819, 1780, 1605, 1350, 995, 541, 5, -549, -985, -1346, -1611, -1780, -1831, -1786, -1621,
              -1357, -988, -550]
    # 月离：先历、后历逐日损益率（负为损：|积|增 1000−率；正为益：|积|减 1000−率），七日、十四日 (初率, 末率)
    MOON_XIAN = [-12, -136, -278, -439, -599, -760, (-937, 992), 900, 733, 565, 394, 235, 110, (231, None)]
    MOON_HOU = [-12, -136, -288, -448, -608, -768, (-937, 992), 900, 732, 564, 404, 252, 120, (231, None)]
    CHU7, CHU14 = Fr(8888, 10002), Fr(7774, 10002)
    # 晨分（常气初，冬至起；秋分 2250、白露 2112 依对称作 2350、2212）
    CHEN = [2748, 2735, 2688, 2612, 2508, 2388, 2350, 2212, 1992, 1888, 1812, 1765, 1752, 1765, 1812, 1888, 1992, 2212, 2350, 2388, 2508,
            2612, 2688, 2735]
    CHEN_ERFEN = 2350

    def __init__(self):
        d = self.DING
        self.ding = d + d[::-1]     # 冬至…芒种，夏至…大雪（对称）

    def ji(self, y):
        return self.JI_BASE + (y - self.Y_BASE)

    def dongzhi_mod(self, y):
        """天正冬至（日，自甲子日子正起，mod 60）"""
        yuanji = self.ji(y) * 730635
        return Fr((yuanji % 840168) * 5, self.F) % 60

    def runyu(self, y):
        yuanji = self.ji(y) * 730635
        return Fr((yuanji % 59073) * 5, self.F)

    def ru_li0(self, y):
        """天正经朔入先后历（日）"""
        yuanji = self.ji(y) * 730635
        tong = yuanji % 59073
        z = Fr(yuanji - tong) % Fr(551201242, 10000)
        return z * 5 / self.F

    zhuan = Fr(551201242, 10000) * 5 / 10002

    def sun_def(self, t):
        """t：距天正冬至（日）。求朔弦望入气（以盈缩准去之），次气先后积 ∓ (定日 − 入气日) × (100 − 损益准)，元法收为分"""
        year = Fr(5 * 730635, self.F)
        t = t % year
        acc = 0.0
        k = 0
        while k < 23 and acc + self.ding[k] <= float(t):
            acc += self.ding[k]
            k += 1
        rem = acc + self.ding[k] - float(t)
        rate = self.SUN_RATE[k]
        c = 100 - abs(rate)
        nxt = self.SUN_JI[(k + 1) % 24]
        first = k < 12
        sun_loss = rate < 0
        # 前半年：损减益加；后半年（后为负）：损则 |后| 增，故符号相反
        adj = rem * c
        if first:
            v = nxt - adj if sun_loss else nxt + adj
        else:
            v = nxt + adj if sun_loss else nxt - adj
        return v   # 元法分，先 +

    def moon_def(self, x):
        """x：入转（日，0…转日）。先历 +、后历 −。"""
        lz = float(self.LI_ZHONG / self.F)
        x = float(x) % float(self.zhuan)
        hou = x >= lz
        y = x - lz if hou else x
        rates = self.MOON_HOU if hou else self.MOON_XIAN
        m = 0.0
        k = int(y)
        for j in range(k):
            m += self._day_total(rates[j], j)
        f = y - k
        m += self._partial(rates[k], k, f)
        return -m if hou else m

    def _change(self, r):
        return (1000 - abs(r)) * (1 if r < 0 else -1)

    def _day_total(self, r, j):
        if isinstance(r, tuple):
            a, b = r
            return self._change(a) + (self._change(b) if b is not None else 0)
        return self._change(r)

    def _partial(self, r, j, f):
        if isinstance(r, tuple):
            a, b = r
            chu = float(self.CHU7 if j == 6 else self.CHU14)
            if f <= chu:
                return self._change(a) * f / chu
            return self._change(a) + (self._change(b) * (f - chu) / (1 - chu) if b is not None else 0)
        return self._change(r) * f

    def chen(self, t):
        return chen_interp(self.CHEN, t, float(self.QI) / self.F)

    erfen_chen = CHEN_ERFEN


def chen_interp(tab, t, qi):
    """晨分逐日（常气初值；「各以氣數相減爲分，自雨水後法十六，霜降後法十五，除分爲中率，二率相減，爲合差；半之，加減中率爲初、末率……
    又以元法除合差，爲日差」）：中率 = 相邻气差 / L（雨水至霜降前 16，霜降至雨水前 15），合差 = 次中率 − 本中率，初率 = 中率 − 合差/2，
    日差 = 合差 / L，逐日累积。t 为距天正冬至日数。"""
    t = t % (qi * 24)
    k = int(t // qi)
    n = int(t - k * qi)              # 入气第 n 日（整日）
    L = lambda kk: 16 if 4 <= kk % 24 < 20 else 15
    zl = lambda kk: (tab[(kk + 1) % 24] - tab[kk % 24]) / L(kk)
    he = zl(k + 1) - zl(k)
    chu = zl(k) - he / 2
    rc = he / L(k)
    v = tab[k]
    for j in range(n):
        v += chu + j * rc
    return v


# ---------------------------------------------------------------- 乾元
class Qianyuan:
    name = "乾元历"
    F = 2940
    JI_BASE, Y_BASE = 30543977, 981
    QI = Fr(5 * 214764, 24)
    # 常气阴阳差与损益率（每气之总）
    YY = [0, 170, 303, 404, 475, 518, 532, 518, 475, 404, 303, 170]
    RATE = [170, 133, 101, 71, 43, 14, -14, -43, -71, -101, -133, -170]
    # 二十八日：每日损益（阳差之增为 +），四七日 (初, 末)
    MOON = [287, 250, 213, 173, 134, 93, (46, -6), -62, -102, -141, -193, -211, -256, (-225, -63),
            -280, -242, -205, -165, -126, -84, (-35, 17), 71, 112, 150, 191, 229, 266, (161, None)]
    CHU = {6: Fr(2612, 2940), 13: Fr(2285, 2940), 20: Fr(1958, 2940), 27: Fr(1632, 2940)}
    zhuan = Fr(162021204, 10000) * 5 / 2940
    CHEN = [808, 802, 786, 761, 732, 699, 666, 624, 589, 558, 534, 519, 515, 519, 534, 558, 585, 624, 666, 699, 730, 762, 786, 803]
    erfen_chen = 666

    def ji(self, y):
        return self.JI_BASE + (y - self.Y_BASE)

    def suiji(self, y):
        return self.ji(y) * 214764

    def dongzhi_mod(self, y):
        return Fr((self.suiji(y) % 70560) * 5, self.F) % 60

    def runyu(self, y):
        return Fr((self.suiji(y) % 17364) * 5, self.F)

    def ru_li0(self, y):
        s = self.suiji(y)
        z = Fr(s - s % 17364) % Fr(162021204, 10000)
        return z * 5 / self.F

    def sun_def(self, t):
        """「以其月气节减经朔大小余，即得入气日及分……以其日损益率乘入气日余分，所得，用损益其日阴阳差为定数」：
        常气阴阳差 + 入气日/气策 × 损益率；前半阳（+）、后半阴（−）。"""
        qi = float(self.QI) / self.F
        t = float(t) % (qi * 24)
        k = int(t // qi)
        f = (t - k * qi) / qi
        kk = k % 12
        v = self.YY[kk] + f * self.RATE[kk]
        return v if k < 12 else -v

    def moon_def(self, x):
        x = float(x) % float(self.zhuan)
        k = min(int(x), 27)
        f = x - k
        v = 0.0
        for j in range(k):
            r = self.MOON[j]
            v += (r[0] + (r[1] or 0)) if isinstance(r, tuple) else r
        r = self.MOON[k]
        if isinstance(r, tuple):
            chu = float(self.CHU[k])
            if f <= chu:
                v += r[0] * f / chu
            else:
                v += r[0] + ((r[1] or 0) * (f - chu) / (1 - chu))
        else:
            v += r * f
        return v      # 阳 +、阴 −（积已含号）

    def chen(self, t):
        return chen_interp(self.CHEN, t, float(self.QI) / self.F)


# ---------------------------------------------------------------- 仪天
class Yitian:
    name = "仪天历"
    F = 10100
    JI_BASE, Y_BASE = 716497, 1001
    SUI = 3688970
    HE = 298259
    LI_ZHONG = Fr(2783010165, 10000 * 2)
    zhuan = Fr(2783010165, 10000) / 10100
    XIANG = Fr(69752541, 10000 * 10100) + 6     # 象限 6 日 8975 秒 2541 少 → 用 历中/2
    L1 = 88 + 8899.5 / 10100      # 盈初缩末限日
    L2 = 93 + 7485.5 / 10100      # 缩初盈末限日
    A_SUN = 1836                  # 进退率（昇平积之极）
    UP = [1086, 916, 746, 576, 406, 236, 75]
    DOWN = [86, 236, 406, 576, 746, 916, 1200]
    TOP = 4031
    QIAN_XIAN = 182.62228         # 前限（日）：二至限
    erfen_chen = None

    def ji(self, y):
        return self.JI_BASE + (y - self.Y_BASE)

    def suiji(self, y):
        return self.ji(y) * self.SUI

    def dongzhi_mod(self, y):
        return Fr(self.suiji(y), self.F) % 60

    def runyu(self, y):
        return Fr(self.suiji(y) % self.HE, self.F)

    def ru_li0(self, y):
        s = self.suiji(y)
        z = Fr(s - s % self.HE) % Fr(2783010165, 10000)
        return z / self.F

    def sun_def(self, t):
        """四限：盈初（冬至起 L1）A(2n/L − n²/L²)、盈末 A(1 − (n/L2)²)、缩初、缩末同形取负；整日按「每日盈縮定分」累积，日内线性"""
        t = float(t) % (2 * (self.L1 + self.L2))
        half = self.L1 + self.L2
        neg = t >= half
        u = t - half if neg else t
        La, Lb = (self.L2, self.L1) if neg else (self.L1, self.L2)

        def par_init(n, L):
            return self.A_SUN * (2 * n / L - n * n / (L * L))

        def par_end(n, L):
            return self.A_SUN * (1 - (n / L) ** 2)

        def interp(fn, n, L):
            i = math.floor(n)
            a, b = fn(i, L), fn(min(i + 1, L), L)
            return a + (b - a) * (n - i)
        if u < La:
            v = interp(par_init, u, La)
        else:
            v = interp(par_end, u - La, Lb)
        return -v if neg else v

    def moon_def(self, x):
        lz = float(self.LI_ZHONG) / self.F
        xiang = lz / 2
        x = float(x) % float(self.zhuan)
        chi = x >= lz
        y = x - lz if chi else x
        mo = y >= xiang
        z = y - xiang if mo else y
        j = min(int(z), 6)
        f = z - j
        if not mo:
            v = sum(self.UP[:j]) + self.UP[j] * f
        else:
            v = self.TOP - sum(self.DOWN[:j]) - self.DOWN[j] * f
        return -v if chi else v

    def chen(self, t):
        """「求每日晷漏損益數」「求每日晷漏母」：入前后限日 u（冬至后、夏至后），初象以下为上限，以上反减前限；(100u)² ÷ 昇法/平法
        为分，副之；与五百五分相减相乘，昇法者二千八百五十除、平法者五千五百五十二除，加副，为损益数；春分后加 1768，秋分后减 2777。"""
        t = float(t) % (2 * self.QIAN_XIAN)
        after_xia = t >= self.QIAN_XIAN
        x = t - self.QIAN_XIAN if after_xia else t
        chu = self.L2 if after_xia else self.L1     # 夏至后初象 93.74、冬至后初象 88.88
        if x <= chu:
            u, fa, den = x, (174003 if after_xia else 156428), (5552 if after_xia else 2850)
            chunfen_hou = after_xia
        else:
            u, fa, den = self.QIAN_XIAN - x, (156428 if after_xia else 174003), (2850 if after_xia else 5552)
            chunfen_hou = not after_xia
        p = (100 * u) ** 2 / fa
        sy = p + (505 - p) * p / den
        return 1768 + sy if chunfen_hou else 2777 - sy

    def erfen(self):
        return self.chen(self.L1)


# ---------------------------------------------------------------- 排月
def dongzhi_abs(li, y, ji_shift=0):
    """冬至岁 y 天正冬至（儒略日数带分）：在儒略历 y−1 年 12 月 16 日前后 ±30 日内找干支相合之日"""
    sh = li.ji
    d = li.dongzhi_mod(y + ji_shift)
    base = jul_jdn(y - 1, 12, 16)
    for j in range(base - 30, base + 31):
        if (j + 49) % 60 == int(d):
            return j + float(d - int(d))
    raise ValueError


def sui_shuo(li, y, ji_shift=0, jin="A"):
    """冬至岁 y 自天正经朔起 14 月：(经朔, 定朔, 进朔后朔日 JDN)"""
    dz = dongzhi_abs(li, y, ji_shift)
    run = float(li.runyu(y + ji_shift))
    jing0 = dz - run
    x0 = float(li.ru_li0(y + ji_shift))
    shuo = 29.530585   # 用于入转、入气累加（各历朔实之日）
    if isinstance(li, Yingtian):
        shuo = 5 * 59073 / 10002
    elif isinstance(li, Qianyuan):
        shuo = 29 + 1560 / 2940
    else:
        shuo = 298259 / 10100
    out = []
    for m in range(14):
        jing = jing0 + m * shuo
        t = jing - dz
        sd = li.sun_def(t)
        md = li.moon_def(x0 + m * shuo)
        ding = jing + (sd + md) / li.F
        day = math.floor(ding)
        xiao = ding - day
        lim = jin_limit(li, day - dz, jin)
        if lim is not None and xiao >= lim:
            day += 1
        out.append((jing, ding, day))
    return out, dz


def jin_limit(li, t_day, mode):
    """进朔限（日之分数）：3/4 −（该日晨分 − 二分晨分）/2 / 日法（A 有号）或 3/4 − |…|/2（B）"""
    if mode is None:
        return None
    c = li.chen(t_day)
    e = li.erfen() if isinstance(li, Yitian) else li.erfen_chen
    diff = (e - c) / 2 if mode == "A" else abs(e - c) / 2
    return 0.75 - diff / li.F


def year_months(li, y, ji_shift=0, jin="A"):
    """冬至岁 y：(朔日 JDN, 月序, 闰否)，自天正至次年天正前；中气用常气"""
    ms0, _ = sui_shuo(li, y - 1, ji_shift, jin)
    ms, dz = sui_shuo(li, y, ji_shift, jin)
    ms2, dz2 = sui_shuo(li, y + 1, ji_shift, jin)
    qi = (dz2 - dz) / 24
    zhong = [math.floor(dz + 2 * k * qi) for k in range(13)]
    days = sorted(set([d for _, _, d in ms0] + [d for _, _, d in ms] + [d for _, _, d in ms2]))
    i0 = max(i for i, d in enumerate(days) if d <= zhong[0])
    i1 = max(i for i, d in enumerate(days) if d <= zhong[12])
    seq = days[i0:i1 + 1]
    n = i1 - i0
    has = [any(seq[i] <= z < seq[i + 1] for z in zhong) for i in range(n)]
    leap = next((i for i in range(1, n) if not has[i]), None) if n == 13 else None
    out, mo = [], 11
    for i in range(n):
        if i == leap:
            out.append((seq[i], out[-1][1], True))
            continue
        out.append((seq[i], mo, False))
        mo = mo % 12 + 1
    return out


LIS = {"应天历": Yingtian(), "乾元历": Qianyuan(), "仪天历": Yitian()}
SPANS = {"应天历": (964, 983), "乾元历": (984, 1000), "仪天历": (1001, 1023)}   # 983 续用应天：《宋史》本纪 982 年「閏月戊子朔」在十二月后，合应天


def lunar_year_months(li, n, ji_shift=0, jin="A"):
    """农历年 n：正月至十二月（含闰）"""
    head = year_months(li, n, ji_shift, jin)
    k = next(i for i, e in enumerate(head) if e[1] == 1 and not e[2])
    out = head[k:]
    for e in year_months(li, n + 1, ji_shift, jin):
        if e[1] < 11:
            break
        out.append(e)
    return out


def zhang():
    out = {}
    for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        y, m, d = int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])
        out.setdefault(int(r["lunar_year"]), []).append((jul_jdn(y, m, d), int(r["lunar_month"]), r["is_leap"] == "1"))
    return out


def benji():
    rows = []
    lines = [ln for ln in open(ROOT / "data/calendar/benji_shuo_check.csv", encoding="utf-8") if not ln.startswith("#")]
    for r in csv.DictReader(lines):
        if r.get("书") != "songshi_benji" or r.get("类") == "其他":
            continue
        try:
            y = int(r["年"])
            j = int(r["本纪朔 JDN"])
        except (ValueError, KeyError):
            continue
        if 964 <= y <= 1023:
            m = r["月"]
            leap = m.startswith("闰") or m.startswith("閏")
            rows.append((y, int(m.lstrip("闰閏")), leap, j, r.get("原文", "")))
    return rows


def evaluate(ji_shift=0, jin="A"):
    zh = zhang()
    res = {}
    for name, li in LIS.items():
        y0, y1 = SPANS[name]
        same = tot = 0
        months = {}
        for n in range(y0, y1 + 1):
            ms = lunar_year_months(li, n, ji_shift, jin)
            months[n] = ms
            t = zh.get(n, [])
            tot += len(ms)
            same += sum(1 for e in ms if e in t)
        res[name] = (same, tot, months)
    return res


JIN = "B"   # 进朔读法：B（「皆」取绝对值）本纪合数较 A 多一条，与张表同月多四；A 列入报告


def ymd_s(j):
    y, m, d = civil_ymd(j)
    return f"{y}-{m:02d}-{d:02d}"


if __name__ == "__main__":
    for name, li in LIS.items():
        y = SPANS[name][0]
        dz = dongzhi_abs(li, y)
        print(name, "冬至岁", y, "天正冬至", ymd_s(math.floor(dz)), GZ[(math.floor(dz) + 49) % 60], f"{dz % 1:.3f}")
    bj, zh = benji(), zhang()
    path = ROOT / "data/calendar/song_early_check.csv"
    with open(path, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 宋初应天、乾元、仪天三历复原（scripts/calendar_data/song_early_dingshuo.py）：逐月朔日与张表比较（类=月），"
                 "《宋史》本纪明书朔日逐条检验（类=本纪）；定朔小余与进朔限以时计，进朔读法 B。\n")
        w = csv.writer(fh)
        w.writerow(["类", "历", "农历年", "月", "闰", "推步朔日", "张表朔日", "本纪朔日", "合张表", "合本纪", "定朔小余（时）", "进朔限（时）", "原文"])
        for name, li in LIS.items():
            y0, y1 = SPANS[name]
            info = {}
            for y in range(y0 - 1, y1 + 3):
                out, dz = sui_shuo(li, y, 0, JIN)
                for _, ding, day in out:
                    info[day] = (ding % 1 * 24, jin_limit(li, math.floor(ding) - dz, JIN) * 24)
            same = tot = 0
            months = {}
            for n in range(y0, y1 + 1):
                ms = lunar_year_months(li, n, 0, JIN)
                months[n] = ms
                t = zh.get(n, [])
                for e in ms:
                    tm = [x for x in t if x[1:] == e[1:]]
                    ok = e in t
                    same += ok
                    tot += 1
                    h, lim = info.get(e[0], (None, None))
                    w.writerow(["月", name, n, e[1], int(e[2]), ymd_s(e[0]), ymd_s(tm[0][0]) if tm else "", "", int(ok), "",
                                f"{h:.2f}" if h is not None else "", f"{lim:.2f}" if lim is not None else "", ""])
            hit = hz = nb = 0
            for (y, m, l, j, txt) in bj:
                if not (y0 <= y <= y1):
                    continue
                got = [e for e in months[y] if e[1:] == (m, l)]
                ok = bool(got) and got[0][0] == j
                okz = any(e == (j, m, l) for e in zh.get(y, []))
                nb += 1
                hit += ok
                hz += okz
                h, lim = info.get(got[0][0], (None, None)) if got else (None, None)
                w.writerow(["本纪", name, y, m, int(l), ymd_s(got[0][0]) if got else "", "", ymd_s(j), int(okz), int(ok),
                            f"{h:.2f}" if h is not None else "", f"{lim:.2f}" if lim is not None else "", txt[:40]])
            print(f"{name} {y0}—{y1}：与张表同月 {same}/{tot}；本纪明书朔日 {nb} 条，推步合 {hit}，张表合 {hz}")
    print(path.relative_to(ROOT))
