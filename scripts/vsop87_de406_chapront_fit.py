#!/usr/bin/env python3
"""
Chapront (2000) 风格拟合：VSOP87B → DE406 残差近似（NSEC=3, MIXT=2）。

实现论文 "Improvements of planetary theories over 6000 years" 的近似方法：
- 标架：VSOP87 动力学黄道 → FK5 → ICRS（SOFA FK5→ICRS 偏置矩阵，Hilton & Hohenkerk 2004）
- 残差 ρ = σ_DE406 − σ_VSOP（同一架下）
- 近似形式：长期项 a0+a1*t+a2*t²+a3*t³ + Σ (C0+C1*t)cos(f*t)+(S0+S1*t)sin(f*t)
- 时间 t = (JD - J2000) / 365250（儒略千年，与 VSOP87 一致）

依赖: numpy, scipy, jplephem
用法: 环境变量 VSOP87B_EAR / DE406_BSP 或命令行参数
      python vsop87_de406_chapront_fit.py [VSOP87B.ear] [de406.bsp]
      默认：终极拟合 run_ultimate_fit_with_bounds（全 MIXT=2，RA 的 a0 约束在 [0, 0.05] 吸收 J2000）。
      CHAPRONT_HYBRID=1 时用混合拟合（RA MIXT=1）；CHAPRONT_UNBOUNDED=1 时用无约束拟合。

参见: doc/chapront2000-fitting-method.md
"""

import numpy as np
import os
from scipy.optimize import least_squares

# 路径：可改为本地路径或通过环境变量覆盖
VSOP_PATH = os.environ.get("VSOP87B_EAR", "data/vsop87/VSOP87B.ear")
DE406_PATH = os.environ.get("DE406_BSP", "data/jpl/de406.bsp")


# ==========================================================================
# 1. VSOP87B 数据加载与频率提取
# ==========================================================================
class VSOP87B_Provider:
    def __init__(self, file_path):
        if not os.path.exists(file_path):
            raise FileNotFoundError(f"File not found: {file_path}")
        self.series = {1: {}, 2: {}, 3: {}}  # 1:L, 2:B, 3:R
        with open(file_path, "r") as f:
            v, p = None, None
            for line in f:
                if line.startswith(" VSOP87"):
                    v, p = int(line[41]), int(line[59])
                    if p not in self.series[v]:
                        self.series[v][p] = []
                elif len(line) > 100:
                    # A, B, C 系数: T^p * A * cos(B + C*T)
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
        """提取主项频率 (T^0)。"""
        terms = self.series[var_idx][0]
        idx = np.argsort(np.abs(terms[:, 0]))[::-1]
        return terms[idx[:n], 2]


# ==========================================================================
# 2. 标架转换 (SOFA FK5→ICRS)
# ==========================================================================
class SOFA_FrameTransformer:
    """
    VSOP87 动力学黄道 → FK5 → ICRS。
    FK5→ICRS：SOFA 偏置矩阵（Hilton & Hohenkerk, A&A 413, 765, 2004）。
    """

    # VSOP87 动力学黄道 → FK5 J2000（Bretagnon & Francou, VSOP87 文档）
    M_V2F = np.array(
        [
            [1.000000000000, 0.000000440360, -0.000000190919],
            [-0.000000479966, 0.917482137087, -0.397776982902],
            [0.000000000000, 0.397776982902, 0.917482137087],
        ]
    )

    # FK5 → ICRS：SOFA 偏置矩阵（Hilton & Hohenkerk 2004），弧度
    M_F2I = np.array([
        [1.0, -8.82462042e-8, -3.85466334e-8],
        [8.82462042e-8, 1.0, -3.30116133e-8],
        [3.85466334e-8, 3.30116133e-8, 1.0],
    ])
    M_V2I = M_F2I @ M_V2F

    @staticmethod
    def transform(l, b, r):
        v = np.array(
            [r * np.cos(b) * np.cos(l), r * np.cos(b) * np.sin(l), r * np.sin(b)]
        )
        v_i = SOFA_FrameTransformer.M_V2I @ v
        d = np.linalg.norm(v_i)
        return (
            np.arctan2(v_i[1], v_i[0]) % (2 * np.pi),
            np.arcsin(v_i[2] / d),
            d,
        )


# ==========================================================================
# 3. Chapront 2000 拟合引擎 (NSEC=3, MIXT=1 或 MIXT=2)
# ==========================================================================
def chapront_fit_model(p, t, freqs):
    """MIXT=2: p = [a0..a3, C0,S0,C1,S1 每频 4 个]."""
    res = p[0] + p[1] * t + p[2] * t**2 + p[3] * t**3
    for i, f in enumerate(freqs):
        idx = 4 + i * 4
        res += (p[idx] + p[idx + 2] * t) * np.cos(f * t) + (
            p[idx + 1] + p[idx + 3] * t
        ) * np.sin(f * t)
    return res


def chapront_fit_model_mixt1(p, t, freqs):
    """MIXT=1: 纯周期项，p = [a0..a3, C0,S0 每频 2 个]，无 C1,S1."""
    res = p[0] + p[1] * t + p[2] * t**2 + p[3] * t**3
    for i, f in enumerate(freqs):
        idx = 4 + i * 2
        res += p[idx] * np.cos(f * t) + p[idx + 1] * np.sin(f * t)
    return res


def chapront_poly_model(p, t, freqs, mixt):
    """统一模型：mixt=1 仅 C0,S0；mixt=2 为 (C0+C1*t)cos+(S0+S1*t)sin。"""
    res = p[0] + p[1] * t + p[2] * t**2 + p[3] * t**3
    step = 2 if mixt == 1 else 4
    for i, f in enumerate(freqs):
        idx = 4 + i * step
        if mixt == 1:
            res += p[idx] * np.cos(f * t) + p[idx + 1] * np.sin(f * t)
        else:
            res += (p[idx] + p[idx + 2] * t) * np.cos(f * t) + (
                p[idx + 1] + p[idx + 3] * t
            ) * np.sin(f * t)
    return res


def run_strict_fit(vsop_file, de406_file):
    provider = VSOP87B_Provider(vsop_file)
    from jplephem.spk import SPK

    kernel = SPK.open(de406_file)

    # Interval (a): 6000 years
    jd_samples = np.linspace(
        2451545.0 - 1095750, 2451545.0 + 365250, 6000
    )
    t_tjy = (jd_samples - 2451545.0) / 365250.0

    diff_ra, diff_dec, diff_r = [], [], []
    AU_KM = 149597870.7

    print(">>> 正在生成 ICRS 标架残差...")
    for jd in jd_samples:
        l, b, r = provider.calculate(jd)
        ra_v, dec_v, r_v = SOFA_FrameTransformer.transform(l, b, r)

        # DE406 (ICRS)：日心系下地球位置
        pos = (kernel[0, 3].compute(jd) + kernel[3, 399].compute(jd)) - kernel[
            0, 10
        ].compute(jd)
        x, y, z = pos / AU_KM
        d_d = np.linalg.norm([x, y, z])
        diff_ra.append(
            ((np.arctan2(y, x) - ra_v + np.pi) % (2 * np.pi) - np.pi)
            * 206264.8
        )
        diff_dec.append((np.arcsin(z / d_d) - dec_v) * 206264.8)
        diff_r.append(d_d - r_v)

    results = {}
    for name, data in zip(["RA", "Dec", "R"], [diff_ra, diff_dec, diff_r]):
        print(f">>> 拟合 {name} (NSEC=3, MIXT=2)...")
        v_idx = 1 if name == "RA" else (2 if name == "Dec" else 3)
        raw_freqs = provider.get_freqs(v_idx, n=25)
        valid_freqs = raw_freqs[np.abs(raw_freqs) > 1.0]

        p_init = np.zeros(4 + 4 * len(valid_freqs))
        lsq = least_squares(
            lambda p: chapront_fit_model(p, t_tjy, valid_freqs) - data,
            p_init,
            xtol=1e-15,
            ftol=1e-15,
        )
        results[name] = (lsq.x, valid_freqs, np.sqrt(np.mean(lsq.fun**2)))

    # 输出符合论文标准的报告
    print("\n" + "=" * 95)
    print("SOFA-STRICT CHAPRONT (2000) IMPROVEMENT OF VSOP87".center(95))
    print("6000 YEARS | NSEC=3, MIXT=2".center(95))
    print("=" * 95)
    for name in ["RA", "Dec", "R"]:
        coeffs, freqs, rmse = results[name]
        unit = "arcsec" if name != "R" else "AU"
        print(f"\n>>> [{name}] RMSE: {rmse:.8f} {unit}")
        print(f"Secular (a0-a3): {list(np.round(coeffs[:4], 10))}")
        print(
            f"{'Freq':>12} | {'C0':>12} | {'S0':>12} | {'C1 (Mixed)':>12} | {'S1 (Mixed)':>12}"
        )
        for i, f in enumerate(freqs):
            row = coeffs[4 + i * 4 : 4 + (i + 1) * 4]
            print(
                f"{f:12.4f} | {row[0]:12.6f} | {row[1]:12.6f} | {row[2]:12.6f} | {row[3]:12.6f}"
            )
    return results


# J2000 权重：使 T=0 处 patch 优先贴近实际偏移，避免在基线仅 ≈0.03" 时给出 -0.17" 过修正
J2000_WEIGHT = 2000.0


# ==========================================================================
# 4. 带约束拟合（避免过修正：基线已 ≈0.03″ 时 patch 不应达 11″）
# ==========================================================================
def run_strict_fit_with_bounds(vsop_file, de406_file):
    """与 run_strict_fit 相同残差定义，但对系数加界：secular ±1.0，周期项 ±0.1（RA/Dec 角秒，R 同量级）。
    残差中对 J2000 (T=0) 施加强权重，使该点 patch 值接近实际偏移，避免单点 Residual 远大于 Total Offset。"""
    provider = VSOP87B_Provider(vsop_file)
    from jplephem.spk import SPK

    kernel = SPK.open(de406_file)
    jd_samples = np.linspace(2451545.0 - 1095750, 2451545.0 + 365250, 6000)
    t_tjy = (jd_samples - 2451545.0) / 365250.0
    AU_KM = 149597870.7
    RAD2AS = 206264.806247
    jd_j2000 = 2451545.0

    print(">>> 正在提取 DE406 残差并执行约束拟合（含 J2000 强权重）...")
    results = {}
    for name in ["RA", "Dec", "R"]:
        diff = []
        for jd in jd_samples:
            l, b, r = provider.calculate(jd)
            ra_v, dec_v, r_v = SOFA_FrameTransformer.transform(l, b, r)
            pos = (kernel[0, 3].compute(jd) + kernel[3, 399].compute(jd)) - kernel[0, 10].compute(jd)
            x, y, z = pos / AU_KM
            dist = np.linalg.norm([x, y, z])
            if name == "RA":
                diff.append(((np.arctan2(y, x) - ra_v + np.pi) % (2 * np.pi) - np.pi) * RAD2AS)
            elif name == "Dec":
                diff.append((np.arcsin(z / dist) - dec_v) * RAD2AS)
            else:
                diff.append(dist - r_v)

        # J2000 单点偏移（样本不含 JD=2451545，需单独算）
        l0, b0, r0 = provider.calculate(jd_j2000)
        ra0_v, dec0_v, r0_v = SOFA_FrameTransformer.transform(l0, b0, r0)
        pos0 = (kernel[0, 3].compute(jd_j2000) + kernel[3, 399].compute(jd_j2000)) - kernel[0, 10].compute(jd_j2000)
        x0, y0, z0 = pos0 / AU_KM
        d0 = np.linalg.norm([x0, y0, z0])
        if name == "RA":
            diff_j2000 = ((np.arctan2(y0, x0) - ra0_v + np.pi) % (2 * np.pi) - np.pi) * RAD2AS
        elif name == "Dec":
            diff_j2000 = (np.arcsin(z0 / d0) - dec0_v) * RAD2AS
        else:
            diff_j2000 = d0 - r0_v

        v_idx = 1 if name == "RA" else (2 if name == "Dec" else 3)
        raw_freqs = provider.get_freqs(v_idx, n=25)
        valid_freqs = raw_freqs[np.abs(raw_freqs) > 1.0]
        n_f = len(valid_freqs)

        if name == "RA":
            # RA: MIXT=1（纯周期项），收紧边界以防过拟合
            # secular ±0.2"，周期项 C0,S0 ±0.03"
            lower = [-0.2] * 4 + [-0.03] * (2 * n_f)
            upper = [0.2] * 4 + [0.03] * (2 * n_f)
            p_init = np.zeros(4 + 2 * n_f)

            def residual_with_j2000(p):
                r = chapront_fit_model_mixt1(p, t_tjy, valid_freqs) - diff
                patch_at_j2000 = chapront_fit_model_mixt1(
                    p, np.array([0.0]), valid_freqs
                )[0]
                r_j2000 = (patch_at_j2000 - diff_j2000) * np.sqrt(J2000_WEIGHT)
                return np.concatenate([r, [r_j2000]])

            lsq = least_squares(
                residual_with_j2000,
                p_init,
                bounds=(lower, upper),
                xtol=1e-15,
                ftol=1e-15,
            )
            n_sample = len(diff)
            rmse = np.sqrt(np.mean(lsq.fun[:n_sample] ** 2))
            results[name] = (lsq.x, valid_freqs, rmse, True)  # True = MIXT=1
        else:
            # Dec, R: MIXT=2，边界 secular ±1.0，周期项 ±0.1
            lower = [-1.0] * 4 + [-0.1] * (4 * n_f)
            upper = [1.0] * 4 + [0.1] * (4 * n_f)
            p_init = np.zeros(4 + 4 * n_f)

            def residual_with_j2000(p):
                r = chapront_fit_model(p, t_tjy, valid_freqs) - diff
                patch_at_j2000 = chapront_fit_model(
                    p, np.array([0.0]), valid_freqs
                )[0]
                r_j2000 = (patch_at_j2000 - diff_j2000) * np.sqrt(J2000_WEIGHT)
                return np.concatenate([r, [r_j2000]])

            lsq = least_squares(
                residual_with_j2000,
                p_init,
                bounds=(lower, upper),
                xtol=1e-15,
                ftol=1e-15,
            )
            n_sample = len(diff)
            rmse = np.sqrt(np.mean(lsq.fun[:n_sample] ** 2))
            results[name] = (lsq.x, valid_freqs, rmse, False)  # False = MIXT=2

    print("\n" + "=" * 95)
    print("FINAL CHAPRONT (2000) COEFFICIENTS (BOUNDED FIT)".center(95))
    print("=" * 95)
    print("若下方验证中 J2000 处 Residual 明显大于 Total Offset，请勿采用该系数。")
    for name in ["RA", "Dec", "R"]:
        entry = results[name]
        coeffs, freqs, rmse = entry[0], entry[1], entry[2]
        mixt1 = entry[3] if len(entry) > 3 else False
        unit = "arcsec" if name != "R" else "AU"
        print(f"\n>>> [{name}] RMSE: {rmse:.8f} {unit}" + (" (MIXT=1)" if mixt1 else " (MIXT=2)"))
        print(f"Secular (a0-a3): {list(np.round(coeffs[:4], 10))}")
        if mixt1:
            print(f"{'Freq':>12} | {'C0':>12} | {'S0':>12}  (MIXT=1, C1=S1=0)")
            for i, f in enumerate(freqs):
                c0, s0 = coeffs[4 + i * 2], coeffs[4 + i * 2 + 1]
                print(f"{f:12.4f} | {c0:12.6f} | {s0:12.6f}")
        else:
            print(f"{'Freq':>12} | {'C0':>12} | {'S0':>12} | {'C1 (Mixed)':>12} | {'S1 (Mixed)':>12}")
            for i, f in enumerate(freqs):
                row = coeffs[4 + i * 4 : 4 + (i + 1) * 4]
                print(f"{f:12.4f} | {row[0]:12.6f} | {row[1]:12.6f} | {row[2]:12.6f} | {row[3]:12.6f}")
    return results, provider, kernel


# ==========================================================================
# 5. 终极约束拟合（全 MIXT=2，RA 的 a0 约束在 [0, 0.05] 吸收 J2000）
# ==========================================================================
def run_ultimate_fit_with_bounds(vsop_file, de406_file):
    """全分量 MIXT=2；RA 强制 a0∈[0, 0.05]、a1..a3∈[-0.2,0.2]、周期项 ±0.05，使 J2000 处 patch 为正且小。"""
    from jplephem.spk import SPK

    provider = VSOP87B_Provider(vsop_file)
    kernel = SPK.open(de406_file)
    jd_samples = np.linspace(2451545.0 - 1095750, 2451545.0 + 365250, 6000)
    t_tjy = (jd_samples - 2451545.0) / 365250.0
    AU_KM = 149597870.7
    RAD2AS = 206264.806247

    results = {}
    print(">>> 正在执行终极拟合 (全 MIXT=2, RA a0 约束 [0, 0.05])...")
    for name in ["RA", "Dec", "R"]:
        diff = []
        for jd in jd_samples:
            l, b, r = provider.calculate(jd)
            ra_v, dec_v, r_v = SOFA_FrameTransformer.transform(l, b, r)
            pos = (kernel[0, 3].compute(jd) + kernel[3, 399].compute(jd)) - kernel[0, 10].compute(jd)
            x, y, z = pos / AU_KM
            dist = np.linalg.norm([x, y, z])
            if name == "RA":
                val = ((np.arctan2(y, x) - ra_v + np.pi) % (2 * np.pi) - np.pi) * RAD2AS
            elif name == "Dec":
                val = (np.arcsin(z / dist) - dec_v) * RAD2AS
            else:
                val = dist - r_v
            diff.append(val)
        diff = np.array(diff)

        v_idx = 1 if name == "RA" else (2 if name == "Dec" else 3)
        valid_freqs = provider.get_freqs(v_idx, n=25)
        valid_freqs = valid_freqs[np.abs(valid_freqs) > 1.0]
        n_f = len(valid_freqs)
        mixt, step = 2, 4

        if name == "RA":
            lower = [0.0, -0.2, -0.2, -0.2] + [-0.05] * (step * n_f)
            upper = [0.05, 0.2, 0.2, 0.2] + [0.05] * (step * n_f)
        else:
            lower = [-1.0] * 4 + [-0.1] * (step * n_f)
            upper = [1.0] * 4 + [0.1] * (step * n_f)

        p_init = np.zeros(4 + step * n_f)
        cost = lambda p: chapront_poly_model(p, t_tjy, valid_freqs, mixt) - diff
        lsq = least_squares(cost, p_init, bounds=(lower, upper), xtol=1e-15, ftol=1e-15)
        rmse = np.sqrt(np.mean(lsq.fun**2))
        results[name] = (lsq.x, valid_freqs, rmse)

    print("\n" + "=" * 95)
    print("ULTIMATE CHAPRONT (2000) COEFFICIENTS FOR SCALA".center(95))
    print("=" * 95)
    for name in ["RA", "Dec", "R"]:
        c, f, r = results[name]
        unit = "arcsec" if name != "R" else "AU"
        print(f"\n// --- Component: {name} (RMSE: {r:.8f} {unit}, Mixt: 2) ---")
        print(f"val secular{name} = Array({', '.join(map(str, c[:4]))})")
        print(f"val freqs{name} = Array({', '.join(map(str, f))})")
        print(f"val periodic{name} = Array(")
        for i in range(len(f)):
            row = c[4 + i * 4 : 4 + (i + 1) * 4]
            line = f"  Array({row[0]}, {row[1]}, {row[2]}, {row[3]})" + ("," if i < len(f) - 1 else "")
            print(line)
        print(")")
    return results, provider, kernel


class ChaprontPatchApplicator:
    """用拟合结果在给定 T（儒略千年）下计算 ΔRA、ΔDec、ΔR。支持混合(RA MIXT=1)与终极(全 MIXT=2)结果。"""
    def __init__(self, fit_results):
        self.patch = fit_results

    def get_correction(self, t_tjy, component):
        entry = self.patch[component]
        coeffs, freqs = entry[0], entry[1]
        mixt1 = entry[3] if len(entry) > 3 else False
        if mixt1:
            return chapront_fit_model_mixt1(coeffs, t_tjy, freqs)
        return chapront_poly_model(coeffs, t_tjy, freqs, mixt=2)


def print_comparison(jd_tdb, vsop_provider, applicator, de406_kernel):
    """打印：总偏差(DE406−VSOP)、Patch 修正量、施加 patch 后残差。"""
    t_tjy = (jd_tdb - 2451545.0) / 365250.0
    AU_KM = 149597870.7
    RAD2AS = 206264.806247

    l, b, r = vsop_provider.calculate(jd_tdb)
    ra_raw, dec_raw, r_raw = SOFA_FrameTransformer.transform(l, b, r)
    d_ra = applicator.get_correction(t_tjy, "RA")
    d_dec = applicator.get_correction(t_tjy, "Dec")
    d_r = applicator.get_correction(t_tjy, "R")
    ra_corr = (ra_raw + (d_ra / RAD2AS)) % (2 * np.pi)
    dec_corr = dec_raw + (d_dec / RAD2AS)
    r_corr = r_raw + d_r

    pos = (de406_kernel[0, 3].compute(jd_tdb) + de406_kernel[3, 399].compute(jd_tdb)) - de406_kernel[0, 10].compute(jd_tdb)
    x, y, z = pos / AU_KM
    r_de = np.linalg.norm([x, y, z])
    ra_de = np.arctan2(y, x) % (2 * np.pi)
    dec_de = np.arcsin(z / r_de)

    print(f"--- JD(TDB): {jd_tdb} | T(tjy): {t_tjy:.6f} ---")
    print(f"Total Offset RA:  {((ra_de - ra_raw + np.pi) % (2 * np.pi) - np.pi) * RAD2AS:10.6f} \"")
    print(f"Delta Patch RA:   {d_ra:10.6f} \"")
    print(f"Residual RA:      {((ra_de - ra_corr + np.pi) % (2 * np.pi) - np.pi) * RAD2AS:10.6f} \"")
    print(f"Total Offset Dec: {(dec_de - dec_raw) * RAD2AS:10.6f} \"")
    print(f"Delta Patch Dec:  {d_dec:10.6f} \"")
    print(f"Residual Dec:     {(dec_de - dec_corr) * RAD2AS:10.6f} \"")
    print(f"Total Offset R:   {(r_de - r_raw):.4e} AU  |  Delta Patch R: {d_r:.4e}  |  Residual R: {(r_de - r_corr):.4e}")
    print("-" * 50)


if __name__ == "__main__":
    import sys

    vsop = os.environ.get("VSOP87B_EAR", VSOP_PATH)
    de406 = os.environ.get("DE406_BSP", DE406_PATH)
    if len(sys.argv) >= 3:
        vsop, de406 = sys.argv[1], sys.argv[2]
    if not os.path.isfile(vsop):
        print(f"VSOP87B 文件不存在: {vsop}", file=sys.stderr)
        sys.exit(1)
    if not os.path.isfile(de406):
        print(f"DE406 文件不存在: {de406}", file=sys.stderr)
        sys.exit(1)

    unbounded = os.environ.get("CHAPRONT_UNBOUNDED", "").lower() in ("1", "true", "yes")
    hybrid = os.environ.get("CHAPRONT_HYBRID", "").lower() in ("1", "true", "yes")

    if unbounded:
        run_strict_fit(vsop, de406)
    elif hybrid:
        fit_res, provider, kernel = run_strict_fit_with_bounds(vsop, de406)
        applicator = ChaprontPatchApplicator(fit_res)
        print("\n>>> 验证（Total Offset / Delta Patch / Residual）")
        for jd in [2451545.0, 2461059.3]:
            print_comparison(jd, provider, applicator, kernel)
    else:
        fit_res, provider, kernel = run_ultimate_fit_with_bounds(vsop, de406)
        applicator = ChaprontPatchApplicator(fit_res)
        print("\n>>> 验证（Total Offset / Delta Patch / Residual）")
        for jd in [2451545.0, 2461059.3]:
            print_comparison(jd, provider, applicator, kernel)
