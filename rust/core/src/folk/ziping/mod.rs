//! 子平命理：四柱、十神、藏干、十二长生、大运小运、胎元命宫、空亡、神煞、月令取格。
//!
//! 只做原典规则可以机械推出的部分，不作吉凶判断；格局、用神只做「月令透干取格」这一条可判定的规则（见 [geju]）。
//! 原典出处、各家异同与逐条检验见 `doc/folk/ziping.md`。
//!
//! 分层（`doc/architecture/three-domains.md`「民俗域」）：
//! - **独立基础层**（只依赖干支序数）：[shishen]（十神）、[canggan]（地支藏干、人元司令）、[changsheng]（十二长生）、
//!   [shensha]（空亡与神煞）、[gong]（胎元、命宫、身宫、胎息、变）。
//! - **聚合层**：[pan]（由出生时刻、节气时刻排四柱，起大运、小运，生前三百日胎元）。节气时刻取天文域
//!   `astronomy::aspects::solar_term_jds_for_year_cached`，平气取历法域 `calendar::gan_zhi_options::pingqi_jds_from_dingqi`。
//! - **派生层**：[geju]（月令取格，依《子平真詮》）。
//! - **仲裁层**：[ZipingOptions] 汇集各项诸家之说的选择，缺省按证据定（理由见各枚举注释与文档）。
//!
//! 干、支、干支均以序数表示：干 0 甲 … 9 癸，支 0 子 … 11 亥，干支 0 甲子 … 59 癸亥。

pub mod canggan;
pub mod changsheng;
pub mod geju;
pub mod gong;
pub mod pan;
pub mod shensha;
pub mod shishen;

#[cfg(test)]
mod cases;

/// 十干
pub const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 十二支
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 干支序的干
#[inline]
pub fn gan_of(gz: u8) -> u8 {
    gz % 10
}

/// 干支序的支
#[inline]
pub fn zhi_of(gz: u8) -> u8 {
    gz % 12
}

/// 干、支 → 干支序（干支须同阴阳）：i ≡ 6g − 5z (mod 60)
#[inline]
pub fn gz_of(gan: u8, zhi: u8) -> u8 {
    debug_assert_eq!(gan % 2, zhi % 2, "干支阴阳不同");
    ((6 * gan as i32 - 5 * zhi as i32).rem_euclid(60)) as u8
}

/// 干支名
pub fn gz_name(gz: u8) -> String {
    format!("{}{}", GAN[gan_of(gz) as usize], ZHI[zhi_of(gz) as usize])
}

/// 由名称解析干支序（测试与命例用）
pub fn parse_gz(s: &str) -> Option<u8> {
    let mut it = s.chars();
    let (a, b) = (it.next()?, it.next()?);
    let g = GAN.iter().position(|x| x.starts_with(a))? as u8;
    let z = ZHI.iter().position(|x| x.starts_with(b))? as u8;
    (g % 2 == z % 2).then(|| gz_of(g, z))
}

/// 五行：0 木、1 火、2 土、3 金、4 水（相生次序）
pub const WU_XING: [&str; 5] = ["木", "火", "土", "金", "水"];

/// 干的五行
#[inline]
pub fn gan_wuxing(gan: u8) -> u8 {
    gan / 2
}

/// 支的五行（本气）：寅卯木、巳午火、申酉金、亥子水、辰戌丑未土
pub fn zhi_wuxing(zhi: u8) -> u8 {
    [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4][(zhi % 12) as usize]
}

/// 干为阳（甲丙戊庚壬）
#[inline]
pub fn gan_is_yang(gan: u8) -> bool {
    gan % 2 == 0
}

/// 四柱：年、月、日、时干支序
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiZhu {
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
}

impl SiZhu {
    pub fn new(year: u8, month: u8, day: u8, hour: u8) -> Self {
        Self { year, month, day, hour }
    }

    /// 由「丙子辛丑癸亥己卯」之类八字串解析（测试与命例用）
    pub fn parse(s: &str) -> Option<Self> {
        fn two(c: &[char]) -> Option<u8> {
            parse_gz(&c.iter().collect::<String>())
        }
        let c: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        if c.len() != 8 {
            return None;
        }
        Some(Self::new(two(&c[0..2])?, two(&c[2..4])?, two(&c[4..6])?, two(&c[6..8])?))
    }

    pub fn as_array(&self) -> [u8; 4] {
        [self.year, self.month, self.day, self.hour]
    }

    pub fn day_gan(&self) -> u8 {
        gan_of(self.day)
    }

    pub fn name(&self) -> String {
        self.as_array().iter().map(|&g| gz_name(g)).collect::<Vec<_>>().join(" ")
    }

    /// 五虎遁：月干是否合年干所起（甲己之年丙作首……）
    pub fn month_consistent(&self) -> bool {
        let yin_gan = (gan_of(self.year) * 2 + 2) % 10;
        let ord = (zhi_of(self.month) + 10) % 12; // 寅 = 0
        gan_of(self.month) == (yin_gan + ord) % 10
    }

    /// 五鼠遁：时干是否合日干所起（甲己還加甲……）
    pub fn hour_consistent(&self) -> bool {
        gan_of(self.hour) == (gan_of(self.day) * 2 + zhi_of(self.hour)) % 10
    }
}

/// 子平诸项选项（仲裁层）：每项诸家之说并列为枚举，缺省按证据定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZipingOptions {
    /// 四柱的年、月、日界与定气平气（历法域选项，缺省取 `folk::ganzhi_presets::zi_ping_ba_zi`：立春、节、子正、定气）
    pub day_boundary: crate::calendar::gan_zhi_options::DayBoundary,
    pub solar_term_mode: crate::calendar::gan_zhi_options::SolarTermMode,
    pub ye_zi_shi: pan::YeZiShi,
    pub qi_yun: pan::QiYunRule,
    pub qi_yun_precision: pan::QiYunPrecision,
    pub xiao_yun: pan::XiaoYunRule,
    pub tai_yuan: pan::TaiYuanRule,
    pub ming_gong_month: gong::MingGongMonth,
    pub cang_gan: canggan::CangGanTable,
    pub si_ling: canggan::SiLingTable,
    pub chang_sheng: changsheng::ChangShengRule,
    pub tu_chang_sheng: changsheng::TuChangSheng,
    pub shensha: shensha::ShenshaOptions,
}

impl Default for ZipingOptions {
    fn default() -> Self {
        let gz = crate::folk::ganzhi_presets::zi_ping_ba_zi();
        Self {
            day_boundary: gz.day_boundary,
            solar_term_mode: gz.solar_term_mode,
            ye_zi_shi: pan::YeZiShi::default(),
            qi_yun: pan::QiYunRule::default(),
            qi_yun_precision: pan::QiYunPrecision::default(),
            xiao_yun: pan::XiaoYunRule::default(),
            tai_yuan: pan::TaiYuanRule::default(),
            ming_gong_month: gong::MingGongMonth::default(),
            cang_gan: canggan::CangGanTable::default(),
            si_ling: canggan::SiLingTable::default(),
            chang_sheng: changsheng::ChangShengRule::default(),
            tu_chang_sheng: changsheng::TuChangSheng::default(),
            shensha: shensha::ShenshaOptions::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gz_roundtrip() {
        for i in 0..60u8 {
            assert_eq!(gz_of(gan_of(i), zhi_of(i)), i);
            assert_eq!(parse_gz(&gz_name(i)), Some(i));
        }
        let s = SiZhu::parse("丙子辛丑癸亥乙卯").unwrap();
        assert_eq!(s.name(), "丙子 辛丑 癸亥 乙卯");
        assert!(s.month_consistent() && s.hour_consistent());
        // 《三命通會》卷三录苏轼「丙子辛丑癸亥己卯」：癸日卯时当为乙卯，「己」字不合五鼠遁
        assert!(!SiZhu::parse("丙子辛丑癸亥己卯").unwrap().hour_consistent());
        // 五虎遁：甲年寅月丙寅；五鼠遁：甲日子时甲子
        assert!(SiZhu::parse("甲子丙寅甲子甲子").unwrap().month_consistent());
        assert!(!SiZhu::parse("甲子戊寅甲子丙子").unwrap().month_consistent());
        assert!(!SiZhu::parse("甲子丙寅甲子丙子").unwrap().hour_consistent());
    }
}
