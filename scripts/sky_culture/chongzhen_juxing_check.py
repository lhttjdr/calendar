"""二十八宿距星：《崇祯历书·恒星历指》（data/sky_culture/chongzhen_juxing.csv）与《钦定天文正义》卷四
（data/sky_culture/tianwen_zhengyi_juxing.csv）对 Hipparcos 的逐宿检验，并与本项目《象林》《灵台仪象志》
《仪象考成》三套距星（data/sky_culture/chinese_lunar_mansion_determinatives.csv）比较。
结果写 doc/astronomy/chongzhen-juxing-checks.md。

方法：Hipparcos（data/stars/hipparcos_bright.csv，ICRS J1991.25）经 erfa.pmsafe 推空间运动至历元，
erfa.pmat06（框架偏差 + IAU 2006 岁差）转当日平赤道平春分点，erfa.ecm06 转当日平黄道。不计章动（≤ 20″）、
光行差（≤ 20″），远小于原书 1′ 的分位与观测误差。

认星：在三套距星与 V ≤ 6 全部星中，取与原书两量（崇祯：赤经、黄经；天文正义：黄经、黄纬）合差最小者；
另列与《灵台仪象志》所取星的差，以见两者是否同星。

运行（仓库根）：uv run --no-project --with pyerfa --with numpy python -I scripts/sky_culture/chongzhen_juxing_check.py
「按原表划宿」一节需两种取法下的《清史稿》检验表，先在 rust/ 下各跑一次
  PHENOMENA_CHECKS=1 PHENOMENA_CHECKS_DIR=<目录>/<table|hipparcos> PHENOMENA_CHONGZHEN_LODGES=<table|hipparcos> \
    cargo test --release -p lunar-core --no-default-features --features real-f64 qingshigao_star_records_check -- --include-ignored
再以 CHONGZHEN_MODE_DIR=<目录> 运行本脚本；不设则略去该节。
"""

from __future__ import annotations

import csv
import math
import os
import re
import statistics

import erfa
import numpy as np

ROOT = os.getcwd()
HIP_CSV = os.path.join(ROOT, "data/stars/hipparcos_bright.csv")
CZ_CSV = os.path.join(ROOT, "data/sky_culture/chongzhen_juxing.csv")
TW_CSV = os.path.join(ROOT, "data/sky_culture/tianwen_zhengyi_juxing.csv")
DET_CSV = os.path.join(ROOT, "data/sky_culture/chinese_lunar_mansion_determinatives.csv")
OUT_MD = os.path.join(ROOT, "doc/astronomy/chongzhen-juxing-checks.md")

CZ_EPOCH = 1628.0  # 《恒星历指》卷二「崇祯元年戊辰为历元」
GONG = {"戌": 0, "酉": 30, "申": 60, "未": 90, "午": 120, "巳": 150, "辰": 180, "卯": 210, "寅": 240, "丑": 270, "子": 300, "亥": 330}
SYSTEMS = [("ming_xianglin", "象林"), ("qing_lingtai", "灵台仪象志"), ("qing", "仪象考成")]
PREC = 50.29  # 角秒/年，只用于由黄经差反推历元的迭代步长


def rows(path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def wrap180(x):
    return (x + 180.0) % 360.0 - 180.0


class Catalog:
    """Hipparcos 推至某历元（儒略年）的当日平赤经、赤纬、黄经、黄纬（度）。"""

    def __init__(self, cat, epoch):
        hips = np.array(sorted(cat))
        f = lambda k: np.array([float(cat[h][k] or 0) for h in hips])
        ra, dec = np.radians(f("ra_deg")), np.radians(f("dec_deg"))
        mas = math.pi / 180 / 3600 / 1000
        pmr = f("pmra_mas_yr") * mas / np.cos(dec)  # erfa 要 dα/dt
        pmd = f("pmdec_mas_yr") * mas
        plx = np.maximum(f("plx_mas"), 0.0) / 1000.0
        jd = 2451545.0 + (epoch - 2000.0) * 365.25
        ra2, dec2, *_ = erfa.pmsafe(ra, dec, pmr, pmd, plx, np.zeros_like(ra), 2448349.0625, 0.0, jd, 0.0)
        p = erfa.s2c(ra2, dec2)
        a, d = erfa.c2s(p @ erfa.pmat06(jd, 0.0).T)
        l, b = erfa.c2s(p @ erfa.ecm06(jd, 0.0).T)
        self.hips = hips
        self.idx = {int(h): i for i, h in enumerate(hips)}
        self.ra, self.dec = np.degrees(erfa.anp(a)), np.degrees(d)
        self.lon, self.lat = np.degrees(erfa.anp(l)), np.degrees(b)
        self.vmag = np.array([float(cat[h]["vmag"] or 99) for h in hips])

    def pos(self, hip):
        i = self.idx[hip]
        return self.ra[i], self.dec[i], self.lon[i], self.lat[i]


def fmt_dm(x):
    s = "−" if x < 0 else ""
    m = round(abs(x) * 60)
    return f"{s}{m // 60}°{m % 60:02d}′"


def stats(xs):
    med = statistics.median(xs)
    mad = statistics.median([abs(x - med) for x in xs])
    return med, mad, math.sqrt(sum(x * x for x in xs) / len(xs)), max(abs(x) for x in xs)


def fit_epoch(cat, hips, lons, guess):
    """黄经差中位数为零的历元。原书黄经比 Hipparcos 大 → 实际历元更晚。"""
    y = guess
    for _ in range(5):
        c = Catalog(cat, y)
        y += statistics.median([wrap180(lon - c.pos(h)[2]) for h, lon in zip(hips, lons)]) * 3600 / PREC
    return y


MODE_DIR = os.environ.get("CHONGZHEN_MODE_DIR", "")  # 含 table/、hipparcos/ 两个子目录的 qingshigao_star_records.csv
CELL = re.compile(r"^(?P<m>[^\d\-]+)(?P<d>-?[\d.]+)（史载 (?P<rm>[^\d]+)(?P<rd>[\d.]+)）$")
MANSION_ORDER = "角亢氐房心尾箕斗牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸"


def mode_rows(path):
    out = {}
    for r in rows(path):
        y = int(r["公历(儒略)"][:4])
        m = CELL.match(r["推算入宿"])
        if not (1645 <= y <= 1673) or not m or r["干支在月"] != "是":
            continue
        out[r["序"]] = (r, m)
    return out


def mode_section(P):
    """两种取法（原表 / Hipparcos）下《清史稿》1645—1673 年记有宿度者的推算入宿对所记之差。"""
    if not MODE_DIR:
        return
    a = mode_rows(os.path.join(MODE_DIR, "table", "qingshigao_star_records.csv"))
    b = mode_rows(os.path.join(MODE_DIR, "hipparcos", "qingshigao_star_records.csv"))
    keys = [k for k in a if k in b]

    def diff(m):
        # 同宿：度差；邻宿：跨宿界换成连续黄道度差（宿度用原表本度近似不可得，此处只计同宿）
        return float(m["d"]) - float(m["rd"]) if m["m"] == m["rm"] else None

    da = [diff(a[k][1]) for k in keys]
    db = [diff(b[k][1]) for k in keys]
    same_a = sum(x is not None for x in da)
    same_b = sum(x is not None for x in db)
    both = [(x, y) for x, y in zip(da, db) if x is not None and y is not None]
    abs_a = statistics.median([abs(x) for x, _ in both])
    abs_b = statistics.median([abs(y) for _, y in both])
    win_a = sum(abs(x) < abs(y) - 1e-9 for x, y in both)
    win_b = sum(abs(y) < abs(x) - 1e-9 for x, y in both)
    P("## 按原表划宿：《清史稿》1645—1673 年所记入宿度")
    P("")
    P("《清史稿·天文志》五星合聚、凌犯所记「同躔某宿某度」多为钦天监按时宪历推步之数。1645—1673 年距星为《崇祯历书》体系，"
      "比较两种距星位置取法下的推算入宿（行星位置同为 VSOP87 推算，黄道 360°）：")
    P("")
    P("- **原表**：距星黄经 = Hipparcos 推算 + 《恒星历指》原表在 1628 年对推算之差（原表误差随岁差带入当时）；")
    P("- **Hipparcos**：距星黄经按 Hipparcos 推算。")
    P("")
    P(f"记有宿度且干支在月者 {len(keys)} 条。推算与所记同宿：原表 {same_a}、Hipparcos {same_b}。"
      f"两法皆同宿者 {len(both)} 条，|推算 − 所记| 中位：原表 {abs_a:.2f}°、Hipparcos {abs_b:.2f}°；"
      f"逐条比，原表更近 {win_a} 条、Hipparcos 更近 {win_b} 条、相等 {len(both) - win_a - win_b} 条。")
    P("")
    # 两法共有系统差（推算普遍比所记大约 0.6°，与距星取法无关）去掉后再比；原表误差大的四宿单列
    ma = statistics.median([x for x, _ in both])
    mb = statistics.median([y for _, y in both])
    big = "尾箕女柳"
    rows_both = [(k, x, y) for k, x, y in zip(keys, da, db) if x is not None and y is not None]
    def cmp(sel):
        ra = [abs(x - ma) for _, x, _ in sel]
        rb = [abs(y - mb) for _, _, y in sel]
        return len(sel), statistics.median(ra), statistics.median(rb), sum(p < q - 1e-9 for p, q in zip(ra, rb)), sum(q < p - 1e-9 for p, q in zip(ra, rb))
    c_all = cmp(rows_both)
    c_big = cmp([t for t in rows_both if a[t[0]][1]["m"] in big])
    P(f"但两法推算普遍比所记大（差的中位：原表 {ma:+.2f}°、Hipparcos {mb:+.2f}°），此系统差与距星取法无关（行星位置、日界、时宪历推步本身），"
      "原表各宿平均偏大 1.7′ 恰把推算拉近所记，上面的「更近」多由此来。去掉各法自身的中位差后再比：")
    P("")
    P("| 范围 | 条数 | 残差绝对值中位：原表 | Hipparcos | 原表更近 | Hipparcos 更近 |")
    P("|---|---|---|---|---|---|")
    P(f"| 全部两法同宿者 | {c_all[0]} | {c_all[1]:.2f}° | {c_all[2]:.2f}° | {c_all[3]} | {c_all[4]} |")
    P(f"| 原表误差大的尾、箕、女、柳四宿 | {c_big[0]} | {c_big[1]:.2f}° | {c_big[2]:.2f}° | {c_big[3]} | {c_big[4]} |")
    P("")
    P("女宿原表比 ε Aqr 偏大 88′，若时宪历用此数，所记女宿度当比按 Hipparcos 推算小约 1.5°；实际所记与 Hipparcos 推算之差"
      "和其他各宿同为 +0.4°—+1.0°，按原表反成 −0.5°—−1.1°。柳宿（原表偏 55′）亦然。故时宪历推步所用距星黄经不是《恒星历指》"
      "这张积度表上的数（当另据恒星经纬表或经校正），缺省取 Hipparcos；原表模式留作对照。")
    P("")
    global MODE_SUMMARY
    MODE_SUMMARY = (len(keys), same_a, same_b, len(both), abs_a, abs_b, win_a, win_b, c_all, c_big)
    P("| 序 | 公历 | 记载 | 所记 | 原表推算 | 差° | Hipparcos 推算 | 差° |")
    P("|---|---|---|---|---|---|---|---|")
    for k, x, y in zip(keys, da, db):
        r, ma = a[k]
        mb = b[k][1]
        fx = "异宿" if x is None else f"{x:+.2f}"
        fy = "异宿" if y is None else f"{y:+.2f}"
        if ma["m"] == mb["m"] and abs(float(ma["d"]) - float(mb["d"])) < 0.005:
            continue  # 两法相同者不列
        P(f"| {k} | {r['公历(儒略)']} | {r['记载']} | {ma['rm']}{float(ma['rd']):g} | {ma['m']}{float(ma['d']):.2f} | {fx} | {mb['m']}{float(mb['d']):.2f} | {fy} |")
    P("")
    P("（只列两法推算不同的记录；未列者所在宿距星的原表误差不足 0.3′。）")
    P("")


MODE_SUMMARY = None


def main():
    cat = {int(r["hip"]): r for r in rows(HIP_CSV)}
    det = {}
    for r in rows(DET_CSV):
        det.setdefault(r["system"], {})[int(r["index"])] = int(r["hip"])
    cz = rows(CZ_CSV)
    tw = rows(TW_CSV) if os.path.exists(TW_CSV) else []
    c28 = Catalog(cat, CZ_EPOCH)
    out = []
    P = out.append
    dm = lambda d, m: int(d) + int(m) / 60

    # ---------------- 崇祯 ----------------
    width_bad = []
    for key in ("ra", "lon"):
        vals = [dm(r[f"{key}_deg"], r[f"{key}_min"]) for r in cz]
        for k, r in enumerate(cz):
            # 宿度 = 距星至东侧下一宿距星；觜在参后则觜无度，毕直至参
            nxt = (k + 2) % 28 if cz[(k + 1) % 28]["mansion"] == "觜" else (k + 1) % 28
            w = 0.0 if r["mansion"] == "觜" else (vals[nxt] - vals[k]) % 360
            wd, wm = r[f"{key}_width"].split(".")
            if round(w * 60) != int(wd) * 60 + int(wm):
                width_bad.append(f"{r['mansion']}（{key}）")

    ident, res = {}, []
    for r in cz:
        i = int(r["index"])
        ra, lon = dm(r["ra_deg"], r["ra_min"]), dm(r["lon_deg"], r["lon_min"])
        d = np.hypot((c28.ra - ra + 180) % 360 - 180, (c28.lon - lon + 180) % 360 - 180)
        d = np.where(c28.vmag <= 6.0, d, np.inf)
        cands = {h for s, _ in SYSTEMS for h in [det[s][i]]} | {int(c28.hips[np.argmin(d)])}
        score = lambda h: math.hypot(wrap180(ra - c28.pos(h)[0]), wrap180(lon - c28.pos(h)[2]))
        # 认星取三套距星中最合者；V ≤ 6 他星只作参照（两量定一点，暗星偶合不足为据），合差不到其一半时列出
        best_sys = min({det[s][i] for s, _ in SYSTEMS}, key=score)
        best_any = min(cands, key=score)
        h = best_sys
        ident[i] = h
        hra, hdec, hlon, hlat = c28.pos(h)
        res.append(dict(i=i, m=r["mansion"], h=h, ra=ra, lon=lon, hra=hra, hlon=hlon, hlat=hlat, hdec=hdec,
                        dra=wrap180(ra - hra) * 60, dlon=wrap180(lon - hlon) * 60,
                        alt=best_any if score(best_any) < 0.5 * score(h) else None, alt_score=score(best_any) * 60))
    s_ra, s_lon = stats([x["dra"] for x in res]), stats([x["dlon"] for x in res])
    ep_cz = fit_epoch(cat, [x["h"] for x in res], [x["lon"] for x in res], CZ_EPOCH)
    lt = det["qing_lingtai"]
    names = {int(r["index"]): r["mansion"] for r in cz}
    same_lt = [i for i in range(1, 29) if lt[i] == ident[i]]

    P("# 二十八宿距星：《崇祯历书》《钦定天文正义》对 Hipparcos 的检验")
    P("")
    P("> 由 `scripts/sky_culture/chongzhen_juxing_check.py` 生成，勿手改。数据 `data/sky_culture/chongzhen_juxing.csv`、"
      "`data/sky_culture/tianwen_zhengyi_juxing.csv`；三套既有距星见 `data/sky_culture/chinese_lunar_mansion_determinatives.csv`。")
    P("")
    P("## 一、《崇祯历书·恒星历指》")
    P("")
    P("- 底本：《崇祯历书》明崇祯刊本 07《恒星历指》二卷（汤若望撰）。卷二「恒星变易度第三·考赤道宿度差」表「今赤道积宿度」"
      "（距星赤经，PDF 第 140—143 页），「恒星黄道经纬度变易第四·考黄道宿度差」表「黄道宿今积度（平度）」（距星黄经，PDF 第 160—163 页）。"
      "周天 360°，至分。原表不给赤纬、黄纬。")
    P("- 历元：同卷「恒星本行表」「崇祯元年戊辰为历元」（PDF 第 114 页），比较取 1628.0。")
    P(f"- 自检：两表所附「今各宿度」「各宿黄道本度」与相邻积度之差逐宿比较，不合 {len(width_bad)} 处"
      + ("。" if not width_bad else "：" + "、".join(width_bad) + "。"))
    P("- 比较：Hipparcos 经空间运动、IAU 2006 岁差推至 1628.0 当日平春分点；不计章动、光行差（各 ≤ 20″）。")
    P("")
    P("### 逐宿（崇祯 − Hipparcos，角分）")
    P("")
    P("认星取三套既有距星中与原书赤经、黄经合差最小者。原书只给两量，两条经圈交于一点，V ≤ 6 的暗星偶有更近者，"
      "列于末栏备查而不改认（距星向取亮星，且这些宿的三套距星本相同）。「灵台」列：本表认星与《灵台仪象志》所取是否同星。")
    P("")
    P("| 宿 | HIP | 崇祯 α | Hipparcos α | Δα′ | 崇祯 λ | Hipparcos λ | Δλ′ | Hipparcos β | 灵台 | 另有更近之星 |")
    P("|---|---|---|---|---|---|---|---|---|---|---|")
    for x in res:
        alt = f"HIP {x['alt']}（合差 {x['alt_score']:.0f}′）" if x["alt"] else ""
        P(f"| {x['m']} | {x['h']} | {fmt_dm(x['ra'])} | {fmt_dm(x['hra'])} | {x['dra']:+.1f} | {fmt_dm(x['lon'])} | {fmt_dm(x['hlon'])} | "
          f"{x['dlon']:+.1f} | {fmt_dm(x['hlat'])} | {'同' if lt[x['i']] == x['h'] else '异 ' + str(lt[x['i']])} | {alt} |")
    P("")
    P("| 量 | 中位数 | 中位绝对偏差 | 均方根 | 最大 |")
    P("|---|---|---|---|---|")
    P(f"| Δα（角分） | {s_ra[0]:+.1f} | {s_ra[1]:.1f} | {s_ra[2]:.1f} | {s_ra[3]:.1f} |")
    P(f"| Δλ（角分） | {s_lon[0]:+.1f} | {s_lon[1]:.1f} | {s_lon[2]:.1f} | {s_lon[3]:.1f} |")
    big = [x for x in res if max(abs(x["dra"]), abs(x["dlon"])) > 10]
    P("")
    P(f"- 以黄经差中位数为零反推历元：{ep_cz:.0f} 年（书题 1628）。")
    P(f"- 与《灵台仪象志》距星同星 {len(same_lt)}/28" + ("" if len(same_lt) == 28 else "，异："
      + "、".join(names[i] for i in range(1, 29) if i not in same_lt)) + "。奎 η And、觜 λ Ori、昴 η Tau 三宿已见于《崇祯历书》，"
      "参仍 δ Ori、觜积度在参后（「参前觜后」）。")
    P(f"- |Δ| > 10′ 者 {len(big)} 宿：" + "；".join(f"{x['m']} Δα {x['dra']:+.0f}′ Δλ {x['dlon']:+.0f}′" for x in big) + "。")
    P("  其中奎、鬼、张、轸 Δλ 在 5′ 内而 Δα 大，像是赤经由黄经纬换算、所用黄纬有误；"
      "尾、柳 Δα、Δλ 同向同量，是黄经本身偏大约 50′；尾（μ¹ Sco，赤纬 −37°）、箕（γ Sgr，−30°）在第谷所在纬度几不出地平，"
      "其位置当另有来源。女宿两量皆远离 ε Aqr，也无亮星与之相合。这些都是原书数字本身的问题（两表互校无误），认星不受影响——"
      "三套距星在这几宿相同。以上为推测，未据他书核对。")
    P("")

    # 宿度
    def widths(lons):
        out = []
        for k in range(28):
            d = (lons[(k + 1) % 28] - lons[k]) % 360
            out.append(0.0 if d > 180 else d)
        for k in range(28):
            if out[k] == 0.0:
                out[(k - 1) % 28] = (lons[(k + 1) % 28] - lons[(k - 1) % 28]) % 360
        return out

    w_tab = [dm(*r["lon_width"].split(".")) for r in cz]
    w_cz = widths([c28.pos(ident[i])[2] for i in range(1, 29)])
    w_lt = widths([c28.pos(lt[i])[2] for i in range(1, 29)])
    P("### 黄道宿度（360° 制，1628.0）")
    P("")
    P("清时宪历以黄经计宿度。「推算」按 Hipparcos 1628.0 黄经之差；距星与《灵台仪象志》同星时两推算列相同。")
    P("")
    P("| 宿 | 崇祯原表本度 | 推算（崇祯认星） | 原表 − 推算′ | 推算（灵台仪象志距星） |")
    P("|---|---|---|---|---|")
    for k, r in enumerate(cz):
        P(f"| {r['mansion']} | {fmt_dm(w_tab[k])} | {fmt_dm(w_cz[k])} | {(w_tab[k] - w_cz[k]) * 60:+.0f} | {fmt_dm(w_lt[k])} |")
    dw = [(w_tab[k] - w_cz[k]) * 60 for k in range(28)]
    sw = stats(dw)
    P("")
    P(f"原表宿度 − 推算：中位 {sw[0]:+.1f}′、中位绝对偏差 {sw[1]:.1f}′、最大 {sw[3]:.0f}′。")
    P("")

    # ---------------- 天文正义 ----------------
    if tw:
        tw_bad = []
        lons = [GONG[r["gong"]] + dm(r["gong_deg"], r["gong_min"]) for r in tw]
        for k, r in enumerate(tw):
            if abs(lons[k] - float(r["lon_deg_total"])) > 1e-3:
                tw_bad.append(f"{r['mansion']}（宫度换算）")
            w = (lons[(k + 1) % 28] - lons[k]) % 360
            if round(w * 60) != int(r["width_deg"]) * 60 + int(r["width_min"]):
                tw_bad.append(f"{r['mansion']}（其间）")
        hips_tw = [int(r["hip"]) for r in tw]
        ep_tw = fit_epoch(cat, hips_tw, lons, float(tw[0]["epoch_guess"]))
        ct = Catalog(cat, round(ep_tw))
        rows_tw, dl, db = [], [], []
        for k, r in enumerate(tw):
            i, h = int(r["index"]), int(r["hip"])
            beta = 90.0 - dm(r["qu_ji_deg"], r["qu_ji_min"])
            _, _, hl, hb = ct.pos(h)
            # 另查三套距星中是否有更合者
            score = lambda hh: math.hypot(wrap180(lons[k] - ct.pos(hh)[2]), beta - ct.pos(hh)[3])
            best = min({det[s][i] for s, _ in SYSTEMS} | {h}, key=score)
            dl.append(wrap180(lons[k] - hl) * 60)
            db.append((beta - hb) * 60)
            rows_tw.append((r, h, lons[k], hl, beta, hb, dl[-1], db[-1], best))
        sl, sb = stats(dl), stats(db)
        P("## 二、《钦定天文正义》卷四「经星去极入宿度」")
        P("")
        P("- 底本：网盘 `/钦定天文正义/` 分册本第三册（卷四，PDF 第 2—63 页）。卷首按语「今用现测各星黄道经纬度分，仍以去极入宿列为表」，"
          "岁差「每年差五十一秒」。每宿首条给距星「去极」（距黄极，β = 90° − 去极）与宫度（黄经），并「其间」宿度。书中未明言历元。")
        P(f"- 自检：宫度换算与「其间」宿度逐宿复核，不合 {len(tw_bad)} 处" + ("。" if not tw_bad else "：" + "、".join(tw_bad) + "。"))
        P(f"- 以黄经差中位数为零反推历元：{ep_tw:.0f} 年；下表按 {round(ep_tw)}.0 比较。")
        P("")
        P("| 宿 | 距星 | HIP | 正义 λ | Hipparcos λ | Δλ′ | 正义 β | Hipparcos β | Δβ′ | 灵台仪象志 | 仪象考成 |")
        P("|---|---|---|---|---|---|---|---|---|---|---|")
        for r, h, l, hl, b, hb, d1, d2, best in rows_tw:
            i = int(r["index"])
            P(f"| {r['mansion']} | {r['star']} | {h}{'' if best == h else f'（{best} 更合）'} | {fmt_dm(l)} | {fmt_dm(hl)} | {d1:+.1f} | "
              f"{fmt_dm(b)} | {fmt_dm(hb)} | {d2:+.1f} | {'同' if lt[i] == h else '异 ' + str(lt[i])} | {'同' if det['qing'][i] == h else '异 ' + str(det['qing'][i])} |")
        P("")
        P("| 量 | 中位数 | 中位绝对偏差 | 均方根 | 最大 |")
        P("|---|---|---|---|---|")
        P(f"| Δλ（角分） | {sl[0]:+.1f} | {sl[1]:.1f} | {sl[2]:.1f} | {sl[3]:.1f} |")
        P(f"| Δβ（角分） | {sb[0]:+.1f} | {sb[1]:.1f} | {sb[2]:.1f} | {sb[3]:.1f} |")
        P("")
        diff_q = [r["mansion"] for r, h, *_ in rows_tw if det["qing"][int(r["index"])] != h]
        diff_l = [r["mansion"] for r, h, *_ in rows_tw if lt[int(r["index"])] != h]
        P(f"- 与《仪象考成》（本项目 `qing`）距星异：{'、'.join(diff_q) or '无'}；与《灵台仪象志》异：{'、'.join(diff_l) or '无'}。")
        P("- 参距「中东星」即 ζ Ori、觜前参后、昴距西星 17 Tau，均为乾隆十九年（1754）改参以后的《仪象考成》体系；"
          "反推历元 1744 与《仪象考成》测星之年（乾隆九年甲子）相合，残差中位绝对偏差仅半角分，当即据《仪象考成》星表，不是清初之物。")
        P("")

    # ---------------- 按原表划宿：《清史稿》1645—1673 所记入宿度 ----------------
    mode_section(P)

    # ---------------- 结论 ----------------
    P("## 三、清初 1645—1673 距星取法与接入")
    P("")
    P("- 顺治二年（1645）行时宪历，宿度依《崇祯历书》（西洋新法历书）黄道宿度。本检验所得《崇祯历书》28 宿认星与 "
      f"`qing_lingtai`（《灵台仪象志》）{'全同' if len(same_lt) == 28 else '同 ' + str(len(same_lt)) + ' 宿'}：奎 η And、觜 λ Ori、昴 η Tau 之改"
      "始于《崇祯历书》，《灵台仪象志》（1674）承之。")
    P("- 据此在距星表新增 `chongzhen` 体系（HIP 同 `qing_lingtai`，由 `scripts/sky_culture/lunar_mansions.py` 生成）。"
      "天象检验 `tests_records.rs::lodge_model`：《清史稿》1644—1673 取 `chongzhen`（黄道 360°）、1674—1753 取 `qing_lingtai`、1754 起取 `qing`。"
      "起点取 1644 而非时宪历颁行的 1645：顺治元年有宿度者两条——正月丙辰「土金同躔壁三度」、三月乙巳「土水同躔壁八度」，"
      "皆在汤若望七月受命掌钦天监、八月朔日食以新法验之前——按陈卓赤道体系推算皆入奎宿（异宿），按崇祯距星黄道 360° 为壁 3.3、壁 9.4（同宿）；"
      "《清史稿》此卷所记当是入关后钦天监按新法整理之数，故整年按新法计，不按月切换。星图年段仍以 1645 颁行为界。"
      "星图仲裁 `system_periods.csv`：《象林》1634—1644、`chongzhen` 1645—1673（崇祯年间朝廷仍行大统历，《崇祯历书》未颁行，"
      "至顺治二年方由时宪历起用；星官与星名用同时代的《象林》）。")
    P("- 距星位置有两种取法，环境变量 `PHENOMENA_CHONGZHEN_LODGES=table|hipparcos`：`table` 按原表黄道积度"
      f"（`EclipticLodgeTable::offsets` 把原表在 1628 年对推算之差加到当时推算黄经上），`hipparcos` 按 Hipparcos。两者宿度差中位 {sw[0]:+.1f}′、"
      f"最大 {sw[3]:.0f}′（牛、女、心、尾、箕诸宿）。")
    if MODE_SUMMARY:
        n, sa, sb_, nb, aa, ab, wa, wb, c_all, c_big = MODE_SUMMARY
        P(f"- 缺省 `hipparcos`，据上节：{n} 条中同宿两法皆 {sa if sa == sb_ else (sa, sb_)}；去掉共有系统差后 Hipparcos 更合"
          f"（{c_all[0]} 条中 {c_all[4]} 对 {c_all[3]}；尾、箕、女、柳四宿 {c_big[0]} 条中 {c_big[4]} 对 {c_big[3]}），"
          "时宪历推步所用距星黄经不随《恒星历指》积度表之误。")
    if tw:
        P(f"- 《钦定天文正义》距星历元约 {ep_tw:.0f} 年、参用 ζ Ori，属《仪象考成》体系，不宜用于 1645—1673。")
    P("")
    with open(OUT_MD, "w", encoding="utf-8") as f:
        f.write("\n".join(out) + "\n")
    print(f"崇祯：认星 {[ident[i] for i in range(1, 29)]}")
    print(f"  Δα 中位 {s_ra[0]:+.2f}′ MAD {s_ra[1]:.2f}′ RMS {s_ra[2]:.1f}′ 最大 {s_ra[3]:.1f}′；Δλ 中位 {s_lon[0]:+.2f}′ MAD {s_lon[1]:.2f}′ RMS {s_lon[2]:.1f}′ 最大 {s_lon[3]:.1f}′")
    print(f"  反推历元 {ep_cz:.1f}；宿度自检不合 {width_bad}；与灵台同 {len(same_lt)}/28；宿度差 中位 {sw[0]:+.1f}′ 最大 {sw[3]:.0f}′")
    if tw:
        print(f"天文正义：反推历元 {ep_tw:.1f}；Δλ 中位 {sl[0]:+.2f}′ MAD {sl[1]:.2f}′ 最大 {sl[3]:.1f}′；Δβ 中位 {sb[0]:+.2f}′ MAD {sb[1]:.2f}′ 最大 {sb[3]:.1f}′；自检不合 {tw_bad}")
        print(f"  与仪象考成异 {diff_q}；与灵台异 {diff_l}")


if __name__ == "__main__":
    main()
