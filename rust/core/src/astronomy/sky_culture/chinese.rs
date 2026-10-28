//! 中国星官：二十八宿距星、宿度与入宿度、去极度；全体星官（成员与距星，多体系）。
//!
//! 距星数据：`data/sky_culture/chinese_lunar_mansion_determinatives.csv`，由 Stellarium 五套中国星空文化（CC BY-SA 4.0，
//! `vendor/stellarium_skycultures/`）提取并逐宿核对，分「体系」并存（陈卓、宋皇祐、元、明象林、清仪象考成）。
//! 汉至明四套相同（与潘鼐《中国恒星观测史》2009 第 14 页表 1.2.1 一致）。康熙《灵台仪象志》（1674）改奎 η And、觜 λ Ori、昴 η Tau，
//! 致「参前觜后」；《仪象考成》（1744 测，第 641 页表 9.1.4）昴复用 17 Tau，乾隆十九年（1754）起参改 ζ Ori 以复「觜前参后」（第 639–640 页）。
//!
//! 宿度：相邻两宿距星赤经差（当日真赤道），按周天度（如四分历 365.25、授时历 365.2575）换算为古度；觜参颠倒时宿度为负，
//! 如实输出。入宿度：天体在其西侧最近距星以东的赤经差（古度）；去极度：90° − 赤纬（古度）。

use crate::astronomy::apparent::{right_ascension_declination, star_apparent_equatorial, ApparentPipelineOptions};
use crate::astronomy::pipeline::EphemerisProvider;
use crate::astronomy::stars::StarCatalog;
use crate::astronomy::time::TimePoint;
use crate::math::real::{real, Real, RealOps};

/// 二十八宿名（自角至轸）。
pub const MANSIONS: [&str; 28] = [
    "角", "亢", "氐", "房", "心", "尾", "箕", "斗", "牛", "女", "虚", "危", "室", "壁", "奎", "娄", "胃", "昴", "毕", "觜", "参", "井", "鬼",
    "柳", "星", "张", "翼", "轸",
];

/// 一套二十八宿距星认定。
#[derive(Clone, Debug)]
pub struct LunarMansionSystem {
    /// 体系 id：`chenzhuo`、`song_huangyou`、`yuan`、`ming_xianglin`、`qing`（仪象考成）、`qing_lingtai`（灵台仪象志 1674）
    pub id: String,
    /// 各宿距星 Hipparcos 编号（自角至轸）
    pub determinatives: [u32; 28],
}

/// 解析距星表（`#` 开头为注释，首个非注释行为表头）。
pub fn parse_lunar_mansion_systems(lines: &[String]) -> Result<Vec<LunarMansionSystem>, String> {
    let mut out: Vec<LunarMansionSystem> = vec![];
    for line in lines.iter().filter(|l| !l.starts_with('#')).skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 4 {
            return Err(format!("列数不足：{line}"));
        }
        let idx: usize = f[1].parse().map_err(|e| format!("{e}：{line}"))?;
        if !(1..=28).contains(&idx) || MANSIONS[idx - 1] != f[2] {
            return Err(format!("宿序与宿名不符：{line}"));
        }
        let hip: u32 = f[3].parse().map_err(|e| format!("{e}：{line}"))?;
        if out.last().is_none_or(|s| s.id != f[0]) {
            out.push(LunarMansionSystem { id: f[0].to_string(), determinatives: [0; 28] });
        }
        out.last_mut().unwrap().determinatives[idx - 1] = hip;
    }
    if let Some(s) = out.iter().find(|s| s.determinatives.contains(&0)) {
        return Err(format!("体系 {} 距星不全", s.id));
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
/// 读取仓库内距星表（[`crate::repo::paths::CHINESE_LUNAR_MANSIONS`]）。
pub fn load_lunar_mansion_systems_from_repo() -> Result<Vec<LunarMansionSystem>, String> {
    let lines = crate::repo::read_lines(crate::repo::paths::CHINESE_LUNAR_MANSIONS).map_err(|e| format!("{e:?}"))?;
    parse_lunar_mansion_systems(&lines)
}

/// 星官成员：Hipparcos 星，或其它天体标记（`DSO:M44` 深空天体、`GAIA:…`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsterismMember {
    Hip(u32),
    Other(String),
}

/// 一个星官（某体系）。
#[derive(Clone, Debug)]
pub struct Asterism {
    pub system: String,
    pub id: String,
    pub name: String,
    pub english: String,
    pub members: Vec<AsterismMember>,
    pub determinative: Option<u32>,
}

impl Asterism {
    pub fn member_hips(&self) -> impl Iterator<Item = u32> + '_ {
        self.members.iter().filter_map(|m| if let AsterismMember::Hip(h) = m { Some(*h) } else { None })
    }
}

/// 解析星官表 `data/sky_culture/chinese_asterisms.csv`（`#` 开头为注释，首个非注释行为表头）。
pub fn parse_asterisms(lines: &[String]) -> Result<Vec<Asterism>, String> {
    let mut out = vec![];
    for line in lines.iter().filter(|l| !l.starts_with('#')).skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 6 {
            return Err(format!("列数不为 6：{line}"));
        }
        let members = f[4]
            .split_whitespace()
            .map(|t| t.parse::<u32>().map(AsterismMember::Hip).unwrap_or_else(|_| AsterismMember::Other(t.to_string())))
            .collect();
        out.push(Asterism {
            system: f[0].into(),
            id: f[1].into(),
            name: f[2].into(),
            english: f[3].into(),
            members,
            determinative: if f[5].is_empty() { None } else { Some(f[5].parse().map_err(|e| format!("{e}：{line}"))?) },
        });
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
/// 读取仓库内星官表（[`crate::repo::paths::CHINESE_ASTERISMS`]）。
pub fn load_asterisms_from_repo() -> Result<Vec<Asterism>, String> {
    let lines = crate::repo::read_lines(crate::repo::paths::CHINESE_ASTERISMS).map_err(|e| format!("{e:?}"))?;
    parse_asterisms(&lines)
}

/// 某历元的二十八宿距星视赤经、视赤纬（当日真赤道，rad）。
#[derive(Clone, Debug)]
pub struct MansionFrame {
    pub ra: [Real; 28],
    pub dec: [Real; 28],
}

impl MansionFrame {
    /// 由星表与历表求距星视位置（`star_apparent_equatorial`：空间运动、周年视差与光行差、岁差章动）。
    pub fn new<P: EphemerisProvider>(
        eph: &P,
        catalog: &StarCatalog,
        system: &LunarMansionSystem,
        t: TimePoint,
        options: &ApparentPipelineOptions,
    ) -> Result<Self, String> {
        let mut ra = [real(0.0); 28];
        let mut dec = [real(0.0); 28];
        for (i, hip) in system.determinatives.iter().enumerate() {
            let star = catalog.by_hip(*hip).ok_or_else(|| format!("星表缺 HIP {hip}（{}宿）", MANSIONS[i]))?;
            let (a, d) = right_ascension_declination(&star_apparent_equatorial(eph, star, t, options));
            ra[i] = a.rad();
            dec[i] = d.rad();
        }
        Ok(Self { ra, dec })
    }

    /// 宿度（古度，周天 `zhou_tian` 度）：下一宿距星与本宿距星的赤经差，取 (−180°, 180°] 内的值换算；觜参颠倒时为负。
    pub fn widths(&self, zhou_tian: Real) -> [Real; 28] {
        let scale = zhou_tian / real(2.0 * core::f64::consts::PI);
        core::array::from_fn(|i| (self.ra[(i + 1) % 28] - self.ra[i]).wrap_to_signed_pi() * scale)
    }

    /// 入宿：赤经 `ra` 所在之宿（其西侧最近的距星）与入宿度（古度）。
    pub fn ru_xiu(&self, ra: Real, zhou_tian: Real) -> (usize, Real) {
        let mut best = (0usize, real(f64::MAX));
        for i in 0..28 {
            let d = (ra - self.ra[i]).wrap_to_2pi();
            if d < best.1 {
                best = (i, d);
            }
        }
        (best.0, best.1 * zhou_tian / real(2.0 * core::f64::consts::PI))
    }
}

/// 天体在二十八宿体系中的位置。
#[derive(Clone, Copy, Debug)]
pub struct MansionLocation {
    /// 所在宿（0 = 角 … 27 = 轸）
    pub mansion: usize,
    /// 入宿度（古度）
    pub ru_xiu_du: Real,
    /// 去极度（古度）
    pub qu_ji_du: Real,
}

impl MansionFrame {
    /// 天体视赤经、赤纬（当日真赤道，rad）的入宿度与去极度。
    pub fn locate(&self, ra: Real, dec: Real, zhou_tian: Real) -> MansionLocation {
        let (mansion, ru_xiu_du) = self.ru_xiu(ra, zhou_tian);
        MansionLocation { mansion, ru_xiu_du, qu_ji_du: qu_ji_du(dec, zhou_tian) }
    }
}

/// 太阳、月亮或行星某时刻（地心视位置）的入宿度与去极度。`frame` 须为同一历元的距星视位置。
pub fn body_mansion_location<P: EphemerisProvider>(
    eph: &P,
    body: crate::astronomy::pipeline::Body,
    frame: &MansionFrame,
    t: TimePoint,
    zhou_tian: Real,
    options: &ApparentPipelineOptions,
) -> MansionLocation {
    let (a, d) = right_ascension_declination(&crate::astronomy::apparent::body_apparent_equatorial(eph, body, t, options));
    frame.locate(a.rad(), d.rad(), zhou_tian)
}

/// 当日黄经（rad，0—2π）：由视赤经、视赤纬与黄赤交角 `eps`（rad）。
pub fn ecliptic_longitude(ra: Real, dec: Real, eps: Real) -> Real {
    (ra.sin() * eps.cos() + dec.tan() * eps.sin()).atan2(ra.cos()).wrap_to_2pi()
}

impl MansionFrame {
    /// 改为黄道宿度框架：各距星赤经换成当日黄经（清时宪历以黄道计宿度，周天 360°）。之后 `ru_xiu` 须传天体黄经；
    /// `dec` 仍为赤纬（去极度不随之改变）。
    pub fn to_ecliptic(&self, eps: Real) -> Self {
        Self { ra: core::array::from_fn(|i| ecliptic_longitude(self.ra[i], self.dec[i], eps)), dec: self.dec }
    }
}

/// 去极度（古度）：90° − 赤纬，按周天 `zhou_tian` 度换算。
pub fn qu_ji_du(dec: Real, zhou_tian: Real) -> Real {
    (real(core::f64::consts::FRAC_PI_2) - dec) * zhou_tian / real(2.0 * core::f64::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
    use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
    use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
    use crate::astronomy::pipeline::Vsop87Planets;
    use crate::astronomy::stars::load_hipparcos_bright_from_repo;
    use crate::astronomy::time::TimeScale;

    fn yuanshi_widths() -> Vec<[f64; 6]> {
        let lines = crate::repo::read_lines("data/sky_culture/yuanshi_lodge_widths.csv").unwrap();
        let rows: Vec<[f64; 6]> = lines
            .iter()
            .filter(|l| !l.starts_with('#'))
            .skip(1)
            .map(|l| {
                let f: Vec<f64> = l.split(',').skip(1).map(|x| x.parse().unwrap()).collect();
                [f[0], f[1], f[2], f[3], f[4], f[5]]
            })
            .collect();
        assert_eq!(rows.len(), 28);
        rows
    }

    #[test]
    fn systems_load() {
        let s = load_lunar_mansion_systems_from_repo().unwrap();
        let ids: Vec<_> = s.iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids, ["chenzhuo", "song_huangyou", "yuan", "ming_xianglin", "qing", "qing_lingtai"]);
        // 清代改奎、觜、参三宿距星，余同
        let (old, qing) = (&s[0], &s[4]);
        let changed: Vec<_> = (0..28).filter(|&i| old.determinatives[i] != qing.determinatives[i]).map(|i| MANSIONS[i]).collect();
        assert_eq!(changed, ["奎", "觜", "参"]);
    }

    /// 《元史·历志一》「周天列宿度」六次实测与本模块推算（同体系距星、同历元）逐宿对照。
    #[test]
    fn widths_vs_yuanshi_table() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let eph = Vsop87Planets { earth: &earth, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let systems = load_lunar_mansion_systems_from_repo().unwrap();
        let sys = |id: &str| systems.iter().find(|s| s.id == id).unwrap();
        let table = yuanshi_widths();
        // (栏, 天文纪年, 体系, 周天)
        let cols = [
            ("汉洛下闳", -103.0, "chenzhuo", 365.25),
            ("唐一行", 724.0, "song_huangyou", 365.25),
            ("宋皇祐", 1052.0, "song_huangyou", 365.25),
            ("宋元丰", 1081.0, "song_huangyou", 365.25),
            ("宋崇宁", 1104.0, "song_huangyou", 365.25),
            ("元至元", 1280.0, "yuan", 365.2575),
        ];
        let mut tab = crate::test_util::CheckTable::new(
            "yuanshi_lodge_widths",
            &["宿", "汉洛下闳 史载", "推算", "差", "唐一行 史载", "推算", "差", "宋皇祐 史载", "推算", "差", "宋元丰 史载", "推算", "差", "宋崇宁 史载", "推算", "差", "元至元 史载", "推算", "差"],
        );
        tab.meta("《元史·历志一》「周天列宿度」历代实测赤道宿度（古度）与推算逐宿对照；推算用同代体系距星、Hipparcos 星表、当日真赤道：");
        tab.meta("汉 前104 陈卓体系；唐 724、皇祐 1052、元丰 1081、崇宁 1104 宋皇祐体系；至元 1280 元体系（周天 365.2575）；余周天 365.25");
        let mut cells: Vec<Vec<String>> = (0..28).map(|i| vec![MANSIONS[i].to_string()]).collect();
        for (c, (name, year, sid, zt)) in cols.iter().enumerate() {
            let jd = 2451545.0 + (year - 2000.0) * 365.25;
            let f = MansionFrame::new(&eph, &cat, sys(sid), TimePoint::new(TimeScale::TT, real(jd)), &ApparentPipelineOptions::default()).unwrap();
            let w = f.widths(real(*zt));
            let diffs: Vec<f64> = (0..28).map(|i| w[i].as_f64() - table[i][c]).collect();
            for i in 0..28 {
                cells[i].extend([format!("{:.4}", table[i][c]), format!("{:.2}", w[i].as_f64()), format!("{:+.2}", diffs[i])]);
            }
            let rms = (diffs.iter().map(|d| d * d).sum::<f64>() / 28.0).sqrt();
            let max = diffs.iter().fold(0.0f64, |m, d| m.max(d.abs()));
            let line: String = (0..28).map(|i| format!("{}{:+.2} ", MANSIONS[i], diffs[i])).collect();
            println!("{name}（{year}，{sid}）均方根 {rms:.2} 最大 {max:.2} 度\n  {line}");
            // 容差按各代测量分辨率：汉唐整度、宋整度至 ¼ 度、至元 0.01 度（郭守敬）；距星认错时相邻两宿常差上度
            let (rms_tol, max_tol) = match c {
                0 | 1 => (1.0, 2.5),
                2 | 3 => (0.7, 1.5),
                4 => (0.35, 0.7),
                _ => (0.15, 0.3),
            };
            assert!(rms < rms_tol && max < max_tol, "{name}: 均方根 {rms:.2} 最大 {max:.2}");
        }
        for r in cells {
            tab.row(r);
        }
        tab.write();
        // 反证：以清代距星（奎 η And、觜 λ Ori、参 ζ Ori）推算至元，奎、觜、参三宿明显偏离实测
        let jd = 2451545.0 + (1280.0 - 2000.0) * 365.25;
        let f = MansionFrame::new(&eph, &cat, sys("qing"), TimePoint::new(TimeScale::TT, real(jd)), &ApparentPipelineOptions::default()).unwrap();
        let w = f.widths(real(365.2575));
        let worst = (0..28).map(|i| (w[i].as_f64() - table[i][5]).abs()).fold(0.0f64, f64::max);
        println!("清代距星推至元：最大差 {worst:.2} 度（奎 {:+.2} 觜 {:+.2} 参 {:+.2}）", w[14].as_f64() - table[14][5], w[19].as_f64() - table[19][5], w[20].as_f64() - table[20][5]);
        assert!(worst > 1.0);
    }

    /// 《元史·历志一·授时历议》（元史/卷052）历代冬至日躔赤道宿度：冬至点赤经 270°（当日真赤道）的入宿度。
    /// 「斗十四度末」取 14.75、「斗九度半」取 9.5。前代所记多由推算或昏旦中星间接定出（原文：「昏旦夜半时刻未易得真」），
    /// 至元一条由月食冲及三年 134 次实测核定。周天：前代 365.25，授时 365.2575。
    #[test]
    fn winter_solstice_lodge_positions_vs_yuanshi() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let eph = Vsop87Planets { earth: &earth, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let systems = load_lunar_mansion_systems_from_repo().unwrap();
        let sys = |id: &str| systems.iter().find(|s| s.id == id).unwrap();
        // (记载, 天文纪年, 体系, 周天, 宿, 入宿度)
        let recs = [
            ("汉元和二年", 85.0, "chenzhuo", 365.25, "斗", 21.0),
            ("晋太元九年", 384.0, "chenzhuo", 365.25, "斗", 17.0),
            ("宋元嘉十年", 433.0, "chenzhuo", 365.25, "斗", 14.75),
            ("梁大同十年", 544.0, "chenzhuo", 365.25, "斗", 12.0),
            ("隋开皇十八年", 598.0, "chenzhuo", 365.25, "斗", 12.0),
            ("唐开元十二年", 724.0, "song_huangyou", 365.25, "斗", 9.5),
            ("元至元", 1280.0, "yuan", 365.2575, "箕", 10.0),
        ];
        let mut tab = crate::test_util::CheckTable::new("yuanshi_winter_solstice", &["记载", "天文纪年", "体系", "史载", "推算", "差（度）"]);
        tab.meta("《元史·历志一》「岁差」节历代冬至日躔赤道宿度与推算（冬至点赤经 270° 的入宿度）；差为推算 − 史载，化到史载之宿起量");
        for (name, year, sid, zt, lodge, du) in recs {
            let jd = 2451545.0 + (year - 2000.0) * 365.25;
            let f = MansionFrame::new(&eph, &cat, sys(sid), TimePoint::new(TimeScale::TT, real(jd)), &ApparentPipelineOptions::default()).unwrap();
            let (i, d) = f.ru_xiu(real(1.5 * core::f64::consts::PI), real(zt));
            let want_i = MANSIONS.iter().position(|m| *m == lodge).unwrap();
            // 化为自同一距星起量，便于比较（跨宿时以该宿宿度相接）
            let w = f.widths(real(zt));
            let got = if i == want_i { d.as_f64() } else if (i + 1) % 28 == want_i { d.as_f64() - w[want_i - 1].as_f64() } else { d.as_f64() + w[want_i].as_f64() };
            println!("{name}（{year}）：推算 {}{:.2} 度，史载 {lodge}{du}，差 {:+.2} 度", MANSIONS[i], d.as_f64(), got - du);
            tab.row(vec![name.to_string(), format!("{year}"), sid.to_string(), format!("{lodge}{du}"), format!("{}{:.2}", MANSIONS[i], d.as_f64()), format!("{:+.2}", got - du)]);
            if name == "元至元" {
                assert!((got - du).abs() < 0.5, "{name}");
            }
        }
        tab.write();
    }

    /// 潘鼐《中国恒星观测史》表 9.1.4：《仪象考成》二十八宿距星 1744 年赤经、赤纬与潘鼐「今测计算值」（1744.0 平位置）。
    /// 本模块平位置（Hipparcos 空间运动 + Vondrák 岁差）对今测值逐星比较；并复算原表的仪象考成误差。
    #[test]
    fn qing_determinatives_vs_pannai_table_9_1_4() {
        use crate::astronomy::apparent::star_mean_equatorial;
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let lines = crate::repo::read_lines("data/sky_culture/pannai_table_9_1_4.csv").unwrap();
        let jd = 2451545.0 + (1744.0 - 2000.0) * 365.25;
        let t = TimePoint::new(TimeScale::TT, real(jd));
        let mut worst: f64 = 0.0;
        let mut tab = crate::test_util::CheckTable::new(
            "pannai_table_9_1_4",
            &["距星", "今名", "HIP", "潘鼐今测 RA°", "潘鼐今测 Dec°", "本模块−今测 RA″", "本模块−今测 Dec″", "仪象考成误差 RA″（以本模块为准）", "Dec″"],
        );
        tab.meta("潘鼐《中国恒星观测史》表 9.1.4（《仪象考成》1744 年二十八宿距星）逐星：本模块平位置（Hipparcos 空间运动 + Vondrák 岁差）对潘鼐「今测计算值」1744.0");
        for l in lines.iter().filter(|l| !l.starts_with('#')).skip(1) {
            let f: Vec<&str> = l.split(',').collect();
            let hip: u32 = f[3].parse().unwrap();
            let (pn_ra, pn_dec): (f64, f64) = (f[7].parse().unwrap(), f[8].parse().unwrap());
            let (ykc_ra, ykc_dec): (f64, f64) = (f[5].parse().unwrap(), f[6].parse().unwrap());
            let p = star_mean_equatorial(cat.by_hip(hip).unwrap(), t, &ApparentPipelineOptions::default());
            let (a, d) = right_ascension_declination(&p);
            let (ra, dec) = (a.rad().as_f64().to_degrees(), d.rad().as_f64().to_degrees());
            let dra = ((ra - pn_ra + 540.0) % 360.0 - 180.0) * 3600.0;
            let ddec = (dec - pn_dec) * 3600.0;
            let ykc = (((ykc_ra - ra + 540.0) % 360.0 - 180.0) * 3600.0, (ykc_dec.abs() - dec.abs()) * 3600.0);
            println!("{} {:8} 本模块 − 潘鼐今测：赤经 {dra:+6.1}″ 赤纬 {ddec:+6.1}″ | 仪象考成误差（以本模块为准）{:+6.0}″ {:+6.0}″", f[1], f[2], ykc.0, ykc.1);
            worst = worst.max(dra.abs()).max(ddec.abs());
            tab.row(vec![f[1].into(), f[2].into(), f[3].into(), format!("{pn_ra:.5}"), format!("{pn_dec:.5}"), format!("{dra:+.1}"), format!("{ddec:+.1}"), format!("{:+.0}", ykc.0), format!("{:+.0}", ykc.1)]);
        }
        tab.write();
        println!("最大差 {worst:.1}″");
        // 潘鼐今测值据老星表推算，至角秒级；双星 λ Ori 差约 15″
        assert!(worst < 20.0, "{worst}");
    }
}
