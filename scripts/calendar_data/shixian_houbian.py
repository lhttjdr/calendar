"""清时宪历《历象考成后编》（雍正癸卯元法）推日躔、月离、节气、合朔，照《清史稿·时宪志》卷五十（雍正癸卯元法上）
与卷四十八（康熙甲子元法中，后编言「同甲子元法」诸条）逐步复原。

角度一律以度为单位的浮点数计算（原法以秒为单位查八线表，浮点精度远高于原表）；中法宫度自冬至起算（= 现代黄经 − 270°）。
日以京师子正为界；「子正」为平子正（日序按平太阳日累计，见卷五十二凌犯视差新法「此言子正者，乃為平子正」）。

步骤（引原文，详注见各函数）：
- 天正冬至：中积 = 周岁 × 积年（积年 = 距元年数 − 1），加气应，满纪法 60 去之，自甲子日子正起算。
- 日躔：平行（年根 = 每日平行 × (1 − 冬至小余)，加每日平行 × 距冬至次日数）；最卑 = 最卑应 + 岁行 × 积年 + 日行 × 距冬至次日数；
  引数 = 平行 − 最卑；以二千万、倍两心差、引数解平三角形得橢圓界角，以小、大半径与引数正切得橢圓差角，界角加减差角为均数。
- 月离：平行、最高、正交（同甲子元）；一、二、三平均得用平行、用最高、用正交；最高本轮均轮得最高实均、本时两心差；
  借平圆引数求实引得初均；二均、三均、末均得白道实行；正交实均、交角减分、距日加分得黄白大距，求升度差得黄道实行。
- 节气：子正实行内插（同甲子元「求节气时刻」）；合朔：本日次日子正日月实行内插（同甲子元「求合朔弦望」）。
- 用时：均数时差（均数变时，反其加减）与升度时差（黄赤经差变时，二分后加、二至后减）相加减为时差总（甲子元「推节气用时法」）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/shixian_houbian.py
"""
import math

# ---------------------------------------------------------------- 用数（卷五十）
ZHOU_SUI = 365.24233442                 # 周岁
SUN_DAILY = 3548.3290897 / 3600         # 太阳每日平行（度）
ZUIBEI_YEAR = 62.9975 / 3600            # 最卑岁行
ZUIBEI_DAY = 0.17248 / 3600             # 最卑日行
SUN_E = 169000.0                        # 两心差（本天大半径一千万）
SUN_B = 9998571.85                      # 本天小半径
ZUIBEI_YING = 8 + 7 / 60 + 32 / 3600 + 22 / 216000   # 最卑应
QI_YING = 32.12254                      # 气应（日）
HUANG_CHI = 23 + 29 / 60                # 黄赤大距

MOON_DAILY = 47435.0234086 / 3600       # 太阴每日平行
GAO_DAILY = 401.070226 / 3600           # 最高每日平行
JIAO_DAILY = 190.63863 / 3600           # 正交每日平行（退行）
SUN_MAX_JUN = 6973 / 3600               # 太阳最大均数
YI_PING = 710 / 3600                    # 太阴最大一平均
GAO_PING = 1196 / 3600                  # 最高最大平均
JIAO_PING = 570 / 3600                  # 正交最大平均
CUBE_GAO = 1051562                      # 太阳最高立方积（以 (r/千万)³ × 一百万计）
CUBE_DA = 101410                        # 太阳高卑立方大较
ER_PING_GAO, ER_PING_BEI = 214 / 3600, 236 / 3600    # 太阳在最高、最卑，太阴最大二平均
SAN_PING = 47 / 3600                    # 太阴最大三平均
GAO_BENLUN, GAO_JUNLUN = 550505.0, 117315.0           # 最高本轮、均轮半径
ER_JUN_GAO, ER_JUN_BEI = 1994 / 3600, 2231 / 3600     # 太阳在最高、最卑，太阴最大二均
SAN_JUN = 145 / 3600                    # 太阴最大三均
# 两最高相距 10°…90°，两弦最大末均（秒）
MO_JUN = [61, 67, 76, 88, 103, 120, 139, 159, 180]
JIAO_BENLUN, JIAO_JUNLUN = 57.5 / 60, 1.5 / 60        # 正交本轮、均轮半径（度）
DA_JU_MAX = 5 + 17 / 60 + 20 / 3600     # 最大黄白大距
DA_JU_BANJIAO = 532.5 / 3600            # 黄白大距半较
JU_RI_JIAFEN_MAX = 163 / 3600           # 最大距日加分
MOON_YING = 5 * 30 + 26 + 27 / 60 + 48 / 3600 + 53 / 216000    # 太阴平行应
GAO_YING = 8 * 30 + 1 + 15 / 60 + 45 / 3600 + 38 / 216000      # 最高应
JIAO_YING = 5 * 30 + 22 + 57 / 60 + 37 / 3600 + 33 / 216000    # 正交应

YUAN_YEAR = 1723        # 雍正元年癸卯：天正冬至在壬寅年（1722）十一月
# 雍正癸卯天正冬至丙申日（干支序 32）：1722-12-22（格里历），儒略日数 2350363（`__main__` 自检）
EPOCH_DZ_JDN = 2350363


def sin(d):
    return math.sin(math.radians(d))


def cos(d):
    return math.cos(math.radians(d))


def tan(d):
    return math.tan(math.radians(d))


def atan2(y, x):
    return math.degrees(math.atan2(y, x))


def norm(d):
    return d % 360.0


def fold(d):
    """入 0—180°（与全周相减用其余）"""
    d = norm(d)
    return d if d <= 180 else 360 - d


def tri_opp(a, b, c_deg):
    """平三角形：两边 a、b 夹角 C，求对 b 之角（度）与第三边"""
    ang = atan2(b * sin(c_deg), a - b * cos(c_deg))
    side = math.sqrt(a * a + b * b - 2 * a * b * cos(c_deg))
    return ang, side


# ---------------------------------------------------------------- 天正冬至
def dongzhi(y):
    """冬至岁 y（天正冬至在 y−1 年十一月）：天正冬至时刻，自 EPOCH 冬至所在日（丙申）前之甲子日子正起算的日数。
    原文：「置周歲，以距元年數減一得積年乘之，得中積分，加氣應得通積分。其日滿紀法去之，餘為天正冬至日分」。"""
    ji_nian = y - YUAN_YEAR
    return ZHOU_SUI * ji_nian + QI_YING, ji_nian


def jdn_of(daynum):
    """自甲子日（EPOCH 冬至前 32 日）子正起算的日序 → 儒略日数"""
    return EPOCH_DZ_JDN - 32 + daynum


def daynum_of(jdn):
    return jdn - (EPOCH_DZ_JDN - 32)


# ---------------------------------------------------------------- 日躔
def sun(t, y):
    """时刻 t（日序，平时）、冬至岁 y：返回 (平行, 引数, 均数, 实行, 日距地心/千万)。
    平行自天正冬至（平）起算，「以周日為一率，太陽每日平行為二率，天正冬至小餘與周日相減餘為三率，求得四率為年根」。"""
    dz, ji_nian = dongzhi(y)
    ping = norm(SUN_DAILY * (t - dz))
    next_zi = math.floor(dz) + 1
    zuibei = ZUIBEI_YING + ZUIBEI_YEAR * ji_nian + ZUIBEI_DAY * (t - next_zi)
    yin = norm(ping - zuibei)
    jun = sun_jun(yin)
    shi = norm(ping + jun)
    # 日距地心（「求用平行」条）：实引 = 引数 ± 均数
    shiyin = norm(yin + jun)
    d2 = 2 * SUN_E
    fen_gu = d2 * cos(shiyin)
    gou = d2 * sin(shiyin)
    s = 2e7 + fen_gu                # 实引三宫内九宫外加，三宫外九宫内减：即 2e7 + 2e·cos
    xian = (s * s + gou * gou) / (2 * s)
    r = 2e7 - xian
    return ping, yin, jun, shi, r / 1e7


def sun_jun(yin):
    """日躔均数（卷五十「求實行」）：「以二千萬為一邊，倍兩心差為一邊，引數為所夾之角……求得對倍兩心差之角，倍之為橢圓界角。
    又以本天小半徑為一率，大半徑為二率，前所夾角正切為三率，求得四率為橢圓之正切，檢表得度分秒。與引數相減，餘為橢圓差角。
    最卑前後各三宮與橢圓界角相加，最高前後各三宮與橢圓界角相減……為均數」；引数初宫至五宫为加，六宫至十一宫为减。"""
    m = fold(yin)
    alpha, _ = tri_opp(2e7, 2 * SUN_E, m)
    jie = 2 * alpha
    x = atan2(1e7 * sin(m), SUN_B * cos(m))     # tan x = (大/小) tan m，同象限
    cha = x - m                                  # 最卑前后（m < 90）为正，最高前后为负：界角加减差角
    jun = jie + cha
    return jun if norm(yin) <= 180 else -jun


# ---------------------------------------------------------------- 月离
def moon(t, y):
    """时刻 t（日序，平时）：返回 dict（用平行、初实行、白道实行、黄道实行等）。平行诸应皆为雍正癸卯天正冬至次日子正之数，
    「置中積分，加氣應小餘，減天正冬至小餘，得積日……又置太陰每日平行，以距天正冬至次日數乘之」，即自该子正起之平日数。"""
    dz0, _ = dongzhi(YUAN_YEAR)
    n = t - (math.floor(dz0) + 1)
    ping = norm(MOON_YING + MOON_DAILY * n)
    gao = norm(GAO_YING + GAO_DAILY * n)
    jiao = norm(JIAO_YING - JIAO_DAILY * n)
    s_ping, s_yin, s_jun, s_shi, s_r = sun(t, y)
    # 求用平行（卷五十末「求用平行」）
    k = abs(s_jun) / SUN_MAX_JUN
    sg = 1 if s_jun >= 0 else -1
    yi_ping = YI_PING * k * sg
    ping2 = ping - yi_ping                      # 太阴与太阳相反
    gao_u = gao + GAO_PING * k * sg             # 最高与太阳同
    jiao_u = jiao - JIAO_PING * k * sg          # 正交与太阳相反
    ri_gao = norm(s_shi - gao_u)                # 日距月最高
    ri_jiao = norm(s_shi - jiao_u)              # 日距正交
    cube = CUBE_GAO - (s_r ** 3) * 1e6          # 本时立方较
    s2 = sin(2 * ri_gao)
    er_gao, er_bei = ER_PING_GAO * abs(s2), ER_PING_BEI * abs(s2)
    er_ping = er_gao + (er_bei - er_gao) * cube / CUBE_DA
    er_ping = -er_ping if norm(2 * ri_gao) < 180 else er_ping      # 倍度不及半周为减
    san_ping = SAN_PING * abs(sin(2 * ri_jiao))
    san_ping = -san_ping if norm(2 * ri_jiao) < 180 else san_ping
    yong = norm(ping2 + er_ping + san_ping)
    # 求初实行：最高实均与本时两心差
    two = norm(2 * ri_gao)
    shi_jun, e_m = tri_opp(GAO_BENLUN, GAO_JUNLUN, 180 - fold(two))
    shi_jun = shi_jun if two < 180 else -shi_jun
    gao_shi = norm(gao_u + shi_jun)
    yin = norm(yong - gao_shi)                  # 太阴引数（自最高起）
    m = fold(yin)
    c = 180 - m
    beta, _ = tri_opp(1e7, e_m, c)
    c2 = c + beta
    q_ang, _ = tri_opp(1e7, e_m, c2)             # 对两心差之角
    ping_yuan = 180 - c2 - q_ang                 # 对半径千万之角：平圆引数
    ratio = math.sqrt(1 - (e_m / 1e7) ** 2)      # 「本時兩心差為正弦，對錶取餘弦」
    shi_yin = atan2(ratio * sin(ping_yuan), cos(ping_yuan))
    chu_jun = m - shi_yin                        # 初均数（以绝对值言）
    chu = norm(yong - chu_jun) if norm(yin) <= 180 else norm(yong + chu_jun)   # 引数初宫至五宫为减
    # 求白道实行
    ju_ri = norm(chu - s_shi)
    s2r = sin(2 * ju_ri)
    e_gao, e_bei = ER_JUN_GAO * abs(s2r), ER_JUN_BEI * abs(s2r)
    er_jun = e_gao + (e_bei - e_gao) * cube / CUBE_DA
    er_jun = er_jun if norm(2 * ju_ri) < 180 else -er_jun       # 倍度不及半周为加
    shi_ju_ri = norm(ju_ri + er_jun)
    ri_zuigao = norm(s_ping - s_yin + 180)       # 太阳最卑平行加减六宫 = 日最高
    h = norm(gao_shi - ri_zuigao)                # 日月最高相距
    zong = norm(shi_ju_ri + h)
    san_jun = SAN_JUN * abs(sin(zong))
    san_jun = san_jun if zong < 180 else -san_jun
    mo_max = mo_jun_max(h)
    mo = mo_max * abs(sin(shi_ju_ri))
    mo = -mo if shi_ju_ri < 180 else mo           # 实月距日不及半周为减
    bai = norm(chu + er_jun + san_jun + mo)
    # 求黄道实行
    two_j = norm(2 * ri_jiao)
    ext = two_j if two_j < 180 else two_j - 180
    kk = (JIAO_BENLUN - JIAO_JUNLUN) / (JIAO_BENLUN + JIAO_JUNLUN)
    half_sum = ext / 2
    half_diff = math.degrees(math.atan(kk * tan(half_sum)))
    jiao_shijun = half_sum - half_diff
    jiao_shi = norm(jiao_u + jiao_shijun) if two_j < 180 else norm(jiao_u - jiao_shijun)
    ju_jiao = norm(bai - jiao_shi)
    zheng_shi = 1 - cos(min(two_j, 360 - two_j))               # 正矢（半径为一）
    jian = DA_JU_BANJIAO * zheng_shi
    jia_cha = JU_RI_JIAFEN_MAX / 2 * zheng_shi
    two_r = norm(2 * shi_ju_ri)
    jia_fen = jia_cha / 2 * (1 - cos(min(two_r, 360 - two_r)))
    da_ju = DA_JU_MAX - jian + jia_fen
    hd = atan2(cos(da_ju) * sin(ju_jiao), cos(ju_jiao))
    sheng = norm(hd - ju_jiao + 180) - 180
    huang = norm(bai + sheng)
    return dict(ping=ping, yong=yong, chu=chu, bai=bai, huang=huang, sun=s_shi, sun_jun=s_jun, e_m=e_m,
                chu_jun=chu_jun, er_jun=er_jun, san_jun=san_jun, mo=mo, jiao_shi=jiao_shi, da_ju=da_ju, sheng=sheng)


def mo_jun_max(h):
    """两弦最大末均：两最高相距 10°…90° 列表，「用中比例」；相距以与半周、全周相减之余入表（对称）。十度以下以十度之数
    与零度连线（原表未列零度，此为本项目假定，零度取 0 合「日月最高同度……止有三均」）。"""
    x = fold(h)
    x = x if x <= 90 else 180 - x
    if x <= 10:
        return MO_JUN[0] / 3600 * x / 10
    i = min(int(x // 10), 8)
    lo = MO_JUN[i - 1]
    hi = MO_JUN[min(i, 8)]
    return (lo + (hi - lo) * (x - 10 * i) / 10) / 3600


# ---------------------------------------------------------------- 节气、合朔、用时
def sui_of(t):
    """时刻 t 所属冬至岁"""
    y = YUAN_YEAR + int(math.floor((t - QI_YING) / ZHOU_SUI))
    while dongzhi(y + 1)[0] <= t:
        y += 1
    while dongzhi(y)[0] > t:
        y -= 1
    return y


def sun_at(t):
    return sun(t, sui_of(t))


def moon_at(t):
    return moon(t, sui_of(t))


def shi_cha(t):
    """时差总（日）：均数时差（均数变时，反其加减）+ 升度时差（黄、赤经差变时，二分后加、二至后减）。用时 = 平时 + 时差总。"""
    _, _, jun, shi, _ = sun_at(t)
    lam = norm(shi + 270)                       # 现代黄经
    ra = norm(atan2(cos(HUANG_CHI) * sin(lam), cos(lam)))
    d = norm(lam - ra + 180) - 180
    return (-jun + d) / 360


def jieqi_times(y):
    """冬至岁 y 的二十四气平时时刻（自天正冬至起，k = 0…24，日躔实行 k×15°）。「皆以子正日躔未交節氣宮度者，為交節氣本日……
    以本日實行與次日實行相減為一率，每日刻下分為二率，本日子正實行與節氣宮度相減為三率」"""
    out = []
    dz, _ = dongzhi(y)
    d = math.floor(dz) - 2
    lam_d = sun_at(d)[3]
    for k in range(25):
        target = (k * 15) % 360
        while True:
            lam_n = sun_at(d + 1)[3]
            a = norm(lam_d - target + 180) - 180
            b = norm(lam_n - target + 180) - 180
            if a < 0 <= b:
                out.append(d + (-a) / (b - a))
                break
            d, lam_d = d + 1, lam_n
    return out


def shuo_times(t0, t1):
    """[t0, t1) 内各合朔平时时刻（日月黄道实行同度；两子正内插，同甲子元「求合朔弦望」）"""
    out = []
    d = math.floor(t0)
    m = moon_at(d)
    e_d = norm(m["huang"] - m["sun"] + 180) - 180
    while d < t1:
        m = moon_at(d + 1)
        e_n = norm(m["huang"] - m["sun"] + 180) - 180
        if e_d < 0 <= e_n:
            out.append(d + (-e_d) / (e_n - e_d))
        d, e_d = d + 1, e_n
    return out


def year_months(y, yongshi_shuo=True, yongshi_qi=True):
    """冬至岁 y：(朔日 JDN, 月序, 闰否) 列表，自天正（含冬至之月）至次年天正前。
    「以前後兩年有冬至之月為準。中積十三月者，以無中氣之月，從前月置閏。一歲中兩無中氣者，置在前無中氣之月為閏。」"""
    q0, q1 = jieqi_times(y), jieqi_times(y + 1)
    zhong = [q0[k] for k in range(0, 24, 2)] + [q1[0]]
    conv = (lambda t: t + shi_cha(t)) if yongshi_qi else (lambda t: t)
    zhong_day = [math.floor(conv(t)) for t in zhong]
    sh = shuo_times(q0[0] - 32, q1[0] + 32)
    convs = (lambda t: t + shi_cha(t)) if yongshi_shuo else (lambda t: t)
    days = [math.floor(convs(t)) for t in sh]
    i0 = max(i for i, dd in enumerate(days) if dd <= zhong_day[0])
    i1 = max(i for i, dd in enumerate(days) if dd <= zhong_day[12])
    ms = days[i0:i1 + 1]
    n = i1 - i0
    has = [any(ms[i] <= z < ms[i + 1] for z in zhong_day) for i in range(n)]
    leap_at = None
    if n == 13:
        leap_at = next(i for i in range(1, n) if not has[i])
    out, mo = [], 11
    for i in range(n):
        if i == leap_at:
            out.append((jdn_of(ms[i]), out[-1][1], True))
            continue
        out.append((jdn_of(ms[i]), mo, False))
        mo = mo % 12 + 1
    return out


def self_check():
    """与志文所载极值对照：日躔最大均数 1°56′12″；月离最大初均 中距 4°57′57″、高卑 7°39′33″；正交最大实均 1°29′42″；
    雍正癸卯天正冬至丙申日丑正三刻十一分。"""
    dz, _ = dongzhi(YUAN_YEAR)
    out = [("雍正癸卯天正冬至", f"{GZ[int(dz) % 60]} {(dz % 1) * 24:.4f} 时", "丙申 丑正三刻十一分（2.933—2.950 时）")]
    mx = max(abs(sun_jun(a / 10)) for a in range(3600))
    out.append(("日躔最大均数", f"{mx * 60:.2f}′", "116.20′（1°56′12″）"))
    for e, want in ((433190.0, "4°57.95′"), (667820.0, "7°39.55′")):
        best = 0
        for a in range(1800):
            m = a / 10
            c = 180 - m
            beta, _ = tri_opp(1e7, e, c)
            q_ang, _ = tri_opp(1e7, e, c + beta)
            py = 180 - (c + beta) - q_ang
            best = max(best, m - atan2(math.sqrt(1 - (e / 1e7) ** 2) * sin(py), cos(py)))
        out.append((f"最大初均（两心差 {e:.0f}）", f"{int(best)}°{(best % 1) * 60:.2f}′", want))
    k = (JIAO_BENLUN - JIAO_JUNLUN) / (JIAO_BENLUN + JIAO_JUNLUN)
    mj = max(x / 100 - math.degrees(math.atan(k * tan(x / 100))) for x in range(9000))
    out.append(("正交最大实均", f"{mj * 60:.2f}′", "89.70′（1°29′42″）"))
    return out


GZ = [("甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12]) for i in range(60)]


def ymd(j):
    a = j + 32044
    b = (4 * a + 3) // 146097
    c = a - 146097 * b // 4
    d = (4 * c + 3) // 1461
    e = c - 1461 * d // 4
    m = (5 * e + 2) // 153
    return f"{100 * b + d - 4800 + m // 10}-{m + 3 - 12 * (m // 10):02d}-{e - (153 * m + 2) // 5 + 1:02d}"


if __name__ == "__main__":
    import csv
    from pathlib import Path
    root = Path(__file__).resolve().parents[2]
    for name, got, want in self_check():
        print(f"{name}: {got}（志文 {want}）")

    def jdn(y, m, d):
        a = (14 - m) // 12
        yy, mm = y + 4800 - a, m + 12 * a - 3
        return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045
    zh = {}
    for r in csv.DictReader(open(root / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        zh[jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))] = (int(r["lunar_month"]), r["is_leap"] == "1")
    y0, y1 = 1724, 1913
    path = root / "data/calendar/shixian_houbian_check.csv"
    n = ny = npg = 0
    with open(path, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 《历象考成后编》复原（scripts/calendar_data/shixian_houbian.py）逐月朔日与张表比较：冬至岁 1724—1913。"
                 "用时 = 合朔、中气化为视太阳时（京师）定日；平时 = 不化。1742 年以前行用《历象考成》，此处只作对照。\n")
        w = csv.writer(fh)
        w.writerow(["冬至岁", "月", "闰", "用时朔日", "平时朔日", "用时合张表", "平时合张表"])
        for y in range(y0, y1 + 1):
            for (ja, m, l), (jb, _, _) in zip(year_months(y, True, True), year_months(y, False, False)):
                a, b = zh.get(ja) == (m, l), zh.get(jb) == (m, l)
                n += 1
                ny += a
                npg += b
                w.writerow([y, m, int(l), ymd(ja), ymd(jb), int(a), int(b)])
    print(path.relative_to(root), f"{n} 月：用时合张表 {ny}，平时合张表 {npg}")
