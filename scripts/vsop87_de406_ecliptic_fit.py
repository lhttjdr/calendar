#!/usr/bin/env python3
"""
VSOP87B → DE406 黄道坐标 (L,B,R) 拟合，J2000 锚点锁定 + 全局约束。

- 阶段 1：在 J2000 求 (dL₀, dB₀) 使 ICRS 零误差。
- 阶段 2：R 独立拟合；L/B 联合拟合，a0 锁在锚点，周期项 ±0.05″。
- 输出：ANCHOR-LOCKED ECLIPTIC COEFFICIENTS，可嵌入黄道补丁。

依赖: numpy, scipy, jplephem
用法: 环境变量 VSOP87B_EAR / DE406_BSP 或命令行参数
      python vsop87_de406_ecliptic_fit.py [VSOP87B.ear] [de406.bsp]

参见: doc/vsop87-de406-ecliptic-fit.md
"""

import numpy as np
import os
from jplephem.spk import SPK
from scipy.optimize import least_squares

# 路径：可改为本地路径或通过环境变量覆盖
VSOP_PATH = os.environ.get("VSOP87B_EAR", "data/vsop87/VSOP87B.ear")
DE406_PATH = os.environ.get("DE406_BSP", "data/jpl/de406.bsp")


# ==========================================================================
# 1. 基础组件
# ==========================================================================
class VSOP87B_Provider:
    def __init__(self, file_path):
        self.series = {1: {}, 2: {}, 3: {}}
        if not os.path.exists(file_path):
            raise FileNotFoundError(f"{file_path} not found")
        with open(file_path, "r") as f:
            v, p = None, None
            for line in f:
                if line.startswith(" VSOP87"):
                    v, p = int(line[41]), int(line[59])
                    if p not in self.series[v]:
                        self.series[v][p] = []
                elif len(line) > 100:
                    self.series[v][p].append(
                        [
                            float(line[79:97]),
                            float(line[97:111]),
                            float(line[111:131]),
                        ]
                    )
        for v in [1, 2, 3]:
            for p in self.series[v]:
                self.series[v][p] = np.array(self.series[v][p])

    def calculate(self, jd):
        t = (jd - 2451545.0) / 365250.0
        res = []
        for v in [1, 2, 3]:
            val = 0.0
            for p, terms in self.series[v].items():
                val += (t**p) * np.sum(
                    terms[:, 0] * np.cos(terms[:, 1] + terms[:, 2] * t)
                )
            res.append(val)
        return res[0] % (2 * np.pi), res[1], res[2]

    def get_freqs(self, var_idx, n=25):
        terms = self.series[var_idx][0]
        idx = np.argsort(np.abs(terms[:, 0]))[::-1]
        return terms[idx[:n], 2]


class SOFA_FrameTransformer:
    AS2R = np.pi / (180.0 * 3600.0)
    _M_V2I = None

    @classmethod
    def get_V2I(cls):
        if cls._M_V2I is None:
            da, xi, et = (
                -0.0146 * cls.AS2R,
                0.0091 * cls.AS2R,
                -0.0199 * cls.AS2R,
            )

            def r_x(a):
                return np.array(
                    [
                        [1, 0, 0],
                        [0, np.cos(a), np.sin(a)],
                        [0, -np.sin(a), np.cos(a)],
                    ]
                )

            def r_y(a):
                return np.array(
                    [
                        [np.cos(a), 0, -np.sin(a)],
                        [0, 1, 0],
                        [np.sin(a), 0, np.cos(a)],
                    ]
                )

            def r_z(a):
                return np.array(
                    [
                        [np.cos(a), np.sin(a), 0],
                        [-np.sin(a), np.cos(a), 0],
                        [0, 0, 1],
                    ]
                )

            m_bias = r_z(-da) @ r_y(xi) @ r_x(-et)
            m_v2f = np.array(
                [
                    [1.0, 0.000000440360, -0.000000190919],
                    [-0.000000479966, 0.917482137087, -0.397776982902],
                    [0.0, 0.397776982902, 0.917482137087],
                ]
            )
            cls._M_V2I = m_bias @ m_v2f
        return cls._M_V2I


def chapront_poly_model(p, t, freqs):
    res = p[0] + p[1] * t + p[2] * t**2 + p[3] * t**3
    step = 4
    for i, f in enumerate(freqs):
        idx = 4 + i * step
        res += (p[idx] + p[idx + 2] * t) * np.cos(f * t) + (
            p[idx + 1] + p[idx + 3] * t
        ) * np.sin(f * t)
    return res


# ==========================================================================
# 2. 核心逻辑: 两步走物理锁定 (Two-Stage Lock)
# ==========================================================================
def run_physically_locked_fit(vsop_file, de406_file):
    provider = VSOP87B_Provider(vsop_file)
    kernel = SPK.open(de406_file)
    AU_KM, RAD2AS = 149597870.7, 206264.806247
    M_V2I = SOFA_FrameTransformer.get_V2I()

    # --- 阶段 1: J2000 锚点搜索 (Find the Anchor) ---
    print(">>> [Phase 1] 寻找 J2000.0 零误差锚点 (dL_0, dB_0)...")

    jd_2000 = 2451545.0
    l_v0, b_v0, r_v0 = provider.calculate(jd_2000)

    pos_de0 = (
        kernel[0, 3].compute(jd_2000)
        + kernel[3, 399].compute(jd_2000)
        - kernel[0, 10].compute(jd_2000)
    )
    x, y, z = pos_de0 / AU_KM
    ra_de0 = np.arctan2(y, x)
    dec_de0 = np.arcsin(z / np.linalg.norm([x, y, z]))

    def cost_j2000(p):
        dL_rad = p[0] / RAD2AS
        dB_rad = p[1] / RAD2AS

        L_new = l_v0 + dL_rad
        B_new = b_v0 + dB_rad

        x_v = r_v0 * np.cos(B_new) * np.cos(L_new)
        y_v = r_v0 * np.cos(B_new) * np.sin(L_new)
        z_v = r_v0 * np.sin(B_new)
        pos_i = M_V2I @ np.array([x_v, y_v, z_v])

        ra_sim = np.arctan2(pos_i[1], pos_i[0])
        dec_sim = np.arcsin(pos_i[2] / np.linalg.norm(pos_i))

        return [ra_sim - ra_de0, dec_sim - dec_de0]

    res_anchor = least_squares(cost_j2000, [0.0, 0.0], ftol=1e-15, xtol=1e-15)
    anchor_dL, anchor_dB = res_anchor.x
    print(f"    -> 锁定锚点: dL = {anchor_dL:.6f}\", dB = {anchor_dB:.6f}\"")

    # --- 阶段 2: 全局约束拟合 (Global Fit with Locked Bounds) ---
    print(">>> [Phase 2] 执行物理约束拟合 (Locked a0, MIXT=2)...")

    jd_samples = np.linspace(
        2451545.0 - 1095750, 2451545.0 + 365250, 6000
    )
    t_tjy = (jd_samples - 2451545.0) / 365250.0

    l_raw, b_raw, r_raw = [], [], []
    ra_de, dec_de, dist_de = [], [], []
    for jd in jd_samples:
        l, b, r = provider.calculate(jd)
        l_raw.append(l)
        b_raw.append(b)
        r_raw.append(r)
        pos = (
            kernel[0, 3].compute(jd)
            + kernel[3, 399].compute(jd)
            - kernel[0, 10].compute(jd)
        )
        x, y, z = pos / AU_KM
        d_val = np.linalg.norm([x, y, z])
        ra_de.append(np.arctan2(y, x))
        dec_de.append(np.arcsin(z / d_val))
        dist_de.append(d_val)

    l_raw = np.array(l_raw)
    b_raw = np.array(b_raw)
    r_raw = np.array(r_raw)
    ra_de = np.unwrap(np.array(ra_de))
    dec_de = np.array(dec_de)
    dist_de = np.array(dist_de)

    f_L = provider.get_freqs(1, 25)
    f_L = f_L[np.abs(f_L) > 1.0]
    f_B = provider.get_freqs(2, 25)
    f_B = f_B[np.abs(f_B) > 1.0]
    f_R = provider.get_freqs(3, 25)
    f_R = f_R[np.abs(f_R) > 1.0]

    final_results = {}

    # --- Step A: 拟合 R ---
    def cost_R(p):
        return (r_raw + chapront_poly_model(p, t_tjy, f_R)) - dist_de

    res_R = least_squares(cost_R, np.zeros(4 + 4 * len(f_R)), xtol=1e-12)
    r_corrected = r_raw + chapront_poly_model(res_R.x, t_tjy, f_R)
    final_results["R"] = (res_R.x, f_R, 0.0)

    # --- Step B: 联合拟合 L, B (应用锚点约束) ---
    n_pL = 4 + 4 * len(f_L)
    n_pB = 4 + 4 * len(f_B)

    L_lower = [-np.inf] * n_pL
    L_upper = [np.inf] * n_pL
    L_lower[0] = anchor_dL - 0.001
    L_upper[0] = anchor_dL + 0.001
    for i in range(4, n_pL):
        L_lower[i] = -0.05
        L_upper[i] = 0.05

    B_lower = [-np.inf] * n_pB
    B_upper = [np.inf] * n_pB
    B_lower[0] = anchor_dB - 0.001
    B_upper[0] = anchor_dB + 0.001
    for i in range(4, n_pB):
        B_lower[i] = -0.05
        B_upper[i] = 0.05

    combined_bounds = (
        np.array(L_lower + B_lower),
        np.array(L_upper + B_upper),
    )

    p0_LB = np.zeros(n_pL + n_pB)
    p0_LB[0] = anchor_dL
    p0_LB[n_pL] = anchor_dB

    def cost_LB_Locked(p_combined):
        pL = p_combined[:n_pL]
        pB = p_combined[n_pL:]

        dL = chapront_poly_model(pL, t_tjy, f_L) / RAD2AS
        dB = chapront_poly_model(pB, t_tjy, f_B) / RAD2AS

        L_corr = l_raw + dL
        B_corr = b_raw + dB

        cosB = np.cos(B_corr)
        sinB = np.sin(B_corr)
        cosL = np.cos(L_corr)
        sinL = np.sin(L_corr)

        X_v = r_corrected * cosB * cosL
        Y_v = r_corrected * cosB * sinL
        Z_v = r_corrected * sinB

        X_i = (
            M_V2I[0, 0] * X_v
            + M_V2I[0, 1] * Y_v
            + M_V2I[0, 2] * Z_v
        )
        Y_i = (
            M_V2I[1, 0] * X_v
            + M_V2I[1, 1] * Y_v
            + M_V2I[1, 2] * Z_v
        )
        Z_i = (
            M_V2I[2, 0] * X_v
            + M_V2I[2, 1] * Y_v
            + M_V2I[2, 2] * Z_v
        )

        D_xy = np.sqrt(X_i**2 + Y_i**2)
        RA_sim = np.unwrap(np.arctan2(Y_i, X_i))
        Dec_sim = np.arctan2(Z_i, D_xy)

        diff_RA = (RA_sim - ra_de) * RAD2AS
        diff_Dec = (Dec_sim - dec_de) * RAD2AS

        return np.concatenate([diff_RA, diff_Dec])

    res_LB = least_squares(
        cost_LB_Locked, p0_LB, bounds=combined_bounds, xtol=1e-6
    )

    final_results["L"] = (res_LB.x[:n_pL], f_L, 0.0)
    final_results["B"] = (res_LB.x[n_pL:], f_B, 0.0)

    # --- Output coefficients ---
    print("\n" + "=" * 95)
    print(f"{'ANCHOR-LOCKED ECLIPTIC COEFFICIENTS':^95}")
    print("=" * 95)
    for name in ["L", "B", "R"]:
        c, f, _ = final_results[name]
        print(f"\n// --- Component: {name} ---")
        print(f"val secular{name} = Array({', '.join(map(str, c[:4]))})")
        print(f"val freqs{name} = Array({', '.join(map(str, f))})")
        print(f"val periodic{name} = Array(")
        for i in range(len(f)):
            p = c[4 + i * 4 : 4 + (i + 1) * 4]
            line = (
                f"  Array({p[0]}, {p[1]}, {p[2]}, {p[3]})"
                + ("," if i < len(f) - 1 else "")
            )
            print(line)
        print(")")

    return final_results, provider, kernel


# ==========================================================================
# 3. 验证与执行
# ==========================================================================
class EclipticPatchApplicator:
    def __init__(self, fit_results_map):
        self.patch = fit_results_map

    def get_correction(self, t_tjy, comp):
        c, f, _ = self.patch[comp]
        return chapront_poly_model(c, t_tjy, f)


def print_full_comparison_ecliptic(jd_tdb, vsop_provider, applicator, de406_kernel):
    t_tjy = (jd_tdb - 2451545.0) / 365250.0
    AU_KM, RAD2AS = 149597870.7, 206264.806247
    l_v, b_v, r_v = vsop_provider.calculate(jd_tdb)
    d_l = applicator.get_correction(t_tjy, "L")
    d_b = applicator.get_correction(t_tjy, "B")
    d_r = applicator.get_correction(t_tjy, "R")
    l_corr = l_v + (d_l / RAD2AS)
    b_corr = b_v + (d_b / RAD2AS)
    r_corr = r_v + d_r
    M_V2I = SOFA_FrameTransformer.get_V2I()
    x_v = r_corr * np.cos(b_corr) * np.cos(l_corr)
    y_v = r_corr * np.cos(b_corr) * np.sin(l_corr)
    z_v = r_corr * np.sin(b_corr)
    pos_i = M_V2I @ np.array([x_v, y_v, z_v])
    r_final = np.linalg.norm(pos_i)
    ra_final = np.arctan2(pos_i[1], pos_i[0]) % (2 * np.pi)
    dec_final = np.arcsin(pos_i[2] / r_final)
    pos_de = (
        de406_kernel[0, 3].compute(jd_tdb)
        + de406_kernel[3, 399].compute(jd_tdb)
        - de406_kernel[0, 10].compute(jd_tdb)
    )
    x, y, z = pos_de / AU_KM
    r_de = np.linalg.norm([x, y, z])
    ra_de = np.arctan2(y, x) % (2 * np.pi)
    dec_de = np.arcsin(z / r_de)
    res_ra = ((ra_de - ra_final + np.pi) % (2 * np.pi) - np.pi) * RAD2AS
    res_dec = (dec_de - dec_final) * RAD2AS
    res_r = r_de - r_final
    x_raw = r_v * np.cos(b_v) * np.cos(l_v)
    y_raw = r_v * np.cos(b_v) * np.sin(l_v)
    z_raw = r_v * np.sin(b_v)
    pos_i_raw = M_V2I @ np.array([x_raw, y_raw, z_raw])
    ra_raw_i = np.arctan2(pos_i_raw[1], pos_i_raw[0]) % (2 * np.pi)
    off_ra = ((ra_de - ra_raw_i + np.pi) % (2 * np.pi) - np.pi) * RAD2AS
    print("============================================================")
    print(f" JD(TDB): {jd_tdb:<15.2f} | T: {t_tjy:<10.6f} tjy")
    print("------------------------------------------------------------")
    print(f"{'Ecliptic Patch':<15} | dL={d_l:>8.5f}\" | dB={d_b:>8.5f}\"")
    print("------------------------------------------------------------")
    print(
        f"{'ICRS CHECK':<12} | {'RA (arcsec)':<14} | {'Dec (arcsec)':<14} | {'R (AU)':<12}"
    )
    print(f"{'Orig Offset':<12} | {off_ra:<14.6f} | {'--':<14} | {'--':<12}")
    print(
        f"{'Final Res':<12} | {res_ra:<14.6f} | {res_dec:<14.6f} | {res_r:<12.10f}"
    )
    print("============================================================\n")


# ==========================================================================
# 4. 执行
# ==========================================================================
if __name__ == "__main__":
    import sys

    vsop_file = sys.argv[1] if len(sys.argv) > 1 else VSOP_PATH
    de406_file = sys.argv[2] if len(sys.argv) > 2 else DE406_PATH

    fit_results, provider, kernel = run_physically_locked_fit(
        vsop_file, de406_file
    )
    applicator = EclipticPatchApplicator(fit_results)
    jd_table = [
        (2444239.5, "1980.2"),
        (2451545.0, "J2000.0"),
        (2455000.0, "2010"),
        (2268922.5, "1500"),
        (2637936.5, "2500"),
        (2458849.5, "2020.0"),
        (2460060.5, "2023.0"),
        (2460512.0, "2024.5"),
        (2461059.3, "2026朔1"),
        (2461148.0, "2026朔4"),
        (2461272.5, "2026.5"),
        (2462500.0, "2028.0"),
    ]
    for jd, _ in jd_table:
        print_full_comparison_ecliptic(jd, provider, applicator, kernel)
