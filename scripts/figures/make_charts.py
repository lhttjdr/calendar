#!/usr/bin/env python3
"""
生成读者文档站（site/）统计图的数据：site/public/charts/<名>.json，由站点的 <Chart> 组件绘制。

数据全部来自仓库内的原始数据或检验结果，不手填：
- VSOP87 → DE406 拟合残差：复用 scripts/ephemeris/vsop87_de_planet_fit.py 的 VSOP87B 全级数与 DE406 读取，
  改正系数读 data/fit/vsop87-de406-planets.txt（与 Rust 管线所用同一文件）；
- ΔT：Espenak–Meeus 2006 多项式（与 rust/core time::scales::delta_t 同系数）、Morrison–Stephenson 2021 逐年表（vendor/delta_t）；
- NASA 日月食典对照、史书天象按朝代总表：data/phenomena/checks/ 与 doc/astronomy/phenomena-checks.md（脚本生成）；
- 以本纪核验历日：doc/calendar/historical-calendar.md 的汇总表（scripts/calendar_data/verify_benji_rili.py 结果）。

用法（仓库根）：uv run --project scripts --only-group chapront-fit python scripts/figures/make_charts.py
DE406 文件：vendor/jpl/de406/de406.bsp（或环境变量 DE406_BSP）。
"""

import csv
import importlib.util
import json
import re
import sys
from pathlib import Path

import numpy as np

OUT = Path("site/public/charts")
RAD2AS = 180 / np.pi * 3600
J2000 = 2451545.0


def save(name: str, spec: dict) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"{name}.json").write_text(json.dumps(spec, ensure_ascii=False, separators=(",", ":")))
    print("→", OUT / f"{name}.json")


def r(v: float, nd: int = 4) -> float:
    return float(f"{v:.{nd}g}")


# ---------------------------------------------------------------- VSOP87 → DE406 残差

def load_fit_module():
    sys.argv = [sys.argv[0], "de406"]
    spec = importlib.util.spec_from_file_location("vsopfit", "scripts/ephemeris/vsop87_de_planet_fit.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def parse_patch(path: str, body: str) -> dict:
    """读改正系数文件中某天体的 L、B、R 段：返回 {变量: (长期系数, [(f, [C0,S0,C1,S1,…]), …])}。"""
    out, cur = {}, None
    for line in open(path):
        line = line.strip()
        m = re.match(r"\[(\w+) ([LBR])\]", line)
        if m:
            cur = m.group(2) if m.group(1) == body else None
            if cur:
                out[cur] = (None, [])
            continue
        if not line or line.startswith("#") or cur is None:
            continue
        nums = [float(x) for x in line.split()]
        sec, per = out[cur]
        if sec is None:
            out[cur] = (nums, per)
        else:
            per.append((nums[0], nums[1:]))
    return out


def eval_patch(seg, t: np.ndarray) -> np.ndarray:
    sec, per = seg
    v = sum(a * t**k for k, a in enumerate(sec))
    for f, cs in per:
        c, s = np.cos(f * t), np.sin(f * t)
        for k in range(len(cs) // 2):
            v = v + t**k * (cs[2 * k] * c + cs[2 * k + 1] * s)
    return v


def charts_vsop_residuals() -> None:
    from jplephem.spk import SPK

    mod = load_fit_module()
    kernel = SPK.open(mod.bsp_path())
    jd = np.linspace(mod.JD_MIN, mod.JD_MAX, 1500)
    years = 2000 + (jd - J2000) / 365.25
    vsop = mod.Vsop87B("vendor/vsop87/VSOP87B.ear")
    t, res, _, _ = mod.residuals(vsop, kernel, None, jd)
    patch = parse_patch("data/fit/vsop87-de406-planets.txt", "ear")
    for vi, (var, label) in enumerate([("L", "日心黄经"), ("B", "日心黄纬")]):
        before = res[vi]
        after = before - eval_patch(patch[var], t)
        save(f"fit-earth-{var.lower()}", {
            "type": "line",
            "x": {"label": "年份"},
            "y": {"label": f"DE406 − VSOP87，{label}", "unit": "″"},
            "series": [
                {"name": "改正前（VSOP87 原样）", "data": [[r(y, 6), r(v)] for y, v in zip(years, before)]},
                {"name": "改正后（VSOP87 + 拟合改正）", "data": [[r(y, 6), r(v)] for y, v in zip(years, after)]},
            ],
            "caption": f"地球{label}：VSOP87B 对 DE406 的残差，改正前后（前 3000—公元 3000 年，1500 个等距时刻）",
        })
        print(f"  {var}: 改正前最大 {np.abs(before).max():.3f}″，改正后最大 {np.abs(after).max():.4f}″")


# ---------------------------------------------------------------- ΔT

def espenak_meeus(y: float) -> float:
    """与 rust/core time::scales::delta_t_eval 同系数（Espenak & Meeus 2006）。"""
    if y < -500:
        u = (y - 1820) / 100
        return -20 + 32 * u * u
    if y <= 500:
        u = y / 100
        return 10583.6 - 1014.41 * u + 33.78311 * u**2 - 5.952053 * u**3 - 0.1798452 * u**4 + 0.022174192 * u**5 + 0.0090316521 * u**6
    if y <= 1600:
        u = (y - 1000) / 100
        return 1574.2 - 556.01 * u + 71.23472 * u**2 + 0.319781 * u**3 - 0.8503463 * u**4 - 0.005050998 * u**5 + 0.0083572073 * u**6
    if y < 1700:
        t = y - 1600
        return 120 - 0.9808 * t - 0.01532 * t**2 + t**3 / 7129
    if y < 1800:
        t = y - 1700
        return 8.83 + 0.1603 * t - 0.0059285 * t**2 + 0.00013336 * t**3 - t**4 / 1174000
    if y < 1860:
        t = y - 1800
        return 13.72 - 0.332447 * t + 0.0068612 * t**2 + 0.0041116 * t**3 - 0.00037436 * t**4 + 0.0000121272 * t**5 - 0.0000001699 * t**6 + 0.000000000875 * t**7
    if y < 1900:
        t = y - 1860
        return 7.62 + 0.5737 * t - 0.251754 * t**2 + 0.01680668 * t**3 - 0.0004473624 * t**4 + t**5 / 233174
    if y < 1920:
        t = y - 1900
        return -2.79 + 1.494119 * t - 0.0598939 * t**2 + 0.0061966 * t**3 - 0.000197 * t**4
    if y < 1941:
        t = y - 1920
        return 21.20 + 0.84493 * t - 0.076100 * t**2 + 0.0020936 * t**3
    if y < 1961:
        t = y - 1950
        return 29.07 + 0.407 * t - t**2 / 233 + t**3 / 2547
    if y < 1986:
        t = y - 1975
        return 45.45 + 1.067 * t - t**2 / 260 - t**3 / 718
    if y < 2005:
        t = y - 2000
        return 63.86 + 0.3345 * t - 0.060374 * t**2 + 0.0017275 * t**3 + 0.000651814 * t**4 + 0.00002373599 * t**5
    if y <= 2050:
        t = y - 2000
        return 62.92 + 0.32217 * t + 0.005589 * t**2
    if y < 2150:
        return -20 + 32 * ((y - 1820) / 100) ** 2 - 0.5628 * (2150 - y)
    u = (y - 1820) / 100
    return -20 + 32 * u * u


def ms2021_table() -> list[tuple[float, float, float]]:
    rows = []
    for line in open("vendor/delta_t/morrison_stephenson_2021/Table-DT-lod4500yrs.2020.txt"):
        p = line.split()
        if len(p) >= 3 and re.fullmatch(r"-?\d+\.\d", p[0]):
            rows.append((float(p[0]), float(p[1]), float(p[2])))
    return rows


def charts_delta_t() -> None:
    ms = [row for row in ms2021_table() if -2000 <= row[0] <= 2100]
    years = [row[0] for row in ms]
    save("delta-t", {
        "type": "line",
        "x": {"label": "年份"},
        "y": {"label": "ΔT = TT − UT", "unit": "s"},
        "series": [
            {"name": "Morrison–Stephenson 2021（阴影为 ±1σ）", "data": [[y, d] for y, d, _ in ms], "band": [[y, d - e, d + e] for y, d, e in ms]},
            {"name": "Espenak–Meeus 2006", "data": [[y, r(espenak_meeus(y), 6)] for y in years]},
        ],
        "caption": "两种 ΔT 模型（前 2000—公元 2100 年）。古代以千秒计，近代为数十秒",
    })
    save("delta-t-diff", {
        "type": "line",
        "x": {"label": "年份"},
        "y": {"label": "Espenak–Meeus − MS2021", "unit": "s"},
        "series": [
            {"name": "两模型之差（阴影为 MS2021 的 ±1σ）", "data": [[y, r(espenak_meeus(y) - d, 6)] for y, d, _ in ms],
             "band": [[y, -e, e] for y, _, e in ms]},
        ],
        "caption": "两模型之差与 MS2021 自身的不确定度相比较：公元前多数年代差异落在 1σ 之内",
    })


# ---------------------------------------------------------------- NASA 日月食典对照

def read_check_csv(path: str) -> list[dict]:
    with open(path) as f:
        lines = [ln for ln in f if not ln.startswith("#")]
    return list(csv.DictReader(lines))


def nasa_year(s: str) -> float:
    m = re.match(r"(-?\d+) (\w{3}) (\d+)", s.strip())
    mon = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].index(m.group(2))
    return int(m.group(1)) + (mon + (int(m.group(3)) - 1) / 31) / 12


def charts_nasa() -> None:
    for kind, label in [("solar", "日食"), ("lunar", "月食")]:
        series = []
        for span, name in [("-0199_-0100", "前 199—前 100 年"), ("2001_2100", "2001—2100 年")]:
            rows = read_check_csv(f"data/phenomena/checks/nasa_{kind}_{span}.csv")
            key = next(k for k in rows[0] if "时刻差" in k)
            series.append((name, [[r(nasa_year(row["NASA 日期(TD)"]), 6), float(row[key])] for row in rows if row[key]]))
        for name, data in series:
            slug = "ancient" if "前" in name else "modern"
            save(f"nasa-{kind}-{slug}", {
                "type": "scatter",
                "x": {"label": "年份"},
                "y": {"label": "极大时刻差（本项目 − NASA）", "unit": "s"},
                "series": [{"name": name, "data": data}],
                "caption": f"NASA 五千年{label}典逐次对照，{name}，共 {len(data)} 次",
            })


# ---------------------------------------------------------------- 史书天象（按朝代）

def charts_records() -> None:
    text = Path("doc/astronomy/phenomena-checks.md").read_text()
    sec = text[text.index("## 七、史书天象记录（按朝代）"):]
    sec = sec[: sec.index("\n### ", sec.index("### 总表") + 5)]
    dyn, moon, planet = [], [], []
    for line in sec.splitlines():
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 6 or cells[0] in ("朝代", "---"):
            continue
        mm = re.match(r"(\d+)，\d+（(\d+)%）", cells[3])
        pm = re.match(r"(\d+)，\d+（(\d+)%）", cells[4])
        if not pm or int(pm.group(1)) < 10:
            continue
        dyn.append(cells[0])
        moon.append(int(mm.group(2)) if mm else 0)
        planet.append(int(pm.group(2)))
    save("records-conjunction", {
        "type": "bar",
        "x": {"label": "朝代"},
        "y": {"label": "角距 ≤ 1° 的比例", "unit": "%", "min": 0, "max": 100},
        "categories": dyn,
        "series": [{"name": "月凌犯", "values": moon}, {"name": "行星凌犯与相合", "values": planet}],
        "caption": "史书凌犯记录按所记日期推算的角距在 1° 以内的比例（行星记录不足 10 条的朝代略去）。随机日期的基线约 1%—2%",
    })

    hist = Path("doc/calendar/historical-calendar.md").read_text()
    blk = hist[hist.index("| 朝代 | 明书朔日 |"):]
    names, rate = [], []
    for line in blk.splitlines()[2:]:
        if not line.startswith("|"):
            break
        c = [x.strip() for x in line.strip("|").split("|")]
        names.append(c[0])
        rate.append(round(100 * int(c[2]) / int(c[1]), 1))
    save("benji-shuo", {
        "type": "bar",
        "x": {"label": "朝代"},
        "y": {"label": "明书朔日与张表相合", "unit": "%", "min": 0, "max": 100},
        "categories": names,
        "series": [{"name": "相合比例", "values": rate}],
        "caption": "正史本纪明书「某月某干支朔」与张表朔日相合的比例",
    })


# ---------------------------------------------------------------- 岁差：P03 与 Vondrák 2011

AS = np.pi / 180 / 3600
EPS0 = 84381.406 * AS
# Vondrák et al. (2011) A&A 534, A22，表 1—2（与 rust/core frame::precession::vondrak2011 同系数）
V_PQ_POL = ([5851.607687, -0.1189000, -0.00028913, 0.000000101], [-1600.886300, 1.1689818, -0.00000020, -0.000000437])
V_PQ_PER = [(708.15, -5486.751211, -684.661560, 667.666730, -5523.863691), (2309.00, -17.127623, 2446.283880, -2354.886252, -549.747450),
            (1620.00, -617.517403, 399.671049, -428.152441, -310.998056), (492.20, 413.442940, -356.652376, 376.202861, 421.535876),
            (1183.00, 78.614193, -186.387003, 184.778874, -36.776172), (622.00, -180.732815, -316.800070, 335.321713, -145.278396),
            (882.00, -87.676083, 198.296071, -185.138669, -34.744450), (547.00, 46.140315, 101.135679, -120.972830, 22.885731)]
V_XY_POL = ([5453.282155, 0.4252841, -0.00037173, -0.000000152], [-73750.930350, -0.7675452, -0.00018725, 0.000000231])
V_XY_PER = [(256.75, -819.940624, 75004.344875, 81491.287984, 1558.515853), (708.15, -8444.676815, 624.033993, 787.163481, 7774.939698),
            (274.20, 2600.009459, 1251.136893, 1251.296102, -2219.534038), (241.45, 2755.175630, -1102.212834, -1257.950837, -2523.969396),
            (2309.00, -167.659835, -2660.664980, -2966.799730, 247.850422), (492.20, 871.855056, 699.291817, 639.744522, -846.485643),
            (396.10, 44.769698, 153.167220, 131.600209, -1393.124055), (288.90, -512.313065, -950.865637, -445.040117, 368.526116),
            (231.10, -819.415595, 499.754645, 584.522874, 749.045012), (1610.00, -538.071099, -145.188210, -89.756563, 444.704518),
            (620.00, -189.793622, 558.116553, 524.429630, 235.934465), (157.87, -402.922932, -23.923029, -13.549067, 374.049623),
            (220.30, 179.516345, -165.405086, -210.157124, -171.330180), (1200.00, -9.814756, 9.344131, -44.919798, -22.899655)]


def _two(pol, per, t):
    a = sum(c * t**k for k, c in enumerate(pol[0]))
    b = sum(c * t**k for k, c in enumerate(pol[1]))
    for P, ac, bc, as_, bs in per:
        th = 2 * np.pi * t / P
        a += ac * np.cos(th) + as_ * np.sin(th)
        b += bc * np.cos(th) + bs * np.sin(th)
    return a * AS, b * AS


def vondrak_matrix(t: float) -> np.ndarray:
    p, q = _two(V_PQ_POL, V_PQ_PER, t)
    z = np.sqrt(max(0.0, 1 - p * p - q * q))
    pecl = np.array([p, -q * np.cos(EPS0) - z * np.sin(EPS0), -q * np.sin(EPS0) + z * np.cos(EPS0)])
    x, y = _two(V_XY_POL, V_XY_PER, t)
    pequ = np.array([x, y, np.sqrt(max(0.0, 1 - x * x - y * y))])
    eqx = np.cross(pequ, pecl)
    eqx /= np.linalg.norm(eqx)
    return np.array([eqx, np.cross(pequ, eqx), pequ])


def _rot(axis: int, a: float) -> np.ndarray:
    c, s = np.cos(a), np.sin(a)
    if axis == 2:
        return np.array([[c, s, 0], [-s, c, 0], [0, 0, 1]])
    return np.array([[c, 0, -s], [0, 1, 0], [s, 0, c]])


def p03_matrix(t: float) -> np.ndarray:
    poly = lambda c: sum(a * t**k for k, a in enumerate(c)) * AS
    zeta = poly([2.650545, 2306.083227, 0.2988499, 0.01801828, -5.971e-6, -3.173e-7])
    z = poly([-2.650545, 2306.077181, 1.0927348, 0.01826837, -0.000028596, -2.904e-7])
    theta = poly([0.0, 2004.191903, -0.4294934, -0.04182264, -7.089e-6, -1.274e-7])
    return _rot(2, -z) @ _rot(1, theta) @ _rot(2, -zeta)


def charts_precession() -> None:
    years = np.arange(-3000, 3001, 25)
    eqx, pole = [], []
    for y in years:
        t = (y - 2000) / 100.0
        a, b = p03_matrix(t), vondrak_matrix(t)
        eqx.append(np.degrees(np.arccos(np.clip(a[0] @ b[0], -1, 1))) * 3600)
        pole.append(np.degrees(np.arccos(np.clip(a[2] @ b[2], -1, 1))) * 3600)
    save("precession-p03-vondrak", {
        "type": "line",
        "x": {"label": "年份"},
        "y": {"label": "P03 与 Vondrák 2011 之差", "unit": "″"},
        "series": [
            {"name": "当日平春分点方向之差", "data": [[int(y), r(v)] for y, v in zip(years, eqx)]},
            {"name": "当日平天极方向之差", "data": [[int(y), r(v)] for y, v in zip(years, pole)]},
        ],
        "caption": "两种岁差模型给出的当日平赤道架之差。J2000 前后几个世纪两者一致到毫角秒级，越往远处 P03 多项式越偏离",
    })
    i = list(years).index(-1000)
    print(f"  岁差：前 1000 年 春分点差 {eqx[i]:.2f}″，天极差 {pole[i]:.2f}″；前 3000 年 {eqx[0]:.1f}″ / {pole[0]:.1f}″")


# ---------------------------------------------------------------- 章动：IAU 2000A 日月项

DELAUNAY = [  # IERS Conventions 2010 式 5.43，角秒，T 的 0—4 次
    [485868.249036, 1717915923.2178, 31.8792, 0.051635, -0.00024470],   # l
    [1287104.79305, 129596581.0481, -0.5532, 0.000136, -0.00001149],    # l'
    [335779.526232, 1739527262.8478, -12.7512, -0.001037, 0.00000417],  # F
    [1072260.70369, 1602961601.2090, -6.3706, 0.006593, -0.00003169],   # D
    [450160.398036, -6962890.5431, 7.4722, 0.007702, -0.00005939],      # Ω
]


def nutation_luni_solar(T: np.ndarray) -> tuple[np.ndarray, np.ndarray, int]:
    """IERS 表 5.3a+5.3b 日月项（vendor/IAU2000/tab5.3a.txt）：Δψ、Δε，单位角秒。"""
    rows = [list(map(float, ln.split())) for ln in open("vendor/IAU2000/tab5.3a.txt") if ln.strip() and not ln.startswith("*")]
    args = [sum(c * T**k for k, c in enumerate(d)) * AS for d in DELAUNAY]
    dpsi = np.zeros_like(T)
    deps = np.zeros_like(T)
    for row in rows:
        n = row[:5]
        A, At, B, Bt, Ac, Act, Bs, Bst = row[6:14]
        th = sum(ni * a for ni, a in zip(n, args))
        dpsi += (A + At * T) * np.sin(th) + (Ac + Act * T) * np.cos(th)
        deps += (B + Bt * T) * np.cos(th) + (Bs + Bst * T) * np.sin(th)
    return dpsi / 1000.0, deps / 1000.0, len(rows)


def charts_nutation() -> None:
    years = np.arange(2000.0, 2040.0, 5 / 365.25)
    T = (years - 2000.0) / 100.0
    dpsi, deps, n = nutation_luni_solar(T)
    save("nutation-series", {
        "type": "line",
        "x": {"label": "年份"},
        "y": {"label": "章动", "unit": "″"},
        "series": [
            {"name": "黄经章动 Δψ", "data": [[r(y, 7), r(v)] for y, v in zip(years, dpsi)]},
            {"name": "交角章动 Δε", "data": [[r(y, 7), r(v)] for y, v in zip(years, deps)]},
        ],
        "caption": f"2000—2040 年的章动（IAU 2000A 日月项 {n} 项，不含毫角秒级的行星项；每 5 日一点）",
        "table": {"maxRows": 30},
    })
    print(f"  章动：{n} 项，Δψ 范围 {dpsi.min():.2f}…{dpsi.max():.2f}″，Δε {deps.min():.2f}…{deps.max():.2f}″")


# ---------------------------------------------------------------- 朔望月长度、定气间隔（参考表 TDBtimes）

TERM_NAMES = ["小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至",
              "小暑", "大暑", "立秋", "处暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪", "冬至"]


def read_tdbtimes(path: str = "vendor/月相和二十四节气的计算/TDBtimes.txt") -> dict[int, dict]:
    out = {}
    with open(path) as f:
        head = f.readline().split()
        for line in f:
            p = line.split()
            row = dict(zip(head, p))
            jd0 = float(row["jd0"])
            terms = [jd0 + float(row["Z11a"])] + [jd0 + float(row[h]) for h in head[3:27]]
            moons = [jd0 + float(row[h]) for h in head if h.startswith("Q0_")]
            out[int(row["year"])] = {"terms": terms, "moons": moons}
    return out


def charts_lunation_terms() -> None:
    tab = read_tdbtimes()
    moons = sorted({round(m, 6) for y in range(1899, 2102) if y in tab for m in tab[y]["moons"]})
    pts = []
    for a, b in zip(moons, moons[1:]):
        yr = 2000 + (a - J2000) / 365.25
        if 1900 <= yr <= 2100 and 29.0 < b - a < 30.0:
            pts.append([r(yr, 7), r(b - a, 6)])
    lens = [p[1] for p in pts]
    save("synodic-month-length", {
        "type": "scatter",
        "x": {"label": "年份"},
        "y": {"label": "朔望月长度", "unit": "日"},
        "series": [{"name": "相邻两次合朔的间隔", "data": pts}],
        "caption": f"1900—2100 年 {len(pts)} 个朔望月的实际长度（参考表 TDBtimes，DE441），{min(lens):.3f}—{max(lens):.3f} 日，平均 {sum(lens) / len(lens):.6f} 日",
        "table": {"maxRows": 40},
    })
    t = tab[2026]["terms"]  # 2025 年冬至起的 25 个节气
    gaps = [round(b - a, 4) for a, b in zip(t, t[1:])]
    save("solar-term-intervals", {
        "type": "bar",
        "x": {"label": "节气（到该节气的间隔）"},
        "y": {"label": "与上一节气相隔", "unit": "日", "min": 14, "max": 16},
        "categories": TERM_NAMES,
        "series": [{"name": "定气间隔", "values": gaps}],
        "caption": "2026 年各节气与前一节气的间隔（定气，参考表 TDBtimes）。近日点在一月初，冬季间隔最短；远日点在七月初，夏季最长",
    })
    print(f"  朔望月 {len(pts)} 个：{min(lens):.4f}—{max(lens):.4f}；定气间隔 {min(gaps)}—{max(gaps)} 日")


# ---------------------------------------------------------------- ELP/MPP02 拟合 DE441（data/fit/elpmpp02-de441-residuals.csv）

def charts_elp_fit() -> None:
    rows = list(csv.DictReader(open("data/fit/elpmpp02-de441-residuals.csv")))
    fit = [r_ for r_ in rows if r_["constants"] == "fit"]
    paper = [r_ for r_ in rows if r_["constants"] != "fit"]
    mid = lambda r_: int(r_["century_start"]) + 50
    save("elp-de441-fit", {
        "type": "line",
        "x": {"label": "年份（世纪中点）"},
        "y": {"label": "ELP/MPP02（拟合 DE441 的常数集）− DE441，月亮黄经", "unit": "″"},
        "series": [
            {"name": "每世纪最大差", "data": [[mid(r_), float(r_["max_abs_dlon_arcsec"])] for r_ in fit]},
            {"name": "每世纪 RMS", "data": [[mid(r_), float(r_["rms_dlon_arcsec"])] for r_ in fit]},
        ],
        "caption": "按两步法拟合 DE441 后，月亮黄经与 DE441 之差（前 3000—公元 3000 年，每世纪 60 个随机时刻）",
    })
    save("elp-paper-vs-de441", {
        "type": "line",
        "x": {"label": "年份（世纪中点）"},
        "y": {"label": "月亮黄经与 DE441 之差，每世纪 RMS", "unit": "″"},
        "series": [
            {"name": "论文常数集（DE405 + Table 6）", "data": [[mid(r_), float(r_["rms_dlon_arcsec"])] for r_ in paper]},
            {"name": "拟合 DE441 的常数集", "data": [[mid(r_), float(r_["rms_dlon_arcsec"])] for r_ in fit]},
        ],
        "caption": "论文常数集贴近 DE406，对 DE441 在古代差到数百角秒；按同法拟合 DE441 后降到 1″ 量级",
    })
    # ΔT 配套量：δΔT = −Δλ(T)/n_月，Δλ 为各常数集对 DE431 的三次式（T 为距 J2000 的儒略世纪）；
    # 系数直接取 rust/core/src/astronomy/time/delta_t.rs 的 lunar_longitude_offset 常数（由 tests_lunar_ndot 断言与实测一致）
    n_moon = 0.549016  # ″/s
    src = open("rust/core/src/astronomy/time/delta_t.rs").read()

    def offset_coeffs(const):
        m = re.search(rf"pub const {const}: LunarLongitudeOffset = LunarLongitudeOffset \{{.*?coefficients: &\[([^\]]*)\]", src, re.S)
        return [float(x) for x in m.group(1).split(",") if x.strip()]

    cubic = {
        "拟合 DE431 的常数集（缺省）": offset_coeffs("ELPMPP02_DE431_MINUS_DE431"),
        "论文常数集（DE406）": offset_coeffs("ELPMPP02_DE406_MINUS_DE431"),
        "拟合 DE441 的常数集": offset_coeffs("ELPMPP02_DE441_MINUS_DE431"),
    }
    years = np.arange(-3000, 3001, 25)
    series = []
    for name, c in cubic.items():
        T = (years - 2000) / 100.0
        dl = c[0] + c[1] * T + c[2] * T**2 + c[3] * T**3
        series.append({"name": name, "data": [[int(y), r(-v / n_moon)] for y, v in zip(years, dl)]})
    save("delta-t-lunar-offset", {
        "type": "line",
        "x": {"label": "年份"},
        "y": {"label": "ΔT 配套修正（配 MS2021）", "unit": "s"},
        "series": series,
        "caption": "按月亮平黄经之差（对 MS2021 所配 DE431 的三次式）折算的 ΔT 配套修正：拟合 DE431 的常数集（缺省）在 0.5 秒以内，论文常数集在 6 秒以内，拟合 DE441 的常数集古代达数分钟",
    })


if __name__ == "__main__":
    charts_elp_fit()
    charts_lunation_terms()
    charts_nutation()
    charts_precession()
    charts_delta_t()
    charts_nasa()
    charts_records()
    charts_vsop_residuals()
