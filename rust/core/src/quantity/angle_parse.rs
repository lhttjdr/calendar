//! 角度字符串解析，8 个 Matcher + dhms 校验（8 个 Matcher + dhms 校验）。

use regex::{Captures, Regex};

use crate::math::real::RealOps;
use crate::math::series::arcsec_to_rad;

fn trim_ws(s: &str) -> String {
    s.split_whitespace().collect::<String>()
}

const UNIT_CHARS: &[char] = &['°', '\u{00B0}', '\u{2032}', '\u{2033}', '度', '分', '秒', '\'', '"'];

fn no_unit_in_number(s: &str) -> Result<(), String> {
    if s.chars().any(|c| UNIT_CHARS.contains(&c)) {
        return Err(format!("Angle parse: unit character in number '{}'", s));
    }
    Ok(())
}

fn parse_f64(s: &str) -> Result<f64, String> {
    no_unit_in_number(s)?;
    let f: f64 = s.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
    if f.is_finite() {
        Ok(f)
    } else {
        Err("non-finite value".to_string())
    }
}

fn is_integer(x: f64) -> bool {
    x.fract() == 0.0 && x.is_finite()
}

#[derive(Clone, Copy)]
struct Parsed {
    neg: bool,
    d: f64,
    m: f64,
    s: f64,
    ha: bool,
}

fn dhms(
    d: Option<&str>,
    m: Option<&str>,
    s: Option<&str>,
) -> Result<(f64, f64, f64), String> {
    match (d, m, s) {
        (None, None, Some(ss)) => Ok((0.0, 0.0, parse_f64(ss)?)),
        (None, Some(mm), None) => Ok((0.0, parse_f64(mm)?, 0.0)),
        (None, Some(mm), Some(ss)) => {
            let bm = parse_f64(mm)?;
            let bs = parse_f64(ss)?;
            if is_integer(bm) && (0.0..60.0).contains(&bs) {
                Ok((0.0, bm, bs))
            } else {
                Err("Illegal Angle!".to_string())
            }
        }
        (Some(dd), None, None) => Ok((parse_f64(dd)?, 0.0, 0.0)),
        (Some(_), None, Some(_)) => Err("Impossible place!".to_string()),
        (Some(dd), Some(mm), None) => {
            let bd = parse_f64(dd)?;
            let bm = parse_f64(mm)?;
            if is_integer(bd) && (0.0..60.0).contains(&bm) {
                Ok((bd, bm, 0.0))
            } else {
                Err("Illegal Angle!".to_string())
            }
        }
        (Some(dd), Some(mm), Some(ss)) => {
            let bd = parse_f64(dd)?;
            let bm = parse_f64(mm)?;
            let bs = parse_f64(ss)?;
            if is_integer(bd) && (0.0..60.0).contains(&bm) && (0.0..60.0).contains(&bs) {
                Ok((bd, bm, bs))
            } else {
                Err("Illegal Angle!".to_string())
            }
        }
        (None, None, None) => Err("Impossible place!".to_string()),
    }
}

fn is_h(u: &str) -> bool {
    matches!(u.chars().next(), Some('h') | Some('°') | Some('度'))
}

fn is_m(u: &str) -> bool {
    matches!(u.chars().next(), Some('m') | Some('\'') | Some('\u{2032}') | Some('分'))
}

fn is_s(u: &str) -> bool {
    matches!(u.chars().next(), Some('s') | Some('"') | Some('\u{2033}') | Some('秒'))
}

fn is_ha(u: &str) -> bool {
    u == "h" || u == "s"
}

fn dhms2sec(d: f64, m: f64, s: f64) -> f64 {
    d * 3600.0 + m * 60.0 + s
}

fn run1(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let num_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let u = c.get(4).map_or("", |m| m.as_str());
    let (d, m, s) = dhms(
        if is_h(u) { Some(num_str.as_str()) } else { None },
        if is_m(u) { Some(num_str.as_str()) } else { None },
        if is_s(u) { Some(num_str.as_str()) } else { None },
    )?;
    Ok(Parsed { neg, d, m, s, ha: is_ha(u) })
}

fn run2(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let v = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(4).map(|m| m.as_str()).unwrap_or("");
    let u = c.get(3).map_or("", |m| m.as_str());
    let (d, m, s) = dhms(
        if is_h(u) { Some(v.as_str()) } else { None },
        if is_m(u) { Some(v.as_str()) } else { None },
        if is_s(u) { Some(v.as_str()) } else { None },
    )?;
    Ok(Parsed { neg, d, m, s, ha: is_ha(u) })
}

fn run3(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let dh_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let (dh, mm, _) = dhms(Some(dh_str.as_str()), c.get(5).map(|m| m.as_str()), None)?;
    let u = c.get(4).map_or("", |m| m.as_str());
    Ok(Parsed { neg, d: dh, m: mm, s: 0.0, ha: is_ha(u) })
}

fn matched2rad(p: Parsed) -> f64 {
    let sec = if p.ha {
        dhms2sec(p.d, p.m, p.s) * 15.0
    } else {
        dhms2sec(p.d, p.m, p.s)
    };
    let rad = arcsec_to_rad(sec).as_f64();
    if p.neg {
        rad
    } else {
        -rad
    }
}

fn run4(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let dh_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let m_val = c.get(5).map_or("", |m| m.as_str()).to_string()
        + c.get(7).map(|m| m.as_str()).unwrap_or("");
    let (dh, mm, _) = dhms(Some(dh_str.as_str()), Some(m_val.as_str()), None)?;
    let u = c.get(4).map_or("", |m| m.as_str());
    Ok(Parsed { neg, d: dh, m: mm, s: 0.0, ha: is_ha(u) })
}

fn run5(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let mm_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let (_, mm, ss) = dhms(None, Some(mm_str.as_str()), c.get(5).map(|m| m.as_str()))?;
    let u = c.get(4).map_or("", |m| m.as_str());
    Ok(Parsed { neg, d: 0.0, m: mm, s: ss, ha: is_ha(u) })
}

fn run6(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let mm_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let s_val = c.get(5).map_or("", |m| m.as_str()).to_string()
        + c.get(7).map(|m| m.as_str()).unwrap_or("");
    let (_, mm, ss) = dhms(None, Some(mm_str.as_str()), Some(s_val.as_str()))?;
    let u = c.get(4).map_or("", |m| m.as_str());
    Ok(Parsed { neg, d: 0.0, m: mm, s: ss, ha: is_ha(u) })
}

fn run7(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let dh_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let (dh, mm, ss) = dhms(
        Some(dh_str.as_str()),
        c.get(5).map(|m| m.as_str()),
        c.get(7).map(|m| m.as_str()),
    )?;
    let u = c.get(4).map_or("", |m| m.as_str());
    Ok(Parsed { neg, d: dh, m: mm, s: ss, ha: is_ha(u) })
}

fn run8(c: &Captures) -> Result<Parsed, String> {
    let neg = c.get(1).map_or("", |m| m.as_str()) != "-";
    let dh_str = c.get(2).map_or("", |m| m.as_str()).to_string()
        + c.get(3).map(|m| m.as_str()).unwrap_or("");
    let s_val = c.get(7).map_or("", |m| m.as_str()).to_string()
        + c.get(9).map(|m| m.as_str()).unwrap_or("");
    let (dh, mm, ss) = dhms(
        Some(dh_str.as_str()),
        c.get(5).map(|m| m.as_str()),
        Some(s_val.as_str()),
    )?;
    let u = c.get(4).map_or("", |m| m.as_str());
    Ok(Parsed { neg, d: dh, m: mm, s: ss, ha: is_ha(u) })
}

/// 禁止重复单位符号（禁止重复单位符号：degree/arcminute/arcsecond symbol at most once）
fn reject_duplicate_units(s: &str) -> Result<(), String> {
    let deg = s.chars().filter(|c| matches!(c, '°' | '度')).count();
    let min = s.chars().filter(|c| matches!(c, '\'' | '\u{2032}' | '分')).count();
    let sec = s.chars().filter(|c| matches!(c, '"' | '\u{2033}' | '秒')).count();
    if deg > 1 || min > 1 || sec > 1 {
        return Err("Angle parse: duplicate unit symbol".to_string());
    }
    Ok(())
}

pub fn plane_angle_parse(s: &str) -> Result<f64, String> {
    let t = trim_ws(s);
    if t.is_empty() {
        return Err("Parse Error!".to_string());
    }
    reject_duplicate_units(&t)?;
    let sign = r"([+-]?)";
    let num = r"([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)";
    let exp_opt = r"(?:([eE][+-]?[0-9]+))?";
    let unit_h = format!("[h°{}{}]", '\u{00B0}', '度');
    let unit_m = format!("[m'{}{}]", '\u{2032}', '分');
    let unit_s = format!("[s\"{}{}]", '\u{2033}', '秒');
    let one_unit = format!("[hms°'\"{}{}{}{}{}{}]", '\u{00B0}', '\u{2032}', '\u{2033}', '度', '分', '秒');

    let m1 = Regex::new(&format!("^{}{}{}({})$", sign, num, exp_opt, one_unit)).map_err(|e| e.to_string())?;
    let m2 = Regex::new(&format!("^([+-]?)([0-9]+)({})(\\.[0-9]+)$", one_unit)).map_err(|e| e.to_string())?;
    let m3 = Regex::new(&format!(
        "{}{}{}({})([0-5]?[0-9](?:\\.[0-9]+)?)({})$",
        sign, num, exp_opt, unit_h, unit_m
    )).map_err(|e| e.to_string())?;
    let m4 = Regex::new(&format!(
        "{}{}{}({})([0-5]?[0-9])({})(\\.[0-9]+)?$",
        sign, num, exp_opt, unit_h, unit_m
    )).map_err(|e| e.to_string())?;
    let m5 = Regex::new(&format!(
        "{}{}{}({})([0-5]?[0-9](?:\\.[0-9]+)?)({})$",
        sign, num, exp_opt, unit_m, unit_s
    )).map_err(|e| e.to_string())?;
    let m6 = Regex::new(&format!(
        "{}{}{}({})([0-5]?[0-9])({})(\\.[0-9]+)?$",
        sign, num, exp_opt, unit_m, unit_s
    )).map_err(|e| e.to_string())?;
    let m7 = Regex::new(&format!(
        "{}{}{}({})([0-5]?[0-9])({})([0-5]?[0-9](?:\\.[0-9]+)?)({})$",
        sign, num, exp_opt, unit_h, unit_m, unit_s
    )).map_err(|e| e.to_string())?;
    let m8 = Regex::new(&format!(
        "{}{}{}({})([0-5]?[0-9])({})([0-5]?[0-9])({})(\\.[0-9]+)?$",
        sign, num, exp_opt, unit_h, unit_m, unit_s
    )).map_err(|e| e.to_string())?;

    // dms (m7/m8) 在 ms (m5/m6) 之前尝试，避免 "18度20分34秒" 被 m5 误匹配
    let parsed = if let Some(cap) = m1.captures(&t) {
        run1(&cap)?
    } else if let Some(cap) = m2.captures(&t) {
        run2(&cap)?
    } else if let Some(cap) = m3.captures(&t) {
        run3(&cap)?
    } else if let Some(cap) = m4.captures(&t) {
        run4(&cap)?
    } else if let Some(cap) = m7.captures(&t) {
        run7(&cap)?
    } else if let Some(cap) = m8.captures(&t) {
        run8(&cap)?
    } else if let Some(cap) = m5.captures(&t) {
        run5(&cap)?
    } else if let Some(cap) = m6.captures(&t) {
        run6(&cap)?
    } else {
        return Err("Parse Error!".to_string());
    };

    Ok(matched2rad(parsed))
}

#[cfg(test)]
mod tests {
    use super::plane_angle_parse;
    use crate::math::angle::deg2rad;
    use crate::math::real::{real, RealOps};
    use crate::math::series::arcsec_to_rad;
    use crate::quantity::angle::PlaneAngle;

    const EPS: f64 = 1e-12;

    fn assert_near_f64(a: f64, b: f64) {
        assert!(real(a).is_near(real(b), EPS), "{} vs {}", a, b);
    }

    #[test]
    fn single_unit_scientific_notation() {
        plane_angle_parse("1.8e-2°").unwrap();
        plane_angle_parse("-1.8e-2'").unwrap();
        plane_angle_parse("+1.8e-2\"").unwrap();
    }

    #[test]
    fn dms_with_decimal_on_seconds() {
        plane_angle_parse("+18°20'34.5\"").unwrap();
        plane_angle_parse("-18°20'34\".5").unwrap();
    }

    #[test]
    fn dm_degree_integer_minute_lt_60() {
        plane_angle_parse("18°20.5'").unwrap();
    }

    #[test]
    fn dms_degree_and_minute_integer() {
        plane_angle_parse("18°20'34\".5").unwrap();
    }

    #[test]
    fn plus_45_deg_equals_pi_over_4() {
        let r = PlaneAngle::parse("+45°").unwrap();
        assert!(r.rad().is_near(real(std::f64::consts::FRAC_PI_4), EPS));
    }

    #[test]
    fn illegal_duplicate_degree_symbol() {
        assert!(PlaneAngle::parse("18°18°20'34.5\"").is_err());
    }

    #[test]
    fn illegal_duplicate_minute_symbol() {
        assert!(PlaneAngle::parse("18°20'20'34\".5").is_err());
    }

    #[test]
    fn illegal_duplicate_second_symbol() {
        assert!(PlaneAngle::parse("18°20'34\"34\".5").is_err());
    }

    #[test]
    fn illegal_degree_not_integer_when_dm() {
        assert!(PlaneAngle::parse("18.5°20.5'").is_err());
    }

    #[test]
    fn illegal_minute_ge_60() {
        assert!(PlaneAngle::parse("18°60'").is_err());
    }

    #[test]
    fn illegal_degree_not_integer_when_dms() {
        assert!(PlaneAngle::parse("-18.5°20'34\".5").is_err());
    }

    #[test]
    fn no_match_parse_error() {
        let e = PlaneAngle::parse("not an angle").unwrap_err();
        assert_eq!(e, "Parse Error!");
    }

    #[test]
    fn convert_deg2rad_360_equals_double_pi() {
        assert!(deg2rad(360.0).is_near(real(std::f64::consts::TAU), EPS));
    }

    #[test]
    fn convert_min2rad_90_60_equals_half_pi() {
        let min_per_rad = 60.0 * 180.0 / std::f64::consts::PI;
        assert_near_f64(90.0 * 60.0 / min_per_rad, std::f64::consts::FRAC_PI_2);
    }

    #[test]
    fn convert_sec2rad_180_3600_equals_pi() {
        assert!(arcsec_to_rad(180.0 * 60.0 * 60.0).is_near(real(std::f64::consts::PI), EPS));
    }

    #[test]
    fn unicode_degree_u00b0_plus_45_as_pi_over_4() {
        let r = plane_angle_parse("+45\u{00B0}").unwrap();
        assert_near_f64(r, std::f64::consts::FRAC_PI_4);
    }

    #[test]
    fn unicode_prime_u2032_30_arcmin() {
        let r = plane_angle_parse("30\u{2032}").unwrap();
        assert!(real(r).is_near(arcsec_to_rad(30.0 * 60.0), EPS));
    }

    #[test]
    fn unicode_double_prime_u2033_45_arcsec() {
        let r = plane_angle_parse("45\u{2033}").unwrap();
        assert!(real(r).is_near(arcsec_to_rad(45.0), EPS));
    }

    #[test]
    fn unicode_dms_18_20_34() {
        let r = plane_angle_parse("18°20\u{2032}34\u{2033}").unwrap();
        let expected = deg2rad(18.0 + 20.0 / 60.0 + 34.0 / 3600.0);
        assert!(real(r).is_near(expected, EPS));
    }

    #[test]
    fn cjk_45_du_equals_pi_over_4() {
        let r = PlaneAngle::parse("45度").unwrap();
        assert!(r.rad().is_near(real(std::f64::consts::FRAC_PI_4), EPS));
    }

    #[test]
    fn cjk_18_du_20_fen_34_miao() {
        let r = PlaneAngle::parse("18度20分34秒").unwrap();
        let expected = deg2rad(18.0 + 20.0 / 60.0 + 34.0 / 3600.0);
        assert!(r.rad().is_near(expected, EPS));
    }
}
