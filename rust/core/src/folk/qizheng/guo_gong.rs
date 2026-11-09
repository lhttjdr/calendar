//! 《欽天監校正授時過宮度圖》（《古今圖書集成·藝術典》卷五百六十九《張果星宗三》书影，
//! `vendor/folk/commons/古今圖書集成/第468冊_djvu第45頁.jpg`）部分转录。
//!
//! 图横列「一度」至「三十度」（第 45 页中栏一至十四度、下栏「授時過宮度」十五至三十度），竖列子至亥十二宫「命」，
//! 每格为该宫第几度所当之宿度（宿名只在换宿处出，余格只记度数；宿名与小字度数常合刻为一字形，如「斗廿三」）。
//! 一度在宫之黄经大端（宫尾），逐度递减，三十度在宫首；偶有跳过一度（如危十一、十后，虛九紧接危一，危初不出）。
//!
//! 书影宽 1920 像素，每格约 30 像素，小字度数（一二三、十几）多处难辨。本次只转录丑、巳、午三宫（《張果星宗·定限度法》
//! 与《星學大成》卷三童限例所需）及子宫（安命度法例所需），逐格据图读出，读法未确者见 [`UNSURE`]；余八宫未转录。

/// 已转录之宫（地支序）与三十格（一度……三十度）：「宿度」，宿用原图字
pub const ROWS: [(u8, &str); 4] = [
    (
        0, // 子
        "危11 危10 危9 危8 危7 危6 危5 危4 危3 危2 危1 虛9 虛8 虛7 虛6 虛5 虛4 虛3 虛2 虛1 女11 女10 女9 女8 女7 女6 女5 女4 女3 女2",
    ),
    (
        1, // 丑
        "女1 女0 牛6 牛5 牛4 牛3 牛2 牛1 牛0 斗23 斗22 斗21 斗20 斗19 斗18 斗17 斗16 斗15 斗14 斗13 斗12 斗11 斗10 斗9 斗8 斗7 斗6 斗5 斗4 斗3",
    ),
    (
        5, // 巳
        "軫9 軫8 軫7 軫6 軫5 軫4 軫3 軫2 軫1 翼19 翼18 翼17 翼16 翼15 翼14 翼13 翼12 翼11 翼10 翼9 翼8 翼7 翼6 翼5 翼4 翼3 翼2 翼1 張16 張15",
    ),
    (
        6, // 午
        "張14 張13 張12 張11 張10 張9 張8 張7 張6 張5 張4 張3 張2 張1 星6 星5 星4 星3 星2 星1 柳13 柳12 柳11 柳10 柳9 柳8 柳7 柳6 柳5 柳4",
    ),
];

/// 读法未确之格：（宫，度列，说明）
pub const UNSURE: [(u8, u8, &str); 6] = [
    (0, 1, "危十一：宿名下小字；二度为十，依递减作十一（亥宫首危十二，相接）"),
    (0, 12, "虛九：宿名下小字；十三度为八，依递减作九（危初不出）"),
    (0, 21, "女十一：宿名下小字；二十二度为十，依递减作十一（虛初不出）"),
    (1, 1, "女一或女二：小字二画三画难分；二度为女初，子宫三十度为女二，作一"),
    (5, 10, "翼十九：宿名下小字「九十」；十一度为十八，依递减作十九（授时翼宽 18.75，本不应有十九度）"),
    (6, 15, "星六：宿名下小字难辨；十六度为五，依递减作六（張初不出）"),
];

/// `gong` 宫第 `col`（1—30）度所当之宿度（宿名，度）；未转录之宫为 None
pub fn guo_gong(gong: u8, col: u8) -> Option<(&'static str, u32)> {
    let (_, row) = ROWS.iter().find(|(g, _)| *g == gong)?;
    let cell = row.split(' ').nth((col as usize).checked_sub(1)?)?;
    let i = cell.char_indices().nth(1)?.0;
    Some((&cell[..i], cell[i..].parse().ok()?))
}

/// 宿度在 `gong` 宫之度列（1—30）；同一宿度出现多格时取最小之列
pub fn guo_gong_col(gong: u8, xiu: &str, du: u32) -> Option<u8> {
    (1..=30).find(|&c| guo_gong(gong, c) == Some((xiu, du)))
}

/// 依图之度列定童限：「以命度在某宮第幾行，則知某歲行限也」（集成卷五百六十七《定限度法》），三度一行，
/// 自宫首（三十度）起第一行十一岁，即 10 + ⌈(31 − 列) ／ 3⌉
pub fn tong_xian_by_col(col: u8) -> u32 {
    10 + (31 - col as u32).div_ceil(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_monotone() {
        for (g, row) in ROWS {
            assert_eq!(row.split(' ').count(), 30, "{g}");
            // 同宿之内逐列递减一度
            for c in 1..30u8 {
                let (a, b) = (guo_gong(g, c).unwrap(), guo_gong(g, c + 1).unwrap());
                if a.0 == b.0 {
                    assert_eq!(a.1, b.1 + 1, "{g} {c}");
                }
            }
        }
        // 三十度（宫首）与《張果星宗》度數所在「女二子」「斗四丑」「張十五巳」「柳三午」、《星學大成》宫界歌「女一」「斗三」「張十四」「柳三」相近
        assert_eq!(guo_gong(1, 30), Some(("斗", 3)));
        assert_eq!(guo_gong(0, 30), Some(("女", 2)));
        assert_eq!(guo_gong(5, 30), Some(("張", 15)));
        assert_eq!(guo_gong(6, 30), Some(("柳", 4)));
    }

    /// 《張果星宗·定限度法》（集成卷五百六十七）午宫诸例与《星學大成》卷三翼十度例，按图之度列
    #[test]
    fn ding_xian_du_fa_by_table() {
        // 星五 → 十五歲：合
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "星", 5).unwrap()), 15);
        // 柳三四五 → 十一歲：柳四、柳五合；柳三不在午宫图内（三十度为柳四）
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "柳", 4).unwrap()), 11);
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "柳", 5).unwrap()), 11);
        assert_eq!(guo_gong_col(6, "柳", 3), None);
        // 張十一、十二、十三 → 二十歲：十二、十三合，十一得十九（差一年）
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "張", 12).unwrap()), 20);
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "張", 13).unwrap()), 20);
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "張", 11).unwrap()), 19);
        // 大成卷三：太陽巳宫翼十度，命丑，「照量天尺本行下推，即十三嵗出童限」：图翼十在第十九列，同列丑宫斗十四，童限十四（书十三，差一年）
        let c = guo_gong_col(5, "翼", 10).unwrap();
        assert_eq!(c, 19);
        assert_eq!(guo_gong(1, c), Some(("斗", 14)));
        assert_eq!(tong_xian_by_col(c), 14);
        // 集成卷五百六十七安命度法「太陽躔子宮虛六度，對著午宮星五度」「日躔女三度，對著午宮柳四度」：
        // 图同列为星六、柳五，书皆取次一列（星五、柳四），二例一致
        let c = guo_gong_col(0, "虛", 6).unwrap();
        assert_eq!((c, guo_gong(6, c)), (15, Some(("星", 6))));
        assert_eq!(guo_gong(6, c + 1), Some(("星", 5)));
        let c = guo_gong_col(0, "女", 3).unwrap();
        assert_eq!((c, guo_gong(6, c)), (29, Some(("柳", 5))));
        assert_eq!(guo_gong(6, c + 1), Some(("柳", 4)));
    }
}
