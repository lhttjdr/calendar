"""《资治通鉴目录》（司马光，朔闰据北宋刘羲叟《长历》）逐年朔闰栏转录的展开与检验。

数据：data/calendar/tongjian_mulu_shuo.csv（逐岁一行：目录原文，看图转录自维基共享资源 SSID-123185xx 扫描本）。
目录体例：每岁只列「前月为小月」之月的朔（即该月朔距上月朔 29 日），未列之月 = 上月朔 + 30 日；
闰月列干支者作「閏N干支」，不列者于岁末记「閏N」。

展开：以本表上岁展开之末月朔为起点（目录自身连续；本表无上岁时取参照表——张表，他国行为分政权推步——上岁末月朔），逐月取 29 或 30 日中干支相合者；列出之干支 29、30 日都不合而 59 日合者，
判为目录脱「閏」（中间补一闰月），其余记「不可展开」。展开后逐月与张培瑜表（vendor 实历朔日表，经 gaiyuan_date 读取）、
本项目中原一线按术推步（calendar::lifa::zhongyuan_year_months，ZhongyuanPreference::Reckoning，临时 crate 调用）比较。

输出 doc/calendar/tongjian-mulu-checks.md（逐岁汇总、不合之月逐条）。
用法（仓库根）：python3 scripts/calendar_data/tongjian_mulu_check.py [--no-reckon | --reckon-csv 预导出推步表]
"""
from __future__ import annotations

import csv
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as gd  # noqa: E402

DATA = ROOT / "data/calendar/tongjian_mulu_shuo.csv"
OUT = ROOT / "doc/calendar/tongjian-mulu-checks.md"
GZ = gd.GANZHI
MN = ["", "正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"]
NUM = {n: i for i, n in enumerate(MN) if n}
COMMONS = "https://commons.wikimedia.org/wiki/File:"

# 临时 crate：导出中原一线逐月推步（不经仲裁：实物年也照推；观象 807—821 术不传，无推步）
RECKON_MAIN = r'''
use lunar_core::calendar::lifa::{zhongyuan_year_months, RiliSource, ZhongyuanCalendarPeriods, ZhongyuanPreference};
use lunar_core::calendar::push_calendar;
fn main() {
    let periods = ZhongyuanCalendarPeriods::embedded();
    for y in -220..=960 {
        let name = periods.name_for_lunar_year(y);
        let r = match zhongyuan_year_months(None, y, ZhongyuanPreference::Reckoning) {
            Some((r, RiliSource::Reckoned(_))) => r,
            Some((_, RiliSource::Artifact(_))) => match push_calendar::reckon_lifa_year(name, y) { Some(r) => r, None => continue },
            _ => continue,
        };
        for (j, m, l) in r { println!("{y},{m},{},{j},{name}", l as u8); }
    }
    // 漢初殷曆、顓頊曆兩行的参照：顓頊曆推步（zhuanxu）、殷曆推步（guliuli，年終閏與無中氣閏兩法）、李忠林復原（qinhan_restore）
    use lunar_core::calendar::{guliuli, zhuanxu, qinhan_restore};
    for y in -210..=-100 {
        for (j, m, l) in zhuanxu::year_months(y) { println!("QH,颛顼推步,{y},{m},{},{j}", l as u8); }
        for (j, m, l) in guliuli::year_months(guliuli::Liu::Yin, y, guliuli::Runfa::Nianzhong) { println!("QH,殷历推步（年终闰）,{y},{m},{},{j}", l as u8); }
        for (j, m, l) in guliuli::year_months(guliuli::Liu::Yin, y, guliuli::Runfa::Wuzhongqi) { println!("QH,殷历推步（无中气闰）,{y},{m},{},{j}", l as u8); }
        if let Some(v) = qinhan_restore::year_months(y) { for (j, m, l) in v { println!("QH,李忠林复原,{y},{m},{},{j}", l as u8); } }
    }
}
'''


QH_REF: dict = {}  # 漢初兩行参照：标签 → {(岁, 当时月序, 闰): 朔 JDN}
QH_GUO = ("殷曆", "顓頊曆")


def reckon_months() -> dict:
    """(岁, 月, 闰) → (朔 JDN, 历名)"""
    with tempfile.TemporaryDirectory() as td:
        (Path(td) / "src").mkdir()
        (Path(td) / "Cargo.toml").write_text(
            '[package]\nname = "tjml_reckon"\nversion = "0.1.0"\nedition = "2021"\n[dependencies]\n'
            f'lunar-core = {{ path = "{ROOT / "rust/core"}", default-features = false, features = ["real-f64"] }}\n[workspace]\n')
        (Path(td) / "src/main.rs").write_text(RECKON_MAIN)
        env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "rust/target"))
        out = subprocess.run(["cargo", "run", "-q", "--release", "--manifest-path", str(Path(td) / "Cargo.toml")],
                             capture_output=True, text=True, env=env, cwd=ROOT, check=True).stdout
    res = {}
    for line in out.splitlines():
        if line.startswith("QH,"):
            _, tag, y, m, l, j = line.split(",")
            QH_REF.setdefault(tag, {})[(int(y), int(m), l == "1")] = int(j)
            continue
        y, m, l, j, name = line.split(",", 4)
        res[(int(y), int(m), l == "1")] = (int(j), name)
    return res


def parse(raw: str):
    """原文 → ([(月, 闰, 干支)], 岁末所记闰月)。"""
    s = raw.split("朔")[0]
    tail = raw[len(s) + 1:] if "朔" in raw else ""
    leap = None
    m = re.match(r"閏(十[一二]|[正二三四五六七八九十])", tail)
    if m:
        leap = NUM[m.group(1)]
    toks = []
    for mm in re.finditer(r"(閏)?(十[一二]|[正二三四五六七八九十])(..)", s):
        if mm.group(3) not in GZ:
            raise ValueError(f"干支不识：{raw}")
        toks.append((NUM[mm.group(2)], bool(mm.group(1)), mm.group(3)))
        if mm.group(1):
            leap = NUM[mm.group(2)]
    if re.sub(r"(閏)?(十[一二]|[正二三四五六七八九十])(..)", "", s):
        raise ValueError(f"原文有不能解析之字：{raw}")
    return toks, leap


def expand(prev: int, raw: str):
    """自上岁末月朔 prev 展开一岁 → ([(月, 闰, JDN, 列出否)], [异常说明])。"""
    toks, leap = parse(raw)
    listed = {(m, l): g for m, l, g in toks}
    slots = []
    for m in range(1, 13):
        slots.append((m, False))
        if m == leap:
            slots.append((m, True))
    out, notes = [], []
    for m, l in slots:
        g = listed.get((m, l))
        if g is None:
            j = prev + 30
            out.append((m, l, j, False))
        else:
            gi = GZ.index(g)
            d = (gi - (prev + 49)) % 60
            if d in (29, 30):
                j = prev + d
            elif d == 59 and leap is None and m > 1:
                # 目录脱「閏」：上月与本月之间补一闰月（大），本月小
                lj = prev + 30
                out.append((m - 1, True, lj, False))
                notes.append(f"{MN[m]}月朔{g}距上月 59 日，目录未记闰，补閏{MN[m - 1]}（推定）")
                leap = m - 1
                j = lj + 29
            elif 27 <= d <= 32:
                # 目录未列之月若干，所列干支与「上月 + 29/30」不合而相差不过一二日：从所列干支（多为上岁末与参照表不同或月长异常）
                notes.append(f"{('閏' if l else '') + MN[m]}月{g}距上月朔 {d} 日，不合大小月，从所列干支")
                j = prev + d
            else:
                notes.append(f"{('閏' if l else '') + MN[m]}月{g}距上月朔{d if d < 45 else d - 60:+d}日，不可展开（按小月处理）")
                j = prev + 29
            out.append((m, l, j, True))
        prev = j
    return out, notes


def load_data():
    rows = []
    with open(DATA, encoding="utf-8") as f:
        for r in csv.DictReader(l for l in f if not l.startswith("#")):
            rows.append(r)
    return rows


REGIME = {"周": "北魏/西魏/北周/隋", "隋": "北魏/西魏/北周/隋", "魏": "北魏/西魏/北周/隋", "西魏": "北魏/西魏/北周/隋", "齊": "东魏/北齐", "東魏": "东魏/北齐", "吳": "吴", "蜀": "蜀"}
BENJI_BOOKS = ("jiutangshu_benji", "xintangshu_benji", "jiuwudaishi_benji", "xinwudaishi_benji")


def benji_shuo() -> dict:
    """新旧唐书、新旧五代史本纪明书朔（剔除 benji_shuo_errata.csv 已考讹误、「其他」类）：(岁, 月, 闰) → [(书, 干支, JDN)]。"""
    er = set()
    with open(ROOT / "data/calendar/benji_shuo_errata.csv", encoding="utf-8") as f:
        for r in csv.DictReader(l for l in f if not l.startswith("#")):
            er.add((r["书"], r["年"], r["月"]))
    out = {}
    with open(ROOT / "data/calendar/benji_shuo_check.csv", encoding="utf-8") as f:
        for r in csv.DictReader(l for l in f if not l.startswith("#")):
            if r["书"] not in BENJI_BOOKS or r["类"] == "其他" or (r["书"], r["年"], r["月"]) in er:
                continue
            try:
                y, j = int(r["年"]), int(r["本纪朔 JDN"])
            except ValueError:
                continue
            mm = r["月"]
            leap = mm.startswith("闰") or mm.startswith("閏")
            try:
                m = int(mm[1:] if leap else mm)
            except ValueError:
                continue
            out.setdefault((y, m, leap), []).append((r["书"].split("_")[0], r["本纪朔"], j))
    return out


def rf_cat(r) -> str:
    """读法归类：独立（含盲读复核、盲读）/参照/两读（盲读与现行读法不同、未定，不计独立）。"""
    v = r.get("读法") or "独立"
    return "两读" if "两读" in v else "参照" if v.startswith("参照") else "独立"


def main():
    rows = load_data()
    bj = benji_shuo()
    zt = {(y, m, l): j for (y, m, l, j) in gd._months()}
    zt_last = {}
    for (y, m, l), j in zt.items():
        zt_last[y] = max(zt_last.get(y, 0), j)
    if "--reckon-csv" in sys.argv:
        # 预先导出的推步表（格式同 RECKON_MAIN 输出：岁,月,闰,JDN,历名），cargo 被占用时用
        rk = {}
        for line in open(sys.argv[sys.argv.index("--reckon-csv") + 1], encoding="utf-8"):
            if line.startswith("QH,"):
                _, tag, y, m, l, j = line.rstrip("\n").split(",")
                QH_REF.setdefault(tag, {})[(int(y), int(m), l == "1")] = int(j)
                continue
            y, m, l, j, name = line.rstrip("\n").split(",", 4)
            if not (807 <= int(y) <= 821):  # 观象术不传，不计推步
                rk[(int(y), int(m), l == "1")] = (int(j), name)
    else:
        rk = {} if "--no-reckon" in sys.argv else reckon_months()
    rk_last = {}
    for (y, m, l), (j, _) in rk.items():
        rk_last[y] = max(rk_last.get(y, 0), j)
    # 南北朝多国并列：目录所标之国对分政权推步（push_calendar，data/calendar/推步政权朔日表.csv 主表，纯推步）
    regs = {}
    with open(gd.REGIME_TABLE, encoding="utf-8") as f:
        for d in csv.DictReader(f):
            if d["regime"] in set(REGIME.values()):
                j = gd.jdn(int(d["gregorian_year"]), int(d["gregorian_month"]), int(d["gregorian_day"]))
                regs.setdefault(d["regime"], {})[(int(d["lunar_year"]), int(d["lunar_month"]), d["is_leap"] == "1")] = (j, d["calendar"])
    reg_last = {k: {} for k in regs}
    for k, t in regs.items():
        for (y, m, l), (j, _) in t.items():
            reg_last[k][y] = max(reg_last[k].get(y, 0), j)
    years = []
    chain = {}
    for r in rows:
        y = int(r["年"])
        if not r["朔闰原文"].strip() or r.get("国", "") in QH_GUO:
            continue
        guo = r.get("国", "")
        reg = REGIME.get(guo) if not (guo == "隋" and y >= 590) and not (guo == "魏" and y < 386) else None
        if reg:
            ztab, rtab, last = {}, regs[reg], reg_last[reg]
        else:
            ztab, rtab, last = zt, rk, zt_last
        key = reg or "中原"
        # 起点取本表上岁展开之末月朔（目录自身连续；参照表上岁末与目录闰位不同时，取参照表会把整岁错开一月，如 591—592 年）；本表无上岁时才取参照表
        prev = chain.get((key, y - 1)) or last.get(y - 1) or (rk_last.get(y - 1) if not reg else None)
        if prev is None:
            years.append((r, None, ["无上岁末月朔可起算"], ztab, rtab))
            continue
        ms, notes = expand(prev, r["校读"] or r["朔闰原文"])
        chain[(key, y)] = ms[-1][2]
        years.append((r, ms, notes, ztab, rtab))
    # 统计
    lines = []
    tot = dict(n=0, z=0, zn=0, p=0, pn=0, bn=0, bm=0, bz=0, bp=0, rn=0, rp=0, rmonths=0)
    diff_rows = []
    year_rows = []
    for r, ms, notes, zt, rk in years:
        y = int(r["年"])
        if ms is None:
            continue
        zc = zn = pc = pn = 0
        dz, dp = [], []
        for m, l, j, lst in ms:
            k = (y, m, l)
            tot["n"] += 1
            name = ("閏" if l else "") + MN[m]
            zj = zt.get(k)
            if zj is not None:
                zn += 1
                if zj == j:
                    zc += 1
                else:
                    dz.append((name, j, zj))
            pj = rk.get(k, (None, ""))[0]
            if pj is not None:
                pn += 1
                if pj == j:
                    pc += 1
                else:
                    dp.append((name, j, pj))
            if (zj is not None and zj != j) or (pj is not None and pj != j) or (zt and zj is None and y >= -104):
                diff_rows.append((y, r, name, j, zj, pj, rk.get(k, (None, ""))[1], bj.get(k, []) if zt else []))
            if not zt:
                tot["rn"] += pj is not None
                tot["rp"] += pj == j and pj is not None
                continue
            for b in bj.get(k, []):
                tot["bn"] += 1
                tot["bm"] += b[2] == j
                tot["bz"] += b[2] == zj
                tot["bp"] += b[2] == pj
        rf = rf_cat(r)
        if zt:
            tot["z"] += zc; tot["zn"] += zn; tot["p"] += pc; tot["pn"] += pn
            for kk, vv in (("z", zc), ("zn", zn), ("p", pc), ("pn", pn), ("yrs", 1)):
                tot[f"{rf}_{kk}"] = tot.get(f"{rf}_{kk}", 0) + vv
        else:
            tot["rmonths"] += len(ms)
            for kk, vv in (("rp", pc), ("rn", pn), ("ryrs", 1)):
                tot[f"{rf}_{kk}"] = tot.get(f"{rf}_{kk}", 0) + vv
        year_rows.append((y, r, len(ms), zc, zn, pc, pn, notes))
    # 分历汇总（按推步历名；观象 807—821 无推步，记「观象历（术不传）」）
    by = {}
    for r, ms, notes, zt, rk in years:
        if ms is None:
            continue
        y = int(r["年"])
        for m, l, j, lst in ms:
            k = (y, m, l)
            nm = rk.get(k, (None, "观象历（术不传）" if 807 <= y <= 821 else "（目录闰位与推步异，无同名月）"))[1]
            if not zt:
                nm = f"{r['国']}（{REGIME[r['国']]}）：{nm}"
            b = by.setdefault(nm, dict(y0=y, y1=y, n=0, z=0, p=0, pn=0, only_p=0, only_z=0, none=0))
            b["y1"] = y
            b["n"] += 1
            if rf_cat(r) != "独立":
                b["ref_n"] = b.get("ref_n", 0) + 1
            zj = zt.get(k)
            pj = rk.get(k, (None, ""))[0]
            b["z"] += zj == j and zj is not None
            if rf_cat(r) == "独立":
                b["iz"] = b.get("iz", 0) + (zj == j and zj is not None)
                b["inz"] = b.get("inz", 0) + (zj is not None)
                b["ip"] = b.get("ip", 0) + (pj == j and pj is not None)
                b["ipn"] = b.get("ipn", 0) + (pj is not None)
            b["nz"] = b.get("nz", 0) + (zj is not None)
            if pj is not None:
                b["pn"] += 1
                b["p"] += pj == j
                b["only_p"] += (pj == j and zj != j)
                b["only_z"] += (zj == j and pj != j)
                b["none"] += (zj != j and pj != j)
    print(tot)
    qh = qinhan_check([r for r in rows if r.get("国", "") in QH_GUO])
    write_doc(year_rows, diff_rows, tot, by, qh)


def gzs(j):
    return "" if j is None else GZ[(j + 49) % 60]


def qh_ref_xia(tag: str) -> dict:
    """参照表换成夏正月名：(天文年, 夏正月, 闰) → JDN。殷曆子正当时月 m → 夏正 m−2（≤0 者属上年）。"""
    out = {}
    for (y, m, l), j in QH_REF.get(tag, {}).items():
        if tag.startswith("殷"):
            mm = m - 2
            yy = y if mm >= 1 else y - 1
            mm = mm if mm >= 1 else mm + 12
            out[(yy, mm, l)] = j
        else:
            out[(y, m, l)] = j
    return out


def qinhan_expand(prev, raw, y):
    """漢初兩行展开：十月岁首，月序 十、十一、十二、正……九，「後九」即閏九；太初元年（−103）只至十二月。"""
    raw = raw.replace("朔後九", "朔閏九").replace("後九", "閏九")
    toks, leap = parse(raw)
    listed = {(m, l): g for m, l, g in toks}
    order = [10, 11, 12] + ([] if y == -103 else list(range(1, 10)))
    slots = []
    for m in order:
        slots.append((m, False))
        if m == leap:
            slots.append((m, True))
    out, notes = [], []
    if prev is None:
        return None, ["无上岁末月朔可起算"]
    key = lambda m, l: (y - 1 if m >= 10 else y, m, l)
    for m, l in slots:
        g = listed.get((m, l))
        if g is None:
            j = prev + 30
        else:
            d = (GZ.index(g) - (prev + 49)) % 60
            if d in (29, 30):
                j = prev + d
            elif d == 59:
                # 目录未记之闰：插在本月之前（十月前即上年後九月）
                pm = 9 if m == 10 else (12 if m == 1 else m - 1)
                pk = (y - 1, 9, True) if m == 10 else key(pm, True)
                out.append((pm, True, prev + 30, pk))
                notes.append(f"{MN[m]}月朔{g}距上月 59 日，目录未记闰，补閏{MN[pm]}（推定）")
                j = prev + 59
            elif 27 <= d <= 32:
                notes.append(f"{('閏' if l else '') + MN[m]}月{g}距上月朔 {d} 日，从所列干支")
                j = prev + d
            else:
                notes.append(f"{('閏' if l else '') + MN[m]}月{g}距上月朔{d if d < 45 else d - 60:+d}日，不可展开（按小月处理）")
                j = prev + 29
        out.append((m, l, j, key(m, l)))
        prev = j
    return out, notes


def qinhan_check(rows):
    """殷曆行对殷曆推步（年终闰、无中气闰），顓頊曆行对顓頊曆推步；两行皆对李忠林复原（漢初实用历之今人复原）。
    比较两种：同名月同日（十至十二月取上一天文年）；朔日见于参照表（不论月名、闰位）。"""
    refs = {t: qh_ref_xia(t) for t in ("颛顼推步", "殷历推步（年终闰）", "殷历推步（无中气闰）", "李忠林复原")}
    plan = {"殷曆": ["殷历推步（年终闰）", "殷历推步（无中气闰）", "李忠林复原"], "顓頊曆": ["颛顼推步", "李忠林复原"]}
    res = {}
    for guo in QH_GUO:
        own = refs[plan[guo][0]]
        chain = {}
        st = {t: dict(same=0, inset=0, n=0) for t in plan[guo]}
        detail = []
        for r in sorted((r for r in rows if r["国"] == guo), key=lambda r: int(r["年"])):
            y = int(r["年"])
            raw = r["校读"] or r["朔闰原文"]
            prev = chain.get(y - 1)
            anchor = own.get((y - 1, 9, True)) or own.get((y - 1, 9, False))
            # 干支六十日一周：两月之差在干支上看不出，本行上岁末与参照相差超过 45 日即视为沿链误差，改从参照起算
            if prev is not None and anchor is not None and abs(prev - anchor) > 45:
                prev = None
            ms, notes = qinhan_expand(prev if prev is not None else anchor, raw, y)
            # 本行上岁末与本岁首列干支接不上（补闰或不可展开）时，改从本行参照上年九月（或後九）朔起算，免一岁误读沿链传下
            if prev is not None and anchor is not None and any(("补閏" in x or "不可展开" in x) for x in notes[:1]):
                ms2, notes2 = qinhan_expand(anchor, raw, y)
                if ms2 is not None and len([x for x in notes2 if "补閏" in x or "不可展开" in x]) < len([x for x in notes if "补閏" in x or "不可展开" in x]):
                    ms, notes = ms2, ["上岁末与本岁首接不上，改从参照上年九月朔起算"] + notes2
            if ms is None:
                detail.append((y, r, None, notes))
                continue
            chain[y] = ms[-1][2]
            mrow = []
            for m, l, j, key in ms:
                cell = []
                for t in plan[guo]:
                    ref = refs[t]
                    st[t]["n"] += 1
                    same = ref.get(key) == j
                    ins = j in set(ref.values())
                    st[t]["same"] += same
                    st[t]["inset"] += ins
                    cell.append((t, same, ins, ref.get(key)))
                mrow.append((m, l, j, cell))
            detail.append((y, r, mrow, notes))
        res[guo] = (st, detail)
    return res


def write_qh(o, qh):
    if not qh:
        return
    o.append("## 漢初殷曆、顓頊曆兩行（卷二、三）\n")
    o.append("目录自漢元年至太初元年逐岁并列殷曆（刻作「般」）、顓頊曆两家朔（序云「今所記月數皆因夏正」），十月岁首、闰在岁末称「後九月」。"
             "两行分别展开（各行自身连续；首岁取本行参照上一年九月朔起算），逐月与参照比较。年为漢紀年所在天文年，十至十二月属上一天文年末。"
             "殷曆行对 `calendar::guliuli`（殷历，子正，年终闰与无中气闰两法，换夏正月名）；顓頊曆行对 `calendar::zhuanxu`；两行皆对李忠林复原历"
             "（`calendar::qinhan_restore`，漢初实用历之今人复原，前 106 年以后与张表同）。「同名月同日」= 夏正同名月朔日相同；「朔见参照」= 该朔日见于参照表任一月（不论月名闰位）。\n")
    o.append("| 行 | 参照 | 月数 | 同名月同日 | 朔见参照 |")
    o.append("|---|---|---|---|---|")
    for guo, (st, _) in qh.items():
        for t, v in st.items():
            o.append(f"| {guo} | {t} | {v['n']} | {v['same']} | {v['inset']} |")
    o.append("")
    o.append("### 逐岁（只列与本行推步或李忠林复原有不同名月同日之月的岁）\n")
    o.append("| 年 | 行 | PDF 页 | 原文 | 校读 | 不合之月（目录朔/参照朔） | 附注 |")
    o.append("|---|---|---|---|---|---|---|")
    for guo, (_, detail) in qh.items():
        for y, r, mrow, notes in detail:
            if mrow is None:
                o.append(f"| {y} | {guo} | {r['PDF页']} | {r['朔闰原文']} | {r['校读']} | 不展开 | {'；'.join(notes)} |")
                continue
            bad = []
            for m, l, j, cell in mrow:
                for t, same, ins, rj in cell:
                    if not same:
                        bad.append(f"{t[:2]}{'閏' if l else ''}{MN[m]}{gzs(j)}/{gzs(rj) or '—'}")
            if bad or notes:
                nt = "；".join([x for x in [r.get('附注', '')] + notes if x])
                o.append(f"| {y} | {guo} | {r['PDF页']} | {r['朔闰原文']} | {r['校读']} | {'，'.join(bad)} | {nt} |")
    o.append("")


def write_doc(year_rows, diff_rows, tot, by, qh=None):
    ys = [y for y, *_ in year_rows]
    o = []
    o.append("# 《资治通鉴目录》朔闰逐条检验\n")
    o.append("> 由 `scripts/calendar_data/tongjian_mulu_check.py` 生成，勿手改。转录数据见 `data/calendar/tongjian_mulu_shuo.csv`。\n")
    o.append("司马光《资治通鉴目录》三十卷逐年上栏记朔闰，据北宋刘羲叟《长历》（原书佚）。体例：每岁只列前月为小月之月的朔，"
             "未列者 = 上月朔 + 30 日；闰月不列干支者岁末记「閏N」。本表据维基共享资源扫描本（SSID-12318597—12318606，"
             "十册 PDF）看图转录，逐月展开后与张培瑜表、本项目中原一线按术推步比较。\n")
    o.append("**可靠性**：目录朔闰是刘羲叟按各代历术（及其所见史料）推排的结果，本身是宋人复原，不是当时颁行历日；"
             "可靠性低于本纪明书朔日、出土历日，只作一家对照（见 `doc/calendar/calendar-compendium.md`「资料可靠性排序」）。"
             "观象历 807—821 段已据目录取为中原一线历日（`calendar::liu_changli`），本检验与 `data/calendar/tongjian_mulu_807_821.csv` 互校。\n")
    o.append(f"## 汇总\n\n已录 {len(ys)} 岁（{min(ys)}—{max(ys)}，中有缺岁见下），展开 {tot['n']} 月；"
             f"与张表可比 {tot['zn']} 月，合 {tot['z']}；与推步可比 {tot['pn']} 月，合 {tot['p']}。"
             f"缺岁：{'、'.join(str(y) for y in range(min(ys), max(ys) + 1) if y not in ys) or '无'}。\n")
    o.append(f"以上合张表、合推步之数只计中原一线（含陳）；南北朝目录所标他国（三国吳、蜀，北魏、東魏、西魏、周、齊、隋 581—589；三国魏与宋、南齊、梁、陳为中原一线）对分政权推步"
             f"（`data/calendar/推步政权朔日表.csv` 主表）：{tot['rmonths']} 月，可比 {tot['rn']}，合 {tot['rp']}。\n")
    g = lambda k: tot.get(k, 0)
    o.append(f"**读法分栏**（CSV「读法」列）：「独立」= 只据书影读出（含东汉 100—219 年经未看张表者盲读复核相同之岁，及以盲读为准之岁）；「参照」= 扫描不清、读时对照张表或推步核定（附注「与張表同」「按干支读」「待核」等）。"
             f"参照之岁与张表、推步的比较是循环的，不能计作独立相合。中原一线：独立 {g('独立_yrs')} 岁，可比张表 {g('独立_zn')} 月合 {g('独立_z')}、"
             f"可比推步 {g('独立_pn')} 月合 {g('独立_p')}；参照 {g('参照_yrs')} 岁，可比张表 {g('参照_zn')} 月合 {g('参照_z')}、可比推步 {g('参照_pn')} 月合 {g('参照_p')}。"
             f"两读未定 {g('两读_yrs')} 岁（盲读与现行读法不同，不计独立）：可比张表 {g('两读_zn')} 月合 {g('两读_z')}、可比推步 {g('两读_pn')} 月合 {g('两读_p')}。"
             f"他国行：独立 {g('独立_ryrs')} 岁，合推步 {g('独立_rp')}/{g('独立_rn')}；参照 {g('参照_ryrs')} 岁，合推步 {g('参照_rp')}/{g('参照_rn')}。\n")
    o.append(f"新旧唐书、新旧五代史本纪明书朔（剔除已考讹误与「其他」类）落在已录各月者 {tot['bn']} 条：合目录 {tot['bm']}，"
             f"合张表 {tot['bz']}，合推步 {tot['bp']}（观象 807—821 无推步）。\n")
    kinds = {}
    for y, r, name, j, zj, pj, pname, bs in diff_rows:
        k = ("合张表" if zj == j else "合推步" if pj == j else "不合张表（无推步）" if pj is None
             else "不合分政权推步" if zj is None else "目录独异" if zj == pj else "三家各异")
        kinds.setdefault(k, []).append((y, name, bs, j))
    solo = kinds.get("目录独异", []) + kinds.get("三家各异", [])
    solo_bj = [(y, nm, bs, j) for y, nm, bs, j in solo if bs]
    o.append("## 要点\n")
    o.append(f"- 不合之月 {len(diff_rows)}：合推步而不合张表 {len(kinds.get('合推步', []))}，合张表而不合推步 "
             f"{len(kinds.get('合张表', []))}，目录独异（张表与推步同而目录不同）{len(kinds.get('目录独异', []))}，"
             f"三家各异 {len(kinds.get('三家各异', []))}，无推步之段（观象）不合张表 {len(kinds.get('不合张表（无推步）', []))}。")
    o.append(f"- 目录独异、三家各异之月有本纪明书朔可证者 {len(solo_bj)} 月，本纪合目录者 "
             f"{sum(any(b[2] == j for b in bs) for _, _, bs, j in solo_bj)}："
             "目录与本纪、张表、推步皆不同处，多为刘羲叟推排或《目录》传刻之异，不足以改张表或推步。")
    o.append("- 合推步而不合张表之月（" + "、".join(f"{y} 年{nm}月" for y, nm, _, _ in kinds.get("合推步", []))
             + "）是目录支持本项目推步的旁证：这些月张表另有所据或所推，目录与按术推步同。")
    o.append("- 目录多处不记闰（展开时由干支间隔 59 日补闰，附注「补閏」），闰月有时列为「閏N干支」、有时岁末记「閏N」；"
             "閏N 之后常附下月中气日（如「閏十　十一二日冬至」），已入附注。五代各年朔闰前或冠「唐」「晉」，并附蜀、南唐等国朔与用历，"
             "本表只取冠首一行。")
    o.append("- 原文干支不可通者（传刻误字、脱月名或脱「一」）在 CSV「校读」列给出按前后月推定之读法，展开用校读；"
             "「二」「三」「十一」「十二」诸字在扫描本中笔画常不清，按干支可通者读，附注说明。\n")
    o.append("- **限制**：读法为「参照」之岁（卷四至六 133—219 年大部、及各段扫描不清处）是对照张表核读的，其相合不是独立证据；"
             "评估目录与张表、推步的异同只应看「独立」栏。东汉 100—219 年已由未看张表者盲读：115 岁原文相同（读法「独立（盲读复核）」），"
             "140、164 两岁以盲读原文加校读为准（终读同现行），127、209、212 三岁两读未定（读法「独立（两读未定）」，附注记两读），检验单列「两读」、不计独立。")
    o.append("- 卷一（战国）为诸侯年表，无朔闰，不录。卷二、三漢元年（前 206）至太初元年（前 104）殷曆、顓頊曆两行另节检验（见「漢初殷曆、顓頊曆兩行」）；"
             "太初元年改用太初曆以后单行，与中原一线同法检验。目录岁名与干支纪年在此段一致（无超辰之差）。\n")
    o.append("## 分历\n")
    o.append("按本项目中原一线行用之历分段（推步不经仲裁，实物年亦照推；观象术不传无推步）；他国行以「国（政权）：历名」标出，只对分政权推步、不对张表。「只合推步」「只合张表」「皆不合」以月计。\n")
    o.append("| 历 | 年段 | 月数 | 合张表 | 合推步 | 独立：合张表 | 独立：合推步 | 参照及两读月数 | 只合推步 | 只合张表 | 皆不合 |")
    o.append("|---|---|---|---|---|---|---|---|---|---|---|")
    for nm, b in sorted(by.items(), key=lambda t: t[1]["y0"]):
        pp = f"{b['p']}/{b['pn']}" if b["pn"] else "—"
        iz = f"{b.get('iz', 0)}/{b['inz']}" if b.get("inz") else "—"
        ip = f"{b.get('ip', 0)}/{b['ipn']}" if b.get("ipn") else "—"
        o.append(f"| {nm} | {b['y0']}—{b['y1']} | {b['n']} | {b['z'] if b.get('nz') else '—'} | {pp} | {iz} | {ip} | {b.get('ref_n', 0)} | {b['only_p'] if b['pn'] else '—'} | "
                 f"{b['only_z'] if b['pn'] else '—'} | {b['none'] if b['pn'] else '—'} |")
    o.append("")
    write_qh(o, qh)
    o.append("## 逐岁\n")
    o.append("| 年 | 岁次 | 国 | 卷 | PDF 页 | 目录原文 | 校读 | 读法 | 月数 | 合张表 | 合推步 | 附注 |")
    o.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for y, r, n, zc, zn, pc, pn, notes in year_rows:
        nt = "；".join([x for x in [r.get("附注", "")] + notes if x])
        o.append(f"| {y} | {r['岁次']} | {r.get('国', '')} | {r['卷']} | {r['PDF页']} | {r['朔闰原文']} | {r['校读']} | {r.get('读法', '')} | {n} | "
                 f"{f'{zc}/{zn}' if zn else '—'} | {f'{pc}/{pn}' if pn else '—'} | {nt} |")
    o.append("\n## 不合之月\n")
    o.append("目录展开后与张表或推步不同之月。「目录独异」= 张表与推步相同而目录不同；书影出处：Commons 文件、PDF 页（1 起算，岁名所在页）。"
             "本纪明书朔取新旧唐书、新旧五代史（剔除已考讹误）。\n")
    o.append("| 年 | 国 | 月 | 目录朔 | 张表朔 | 推步朔 | 推步历 | 合 | 本纪明书朔 | 书影 |")
    o.append("|---|---|---|---|---|---|---|---|---|---|")
    for y, r, name, j, zj, pj, pname, bs in diff_rows:
        who = []
        if zj == j:
            who.append("合张表")
        if pj == j:
            who.append("合推步")
        if not who:
            who.append("不合张表（无推步）" if pj is None else "不合分政权推步" if zj is None else "目录独异" if zj == pj else "三家各异")
        o.append(f"| {y} | {r.get('国', '')} | {name} | {gzs(j)} | {gzs(zj) or '—'} | {gzs(pj) or '—'} | {pname or '—'} | "
                 f"{'、'.join(who)} | {' '.join(f'{b[1]}（{b[0]}）' for b in bs) or '—'} | {r['文件']} p{r['PDF页']} |")
    OUT.write_text("\n".join(o) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
