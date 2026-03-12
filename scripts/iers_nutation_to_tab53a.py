#!/usr/bin/env python3
"""
从 IERS 官方 Table 5.3a / 5.3b 生成 data/IAU2000/tab5.3a.txt（合并格式，仅月日项）。
用法：在仓库根目录执行 python3 scripts/iers_nutation_to_tab53a.py
输出覆盖 data/IAU2000/tab5.3a.txt。
"""
import urllib.request
import re
from pathlib import Path
from collections import defaultdict

IERS_53A = "https://iers-conventions.obspm.fr/content/chapter5/additional_info/tab5.3a.txt"
IERS_53B = "https://iers-conventions.obspm.fr/content/chapter5/additional_info/tab5.3b.txt"

# 77 项顺序（与 iau2000b LUNI_SOLAR_77_ROWS 一致），保证前 77 行与 nutation_77 一致
ORDER_77 = [
    (0, 0, 0, 0, 1), (0, 0, 2, -2, 2), (0, 0, 2, 0, 2), (0, 0, 0, 0, 2), (0, 1, 0, 0, 0),
    (0, 1, 2, -2, 2), (1, 0, 0, 0, 0), (0, 0, 2, 0, 1), (1, 0, 2, 0, 2), (0, -1, 2, -2, 2),
    (0, 0, 2, -2, 1), (-1, 0, 2, 0, 2), (-1, 0, 0, 2, 0), (1, 0, 0, 0, 1), (-1, 0, 0, 0, 1),
    (-1, 0, 2, 2, 2), (1, 0, 2, 0, 1), (-2, 0, 2, 0, 1), (0, 0, 0, 2, 0), (0, 0, 2, 2, 2),
    (0, -2, 2, -2, 2), (-2, 0, 0, 2, 0), (2, 0, 2, 0, 2), (1, 0, 2, -2, 2), (-1, 0, 2, 0, 1),
    (2, 0, 0, 0, 0), (0, 0, 2, 0, 0), (0, 1, 0, 0, 1), (-1, 0, 0, 2, 1), (0, 2, 2, -2, 2),
    (0, 0, -2, 2, 0), (1, 0, 0, -2, 1), (0, -1, 0, 0, 1), (-1, 0, 2, 2, 1), (0, 2, 0, 0, 0),
    (1, 0, 2, 2, 2), (-2, 0, 2, 0, 0), (0, 1, 2, 0, 2), (0, 0, 2, 2, 1), (0, -1, 2, 0, 2),
    (0, 0, 0, 2, 1), (1, 0, 2, -2, 1), (2, 0, 2, -2, 2), (-2, 0, 0, 2, 1), (2, 0, 2, 0, 1),
    (0, -1, 2, -2, 1), (0, 0, 0, -2, 1), (-1, -1, 0, 2, 0), (2, 0, 0, -2, 1), (1, 0, 0, 2, 0),
    (0, 1, 2, -2, 1), (1, -1, 0, 0, 0), (-2, 0, 2, 0, 2), (3, 0, 2, 0, 2), (0, -1, 0, 2, 0),
    (1, -1, 2, 0, 2), (0, 0, 0, 1, 0), (-1, -1, 2, 2, 2), (-1, 0, 2, 0, 0), (0, -1, 2, 2, 2),
    (-2, 0, 0, 0, 1), (1, 1, 2, 0, 2), (2, 0, 0, 0, 1), (-1, 1, 0, 1, 0), (1, 1, 0, 0, 0),
    (1, 0, 2, 0, 0), (-1, 0, 2, -2, 1), (1, 0, 0, 0, 2), (-1, 0, 0, 1, 0), (0, 0, 2, 1, 2),
    (-1, 0, 2, 4, 2), (-1, 1, 0, 1, 1), (0, -2, 2, -2, 1), (1, 0, 2, 2, 1), (-2, 0, 2, 2, 2),
    (-1, 0, 0, 0, 2), (1, 1, 2, -2, 2),
]


def fetch(url: str) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (compatible; IERS-table-fetcher/1.0)"})
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.read().decode("utf-8", errors="replace")


def parse_53a_j0(lines, start: int):
    """Parse tab5.3a j=0: i A_i A"_i l l' F D Om (9 planetary). Return list of (key, A_i, A"_i)."""
    out = []
    for i in range(start, len(lines)):
        line = lines[i].strip()
        if not line or line.startswith("-") or "j = " in line:
            break
        parts = line.split()
        if len(parts) < 17:
            continue
        try:
            l, lp, f, d, om = int(parts[3]), int(parts[4]), int(parts[5]), int(parts[6]), int(parts[7])
            plan = [int(parts[k]) for k in range(8, 17)]
            if any(plan):
                continue
            a_i = float(parts[1])
            a_pp = float(parts[2])
            out.append(((l, lp, f, d, om), (a_i, a_pp)))
        except (ValueError, IndexError):
            continue
    return out


def parse_53a_j1(lines, start: int):
    """Parse tab5.3a j=1: i A'_i A"'_i l l' F D Om (9 planetary)."""
    out = []
    for i in range(start, len(lines)):
        line = lines[i].strip()
        if not line or line.startswith("-") or "j = " in line:
            break
        parts = line.split()
        if len(parts) < 17:
            continue
        try:
            l, lp, f, d, om = int(parts[3]), int(parts[4]), int(parts[5]), int(parts[6]), int(parts[7])
            plan = [int(parts[k]) for k in range(8, 17)]
            if any(plan):
                continue
            ap = float(parts[1])
            appp = float(parts[2])
            out.append(((l, lp, f, d, om), (ap, appp)))
        except (ValueError, IndexError):
            continue
    return out


def parse_53b_j0(lines, start: int):
    """Parse tab5.3b j=0: i B"_i B_i l l' F D Om. Δε = B·cos + B''·sin."""
    out = []
    for i in range(start, len(lines)):
        line = lines[i].strip()
        if not line or line.startswith("-") or "j = " in line:
            break
        parts = line.split()
        if len(parts) < 17:
            continue
        try:
            l, lp, f, d, om = int(parts[3]), int(parts[4]), int(parts[5]), int(parts[6]), int(parts[7])
            plan = [int(parts[k]) for k in range(8, 17)]
            if any(plan):
                continue
            b_pp = float(parts[1])
            b_i = float(parts[2])
            out.append(((l, lp, f, d, om), (b_i, b_pp)))
        except (ValueError, IndexError):
            continue
    return out


def parse_53b_j1(lines, start: int):
    """Parse tab5.3b j=1: i B"'_i B'_i l l' F D Om."""
    out = []
    for i in range(start, len(lines)):
        line = lines[i].strip()
        if not line or line.startswith("-") or "j = " in line:
            break
        parts = line.split()
        if len(parts) < 17:
            continue
        try:
            l, lp, f, d, om = int(parts[3]), int(parts[4]), int(parts[5]), int(parts[6]), int(parts[7])
            plan = [int(parts[k]) for k in range(8, 17)]
            if any(plan):
                continue
            bppp = float(parts[1])
            bp = float(parts[2])
            out.append(((l, lp, f, d, om), (bp, bppp)))
        except (ValueError, IndexError):
            continue
    return out


def _section_marker_ok(line: str) -> bool:
    """IERS 表头：标准为 'Number of terms'，部分表用 'Nb of terms'；tab5.3b 可能为 'Number  of terms'（双空格）。"""
    return bool(re.search(r"Number\s+of\s+terms", line)) or "Nb of terms" in line


def find_section(lines, marker: str) -> int:
    for i, line in enumerate(lines):
        if marker in line and _section_marker_ok(line):
            return i
    return -1


def main():
    repo = Path(__file__).resolve().parent.parent
    out_path = repo / "data" / "IAU2000" / "tab5.3a.txt"
    out_path.parent.mkdir(parents=True, exist_ok=True)

    print("Fetching IERS tab5.3a ...")
    raw_a = fetch(IERS_53A)
    print("Fetching IERS tab5.3b ...")
    raw_b = fetch(IERS_53B)
    lines_a = raw_a.splitlines()
    lines_b = raw_b.splitlines()

    # 若返回 HTML 或非表格式，则无 "j = 0" / "Number of terms" 或 "Nb of terms"
    j0_a = find_section(lines_a, "j = 0")
    j1_a = find_section(lines_a, "j = 1")
    j0_b = find_section(lines_b, "j = 0")
    j1_b = find_section(lines_b, "j = 1")
    if j0_a < 0 or j1_a < 0 or j0_b < 0:
        err_which = []
        if j0_a < 0:
            err_which.append("tab5.3a j=0")
        if j1_a < 0:
            err_which.append("tab5.3a j=1")
        if j0_b < 0:
            err_which.append("tab5.3b j=0")
        if j1_b < 0:
            err_which.append("tab5.3b j=1 (optional)")
        def looks_like_html(text: str) -> bool:
            t = text.strip()[:500].lower()
            return t.startswith("<!") or "<html" in t or "<!doctype" in t

        err = [
            "Could not find j=0/j=1 sections in IERS files (expected 'j = 0' and 'Number of terms' or 'Nb of terms').",
            f"Missing: {', '.join(err_which)}. Indices: j0_a={j0_a} j1_a={j1_a} j0_b={j0_b} j1_b={j1_b}.",
        ]
        if looks_like_html(raw_a) or looks_like_html(raw_b):
            err.append("Response looks like HTML (e.g. IERS error page or redirect). Check URL or network.")
        err.append(f"tab5.3a line count: {len(lines_a)}")
        err.append("tab5.3a first 5 and line 19:")
        for i in [0, 1, 2, 3, 4]:
            if i < len(lines_a):
                err.append(f"  {i+1}: {lines_a[i][:80]!r}")
        if len(lines_a) > 19:
            err.append(f"  19: {lines_a[18][:80]!r}")
        err.append("tab5.3b first 3 lines:")
        for i, line in enumerate(lines_b[:3]):
            err.append(f"  {i+1}: {line[:80]!r}")
        raise SystemExit("\n".join(err))

    # skip header rows after "Number of terms" (--- and column header)
    start_j0_a = j0_a + 3
    while start_j0_a < len(lines_a) and (not lines_a[start_j0_a].strip() or lines_a[start_j0_a].strip().startswith("-") or "i " in lines_a[start_j0_a]):
        start_j0_a += 1
    start_j1_a = j1_a + 3
    while start_j1_a < len(lines_a) and (not lines_a[start_j1_a].strip() or lines_a[start_j1_a].strip().startswith("-") or "i " in lines_a[start_j1_a]):
        start_j1_a += 1
    start_j0_b = j0_b + 3
    while start_j0_b < len(lines_b) and (not lines_b[start_j0_b].strip() or lines_b[start_j0_b].strip().startswith("-") or "i " in lines_b[start_j0_b]):
        start_j0_b += 1
    start_j1_b = j1_b + 3
    while start_j1_b < len(lines_b) and (not lines_b[start_j1_b].strip() or lines_b[start_j1_b].strip().startswith("-") or "i " in lines_b[start_j1_b]):
        start_j1_b += 1

    a0 = parse_53a_j0(lines_a, start_j0_a)
    a1 = parse_53a_j1(lines_a, start_j1_a)
    b0 = parse_53b_j0(lines_b, start_j0_b)
    b1 = parse_53b_j1(lines_b, start_j1_b)

    merge = defaultdict(lambda: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0))
    for key, (ai, app) in a0:
        v = merge[key]
        merge[key] = (ai, app, v[2], v[3], v[4], v[5], v[6], v[7])
    for key, (ap, appp) in a1:
        v = merge[key]
        merge[key] = (v[0], v[1], ap, appp, v[4], v[5], v[6], v[7])
    for key, (bi, bpp) in b0:
        v = merge[key]
        merge[key] = (v[0], v[1], v[2], v[3], bi, bpp, v[6], v[7])
    for key, (bp, bppp) in b1:
        v = merge[key]
        merge[key] = (v[0], v[1], v[2], v[3], v[4], v[5], bp, bppp)

    # µas -> mas
    def to_mas(x):
        return x / 1000.0

    ordered_keys = []
    seen = set()
    for k in ORDER_77:
        if k in merge and k not in seen:
            ordered_keys.append(k)
            seen.add(k)
    for k in sorted(merge.keys()):
        if k not in seen:
            ordered_keys.append(k)

    header = """* IERS Conventions Table 5.3a+5.3b (IAU 2000A nutation, luni-solar terms only)
* Source: https://iers-conventions.obspm.fr/ chapter 5 additional_info
*  L Lm  F  D Om       Period               In Phase                          Out of phase
*                      (days)       Psi     dPsi/dt   Eps    dEps/dt   Psi   dPsi/dt    Eps  dEps/dt
*                                   (mas)   (mas/c)    (mas)   (mas/c)  (mas)  (mas/c)  (mas)  (mas/c)
"""
    with open(out_path, "w", encoding="utf-8") as outf:
        outf.write(header)
        for key in ordered_keys:
            l, lp, f, d, om = key
            ai, app, ap, appp, bi, bpp, bp, bppp = merge[key]
            period = 0.0
            row = (
                f"{l:4d} {lp:4d} {f:4d} {d:4d} {om:4d} "
                f"{period:12.3f} "
                f"{to_mas(ai):12.4f} {to_mas(ap):8.4f} {to_mas(bi):12.4f} {to_mas(bp):8.4f} "
                f"{to_mas(app):8.4f} {to_mas(appp):8.4f} {to_mas(bpp):8.4f} {to_mas(bppp):8.4f}\n"
            )
            outf.write(row)

    print(f"Wrote {len(ordered_keys)} terms to {out_path}")


if __name__ == "__main__":
    main()
