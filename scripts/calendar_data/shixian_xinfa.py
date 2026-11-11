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
- 月离：《新法算书》月离表（卷三十二至三十五）维基文库本只录说明，数字表不存；月离按表推，不能以《历象考成》作图公式代替
  （表的取整、分段、插值与公式不同，近子夜之月可差一日；允禄「其數……仍新法算書西人第穀之舊」只言常数）。
  故本脚本**不推月离、不排月**，只复原日躔并核原文算例。月离表可得途径：文渊阁四库影印本《新法算书》月离诸卷，
  或顺治刊本《西洋新法历书》。下列太阴常数仅记录《清史稿》所载，暂不使用。

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

# ---------------------------------------------------------------- 太阴常数（清史稿卷四十五；月离表未得，暂不使用）
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
