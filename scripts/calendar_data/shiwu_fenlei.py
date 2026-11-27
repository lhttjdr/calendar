"""实物（出土、传世历日与当时文书）比对的分类：推步（或该段所用复原）与张表逐月相同、互异，按实物分类。

原则同第一批（doc/calendar/dingshuo-zhangbiao-diffs.md、benji-checks.md）：
- 互异之月：①有硬据证张表误（实物合推步）；②有据证推步误（实物合张表）；③无据待核（无实物，或实物两家皆不合，或证据冲突，或只一见而等级不足）。
- 相同之月：有证（实物合两家）；同误（实物与两家皆不合，按语注判实物抄误或待核）；无证（两家相同而已，或只有不作证据之条）。
- 证据等级：A 实物历日（历书、历谱、具注历）；B 当时文字（碑志、墓志、墓表、题记、简牍文书）；C 史书明书朔；D 纪日（同月两条以上独立才判）；
  E 后人追修。复原材料只作比对、不作证据。
一个月只给一个分类；独立之条各算一次（同件、同简异本只算一条，由调用方去重）。
- 史书之条须两条者（D；分政权、中原一线逐月分类中 E 亦然）须异源两条：同一部书（含同源诸书，如《北史》与魏、齐、周、隋书）同月诸事
  只算一条——同书系月之误会连带同月诸条（2026-11-27 定；`rust/core/examples/regime_benji_eval.rs`、`zhongyuan_reckoning_eval.rs`、
  `dingshuo_zhangbiao_diffs.rs` 同）。本模块不知史源，由调用方归并后传入；NEED 之 E 仍记 1（实物比对诸脚本沿用，未改）。
"""

CATS = ["有证", "同误", "无证", "①", "②", "③"]
NEED = {"A": 1, "B": 1, "C": 1, "D": 2, "E": 1}
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]


def off_note(o, ref):
    """所记朔与两家之朔相差几日 → 判语：差一月者多是月名、年号抄误或闰位；差一二日者待核；余为抄误或释读误。"""
    if o not in GZ:
        return "待核"
    if ref not in GZ:
        return "两家皆无此月，待核"
    d = (GZ.index(o) - GZ.index(ref) + 30) % 60 - 30
    if abs(d) in (29, 30):
        return "合前后月：月名、年号抄误或闰位异"
    if abs(d) <= 2:
        return f"差 {d:+d} 日，待核"
    return f"差 {d:+d} 日，判抄误或释读误"


def classify(tui, zhang, obs, grade="A", what="实物"):
    """tui、zhang：两家该月朔干支（空 = 该家无此月）；obs：该月独立证据所记朔干支（已剔除不作证据之条）。
    → (分类, 注)；两家缺一返回 (None, "")。"""
    if not tui or not zhang:
        return None, ""
    need = NEED.get(grade, 1)
    n = len(obs)
    p = sum(o == tui for o in obs)
    z = sum(o == zhang for o in obs)
    if tui == zhang:
        if n == 0:
            return "无证", ""
        if p >= need:
            return "有证", (f"另 {n - p} 条不合" if p < n else "")
        if p == 0 and n >= need:
            return "同误", f"{what}与两家皆不合，{off_note(obs[0], tui)}"
        return "无证", f"{what}一见"
    if p and z:
        return "③", f"证据冲突：合推步 {p}、合张表 {z}"
    if p >= need:
        return "①", (f"另 {n - p} 条两家皆不合" if p < n else "")
    if z >= need:
        return "②", (f"另 {n - z} 条两家皆不合" if z < n else "")
    if p or z:
        return "③", f"{what}一见，倾向{'推步' if p else '张表'}"
    if n:
        return "③", f"{what}与两家皆不合"
    return "③", ""


def headline(cats, tui_name="推步", u="月"):
    """cats：[(分类, 等级)] → 一句汇总。"""
    c = {k: sum(x == k for x, _ in cats) for k in CATS}
    same = c["有证"] + c["同误"] + c["无证"]
    diff = c["①"] + c["②"] + c["③"]
    return (f"{tui_name}与张表**相同 {same}** {u}（有证 {c['有证']}／同误 {c['同误']}／无证 {c['无证']}），"
            f"**互异 {diff}** {u}（①证张表误 {c['①']}／②证{tui_name}误 {c['②']}／③无据待核 {c['③']}）")


def grade_table(cats):
    """按证据等级给数（无证、③无据者等级记「—」）。"""
    gs = sorted({g for _, g in cats})
    out = ["| 等级 | 有证 | 同误 | 无证 | ①证张表误 | ②证推步误 | ③无据待核 |", "|---|---|---|---|---|---|---|"]
    for g in gs:
        row = [sum(1 for x, y in cats if x == k and y == g) for k in CATS]
        out.append(f"| {g} | " + " | ".join(map(str, row)) + " |")
    tot = [sum(1 for x, _ in cats if x == k) for k in CATS]
    out.append("| 合计 | " + " | ".join(map(str, tot)) + " |")
    return out


RULE = ("分类（每月一个）：两家**相同**之月——有证（实物合两家）、同误（实物与两家皆不合，判实物抄误或待核）、无证（两家相同而已，"
        "或只有推得、定年等不作证据之条）；两家**互异**之月——①实物证张表误（合推步）、②实物证推步误（合张表）、③无据待核"
        "（实物两家皆不合、证据冲突，或只一见而等级不足）。实物本身是证据，复原（今人诸表、外推）只作比对。")
