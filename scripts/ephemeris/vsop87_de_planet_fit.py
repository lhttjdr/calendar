#!/usr/bin/env python3
"""
VSOP87B 各行星（含地球）→ JPL 数值历表（DE406 或 DE441）改正拟合，方法按 Chapront (2000)《Improvements of planetary theories over 6000 years》
（doc/references/ephemeris/chapront2000.pdf）：

- 残差 ρ = σ_DE − σ_VSOP87，σ 取 VSOP87B 自身的日心变量 L、B、R（J2000 动力学黄道与春分点）；
- 近似形式同论文式 (1)：长期项 Σ_{k≤NSEC} a_k t^k + Σ_f Σ_{k≤MIXT} t^k (C_k cos f t + S_k sin f t)；
  频率 f 取该行星该变量 VSOP87 级数中振幅最大的若干项（论文：「参数列表限于主要项」）；
- 区间取论文 (a)：前 3000 年—3000 年（DE406 全程；DE441 覆盖更长，但 VSOP87 本身超出此区间劣化更快，两者取同一区间以便比较）。论文以 NSEC = 3、MIXT = 2 为例，并说明「可因行星与变量而略有不同」；
  这里水星、金星、地球、天王星、海王星用 NSEC = 3、MIXT = 2、60 个频率；火星、木星、土星的 VSOP87 在远端劣化更快
  （高次长期项与大不等项等频率误差产生的高次混合项），用 NSEC = 5、MIXT = 4、300 个频率。参数按独立检验点（不参与拟合）
  的最大残差选定，改正后与论文表 III 同一量级；
- 模型对系数线性，直接用线性最小二乘，不加人为边界。

JPL 历表（ICRS）→ VSOP87 架：与 rust/core 管线严格一致——管线把 VSOP87 的 MeanEcliptic(J2000) 经 R_x(ε₀)（ε₀ = 84381.406″，
IAU 2006/P03，`frame::precession::mean_obliquity(0)`）到 FK5，再乘 SOFA FK5→ICRS 偏差矩阵（`frame::fixed::fk5_icrs`）到 ICRS；
这里取其逆把 DE406 或 DE441（ICRS）转回，使改正与管线配套。

时间 t = (JD_TDB − J2000)/365250（儒略千年，同 VSOP87）。L、B 改正单位角秒，R 改正单位 AU。

依赖：numpy、jplephem（uv dependency-group chapront-fit）
用法（仓库根）：uv run --project scripts --only-group chapront-fit python scripts/ephemeris/vsop87_de_planet_fit.py [de406|de441]
目标历表缺省 de406。DE406：vendor/jpl/de406/de406.bsp（或环境变量 DE406_BSP）；DE441：vendor/jpl/de441/de441.bsp（本地验证用，
不入库，见该目录 README），每个天体分两段（以 JD 2440432.5 为界），按时刻选段。
输出：data/fit/vsop87-<目标>-planets.txt（两套并存，Rust 侧 `Vsop87DePatch::from_repo(PatchTarget)` 选用）
"""

import os
import sys

import numpy as np

VSOP_DIR = "vendor/vsop87"
TARGET = (sys.argv[1] if len(sys.argv) > 1 else "de406").lower()
if TARGET not in ("de406", "de441"):
    sys.exit("目标历表须为 de406 或 de441")
OUT = f"data/fit/vsop87-{TARGET}-planets.txt"
BODIES = [  # (VSOP87 后缀, NAIF 质心号；地球特殊处理)
    ("mer", 1), ("ven", 2), ("ear", None), ("mar", 4), ("jup", 5), ("sat", 6), ("ura", 7), ("nep", 8),
]
# 各体 (NSEC, MIXT, 频率数)
PARAMS = {"mar": (5, 4, 300), "jup": (5, 4, 300), "sat": (5, 4, 300)}
DEFAULT_PARAMS = (3, 2, 60)
N_SAMPLES = 30000
J2000 = 2451545.0
# DE406 覆盖 JD 625360.5—2816848.5；两端各留 1 年
JD_MIN, JD_MAX = 625360.5 + 366, 2816848.5 - 366
AU_KM = 149597870.7
RAD2AS = 180 / np.pi * 3600

EPS0 = 84381.406 / RAD2AS
RX = np.array([[1, 0, 0], [0, np.cos(EPS0), -np.sin(EPS0)], [0, np.sin(EPS0), np.cos(EPS0)]])
BIAS = np.array([
    [1.0, -8.82462042e-8, -3.85466334e-8],
    [8.82462042e-8, 1.0, -3.30116133e-8],
    [3.85466334e-8, 3.30116133e-8, 1.0],
])
M_VSOP_TO_ICRS = BIAS @ RX
M_ICRS_TO_VSOP = np.linalg.inv(M_VSOP_TO_ICRS)


def bsp_path():
    cands = {
        "de406": (os.environ.get("DE406_BSP"), "vendor/jpl/de406.bsp", "vendor/jpl/de406/de406.bsp"),
        "de441": (os.environ.get("DE441_BSP"), "vendor/jpl/de441/de441.bsp"),
    }[TARGET]
    for p in cands:
        if p and os.path.isfile(p):
            return p
    sys.exit(f"找不到 {TARGET} BSP（见脚本说明）")


def compute(kernel, center, target, jd):
    """(center, target) 状态位置（km），按时刻选段（DE441 每对分两段；jplephem 的 kernel[c, t] 只给其中一段）。"""
    out = np.full((3, len(jd)), np.nan)
    for seg in kernel.segments:
        if (seg.center, seg.target) != (center, target):
            continue
        m = (jd >= seg.start_jd) & (jd <= seg.end_jd) & np.isnan(out[0])
        if m.any():
            out[:, m] = seg.compute(jd[m])
    if np.isnan(out).any():
        sys.exit(f"({center},{target}) 有时刻不在任何段内")
    return out


class Vsop87B:
    """VSOP87B 全级数（与 rust/core 的 Vsop87Parse 读同一文件、不截断）。"""

    def __init__(self, path):
        self.series = {1: {}, 2: {}, 3: {}}
        v = p = None
        with open(path) as f:
            for line in f:
                if line.startswith(" VSOP87"):
                    v, p = int(line[41]), int(line[59])
                    self.series[v].setdefault(p, [])
                elif len(line) > 100:
                    self.series[v][p].append([float(line[79:97]), float(line[97:111]), float(line[111:131])])
        for v in self.series:
            for p in self.series[v]:
                self.series[v][p] = np.array(self.series[v][p])

    def lbr(self, t):
        out = []
        for v in (1, 2, 3):
            val = np.zeros_like(t)
            for p, terms in self.series[v].items():
                for chunk in np.array_split(terms, max(1, len(terms) // 400)):
                    val += t**p * np.sum(chunk[:, 0][:, None] * np.cos(chunk[:, 1][:, None] + chunk[:, 2][:, None] * t[None, :]), axis=0)
            out.append(val)
        return out

    def freqs(self, v, n):
        """该变量各 t^k 级数中振幅最大的 n 个不同频率（|f| > 1 rad/千年，即周期 < 6000 年者另由长期项吸收）。"""
        terms = np.vstack([t for t in self.series[v].values()])
        terms = terms[np.argsort(-np.abs(terms[:, 0]))]
        out = []
        for a, _, f in terms:
            if abs(f) > 1.0 and all(abs(f - g) > 1e-6 * max(1.0, abs(f)) for g in out):
                out.append(f)
            if len(out) == n:
                break
        return np.array(out)


def design(t, freqs, nsec, mixt):
    cols = [t**k for k in range(nsec + 1)]
    for f in freqs:
        c, s = np.cos(f * t), np.sin(f * t)
        for k in range(mixt + 1):
            cols += [t**k * c, t**k * s]
    return np.column_stack(cols)


def de_helio_vsop(kernel, naif, jd):
    sun = compute(kernel, 0, 10, jd)
    body = compute(kernel, 0, 3, jd) + compute(kernel, 3, 399, jd) if naif is None else compute(kernel, 0, naif, jd)
    v = M_ICRS_TO_VSOP @ ((body - sun) / AU_KM)
    r = np.linalg.norm(v, axis=0)
    return np.arctan2(v[1], v[0]), np.arcsin(v[2] / r), r


def residuals(vsop, kernel, naif, jd):
    t = (jd - J2000) / 365250.0
    l, b, r = vsop.lbr(t)
    ld, bd, rd = de_helio_vsop(kernel, naif, jd)
    dl = ((ld - l + np.pi) % (2 * np.pi) - np.pi) * RAD2AS
    return t, [dl, (bd - b) * RAD2AS, rd - r], (l, b, r), (ld, bd, rd)


def ang_sep_as(l1, b1, l2, b2):
    v1 = np.array([np.cos(b1) * np.cos(l1), np.cos(b1) * np.sin(l1), np.sin(b1)])
    v2 = np.array([np.cos(b2) * np.cos(l2), np.cos(b2) * np.sin(l2), np.sin(b2)])
    cr = np.linalg.norm(np.cross(v1.T, v2.T), axis=1)
    return np.arctan2(cr, np.sum(v1 * v2, axis=0)) * RAD2AS


def main():
    from jplephem.spk import SPK

    kernel = SPK.open(bsp_path())
    rng = np.random.default_rng(20001)
    jd_fit = np.sort(rng.uniform(JD_MIN, JD_MAX, N_SAMPLES))
    jd_chk = np.sort(rng.uniform(JD_MIN, JD_MAX, 4000))  # 独立检验点
    lines = [
        f"# VSOP87B → {TARGET.upper()} 日心 L、B、R 改正（Chapront 2000 方法，前 3000 年—3000 年；各体 NSEC、MIXT 见脚本 PARAMS）。",
        f"# 由 scripts/ephemeris/vsop87_de_planet_fit.py {TARGET} 生成。t = (JD_TDB − J2000)/365250（儒略千年）。",
        "# Δ = Σ_{k≤NSEC} a_k t^k + Σ_f Σ_{k≤MIXT} t^k (C_k cos f t + S_k sin f t)；L、B 单位角秒，R 单位 AU。",
        "# 改正施于 VSOP87B 的 L、B、R（J2000 动力学黄道；与 rust 管线 R_x(84381.406″)+FK5→ICRS 偏差配套）。",
        "# 段 [体 变量]：首行 a0..a_NSEC；其后每行 f C0 S0 C1 S1 …（共 MIXT+1 对）。",
        "",
    ]
    summary = []
    for suffix, naif in BODIES:
        vsop = Vsop87B(f"{VSOP_DIR}/VSOP87B.{suffix}")
        nsec, mixt, n_freq = PARAMS.get(suffix, DEFAULT_PARAMS)
        t, res, _, _ = residuals(vsop, kernel, naif, jd_fit)
        coefs = []
        for vi, name in enumerate("LBR"):
            fr = vsop.freqs(vi + 1, n_freq)
            x, *_ = np.linalg.lstsq(design(t, fr, nsec, mixt), res[vi], rcond=None)
            coefs.append((name, fr, x))
            lines.append(f"[{suffix} {name}]")
            lines.append(" ".join(repr(float(v)) for v in x[: nsec + 1]))
            for i, f in enumerate(fr):
                row = x[nsec + 1 + i * 2 * (mixt + 1): nsec + 1 + (i + 1) * 2 * (mixt + 1)]
                lines.append(" ".join(repr(float(v)) for v in [f, *row]))
            lines.append("")
        # 独立检验：日心方向角距，改正前后
        tc, rc, (l, b, r), (ld, bd, rd) = residuals(vsop, kernel, naif, jd_chk)
        corr = [design(tc, fr, nsec, mixt) @ x for _, fr, x in coefs]
        before = ang_sep_as(l, b, ld, bd)
        after = ang_sep_as(l + corr[0] / RAD2AS, b + corr[1] / RAD2AS, ld, bd)
        dr_after = np.abs(r + corr[2] - rd)
        years = 2000 + (jd_chk - J2000) / 365.25
        mid = (years >= -1000) & (years <= 2000)
        summary.append((suffix, before.max(), after.max(), before[mid].max(), after[mid].max(), dr_after.max() * AU_KM))
        print(f"{suffix}（NSEC={nsec} MIXT={mixt} 频率 {n_freq}）: 日心方向差 前3000–3000 年 改正前 {before.max():.3f}″ → 后 {after.max():.4f}″；"
              f"前1000–2000 年 {before[mid].max():.3f}″ → {after[mid].max():.4f}″；R 残差最大 {dr_after.max() * AU_KM:.0f} km", flush=True)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w") as f:
        f.write("\n".join(lines))
    print(f"→ {OUT}")


if __name__ == "__main__":
    main()
