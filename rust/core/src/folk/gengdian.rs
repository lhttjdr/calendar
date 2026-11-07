//! 更点：夜间五更（每更五点）的起讫时刻。
//!
//! 两家，均据《欽定協紀辨方書》卷十三「更漏中星」：
//! - **元授时法**：「於日入後日出前減去晨昏分各二刻半，餘為夜時，分為五更，每更又分為五點，於初更三點時起更，
//!   五更三點時攢點」。《元史》卷五十五《授時曆經下》：「以昏明分減日出分，餘為晨分；加日入分，為昏分」，
//!   「置晨分，倍之，五約，為更率；又五約更率，為點率」，「置所求更點數，以更點率乘之，加其日昏分」。
//!   授时一日百刻，二刻半 = 36 分钟。
//! - **清台官法**：「今臺官相傳之法，則於日入後八刻起更，日出前九刻攢點，計起更至攢點共若干時刻，五分之以為五更。
//!   日出前減朦影刻分為旦刻，日入後加朦影刻分為昏刻」；朦影即太阳在地平下十八度（「其地平下為十八度」）。
//!   清制一日九十六刻，八刻 = 2 小时，九刻 = 2 小时 15 分。
//!
//! 日出入取日心在地平、不计折射（协纪卷十二春分「日出卯正初刻，晝四十八刻」），时刻为当地视太阳时。
//! 输入为纬度与当日太阳赤纬，本模块不碰天文；以协纪卷十二、十三京师逐节气表核对。

use crate::math::real::{real, real_const, Real, RealOps};

/// 日出入、昏旦与各更时刻（当地视太阳时，小时；跨午夜者 ≥ 24）
#[derive(Clone, Debug)]
pub struct GengDian {
    pub ri_chu: Real,
    pub ri_ru: Real,
    /// 昏（授时：日入 + 二刻半；清：日入 + 朦影）
    pub hun: Real,
    /// 旦／晨（次日）
    pub dan: Real,
    /// 一更至五更的起点
    pub geng: [Real; 5],
    /// 起更（授时：初更三点；清：一更）
    pub qi_geng: Real,
    /// 攢点（次日）
    pub zan_dian: Real,
    /// 每点时长（小时）
    pub dian: Real,
}

fn deg(x: Real) -> Real {
    x * Real::pi() / real_const(180.0)
}

/// 地平下 `depression_deg` 度时的半日弧（小时）。极昼极夜时截到 12 或 0。
pub fn half_arc_hours(lat_deg: Real, dec_deg: Real, depression_deg: Real) -> Real {
    let (p, d, h) = (deg(lat_deg), deg(dec_deg), deg(-depression_deg));
    let mut c = (h.sin() - p.sin() * d.sin()) / (p.cos() * d.cos());
    if c > real_const(1.0) {
        c = real_const(1.0);
    }
    if c < real_const(-1.0) {
        c = real_const(-1.0);
    }
    let acos = (real_const(1.0) - c * c).sqrt().atan2(c);
    acos * real_const(12.0) / Real::pi()
}

/// 元授时法
pub fn shou_shi(lat_deg: Real, dec_deg: Real) -> GengDian {
    let h0 = half_arc_hours(lat_deg, dec_deg, real_const(0.0));
    let (ri_chu, ri_ru) = (real_const(12.0) - h0, real_const(12.0) + h0);
    let hun_ming = real_const(2.5 * 24.0 / 100.0); // 二刻半（百刻制）
    let (hun, dan) = (ri_ru + hun_ming, real_const(24.0) + ri_chu - hun_ming);
    let geng_lv = (dan - hun) / real_const(5.0);
    let dian = geng_lv / real_const(5.0);
    let geng = core::array::from_fn(|k| hun + real(k as f64) * geng_lv);
    GengDian {
        ri_chu,
        ri_ru,
        hun,
        dan,
        geng,
        qi_geng: hun + real_const(2.0) * dian,
        zan_dian: hun + real_const(4.0) * geng_lv + real_const(2.0) * dian,
        dian,
    }
}

/// 清台官法
pub fn qing(lat_deg: Real, dec_deg: Real) -> GengDian {
    let h0 = half_arc_hours(lat_deg, dec_deg, real_const(0.0));
    let h18 = half_arc_hours(lat_deg, dec_deg, real_const(18.0));
    let (ri_chu, ri_ru) = (real_const(12.0) - h0, real_const(12.0) + h0);
    let meng = h18 - h0;
    let qi = ri_ru + real_const(2.0); // 日入后八刻（九十六刻制）
    let zan = real_const(24.0) + ri_chu - real_const(2.25); // 日出前九刻
    let geng_lv = (zan - qi) / real_const(5.0);
    let geng = core::array::from_fn(|k| qi + real(k as f64) * geng_lv);
    GengDian {
        ri_chu,
        ri_ru,
        hun: ri_ru + meng,
        dan: real_const(24.0) + ri_chu - meng,
        geng,
        qi_geng: qi,
        zan_dian: zan,
        dian: geng_lv / real_const(5.0),
    }
}

/// 时刻（小时）写成清制「某初/正某刻某分」（一时八刻，一刻十五分）。
pub fn qing_shike(hours: Real) -> String {
    const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    const NUM: [&str; 15] = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二", "十三", "十四"];
    let minutes = (hours * real_const(60.0) + real_const(0.5)).floor().as_f64() as i64;
    let total = minutes.rem_euclid(1440);
    let (h, m) = (total / 60, total % 60);
    let zhi = ZHI[(((h + 1) / 2) % 12) as usize];
    let chu_zheng = if h % 2 == 1 { "初" } else { "正" };
    let (ke, fen) = (m / 15, m % 15);
    let ke_s = if ke == 0 { "初" } else { NUM[ke as usize] };
    if fen == 0 { format!("{zhi}{chu_zheng}{ke_s}刻") } else { format!("{zhi}{chu_zheng}{ke_s}刻{}分", NUM[fen as usize]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 京师北极出地三十九度五十五分（协纪卷十二）；黄赤交角取 23.5°（节气按黄经）
    const LAT: f64 = 39.0 + 55.0 / 60.0;
    fn dec(lon_deg: f64) -> Real {
        real((23.5f64.to_radians().sin() * lon_deg.to_radians().sin()).asin().to_degrees())
    }

    /// 协纪卷十三「更漏中星」京师表：春分、夏至、冬至三条逐项核对（允差一分钟）。
    #[test]
    fn qing_matches_xieji_table() {
        let cases: [(f64, [&str; 9]); 3] = [
            (0.0, ["戌初二刻五分", "戌正初刻", "亥初二刻三分", "子初初刻六分", "子正二刻九分", "丑正初刻十二分", "寅初三刻", "寅正一刻十分", "卯正初刻"]),
            (90.0, ["亥初二刻四分", "亥初一刻十分", "亥正一刻九分", "子初一刻八分", "子正一刻七分", "丑初一刻六分", "丑正一刻五分", "丑正一刻十一分", "寅正二刻五分"]),
            (270.0, ["酉正一刻二分", "酉正二刻五分", "戌正二刻十二分", "亥正三刻四分", "子正三刻十一分", "寅初初刻三分", "卯初初刻十分", "卯初二刻十三分", "辰初一刻十分"]),
        ];
        for (lon, expect) in cases {
            let g = qing(real(LAT), dec(lon));
            let got = [g.hun, g.geng[0], g.geng[1], g.geng[2], g.geng[3], g.geng[4], g.zan_dian, g.dan, g.ri_chu];
            for (k, (v, e)) in got.iter().zip(expect.iter()).enumerate() {
                let near = [-1.0, 0.0, 1.0].iter().any(|dm| qing_shike(*v + real(dm / 60.0)) == *e);
                assert!(near, "黄经 {lon} 第 {k} 项：算得 {}，表 {e}", qing_shike(*v));
            }
        }
    }

    #[test]
    fn shou_shi_equinox() {
        let g = shou_shi(real(LAT), real(0.0));
        let f = |x: Real| x.as_f64();
        assert!((f(g.hun) - 18.6).abs() < 1e-9); // 日入 18:00 + 36 分
        assert!((f(g.geng[0]) - f(g.hun)).abs() < 1e-12); // 初更自昏起
        assert!(((f(g.dan) - f(g.hun)) - 10.8).abs() < 1e-9); // 夜 = 24 − 12 − 1.2
        assert!((f(g.zan_dian) - (f(g.hun) + 4.0 * 2.16 + 2.0 * 0.432)).abs() < 1e-9);
    }

    #[test]
    fn shike_format() {
        assert_eq!(qing_shike(real(20.0)), "戌正初刻");
        assert_eq!(qing_shike(real(21.0 + 33.0 / 60.0)), "亥初二刻三分");
        assert_eq!(qing_shike(real(23.0 + 6.0 / 60.0)), "子初初刻六分");
        assert_eq!(qing_shike(real(7.0 + 25.0 / 60.0)), "辰初一刻十分");
    }
}
