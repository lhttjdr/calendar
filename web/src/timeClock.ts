/**
 * 时间流转（排盘页「实时」）：太阳时角。与方位（罗盘，CompassBar）合成时空四维的动态盘：
 * 方位由陀螺仪指南针转整盘；随天而行之天盘（七政、六壬）按太阳时角随时刻连续转，循规则步进者（奇门、太乙）逢时辰交界转过。
 *
 * 太阳位置用低精度公式（《天文年历》附低精度太阳位置：平黄经 L、平近点角 g、λ = L + 1.915° sin g + 0.020° sin 2g），
 * 精度约 0.01°，只供转盘之显示；排盘的计算仍以 Rust 历表为准。时角 H = 格林尼治平恒星时 + 东经 − 太阳赤经（即地方视太阳时角）。
 * 盘上方位：南上北下，太阳方位角取 180° + H（真午居午、卯初居东、子夜居子），即时辰与地支方位之对应。
 */

const D2R = Math.PI / 180
/** 当前时刻之 UT 儒略日 */
export const nowJdUt = () => Date.now() / 86400000 + 2440587.5

/** 太阳低精度视黄经、赤经（度） */
export function sunLonRa(jdUt: number) {
  const n = jdUt - 2451545.0
  const L = 280.46 + 0.9856474 * n
  const g = (357.528 + 0.9856003 * n) * D2R
  const lam = (L + 1.915 * Math.sin(g) + 0.02 * Math.sin(2 * g)) * D2R
  const eps = (23.439 - 0.0000004 * n) * D2R
  const ra = Math.atan2(Math.cos(eps) * Math.sin(lam), Math.cos(lam)) / D2R
  return { lon: ((lam / D2R) % 360 + 360) % 360, ra }
}

/** 太阳地方时角（度，−180—180；正午 0，上午为负） */
export function sunHourAngle(jdUt: number, lonDeg: number) {
  const gmst = 280.46061837 + 360.98564736629 * (jdUt - 2451545.0)
  const h = gmst + lonDeg - sunLonRa(jdUt).ra
  return ((((h + 180) % 360) + 360) % 360) - 180
}

/** 太阳在南上北下盘上的方位角（度，罗盘方位：北 0、东 90） */
export const sunBoardAz = (jdUt: number, lonDeg: number) => (180 + sunHourAngle(jdUt, lonDeg) + 360) % 360
