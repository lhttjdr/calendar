"""《崇禎曆書·五緯曆指》卷九崇禎七—九年（1634—1636）五星記錄逐条检验：新法、大統推算与部监实测对 DE406。

录文：data/phenomena/chongzhen_wuwei_observations.csv（18 条，书页 309—313，《故宫珍本丛刊》第 382 册）。
历表：vendor/jpl/de406/de406.bsp（skyfield load_file；ΔT 用 skyfield 内置长期表）。恒星：data/stars/hipparcos_bright.csv
（I/311，含自行视差）；积尸气（M44）取星团中心 RA 8h40m24s、Dec +19°40′（J2000），不计自行。
时刻：北京（116.4°E、39.9°N）地方真太阳时；曆局用九十六刻制，一时辰八刻（初、正各四刻），卯正二刻 = 06:30。
坐标（皆地心视位置、当日真春分点）：
  新法黄道：黄经 360°；宫名十二宫每宫 30°（鶉火 = 120°—150°，析木 = 240°—270°）；黄道宿度 = 黄经 − 距星黄经。
  新法赤道宿度：赤经 − 距星赤经（360°）；「出極一直線」= 同赤经；「距夏至」= 赤经 − 90°。
  大統宿度：授时—大统黄道宿度沿赤经圈投影（古黄道），即 行星黄经 − 距星赤经圈与黄道交点之黄经，化古度（×365.25/360）；
    兼给赤道古度（赤经差 ×365.25/360）备参。距星：鬼 θ Cnc（HIP 41822）、張 υ¹ Hya（48356）、角 α Vir（65474）、軫 γ Crv（59803），
    清代与明以前相同（data/sky_culture/chinese_lunar_mansion_determinatives.csv）。
见伏：太阳在地平下 7° 时行星地平高度 ≥ 阈值（水星 5°、木星 3°、金星 2°）记为可见。此为粗判据，只用来定见伏日期的大致所在。

用法（仓库根）：uv run --no-project --with skyfield python -I scripts/phenomena/chongzhen_wuwei_check.py
输出：doc/astronomy/chongzhen-wuwei-checks.md
"""
import csv
import datetime as dt
import math
import statistics
import sys
from pathlib import Path

import numpy as np
from skyfield.api import Star, load, load_file, wgs84
from skyfield.framelib import ecliptic_frame

LON, LAT = 116.4, 39.9
GU = 365.25 / 360.0
ts = load.timescale(builtin=True)
eph = load_file("vendor/jpl/de406/de406.bsp")
earth = eph[399]
site = earth + wgs84.latlon(LAT, LON)
BODY = {"水": eph[1], "金": eph[2], "火": eph[4], "木": eph[5], "土": eph[6], "日": eph[10], "月": eph[301]}


def load_stars():
    out = {}
    for line in open("data/stars/hipparcos_bright.csv", encoding="utf-8"):
        if line[0] in "#h":
            continue
        p = line.split(",")
        out[int(p[0])] = Star(ra_hours=float(p[1]) / 15, dec_degrees=float(p[2]), ra_mas_per_year=float(p[4]),
                              dec_mas_per_year=float(p[5]), parallax_mas=float(p[3]), epoch=ts.tt_jd(2448349.0625))
    return out


ST = load_stars()
STAR = {"鬼": ST[41822], "鬼二": ST[41909], "鬼三": ST[42806], "鬼四": ST[42911], "張": ST[48356], "角": ST[65474],
        "軫": ST[59803], "軒轅": ST[49669], "天江四": ST[85340],
        "積尸": Star(ra_hours=8 + 40 / 60 + 24 / 3600, dec_degrees=19 + 40 / 60)}


def obj(name):
    return BODY.get(name) or STAR[name]


# ---------- 时间 ----------
def ut_of(date, hours):
    """北京地方真太阳时 date（datetime.date）hours 时 → skyfield Time。"""
    base = ts.ut1(date.year, date.month, date.day, hours - LON / 15)
    for _ in range(2):
        app = earth.at(base).observe(BODY["日"]).apparent()
        ra, _, _ = app.radec(epoch="date")
        ha = (base.gast * 15 + LON - ra._degrees) % 360  # 太阳时角
        lat_ = (ha / 15 + 12) % 24  # 地方真太阳时
        lmt = (base.ut1 - 0.5 - math.floor(base.ut1 - 0.5)) * 24 + LON / 15
        eot = ((lat_ - lmt + 12) % 24) - 12
        base = ts.ut1(date.year, date.month, date.day, hours - eot - LON / 15)
    return base


def local_str(t):
    """Time → 北京地方真太阳时字符串（近似：平时 + 时差）。"""
    app = earth.at(t).observe(BODY["日"]).apparent()
    ra, _, _ = app.radec(epoch="date")
    ha = (t.gast * 15 + LON - ra._degrees) % 360
    hours = (ha / 15 + 12) % 24
    jd_loc = t.ut1 + LON / 360
    d = dt.date.fromordinal(int(math.floor(jd_loc + 0.5)) - 1721425)
    # 真太阳时在日界附近可能与平时差一日，按平时日期修正
    lmt = ((jd_loc + 0.5) % 1) * 24
    if lmt - hours > 12:
        d += dt.timedelta(1)
    elif hours - lmt > 12:
        d -= dt.timedelta(1)
    return f"{d.isoformat()} {int(hours):02d}:{int(round((hours % 1) * 60)) % 60:02d}"


def local_days(t):
    """Time → 以北京地方平时计的连续日数（用于日数差）。"""
    return t.ut1 + LON / 360


def date_days(date, hours=12.0):
    return ut_of(date, hours).ut1 + LON / 360


# ---------- 坐标 ----------
def apparent(name, t, topo=False):
    o = site if topo else earth
    a = o.at(t).observe(obj(name)).apparent()
    lat, lon, _ = a.frame_latlon(ecliptic_frame)
    ra, dec, _ = a.radec(epoch="date")
    return {"lon": lon.degrees, "lat": lat.degrees, "ra": ra._degrees, "dec": dec.degrees, "vec": a}


def wrap(x):
    return (np.asarray(x) + 180) % 360 - 180


def obliquity(t):
    T = (t.tt - 2451545.0) / 36525
    return 23.439291 - 0.0130042 * T


def foot_lon(ra, t):
    """赤经圈与黄道交点之黄经（古黄道宿度的距星所在）。"""
    e = math.radians(obliquity(t))
    return math.degrees(math.atan2(math.sin(math.radians(ra)), math.cos(math.radians(ra)) * math.cos(e))) % 360


def lodge(name, mansion, t):
    """返回 新法黄道宿度（360°）、新法赤道宿度（360°）、古黄道宿度（古度）、赤道古度。"""
    p, s = apparent(name, t), apparent(mansion, t)
    hd = float(wrap(p["lon"] - s["lon"]))
    cd = float(wrap(p["ra"] - s["ra"]))
    gh = float(wrap(p["lon"] - foot_lon(s["ra"], t))) * GU
    return {"黄": hd, "赤": cd, "古黄": gh, "赤古": cd * GU}


def times(t0, t1, step_h=1.0):
    n = int((t1 - t0) * 24 / step_h) + 1
    return ts.ut1_jd(t0 + np.arange(n) * step_h / 24)


def root(f, t0, t1, step_h=1.0):
    """在 [t0,t1]（UT1 JD）内找 f 变号处（线性插值），返回全部根的 JD 列表。"""
    tt = times(t0, t1, step_h)
    y = np.asarray(f(tt))
    out = []
    for i in range(len(y) - 1):
        if y[i] == 0 or (y[i] < 0) != (y[i + 1] < 0):
            if abs(y[i] - y[i + 1]) > 90:
                continue
            a, b = tt.ut1[i], tt.ut1[i + 1]
            out.append(a + (b - a) * y[i] / (y[i] - y[i + 1]))
    return out


def lon_diff(a, b):
    return lambda tt: wrap(apparent(a, tt)["lon"] - apparent(b, tt)["lon"])


def ra_diff(a, b):
    return lambda tt: wrap(apparent(a, tt)["ra"] - apparent(b, tt)["ra"])


def side_of_line(name, s1, s2):
    """行星在过两星之大圆哪一侧（以当日赤道单位矢量叉积判断）。"""
    def f(tt):
        p = apparent(name, tt)["vec"].position.au
        a = apparent(s1, tt)["vec"].position.au
        b = apparent(s2, tt)["vec"].position.au
        n = np.cross(a, b, axis=0)
        p = p / np.linalg.norm(p, axis=0)
        return np.degrees(np.arcsin(np.sum(n * p, axis=0) / np.linalg.norm(n, axis=0)))
    return f


def sep(a, b, tt, topo=False):
    pa, pb = apparent(a, tt, topo)["vec"], apparent(b, tt, topo)["vec"]
    return pa.separation_from(pb).degrees


def daily_motion(name, t):
    a = apparent(name, ts.ut1_jd(t.ut1 - 0.5))["lon"]
    b = apparent(name, ts.ut1_jd(t.ut1 + 0.5))["lon"]
    return float(wrap(b - a))


# ---------- 见伏 ----------
def twilight_alt(name, date, evening, depression=None):
    """太阳在地平下 depression 度（缺省取该行星之视界弧 ARCUS）时（晨或昏）行星几何高度；返回 (高度, 距角)。"""
    if depression is None:
        depression = ARCUS[name]
    hrs = np.arange(15.0, 23.0, 2 / 60) if evening else np.arange(2.0, 9.0, 2 / 60)
    t0 = ut_of(date, 0).ut1
    tt = ts.ut1_jd(t0 + hrs / 24)
    sun = site.at(tt).observe(BODY["日"]).apparent().altaz()[0].degrees
    idx = np.where(np.diff(np.sign(sun + depression)) != 0)[0]
    if len(idx) == 0:
        return None
    i = idx[0]
    t = ts.ut1_jd(tt.ut1[i])
    alt = site.at(t).observe(obj(name)).apparent().altaz()[0].degrees
    el = sep(name, "日", t)
    return float(alt), float(el)


# 见伏判据：太阳在地平下 h 度（视界弧）时行星几何高度 ≥ 2° 为可见。视界弧取宽、严两档（宽：金 4°、木 7°、水 8°；
# 严：金 6°、木 10°、水 11°，后者近托勒密所用值），两档之间的日期即 DE406 所定见伏日之不定范围。
ARCUS = {"宽": {"水": 8.0, "木": 7.0, "金": 4.0}, "严": {"水": 11.0, "木": 10.0, "金": 6.0}}
VIS_ALT = 2.0


def is_visible(name, date, evening, kind):
    r = twilight_alt(name, date, evening, ARCUS[kind][name])
    return r is not None and r[0] >= VIS_ALT


def window(name, d0, d1, evening, kind):
    """[d0,d1] 内按某档判据可见之日的最长连续段 (起, 止)；无则 None。"""
    seg, cur, d = [], None, d0
    while d <= d1:
        v = is_visible(name, d, evening, kind)
        if v and cur is None:
            cur = [d, d]
        elif v:
            cur[1] = d
        elif cur:
            seg.append(tuple(cur)); cur = None
        d += dt.timedelta(1)
    if cur:
        seg.append(tuple(cur))
    return max(seg, key=lambda x: (x[1] - x[0]).days) if seg else None


def judge(name, date, evening, claim_visible):
    """实测「见」或「不见」对 DE406：严档亦可见 / 宽档亦不可见 为合，两档之间为两可。"""
    s, w = is_visible(name, date, evening, "严"), is_visible(name, date, evening, "宽")
    if claim_visible:
        return True if s else ("两可" if w else False)
    return True if not w else ("两可" if not s else False)


# ---------- 记录 ----------
D = dt.date
O = dt.date.toordinal
items = []  # 数值子项：序、项目、新法、大统、DE406、新法误差、大统误差、单位、实测、实测判定
notes = {}  # 序 → 按语


def iv_err(x, iv):
    lo, hi = iv
    return 0.0 if lo <= x <= hi else (x - hi if x > hi else x - lo)


def item(no, what, unit, de, xf=None, dt_=None, obs=None, obs_ok=None, xf_txt=None, dt_txt=None, de_txt=None, cat=None, de_iv=None):
    if obs_ok is not None and not isinstance(obs_ok, str):
        obs_ok = bool(obs_ok)
    if de_iv is not None:
        e_x = None if xf is None else iv_err(xf, de_iv)
        e_d = None if dt_ is None else iv_err(dt_, de_iv)
        items.append({"序": no, "类": cat or "见伏", "项目": what, "单位": unit, "新法": xf_txt or "", "大统": dt_txt or "",
                      "DE406": de_txt, "新法误差": e_x, "大统误差": e_d, "实测": obs or "", "实测判定": obs_ok})
        return
    if cat is None:
        cat = "见伏" if ("见" in what or "見" in what) and unit == "日" else ("时刻" if unit == "日" else ("位置" if unit in ("度", "古度") else "其他"))
    items.append({"序": no, "类": cat, "项目": what, "单位": unit, "新法": xf_txt if xf_txt is not None else ("" if xf is None else f"{xf:.2f}"),
                  "大统": dt_txt if dt_txt is not None else ("" if dt_ is None else f"{dt_:.2f}"),
                  "DE406": de_txt if de_txt is not None else f"{de:.2f}",
                  "新法误差": None if xf is None else xf - de, "大统误差": None if dt_ is None else dt_ - de,
                  "实测": obs or "", "实测判定": obs_ok})


def fmt_err(v, unit):
    if v is None:
        return ""
    return f"{v:+.2f}"


def conj(no, a, b, d_xf, h_xf, d_dt, latdiff_xf):
    """会合：DE406 黄经相等时刻；新法时刻误差（日）、大统日期误差（日，按当日正午）、新法纬差误差。"""
    t0 = ut_of(d_xf, h_xf)
    rs = root(lon_diff(a, b), t0.ut1 - 12, t0.ut1 + 12)
    tc = ts.ut1_jd(min(rs, key=lambda r: abs(r - t0.ut1)))
    pa, pb = apparent(a, tc), apparent(b, tc)
    de_days = local_days(tc)
    item(no, f"{a}{b}同度（黄经合）时刻", "日", de_days, xf=local_days(t0), dt_=date_days(d_dt),
         xf_txt=f"{d_xf} {h_xf:05.2f}h", dt_txt=f"{d_dt}", de_txt=local_str(tc))
    item(no, f"合时南北相距（{a}纬 − {b}纬 绝对值）", "度", abs(pa["lat"] - pb["lat"]), xf=latdiff_xf)
    return tc


def run():
    # 1—3 七年九月三会合
    tc1 = conj(1, "火", "土", D(1634, 10, 25), 18.0, D(1634, 10, 28), 1 + 54 / 60)
    t = ut_of(D(1634, 10, 25), 18.5)
    s = float(sep("火", "天江四", t))
    item(1, "九月初四昏 火星距天江四（44 Oph）", "度", s, obs="天江第四星為火星所掩不見", obs_ok=s < 1.0)
    notes[1] = (f"DE406 火土黄经合在 {local_str(tc1)}（初五夜至初六晨），新法早一日余，大統晚一日半。"
                f"初四昏火星距天江四 {s:.2f}°，天江四仅 4.2 等，图注「為火星所掩不見」当是为火星光芒所掩。")
    tc = conj(2, "金", "土", D(1634, 10, 28), 6.5, D(1634, 10, 27), 3.5)
    notes[2] = f"DE406 金土黄经合在 {local_str(tc)}，新法早约十九小时，大統早一日半；纬差新法 3°30′ 与 DE406 几乎全合。"
    tc = conj(3, "金", "火", D(1634, 11, 1), 19.0, D(1634, 10, 24), 1.5)
    notes[3] = f"DE406 金火黄经合在 {local_str(tc)}，新法晚约十五小时，大統早七日余。"

    # 4 閏八月二十四 木犯积尸
    t4 = ut_of(D(1634, 10, 15), 4.0)
    s4 = float(sep("木", "積尸", t4))
    rs = root(lon_diff("木", "積尸"), t4.ut1 - 30, t4.ut1 + 30)
    tt = times(t4.ut1 - 40, t4.ut1 + 40, 6)
    ss = sep("木", "積尸", tt)
    tmin = ts.ut1_jd(tt.ut1[int(np.argmin(ss))])
    item(4, "閏八月二十四日寅时 木星距積尸氣", "度", s4, xf_txt="犯（<1°）", obs="與本局所推悉合（總述）", obs_ok=s4 < 1.0)
    item(4, "木星与積尸氣黄经同度日", "日", local_days(ts.ut1_jd(rs[0])), xf=date_days(D(1634, 10, 15)),
         xf_txt="1634-10-15", de_txt=local_str(ts.ut1_jd(rs[0])))
    notes[4] = (f"木星此时顺行入鬼，DE406 黄经与積尸同度在 {local_str(ts.ut1_jd(rs[0]))}，最近 {float(ss.min()):.2f}°（{local_str(tmin)}）；"
                f"新法犯日早于黄经同度二日余，然是日木星距積尸已不足一度。")

    # 5 九月初一 木在鬼宿
    t5 = ut_of(D(1634, 10, 22), 4.0)
    L = lodge("木", "鬼", t5)
    item(5, "九月初一 木星鬼宿度（新法：黄道 360°）", "度", L["黄"], xf=2.0, xf_txt="二度有奇（≥2）")
    item(5, "九月初一 木星鬼宿度（大統：古黄道，古度）", "古度", L["古黄"], dt_=0.0)
    r_xf = root(lambda tt: wrap(apparent("木", tt)["lon"] - apparent("鬼", tt)["lon"]), t5.ut1 - 40, t5.ut1 + 5)
    # 古黄道入鬼：行星黄经 = 鬼距星赤经圈交黄道之黄经
    def f_gu(tt):
        p = apparent("木", tt)["lon"]
        s = apparent("鬼", tt)["ra"]
        fl = np.degrees(np.arctan2(np.sin(np.radians(s)), np.cos(np.radians(s)) * np.cos(np.radians(obliquity(tt))))) % 360
        return wrap(p - fl)
    r_dt = root(f_gu, t5.ut1 - 40, t5.ut1 + 10)
    item(5, "木星入鬼初度日（新法：黄道）", "日", local_days(ts.ut1_jd(r_xf[0])), xf=date_days(D(1634, 10, 6)),
         xf_txt="閏八月十五（1634-10-06）", de_txt=local_str(ts.ut1_jd(r_xf[0])))
    item(5, "木星入鬼初度日（大統：古黄道）", "日", local_days(ts.ut1_jd(r_dt[0])), dt_=date_days(D(1634, 10, 15)),
         dt_txt="閏八月二十四（1634-10-15）", de_txt=local_str(ts.ut1_jd(r_dt[0])))
    notes[5] = (f"九月初一木星黄道鬼宿 {L['黄']:.2f}°、赤道 {L['赤']:.2f}°、古黄道 {L['古黄']:.2f} 古度。"
                f"新法入宿与「二度有奇」相洽；大統所推入宿日即使按古黄道（赤经圈投影）计亦晚。")

    # 6、7 木星过鬼宿
    for no, dref, d_ra, d_lon, motion, d_in, h_in, in_line, d_out, h_out, out_line, mind in [
        (6, D(1634, 12, 22), D(1634, 12, 22), D(1634, 12, 24), 0.11, D(1634, 12, 10), 12.0, ("鬼四", "鬼三"),
         D(1635, 1, 8), 12.0, ("鬼", "鬼二"), 0.54),
        (7, D(1635, 6, 7), D(1635, 6, 7), D(1635, 6, 8), 0.19, D(1635, 6, 7), 12.0, ("鬼", "鬼二"),
         D(1635, 6, 14), 18.0, ("鬼四", "鬼三"), 0.28)]:
        t0 = ut_of(dref, 0)
        rr = root(ra_diff("木", "積尸"), t0.ut1 - 30, t0.ut1 + 30)
        rl = root(lon_diff("木", "積尸"), t0.ut1 - 30, t0.ut1 + 30)
        tra = ts.ut1_jd(min(rr, key=lambda r: abs(r - t0.ut1)))
        tla = ts.ut1_jd(min(rl, key=lambda r: abs(r - t0.ut1)))
        item(no, "木星与積尸氣赤经同度日", "日", local_days(tra), xf=date_days(d_ra), xf_txt=str(d_ra), de_txt=local_str(tra))
        item(no, "木星与積尸氣黄经同度日", "日", local_days(tla), xf=date_days(d_lon), xf_txt=str(d_lon), de_txt=local_str(tla))
        item(no, "是日木星细行（黄经日行，绝对值）", "度/日", abs(daily_motion("木", ut_of(d_ra, 12))), xf=motion)
        ti = ut_of(d_in, h_in)
        ri = root(side_of_line("木", *in_line), ti.ut1 - 25, ti.ut1 + 25)
        to = ut_of(d_out, h_out)
        ro = root(side_of_line("木", *out_line), to.ut1 - 25, to.ut1 + 25)
        tin = ts.ut1_jd(min(ri, key=lambda r: abs(r - ti.ut1)))
        tout = ts.ut1_jd(min(ro, key=lambda r: abs(r - to.ut1)))
        item(no, "入鬼宿座（过两星连线）" + ("：正文「二十三日午時」" if no == 7 else ""), "日", local_days(tin), xf=local_days(ti),
             xf_txt=f"{d_in} {h_in:g}h", cat="存疑" if no == 7 else None,
             de_txt=local_str(tin))
        item(no, "出鬼宿座（过两星连线）", "日", local_days(tout), xf=local_days(to), xf_txt=f"{d_out} {h_out:g}h",
             de_txt=local_str(tout))
        tt = times(t0.ut1 - 20, t0.ut1 + 20, 2)
        ss = sep("木", "積尸", tt)
        item(no, "木星距積尸氣最近", "度", float(ss.min()), xf=mind, xf_txt=f"{mind:.2f}" + ("（小注 0.38）" if no == 7 else ""))
        if no == 7:
            item(7, "入鬼宿座（按 312 上图「癸巳」= 四月十四）", "日", local_days(tin), xf=date_days(D(1635, 5, 29)),
                 xf_txt="1635-05-29（癸巳）", de_txt=local_str(tin))
            notes[7] = (f"DE406 入宿 {local_str(tin)}、出宿 {local_str(tout)}，最近 {float(ss.min()):.2f}°。"
                        "正文「自二十三日午時」入宿与同页图（312 上）标癸巳（十四日）入宿相差九日，DE406 支持图，正文「二十三」当为「十三」或「十四」之讹；"
                        "距积尸「百分度之二十八」与小注「三十八」两歧，DE406 最近距离较二者皆大。"
                        f"又正文赤道二十三日、黄道二十四日同度，DE406 则黄经同度（{local_str(tla)}）在赤经同度（{local_str(tra)}）之前："
                        "次序与正文相反（七年十一月逆行条之次序则与 DE406 同）。")
        else:
            notes[6] = f"DE406 入宿 {local_str(tin)}、出宿 {local_str(tout)}，最近 {float(ss.min()):.2f}°。图 312 上标十月二十日癸卯入宿、十一月二十日壬申出宿，与正文同。"

    def vis_item(no, what, name, d0, d1, evening, end, xf=None, dt_=None, xf_txt=None, dt_txt=None):
        """见伏日：end=True 取可见段末日（伏前一日），否则取始日；DE406 给宽严两档之间的区间。"""
        ws, ww = window(name, d0, d1, evening, "严"), window(name, d0, d1, evening, "宽")
        if end:
            lo, hi = (ws[1] if ws else ww[0]), ww[1]
        else:
            lo, hi = ww[0], (ws[0] if ws else ww[1])
        item(no, what, "日", None, xf=None if xf is None else O(xf), dt_=None if dt_ is None else O(dt_), xf_txt=xf_txt, dt_txt=dt_txt,
             de_txt=f"{lo}—{hi}" if lo != hi else str(lo), de_iv=(O(lo), O(hi)))
        return ws, ww

    def alt_txt(name, d, evening):
        r1 = twilight_alt(name, d, evening, ARCUS["宽"][name])
        r2 = twilight_alt(name, d, evening, ARCUS["严"][name])
        return f"日下{ARCUS['宽'][name]:g}°时高 {r1[0]:.1f}°、日下{ARCUS['严'][name]:g}°时高 {r2[0]:.1f}°，距角 {r1[1]:.1f}°"

    # 8 八年水星晨见（三至六月）：新法俱不見，大統三月十八—四月二十一晨見
    ww = window("水", D(1635, 4, 17), D(1635, 8, 12), False, "宽")
    best = max(((d, twilight_alt("水", d, False, ARCUS["宽"]["水"])) for d in
                (D(1635, 4, 17) + dt.timedelta(k) for k in range(0, 118, 2))), key=lambda x: x[1][0])
    ok = True if ww is None else ("两可" if window("水", D(1635, 4, 17), D(1635, 8, 12), False, "严") is None else False)
    item(8, f"三至六月水星晨时最大高度（日下 {ARCUS['宽']['水']:g}°）", "度", best[1][0], xf_txt="俱晨不見", dt_txt="三月十八—四月二十一晨見",
         obs="委無水星出見", obs_ok=ok, cat="其他")
    notes[8] = (f"三至六月水星晨时（日下 {ARCUS['宽']['水']:g}°）最高 {best[1][0]:.1f}°（{best[0]}，距角 {best[1][1]:.1f}°）；"
                f"宽档可见段：{'—'.join(map(str, ww)) if ww else '无'}。北半球春夏之交晨见，黄道与东方地平交角小，水星距角虽大而高度甚低，"
                "大統所载晨见（5 月 4 日—6 月 5 日）期内宽档亦仅末二日勉强可见，新法「俱晨不見」与 DE406 大体合。")
    # 9 水星晨见（七至九月）
    ws, ww = vis_item(9, "水星晨见始日", "水", D(1635, 8, 25), D(1635, 10, 31), False, False, xf=D(1635, 9, 6),
                      xf_txt="七月二十五（1635-09-06）")
    vis_item(9, "水星晨见末日（晨不見前一日）", "水", D(1635, 8, 25), D(1635, 10, 31), False, True, xf=D(1635, 10, 2),
             dt_=D(1635, 9, 16), xf_txt="八月二十三晨不見（1635-10-03）", dt_txt="八月初七晨伏（1635-09-17）")
    item(9, "八月二十二（1635-10-02）晨 水星", "度", twilight_alt("水", D(1635, 10, 2), False, ARCUS["宽"]["水"])[0],
         obs="八月二十三日以前皆晨見", obs_ok=judge("水", D(1635, 10, 2), False, True), cat="其他")
    notes[9] = f"八月二十二晨 {alt_txt('水', D(1635, 10, 2), False)}。大統八月初七即伏，早二周。"
    # 13 金星晨见
    vis_item(13, "金星晨见末日（晨不見前一日）", "金", D(1635, 10, 1), D(1635, 12, 20), False, True, xf=D(1635, 11, 11),
             dt_=D(1635, 10, 18), xf_txt="十月初三始晨不見（1635-11-12）", dt_txt="九月初九晨伏（1635-10-19）")
    item(13, "九月十七（1635-10-27）晨 金星", "度", twilight_alt("金", D(1635, 10, 27), False, ARCUS["宽"]["金"])[0],
         obs="委見金星曉出", obs_ok=judge("金", D(1635, 10, 27), False, True), cat="其他")
    notes[13] = f"九月十七晨 {alt_txt('金', D(1635, 10, 27), False)}。金星上合在 1635 年 12 月。"
    # 14 水星夕见
    vis_item(14, "水星夕见始日", "水", D(1635, 10, 20), D(1635, 12, 31), True, False, xf=D(1635, 11, 15), dt_=D(1635, 10, 31),
             xf_txt="十月初六（1635-11-15）", dt_txt="九月二十一（1635-10-31）")
    for dd, lab in [(D(1635, 10, 30), "九月二十"), (D(1635, 11, 7), "九月二十八")]:
        item(14, f"{lab}（{dd}）昏 水星", "度", twilight_alt("水", dd, True, ARCUS["宽"]["水"])[0], obs="委無水星出見",
             obs_ok=judge("水", dd, True, False), cat="其他")
    notes[14] = (f"九月二十昏 {alt_txt('水', D(1635, 10, 30), True)}；九月二十八昏 {alt_txt('水', D(1635, 11, 7), True)}。"
                 "大統夕见早约一月，新法亦早，均因秋季昏时黄道与西方地平交角小。")
    # 16 九年水星夕见
    vis_item(16, "水星夕见末日", "水", D(1636, 3, 1), D(1636, 4, 25), True, True, xf=D(1636, 4, 1), dt_=D(1636, 3, 17),
             xf_txt="至二月二十六皆見（1636-04-01）", dt_txt="二月十二夕伏（1636-03-18）")
    item(16, "二月十二（1636-03-18）昏 水星", "度", twilight_alt("水", D(1636, 3, 18), True, ARCUS["宽"]["水"])[0],
         obs="委見水星未伏", obs_ok=judge("水", D(1636, 3, 18), True, True), cat="其他")
    notes[16] = (f"二月十二昏 {alt_txt('水', D(1636, 3, 18), True)}。原文「推木星」，木星时近冲日（见第 15 条），不得言夕伏，"
                 "下文又云「見水星未伏」，当作水星。")
    # 18 九年木星夕伏
    vis_item(18, "木星夕见末日（夕不見前一日）", "木", D(1636, 7, 10), D(1636, 9, 5), True, True, xf=D(1636, 8, 13),
             dt_=D(1636, 8, 22), xf_txt="七月十四夕不見（1636-08-14）", dt_txt="七月二十三始夕不見（1636-08-23）")
    item(18, "七月十四（1636-08-14）昏 木星", "度", twilight_alt("木", D(1636, 8, 14), True, ARCUS["宽"]["木"])[0],
         obs="委無木星出見", obs_ok=judge("木", D(1636, 8, 14), True, False), cat="其他")
    notes[18] = f"七月十四昏 {alt_txt('木', D(1636, 8, 14), True)}。木星合日在 1636 年 9 月。"

    # 10 木星合轩辕
    t10 = ut_of(D(1635, 9, 23), 0.0)
    J, R = apparent("木", t10), apparent("軒轅", t10)
    item(10, "木星黄经（鶉火宫度）", "度", J["lon"] - 120, xf=24 + 39 / 60)
    item(10, "木星黄纬", "度", J["lat"], xf=50 / 60)
    item(10, "軒轅大星黄经（鶉火宫度）", "度", R["lon"] - 120, xf=24 + 47 / 60)
    item(10, "軒轅大星黄纬", "度", R["lat"], xf=27 / 60)
    item(10, "木星赤经 − 軒轅赤经（出極一直線 = 0）", "度", float(wrap(J["ra"] - R["ra"])), xf=0.0,
         obs="同度同分", obs_ok=abs(float(wrap(J["ra"] - R["ra"]))) < 0.17)
    L = lodge("木", "張", t10)
    item(10, "木星張宿度（新法赤道 360°；大統古黄道古度）", "度", L["赤"], xf=4.0)
    item(10, "木星張宿度（大統：古黄道，古度）", "古度", L["古黄"], dt_=1.0)
    rr = root(ra_diff("木", "軒轅"), t10.ut1 - 10, t10.ut1 + 10)
    notes[10] = (f"DE406 木星与軒轅赤经相等在 {local_str(ts.ut1_jd(rr[0]))}；子正初刻木星赤道張宿 {L['赤']:.2f}°、"
                 f"古黄道 {L['古黄']:.2f} 古度、赤道古度 {L['赤古']:.2f}。")

    # 11 木火合
    t0 = ut_of(D(1635, 10, 7), 14.0)
    rs = root(lon_diff("火", "木"), t0.ut1 - 5, t0.ut1 + 5)
    tc = ts.ut1_jd(rs[0])
    J, M = apparent("木", tc), apparent("火", tc)
    item(11, "木火黄经合时刻", "日", local_days(tc), xf=local_days(t0), dt_=date_days(D(1635, 10, 9)),
         xf_txt="八月二十七未時（1635-10-07 14h）", dt_txt="二十九日（1635-10-09）", de_txt=local_str(tc))
    item(11, "合时黄经（鶉火宫度）", "度", J["lon"] - 120, xf=27 + 26 / 60)
    item(11, "合时火纬 − 木纬", "度", M["lat"] - J["lat"], xf=0.5)
    L = lodge("木", "張", t0)
    Lm = lodge("火", "張", t0)
    item(11, "木星赤道張宿度（新法 360°）", "度", L["赤"], xf=6 + 23 / 60)
    item(11, "火星赤道張宿度（新法 360°）", "度", Lm["赤"], xf=6 + 23 / 60)
    item(11, "木星張宿度（大統：古黄道，古度）", "古度", L["古黄"], dt_=4.0)
    item(11, "火星張宿度（大統：古黄道，古度）", "古度", Lm["古黄"], dt_=3.0)
    rr = root(ra_diff("火", "木"), t0.ut1 - 5, t0.ut1 + 5)
    tr = ts.ut1_jd(rr[0])
    item(11, "木火赤经相等（出極一直線）时刻", "日", local_days(tr), xf=date_days(D(1635, 10, 8), 0.0),
         xf_txt="子正（1635-10-08 00h）", de_txt=local_str(tr))
    item(11, "子正时 木星赤经 − 90°（距夏至）", "度", apparent("木", ut_of(D(1635, 10, 8), 0))["ra"] - 90, xf=59 + 50 / 60)
    notes[11] = (f"未時木星古黄道張 {L['古黄']:.2f}、火星 {Lm['古黄']:.2f} 古度（赤道古度木 {L['赤古']:.2f}、火 {Lm['赤古']:.2f}）。"
                 "大統所云木差二度半、火差三度半，与 DE406 相较大致不差。")

    # 12 月木火
    t0 = ut_of(D(1635, 10, 7), 6.0)
    rs = root(lon_diff("月", "木"), t0.ut1 - 1, t0.ut1 + 1, 0.25)
    tc = ts.ut1_jd(rs[0])
    Mo, J = apparent("月", tc), apparent("木", tc)
    item(12, "月木黄经合时刻（地心）", "日", local_days(tc), xf=local_days(t0), xf_txt="卯正初刻（06:00）", de_txt=local_str(tc))
    item(12, "合时月纬 − 木纬（月在南为负）", "度", Mo["lat"] - J["lat"], xf=-36 / 60)

    def coll(tt):  # 站心：月心至木火大圆之角距
        m = site.at(tt).observe(BODY["月"]).apparent().position.au
        a = site.at(tt).observe(BODY["木"]).apparent().position.au
        b = site.at(tt).observe(BODY["火"]).apparent().position.au
        n = np.cross(a, b, axis=0)
        return np.degrees(np.arcsin(np.sum(n * m, axis=0) / np.linalg.norm(n, axis=0) / np.linalg.norm(m, axis=0)))
    t1 = ut_of(D(1635, 10, 7), 4.5)
    rr = root(coll, t1.ut1 - 0.3, t1.ut1 + 0.3, 0.05)
    if rr:
        tl = ts.ut1_jd(rr[0])
        item(12, "站心 月木火一直线（月心在木火大圆上）时刻", "日", local_days(tl), xf=local_days(t1), xf_txt="寅正二刻（04:30）",
             de_txt=local_str(tl), obs="寅時親測月木火皆為一直線", obs_ok=3.0 <= ((local_days(tl) + 0.5) % 1) * 24 <= 5.5)
    t_obs = ut_of(D(1635, 10, 7), 4.0)
    item(12, "寅正（04:00）站心月心距木火大圆", "度", float(coll(t_obs)), obs="一直線", obs_ok=abs(float(coll(t_obs))) < 0.5)
    alt_m = site.at(t_obs).observe(BODY["月"]).apparent().altaz()[0].degrees
    notes[12] = f"寅正月高 {alt_m:.1f}°（已出地平）。地心合与站心「一直線」相差约二小时，正是月视差之效，曆局用视差改正亦合。"

    # 15 九年二月 木星与轩辕
    rows = []
    for d in [D(1636, 3, 18), D(1636, 3, 19), D(1636, 3, 20)]:
        t = ut_of(d, 22.0)
        J, R = apparent("木", t), apparent("軒轅", t)
        rows.append((float(wrap(J["ra"] - R["ra"])), float(wrap(J["lon"] - R["lon"])), lodge("木", "張", t), lodge("軒轅", "張", t)))
    dra = statistics.mean(r[0] for r in rows)
    dlon = statistics.mean(r[1] for r in rows)
    LJ = rows[1][2]; LR = rows[1][3]
    item(15, "木星赤经 − 軒轅赤经（东为正）", "度", dra, xf=6 + 1 / 6 - 4, dt_=(2 - 3.4) / GU,
         obs="木星果在軒轅大星之東", obs_ok=dra > 0)
    item(15, "木星張宿度（新法赤道 360°）", "度", LJ["赤"], xf=6 + 1 / 6)
    item(15, "軒轅張宿度（新法赤道 360°）", "度", LR["赤"], xf=4.0)
    item(15, "木星張宿度（大統：古黄道，古度）", "古度", LJ["古黄"], dt_=2.0)
    item(15, "軒轅張宿度（舊法：古黄道，古度）", "古度", LR["古黄"], dt_=3.4)
    t0 = ut_of(D(1636, 3, 19), 0)
    st = root(lambda tt: np.asarray([daily_motion("木", ts.ut1_jd(x)) for x in np.atleast_1d(tt.ut1)]), t0.ut1 - 20, t0.ut1 + 120, 24)
    notes[15] = (f"二月十三夜木星在軒轅东 {dra:.2f}°（赤经）、黄经差 {dlon:.2f}°；木星赤道張 {LJ['赤']:.2f}°、軒轅 {LR['赤']:.2f}°；"
                 f"古黄道木 {LJ['古黄']:.2f}、軒轅 {LR['古黄']:.2f} 古度。木星此时逆行，留在 {', '.join(local_str(ts.ut1_jd(x))[:10] for x in st)}，"
                 "「將留」略早。")

    # 17 九年火星逆行近角宿
    dd = [D(1636, 5, 2) + dt.timedelta(k) for k in range(0, 40, 3)]
    vals = []
    for d in dd:
        t = ut_of(d, 20.0)
        M, S = apparent("火", t), apparent("角", t)
        vals.append((d, float(wrap(M["ra"] - S["ra"])), float(sep("火", "角", t)), lodge("火", "角", t)))
    ra_min = min(v[1] for v in vals); ra_max = max(v[1] for v in vals)
    s_min = min(v[2] for v in vals); s_max = max(v[2] for v in vals)
    hd = [v[3]["黄"] for v in vals]; gh = [v[3]["古黄"] for v in vals]
    t_mid = ut_of(D(1636, 5, 21), 20)
    zhen_w = float(wrap(foot_lon(apparent("角", t_mid)["ra"], t_mid) - foot_lon(apparent("軫", t_mid)["ra"], t_mid))) * GU
    item(17, "火星角宿度（黄道 360°，期中均值）", "度", statistics.mean(hd), xf=2.0, xf_txt="角一二三度（取 2）")
    item(17, "火星角宿度（古黄道古度，期中均值；大統軫十六七度化角宿度）", "古度", statistics.mean(gh), dt_=16.5 - zhen_w,
         dt_txt=f"軫16.5（= 角{16.5 - zhen_w:.2f}）")
    item(17, "戌时火星赤经 − 角宿大星赤经（期内最小）", "度", ra_min, obs="果在角宿大星之東", obs_ok=ra_min > 0, cat="其他")
    item(17, "戌时火星赤经 − 角宿大星赤经（期内最小，东西相距）", "度", ra_min, obs="相距不過一度", obs_ok=0 < ra_min <= 1.0, cat="其他")
    item(17, "戌时火星距角宿大星角距（期内最小—最大）", "度", s_min, de_txt=f"{s_min:.2f}—{s_max:.2f}", cat="其他")
    t0 = ut_of(D(1636, 5, 2), 0)
    st = root(lambda tt: np.asarray([daily_motion("火", ts.ut1_jd(x)) for x in np.atleast_1d(tt.ut1)]), t0.ut1 - 60, t0.ut1 + 60, 24)
    op = root(lambda tt: wrap(apparent("火", tt)["lon"] - apparent("日", tt)["lon"] - 180), t0.ut1 - 60, t0.ut1 + 60, 12)
    notes[17] = (f"三月二十七至五月初八（1636-05-02—06-10）戌时火星在角宿大星东 {ra_min:.2f}°—{ra_max:.2f}°（赤经），"
                 f"角距 {s_min:.2f}°—{s_max:.2f}°；黄道角宿 {min(hd):.2f}°—{max(hd):.2f}°，古黄道 {min(gh):.2f}—{max(gh):.2f} 古度，"
                 f"按古黄道始终未入軫（軫宿古黄道宽 {zhen_w:.2f} 古度），按黄经则留前后数日在角距星之西不足一度（火星纬度偏北，赤经仍在其东）。火星冲日 {', '.join(local_str(ts.ut1_jd(x))[:10] for x in op)}，"
                 f"留 {', '.join(local_str(ts.ut1_jd(x))[:10] for x in st)}。")


CATS = [("位置", "位置（度/古度）"), ("时刻", "会合、凌犯、入宿时刻（日）"), ("见伏", "见伏日期（日，粗判）")]


def summarize():
    def med(key, cat):
        v = [abs(i[key]) for i in items if i[key] is not None and i["类"] == cat]
        return (statistics.median(v), max(v), len(v)) if v else (float("nan"), float("nan"), 0)
    return {cat: (med("新法误差", cat), med("大统误差", cat)) for cat, _ in CATS}


def esc(s):
    return str(s).replace("|", "\\|").replace("\n", " ")


def write_md(recs):
    out = []
    w = out.append
    w("# 《崇禎曆書·五緯曆指》崇禎七—九年五星記錄检验\n")
    w("> 生成：`uv run --no-project --with skyfield python -I scripts/phenomena/chongzhen_wuwei_check.py`（仓库根）。录文："
      "`data/phenomena/chongzhen_wuwei_observations.csv`。勿手改本文件。\n")
    w("## 来源\n")
    w("《崇禎曆書·五緯曆指》卷九（第八章「五星盈縮立成考」之后至第十章末），曆局所录崇禎七年（1634）至九年（1636）新法（西法）与大統曆"
      "推算之五星会合、凌犯、入宿与见伏，及「會同部監公同測驗」之结果（卷九，《故宫珍本丛刊》第 382 册书页 309—313）。卷末自云"
      "「此上所錄，皆係會同部監公同測驗過者，其未經測者，每年相差甚多，茲不備錄」，故所录本是择优之例，用以论证新法之密，不是随机样本。\n")
    w("原书所记以推算值为主：新法、大統各给时日、宫度、宿度与纬差，实测多为「果合」「委無水星出見」之类的定性结论，"
      "数值实测只有「相距不過一度」等少数。故本检验以 DE406 分别检核新法、大統推算之误差，实测则逐条判其与 DE406 天象是否相符。\n")
    w("## 方法\n")
    w("- 时刻：北京（116.4°E、39.9°N）地方真太阳时；曆局用九十六刻制，一时辰八刻（初、正各四刻），卯正二刻 = 06:30。"
      "历表 DE406（skyfield，ΔT 用其内置长期表）；恒星用 Hipparcos（I/311，含自行视差），積尸氣（M44）取星团中心。")
    w("- 坐标皆地心视位置、当日真春分点。新法黄道：黄经 360°，十二宫每宫 30°（鶉火 120°—150°、析木 240°—270°）；黄道宿度 = 行星黄经 − 距星黄经。"
      "新法赤道宿度 = 赤经差（360°），「出極一直線」即同赤经，「距夏至」= 赤经 − 90°。")
    w("- 大統宿度按授时—大统「黄道宿度」之定义：距星赤经圈投影到黄道之点起算（古黄道），化古度（×365.25/360）；兼给赤道古度备参。"
      "距星鬼 θ Cnc、張 υ¹ Hya、角 α Vir、軫 γ Crv（明清相同）。大統以至元十七年宿度为常数，不随岁差重算，本检验按当日距星位置投影，"
      "所差在十分之一度量级。")
    w("- 见伏：太阳在地平下 h 度（视界弧）时行星几何高度 ≥ 2° 为可见。h 取宽、严两档（宽：金 4°、木 7°、水 8°；严：金 6°、木 10°、水 11°，"
      "严档近托勒密所用值），DE406 见伏日给两档之间的区间，推算日落在区间内误差记 0，否则记到较近一端之日数。实测「见」须严档亦可见、"
      "「不见」须宽档亦不可见方判「合」，介于两档之间判「两可」。北京平地平、不计大气消光与星等变化，只作粗判。")
    w("- 会合、同度：在 ±12 日内求黄经差（或赤经差）变号之时刻。入鬼宿座：木星过「两星中线」即过两距星所定大圆之时刻"
      "（西侧鬼宿一 θ、鬼宿二 η；东侧鬼宿三 γ、鬼宿四 δ）。")
    w("- 误差 = 推算 − DE406；时日误差以日计（大統只给日，按当日正午计）。\n")
    s = summarize()
    w("## 汇总\n")
    w("| 类别 | 新法项数 | 新法 \\|误差\\| 中位数 | 新法最大 | 大統项数 | 大統 \\|误差\\| 中位数 | 大統最大 |")
    w("|---|---|---|---|---|---|---|")
    for cat, label in CATS:
        (xm, xx, xn), (dm, dx, dn) = s[cat]
        w(f"| {label} | {xn} | {xm:.2f} | {xx:.2f} | {dn} | {dm:.2f} | {dx:.2f} |")
    obs = [i for i in items if i["实测判定"] is not None]
    ok = sum(1 for i in obs if i["实测判定"] is True)
    amb = sum(1 for i in obs if i["实测判定"] == "两可")
    w("")
    w(f"实测（定性）{len(obs)} 项：与 DE406 天象相符 {ok} 项，见伏判据宽严之间两可 {amb} 项，不合 {len(obs) - ok - amb} 项。")
    # 成对比较：同一项目新法、大統皆有值者
    pair = [i for i in items if i["新法误差"] is not None and i["大统误差"] is not None]
    better = sum(1 for i in pair if abs(i["新法误差"]) < abs(i["大统误差"]))
    w(f"新法、大統同给数值之项 {len(pair)} 个，新法误差较小者 {better} 个。\n")
    w("## 逐条\n")
    w("| 序 | 书页 | 纪年日期 | 公历 | 天体 | 事项 | 原文 | 按语 |")
    w("|---|---|---|---|---|---|---|---|")
    for r in recs:
        w(f"| {r['序']} | {r['书页']} | {esc(r['纪年日期'])}{('（' + r['干支'] + '）') if r['干支'] else ''} | {r['公历']} | {esc(r['天体'])} | {esc(r['事项'])} | {esc(r['原文'])} | {esc(notes.get(int(r['序']), ''))} |")
    w("\n## 数值子项\n")
    w("| 序 | 项目 | 单位 | 新法 | 大統 | DE406 | 新法误差 | 大統误差 | 实测 | 实测判定 |")
    w("|---|---|---|---|---|---|---|---|---|---|")
    for i in items:
        jd = {None: "", True: "合", False: "不合", "两可": "两可"}[i["实测判定"]]
        w(f"| {i['序']} | {esc(i['项目'])} | {i['单位']} | {esc(i['新法'])} | {esc(i['大统'])} | {esc(i['DE406'])} | "
          f"{fmt_err(i['新法误差'], i['单位'])} | {fmt_err(i['大统误差'], i['单位'])} | {esc(i['实测'])} | {jd} |")
    w("\n## 限制\n")
    w("- 原书所录是曆局为论证新法而择取之例（卷末明言），不能据以统计新法、大統的一般精度。")
    w("- 大統之宿度用今算距星位置投影，未按大統曆本身的至元宿度常数与岁差处理；大統五星推步本身未在本仓库复原，故大統误差只就书中所引数字论。")
    w("- 见伏判据粗，日期误差有 ±数日之不定；各书页之经纬图（309 下、310 上下、311 上、312 上）未量读图中星位。")
    w("- 时刻按九十六刻制、北京地方真太阳时；「昏初」取 18 时，「未時」取未正 14 时。")
    Path("doc/astronomy/chongzhen-wuwei-checks.md").write_text("\n".join(out) + "\n", encoding="utf-8")


def main():
    recs = list(csv.DictReader(l for l in open("data/phenomena/chongzhen_wuwei_observations.csv", encoding="utf-8")
                               if not l.startswith("#")))
    run()
    write_md(recs)
    for k, v in summarize().items():
        print(k, v)
    for i in items:
        print(i["序"], i["类"], i["项目"], i["新法"], i["大统"], i["DE406"], fmt_err(i["新法误差"], ""), fmt_err(i["大统误差"], ""), i["实测判定"])


if __name__ == "__main__":
    sys.exit(main())
