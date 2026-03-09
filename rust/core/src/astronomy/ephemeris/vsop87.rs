use crate::astronomy::constant::AU_METERS;
use crate::astronomy::time::{jd_to_t, JulianMillennia, TimePoint};
use crate::math::real::{real, Real, RealOps};
use crate::platform::{DataLoader, LoadError};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::unit::{AngularRateUnit, LengthUnit, SpeedUnit};
use crate::quantity::{angular_rate::AngularRate, length::Length, speed::Speed};

// --------------- VSOP87 振幅与项：物理量 ---------------

/// VSOP87 振幅：L/B 用平面角，R 用长度(AU)。
#[derive(Clone, Debug, PartialEq)]
pub enum Vsop87Amplitude {
    Angle(PlaneAngle),
    Length(Length),
}

/// VSOP87 项：振幅（角或长）、相位（角）、频率（角频率，单位可为 rad/儒略千年）。
#[derive(Clone, Debug)]
pub struct VsopTerm {
    pub amplitude: Vsop87Amplitude,
    pub phase: PlaneAngle,
    pub frequency: AngularRate,
}

#[derive(Clone, Debug)]
pub struct VsopBlock {
    pub coords: u8,
    pub alpha_t: u8,
    pub terms: Vec<VsopTerm>,
}

#[derive(Clone, Debug)]
pub struct Vsop87 {
    pub blocks: Vec<VsopBlock>,
}

/// VSOP87 球面位置：L/B 为角，R 为长。标量 Real。
#[derive(Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct Vsop87SphericalPosition {
    pub L: PlaneAngle,
    pub B: PlaneAngle,
    pub R: Length,
}

/// VSOP87 球面速度：dL/dB 为角速率（rad/日），dR 为速率（au/日）。标量 Real。
#[derive(Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct Vsop87SphericalVelocity {
    pub d_l: AngularRate,
    pub d_b: AngularRate,
    pub d_r: Speed,
}

impl Vsop87 {
    /// 位置与速度一次算出；全程 Real。
    pub fn position_and_velocity_jd(&self, jd: Real) -> (Vsop87SphericalPosition, Vsop87SphericalVelocity) {
        let JulianMillennia(t) = jd_to_t(jd);
        let t_pow: [Real; 6] = [
            real(1),
            t,
            t * t,
            t * t * t,
            t * t * t * t,
            t * t * t * t * t,
        ];

        let mut sum_l = PlaneAngle::from_rad(real(0));
        let mut sum_b = PlaneAngle::from_rad(real(0));
        let mut sum_r = Length::from_value(real(0), LengthUnit::Meter);
        let mut sum_dl = AngularRate::from_value(real(0), AngularRateUnit::RadPerJulianMillennium);
        let mut sum_db = AngularRate::from_value(real(0), AngularRateUnit::RadPerJulianMillennium);
        let mut sum_dr = Speed::from_value(real(0), SpeedUnit::MPerJulianMillennium);

        for block in &self.blocks {
            let coord_idx = (block.coords as usize).saturating_sub(1);
            if coord_idx >= 3 {
                continue;
            }
            let a = block.alpha_t as usize;
            let t_pow_a = t_pow.get(a).copied().unwrap_or(real(0));
            let t_pow_am1 = if a == 0 { real(0) } else { t_pow.get(a - 1).copied().unwrap_or(real(0)) };

            for term in &block.terms {
                let phase_t = term.phase + term.frequency.angle_for_t_julian_millennia(t);
                let phase_reduced = phase_t.wrap_to_2pi().rad();
                let cos_b = phase_reduced.cos();
                let sin_b = phase_reduced.sin();
                let vel_scalar = if a == 0 {
                    real(0)
                } else {
                    real(a as f64) * t_pow_am1 * cos_b
                        - term.frequency.rad_per_julian_millennium() * t_pow_a * sin_b
                };

                match &term.amplitude {
                    Vsop87Amplitude::Angle(angle) => {
                        let pos = angle.scale(t_pow_a * cos_b);
                        let rate_contrib = AngularRate::from_value(angle.rad() * vel_scalar, AngularRateUnit::RadPerJulianMillennium);
                        match block.coords {
                            1 => {
                                sum_l = sum_l + pos;
                                sum_dl = sum_dl + rate_contrib;
                            }
                            2 => {
                                sum_b = sum_b + pos;
                                sum_db = sum_db + rate_contrib;
                            }
                            _ => {}
                        }
                    }
                    Vsop87Amplitude::Length(length) => {
                        if block.coords != 3 {
                            continue;
                        }
                        sum_r = Length::from_value(sum_r.meters() + length.scale(t_pow_a * cos_b).meters(), LengthUnit::Meter);
                        sum_dr = sum_dr + Speed::from_value(length.meters() * vel_scalar, SpeedUnit::MPerJulianMillennium);
                    }
                }
            }
        }

        let pos = Vsop87SphericalPosition {
            L: sum_l.wrap_to_2pi(),
            B: sum_b.wrap_to_signed_pi(),
            R: sum_r,
        };
        let vel = Vsop87SphericalVelocity {
            d_l: AngularRate::from_value(sum_dl.in_unit(AngularRateUnit::RadPerDay), AngularRateUnit::RadPerDay),
            d_b: AngularRate::from_value(sum_db.in_unit(AngularRateUnit::RadPerDay), AngularRateUnit::RadPerDay),
            d_r: Speed::from_value(sum_dr.in_unit(SpeedUnit::MPerS), SpeedUnit::MPerS),
        };
        (pos, vel)
    }

    /// 球面位置（物理量）；jd 为 TDB。
    pub fn position_jd(&self, jd: Real) -> Vsop87SphericalPosition {
        self.position_and_velocity_jd(jd).0
    }

    /// 球面位置（物理量）；历表时间按 VSOP87 约定用 TDB。
    pub fn position(&self, t: TimePoint) -> Vsop87SphericalPosition {
        self.position_jd(t.jd_tdb())
    }

    /// 球面速度（物理量：rad/日、au/日）。
    pub fn velocity_jd(&self, jd: Real) -> Vsop87SphericalVelocity {
        self.position_and_velocity_jd(jd).1
    }

    /// 球面速度（物理量）；历表时间用 TDB。
    pub fn velocity(&self, t: TimePoint) -> Vsop87SphericalVelocity {
        self.velocity_jd(t.jd_tdb())
    }

    /// 直角速度（m/s）在 J2000 平黄道架。
    pub fn velocity_ecliptic_j2000_m_per_s(&self, jd: Real) -> [Real; 3] {
        let (pos, vel) = self.position_and_velocity_jd(jd);
        let l = pos.L.rad();
        let b = pos.B.rad();
        let r_au = pos.R.meters() / AU_METERS;
        let (cl, sl) = (l.cos(), l.sin());
        let (cb, sb) = (b.cos(), b.sin());
        let dl = vel.d_l.rad_per_day();
        let db = vel.d_b.rad_per_day();
        let dr = vel.d_r.au_per_day(crate::astronomy::constant::AU_METERS);
        let au_m = AU_METERS;
        let day_s = 86400.0;
        let vx = (dr * cb * cl - r_au * sb * cl * db - r_au * cb * sl * dl) * au_m / day_s;
        let vy = (dr * cb * sl - r_au * sb * sl * db + r_au * cb * cl * dl) * au_m / day_s;
        let vz = (dr * sb + r_au * cb * db) * au_m / day_s;
        [vx, vy, vz]
    }

    /// 从二进制格式加载（零解析：头 + f64 小端数组）。格式见 doc/13-ephemeris-binary-format.md。
    pub fn from_binary(bytes: &[u8]) -> Result<Vsop87, LoadError> {
        const MAGIC: &[u8; 4] = b"VSB1";
        const HEADER_LEN: usize = 4 + 4 + 4; // magic + version + num_blocks
        if bytes.len() < HEADER_LEN {
            return Err(LoadError::Io("VSOP87 binary too short (header)".into()));
        }
        if &bytes[0..4] != MAGIC {
            return Err(LoadError::Io("VSOP87 binary bad magic".into()));
        }
        let version = read_u32_le(bytes, 4)?;
        if version != 1 {
            return Err(LoadError::Io(format!("VSOP87 binary unsupported version {}", version)));
        }
        let num_blocks = read_u32_le(bytes, 8)? as usize;
        let mut blocks = Vec::with_capacity(num_blocks);
        let mut pos = 12_usize;
        for _ in 0..num_blocks {
            if pos + 8 > bytes.len() {
                return Err(LoadError::Io("VSOP87 binary block header truncated".into()));
            }
            let coords = bytes[pos];
            let alpha_t = bytes[pos + 1];
            pos += 4;
            let term_count = read_u32_le(bytes, pos)? as usize;
            pos += 4;
            let mut terms = Vec::with_capacity(term_count);
            for _ in 0..term_count {
                if pos + 24 > bytes.len() {
                    return Err(LoadError::Io("VSOP87 binary term truncated".into()));
                }
                let amp = read_f64_le(bytes, pos)?;
                let phase_rad = read_f64_le(bytes, pos + 8)?;
                let freq = read_f64_le(bytes, pos + 16)?;
                pos += 24;
                let amplitude = if coords == 3 {
                    Vsop87Amplitude::Length(Length::from_value(real(amp) * AU_METERS, LengthUnit::Meter))
                } else {
                    Vsop87Amplitude::Angle(PlaneAngle::from_rad(real(amp)))
                };
                terms.push(VsopTerm {
                    amplitude,
                    phase: PlaneAngle::from_rad(real(phase_rad)),
                    frequency: AngularRate::from_value(real(freq), AngularRateUnit::RadPerJulianMillennium),
                });
            }
            blocks.push(VsopBlock {
                coords,
                alpha_t,
                terms,
            });
        }
        if blocks.is_empty() {
            return Err(LoadError::Io("VSOP87 binary no blocks".into()));
        }
        Ok(Vsop87 { blocks })
    }

    /// 序列化为二进制格式（与 from_binary 对称，供构建脚本生成 .ear.bin）。
    pub fn to_binary(&self) -> Vec<u8> {
        const MAGIC: &[u8; 4] = b"VSB1";
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend_from_slice(&(self.blocks.len() as u32).to_le_bytes());
        for block in &self.blocks {
            out.push(block.coords);
            out.push(block.alpha_t);
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&(block.terms.len() as u32).to_le_bytes());
            for term in &block.terms {
                let amp_f64: f64 = match &term.amplitude {
                    Vsop87Amplitude::Angle(a) => a.rad().as_f64(),
                    Vsop87Amplitude::Length(l) => l.meters().as_f64() / AU_METERS.as_f64(),
                };
                out.extend_from_slice(&amp_f64.to_le_bytes());
                out.extend_from_slice(&term.phase.rad().as_f64().to_le_bytes());
                out.extend_from_slice(&term.frequency.rad_per_julian_millennium().as_f64().to_le_bytes());
            }
        }
        out
    }
}

fn read_u32_le(b: &[u8], i: usize) -> Result<u32, LoadError> {
    if i + 4 > b.len() {
        return Err(LoadError::Io("read u32 out of bounds".into()));
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&b[i..i + 4]);
    Ok(u32::from_le_bytes(arr))
}

fn read_f64_le(b: &[u8], i: usize) -> Result<f64, LoadError> {
    if i + 8 > b.len() {
        return Err(LoadError::Io("read f64 out of bounds".into()));
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&b[i..i + 8]);
    Ok(f64::from_le_bytes(arr))
}

/// 最小 3 块（L/B/R 各一常数项）地球 VSOP87，供测试用；供测试用。
pub fn minimal_earth_vsop() -> Vsop87 {
    Vsop87 {
        blocks: vec![
            VsopBlock {
                coords: 1,
                alpha_t: 0,
                terms: vec![VsopTerm {
                    amplitude: Vsop87Amplitude::Angle(PlaneAngle::from_rad(real(1))),
                    phase: PlaneAngle::from_rad(real(0)),
                    frequency: AngularRate::from_value(real(0), AngularRateUnit::RadPerJulianMillennium),
                }],
            },
            VsopBlock {
                coords: 2,
                alpha_t: 0,
                terms: vec![VsopTerm {
                    amplitude: Vsop87Amplitude::Angle(PlaneAngle::from_rad(real(0))),
                    phase: PlaneAngle::from_rad(real(0)),
                    frequency: AngularRate::from_value(real(0), AngularRateUnit::RadPerJulianMillennium),
                }],
            },
            VsopBlock {
                coords: 3,
                alpha_t: 0,
                terms: vec![VsopTerm {
                    amplitude: Vsop87Amplitude::Length(Length::from_value(AU_METERS, LengthUnit::Meter)),
                    phase: PlaneAngle::from_rad(real(0)),
                    frequency: AngularRate::from_value(real(0), AngularRateUnit::RadPerJulianMillennium),
                }],
            },
        ],
    }
}

pub struct Vsop87Parse;

impl Vsop87Parse {
    fn parse_head(s: &str) -> Result<(u8, u8, usize), LoadError> {
        if s.len() < 67 {
            return Err(LoadError::Io(format!("Invalid block descriptor: {}", s.len())));
        }
        let coord_index = s.get(41..42).and_then(|c| c.trim().parse().ok())
            .ok_or_else(|| LoadError::Io("coordIndex".into()))?;
        let alpha_t = s.get(59..60).and_then(|c| c.trim().parse().ok())
            .ok_or_else(|| LoadError::Io("alphaT".into()))?;
        let terms_count = s.get(60..67).and_then(|c| c.trim().parse().ok())
            .ok_or_else(|| LoadError::Io("termsCount".into()))?;
        Ok((coord_index, alpha_t, terms_count))
    }

    fn parse_term(s: &str, coord_index: u8) -> Result<VsopTerm, LoadError> {
        if s.len() < 131 {
            return Err(LoadError::Io(format!("Term line too short: {}", s.len())));
        }
        let amp_f64 = s.get(79..97).and_then(|c| c.trim().parse::<f64>().ok())
            .ok_or_else(|| LoadError::Io("amplitude".into()))?;
        let phase = PlaneAngle::from_rad(real(
            s.get(97..111).and_then(|c| c.trim().parse::<f64>().ok())
                .ok_or_else(|| LoadError::Io("phase".into()))?,
        ));
        let frequency = AngularRate::from_value(
            real(s.get(111..131).and_then(|c| c.trim().parse::<f64>().ok())
                .ok_or_else(|| LoadError::Io("frequency".into()))?),
            AngularRateUnit::RadPerJulianMillennium,
        );
        let amplitude = if coord_index == 3 {
            Vsop87Amplitude::Length(Length::from_value(real(amp_f64) * AU_METERS, LengthUnit::Meter))
        } else {
            Vsop87Amplitude::Angle(PlaneAngle::from_rad(real(amp_f64)))
        };
        Ok(VsopTerm { amplitude, phase, frequency })
    }

    fn is_header(line: &str) -> bool {
        line.contains("VSOP87") && line.len() < 140
    }

    pub fn parse_from_lines(lines: &[String]) -> Result<Vsop87, LoadError> {
        let mut blocks = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() {
                i += 1;
                continue;
            }
            if Self::is_header(line) {
                let (coord_index, alpha_t, terms_count) = Self::parse_head(&lines[i])?;
                i += 1;
                let mut terms = Vec::with_capacity(terms_count);
                for _ in 0..terms_count {
                    if i >= lines.len() {
                        return Err(LoadError::Io("Unexpected end in block".into()));
                    }
                    terms.push(Self::parse_term(&lines[i], coord_index)?);
                    i += 1;
                }
                blocks.push(VsopBlock { coords: coord_index, alpha_t, terms });
            } else {
                i += 1;
            }
        }
        if blocks.is_empty() {
            return Err(LoadError::Io("No blocks parsed".into()));
        }
        Ok(Vsop87 { blocks })
    }

    pub fn parse(loader: &dyn DataLoader, path: &str) -> Result<Vsop87, LoadError> {
        let lines = loader.read_lines(path)?;
        Self::parse_from_lines(&lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::constant::J2000;
    use crate::math::real::{real, RealOps, ToReal};

    #[test]
    fn jd_to_t_j2000() {
        let JulianMillennia(t) = jd_to_t(J2000);
        assert!(t.abs() < real(0.0001));
    }

    fn term_angle(a: impl ToReal, p: impl ToReal, f: impl ToReal) -> VsopTerm {
        VsopTerm {
            amplitude: Vsop87Amplitude::Angle(PlaneAngle::from_rad(real(a))),
            phase: PlaneAngle::from_rad(real(p)),
            frequency: AngularRate::from_value(real(f), AngularRateUnit::RadPerJulianMillennium),
        }
    }

    fn term_length_au(au: impl ToReal, p: impl ToReal, f: impl ToReal) -> VsopTerm {
        VsopTerm {
            amplitude: Vsop87Amplitude::Length(Length::from_value(real(au) * AU_METERS, LengthUnit::Meter)),
            phase: PlaneAngle::from_rad(real(p)),
            frequency: AngularRate::from_value(real(f), AngularRateUnit::RadPerJulianMillennium),
        }
    }

    #[test]
    fn position_minimal_blocks() {
        let vsop = Vsop87 {
            blocks: vec![
                VsopBlock { coords: 1, alpha_t: 0, terms: vec![term_angle(real(1), real(0), real(0))] },
                VsopBlock { coords: 2, alpha_t: 0, terms: vec![term_angle(real(0.1), real(0), real(0))] },
                VsopBlock { coords: 3, alpha_t: 0, terms: vec![term_length_au(real(1), real(0), real(0))] },
            ],
        };
        let pos = vsop.position_jd(J2000);
        assert!(pos.L.rad().abs() > real(0) && pos.B.rad().abs() > real(0) && (pos.R.meters() / AU_METERS).abs() > real(0));
    }

    /// 最小 VSOP 太阳黄经测试
    #[test]
    fn sun_ecliptic_longitude_minimal_vsop() {
        use crate::astronomy::aspects::sun_ecliptic_longitude;
        use crate::astronomy::time::j2000_tt;
        let vsop = Vsop87 {
            blocks: vec![VsopBlock {
                coords: 1,
                alpha_t: 0,
                terms: vec![term_angle(real(1), real(0), real(0))],
            }],
        };
        let sun_l = sun_ecliptic_longitude(&vsop, j2000_tt());
        assert!(sun_l.rad().abs() > real(0));
    }

    #[test]
    fn binary_roundtrip() {
        let vsop = minimal_earth_vsop();
        let bin = vsop.to_binary();
        let vsop2 = Vsop87::from_binary(&bin).unwrap();
        assert_eq!(vsop2.blocks.len(), vsop.blocks.len());
        for (b1, b2) in vsop.blocks.iter().zip(vsop2.blocks.iter()) {
            assert_eq!(b1.coords, b2.coords);
            assert_eq!(b1.alpha_t, b2.alpha_t);
            assert_eq!(b1.terms.len(), b2.terms.len());
        }
    }

    #[test]
    fn parse_from_lines_real_format() {
        let mut header = [b' '; 67];
        let s = " VSOP87 VERSION B2    EARTH     VARIABLE 1 (LBR)       *T**0    623 TERMS    ";
        for (i, b) in s.bytes().enumerate().take(67) {
            header[i] = b;
        }
        header[41] = b'1';
        header[59] = b'0';
        header[60] = b'1';
        header[61] = b' ';
        header[62] = b' ';
        header[63] = b' ';
        header[64] = b' ';
        header[65] = b' ';
        header[66] = b' ';
        let header_line = std::str::from_utf8(&header).unwrap().to_string();
        let term_line = format!(
            "{:79}{:18.11}{:14.11}{:20.11}",
            " 2310    1  0  0  0  0  0  0  0  0  0  0  0  0  0.00000000000     ",
            1.75347045673_f64,
            1.75347045673,
            0.00000000000
        );
        let lines = vec![header_line, term_line];
        let vsop = Vsop87Parse::parse_from_lines(&lines).unwrap();
        assert_eq!(vsop.blocks.len(), 1);
        assert_eq!(vsop.blocks[0].coords, 1);
        assert_eq!(vsop.blocks[0].alpha_t, 0);
        assert_eq!(vsop.blocks[0].terms.len(), 1);
        match &vsop.blocks[0].terms[0].amplitude {
            Vsop87Amplitude::Angle(a) => assert!((a.rad() - real(1.75347045673)).abs() < real(1e-10)),
            _ => panic!("expected angle amplitude"),
        }
    }
}
