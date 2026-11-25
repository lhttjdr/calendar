//! 《增補鰲頭通書大全》日家神（按月起例）：只列神，不推宜忌。
//!
//! 数据 `data/folk/aotou_rijia_shen.csv` 由 `scripts/folk/aotou_qili.py` 从鳌头书影录文（`aotou_shensha_qili.csv`、
//! `aotou_xiongshen.csv`）汇出：卷二、卷三、卷四各门表前凶神吉神栏与卷五日家凶神、日家吉神定局中**按月（或按季）起例**之神，
//! 同名同值并为一行，出处并列；同名而值异者（如天火卷五子卯午酉、種植作竈表子午卯酉）各占一行。
//! 月以节为界，正月 = 寅月；值为单字干者比日干、单字支者比日支，两字比干支（与 [`super::shensha`] 同法）。
//! 「协纪同义名」列：720 日与协纪月表（或 `shensha_month.csv`）某神全同者记其名，见 [`xieji_alias`]。
//! 逐条与协纪、项目实现之比对见 `doc/folk/aotou-qili-checks.md`。

use once_cell::sync::Lazy;

const CSV: &str = include_str!("../../../../data/folk/aotou_rijia_shen.csv");
const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";

/// 鳌头一神（一行）
#[derive(Clone, Debug)]
pub struct AoTouShen {
    pub name: String,
    pub ji: bool,
    /// 正月（寅）至十二月（丑）各月取值（可多个；空表示该月无）
    pub vals: [Vec<String>; 12],
    /// 出处（卷、表、册页），多处以「；」分
    pub source: String,
    /// 协纪同义名（720 日全同者；无则空）
    pub xieji: String,
}

pub static AOTOU_SHEN: Lazy<Vec<AoTouShen>> = Lazy::new(|| {
    CSV.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            assert_eq!(f.len(), 5, "{l}");
            let ms: Vec<&str> = f[2].split('|').collect();
            assert_eq!(ms.len(), 12, "{}", f[0]);
            let vals = core::array::from_fn(|k| {
                if ms[k] == "-" { Vec::new() } else { ms[k].split('+').map(str::to_string).collect() }
            });
            AoTouShen { name: f[0].to_string(), ji: f[1] == "吉", vals, source: f[3].to_string(), xieji: f[4].to_string() }
        })
        .collect()
});

fn hit(v: &str, day_gz: u8) -> bool {
    let g = GAN.chars().nth((day_gz % 10) as usize).unwrap();
    let z = ZHI.chars().nth((day_gz % 12) as usize).unwrap();
    let cs: Vec<char> = v.chars().collect();
    match cs.as_slice() {
        [a, b] => *a == g && *b == z,
        [c] if GAN.contains(*c) => *c == g,
        [c] => *c == z,
        _ => false,
    }
}

/// 某月（`month_ord`：寅月 = 0）某日（干支序 `day_gz`）所值之鳌头日家神
pub fn aotou_shen_for_day(month_ord: u8, day_gz: u8) -> Vec<&'static AoTouShen> {
    let m = (month_ord % 12) as usize;
    AOTOU_SHEN.iter().filter(|s| s.vals[m].iter().any(|v| hit(v, day_gz))).collect()
}

/// 鳌头神名 → 协纪同义名（720 日全同者，如焦坎→九焦、天對天德→寶光、敬心→敬安）；同名或无对应者 `None`
pub fn xieji_alias(name: &str) -> Option<&'static str> {
    AOTOU_SHEN.iter().find(|s| s.name == name && !s.xieji.is_empty() && s.xieji != name).map(|s| s.xieji.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folk::shensha::{shen_for_day_with, ShenshaOptions};
    use crate::folk::yiji::yuebiao_day;

    fn days(name_pred: impl Fn(&AoTouShen) -> bool) -> Vec<(u8, u8)> {
        let mut v = Vec::new();
        for m in 0..12u8 {
            for d in 0..60u8 {
                if aotou_shen_for_day(m, d).iter().any(|s| name_pred(s)) {
                    v.push((m, d));
                }
            }
        }
        v
    }

    #[test]
    fn table_parses() {
        assert!(AOTOU_SHEN.len() > 150);
        assert!(AOTOU_SHEN.iter().all(|s| s.vals.iter().any(|v| !v.is_empty())), "每神至少一月有值");
        // 正月丁卯：天德（丁）、天寶金匱非、六合非（亥）
        let names: Vec<&str> = aotou_shen_for_day(0, 3).iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"天德"), "{names:?}");
    }

    /// 「协纪同义名」所记诸神，与协纪月表（或 shensha 起例）720 日逐日全同
    #[test]
    fn alias_equals_xieji() {
        let mut n = 0;
        for s in AOTOU_SHEN.iter().filter(|s| !s.xieji.is_empty()) {
            for m in 0..12u8 {
                for d in 0..60u8 {
                    let ours = s.vals[m as usize].iter().any(|v| hit(v, d));
                    let yb = yuebiao_day(m, d);
                    let jc = format!("{}日", yb.jian_chu);
                    let theirs = yb.ji_shen.iter().chain(yb.xiong_shen.iter()).any(|x| *x == s.xieji)
                        || jc == s.xieji
                        || shen_for_day_with(m, d, &ShenshaOptions::default()).iter().any(|(x, _)| *x == s.xieji);
                    assert_eq!(ours, theirs, "{} / {} {m} {d}", s.name, s.xieji);
                }
            }
            n += 1;
        }
        assert!(n > 50, "{n}");
        assert_eq!(xieji_alias("焦坎"), Some("九焦"));
        assert_eq!(xieji_alias("天對天德"), Some("寶光"));
        assert_eq!(xieji_alias("敬心"), Some("敬安"));
        assert_eq!(xieji_alias("月德"), None);
    }

    /// 鳌头卷五日家凶神天火（顺行四仲）即 `ShenshaOptions::aotou()` 之天火；種植、作竈表另作子午卯酉
    #[test]
    fn tian_huo_two_readings() {
        let ao = ShenshaOptions::aotou();
        let juan5 = days(|s| s.name == "天火" && s.source.contains("第6册第22页"));
        let opt: Vec<(u8, u8)> = (0..12u8)
            .flat_map(|m| (0..60u8).map(move |d| (m, d)))
            .filter(|&(m, d)| shen_for_day_with(m, d, &ao).iter().any(|(x, _)| x == "天火"))
            .collect();
        assert_eq!(juan5, opt);
        let zz = days(|s| s.name == "天火" && s.source.contains("種植"));
        assert_eq!(zz.len(), 60);
        assert_eq!(juan5.iter().filter(|x| zz.contains(x)).count(), 30);
    }
}
