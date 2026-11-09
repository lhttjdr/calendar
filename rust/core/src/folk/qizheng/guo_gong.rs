//! 《欽天監校正授時過宮度圖》（《古今圖書集成·藝術典》卷五百六十九《張果星宗三》）全表转录。
//!
//! 图横列「一度」至「三十度」（第 45 页中栏一至十四度、下栏「授時過宮度」十五至三十度），竖列子至亥十二宫「命」，
//! 每格为该宫第几度所当之宿度（宿名只在换宿处出，余格只记度数；宿名与小字度数常合刻为一字形，如「斗廿三」）。
//! 一度在宫之黄经大端（宫尾），逐度递减，三十度在宫首。
//!
//! 底本：维基共享资源 `File:Gujin Tushu Jicheng, Volume 468 (1700-1725).djvu` 第 45 页原件（JB2 二值图 3600×6193，
//! 每格约 60 像素，存 `vendor/folk/commons/古今圖書集成/第468冊_djvu第45頁_原件.png`；同目录 `第468冊_djvu第45頁.jpg` 为 1920 像素缩略图）。十二宫 360 格皆据原件逐格读出，
//! 未有不可辨之格；读法略有疑者见 [`UNSURE`]。与推算之比较见 [`tui_suan`] 与 doc/folk/qizheng.md §4.3.2。

use super::{GONG_JIE_SHOUSHI, SHOUSHI_CHIDAO_XIU, SHOUSHI_HUANGDAO_XIU};
use crate::math::real::{real, Real, RealOps};

/// 十二宫（地支序 子…亥）三十格（一度……三十度）：「宿度」，宿用原图字
pub const ROWS: [&str; 12] = [
    // 子
    "危11 危10 危9 危8 危7 危6 危5 危4 危3 危2 危1 虛9 虛8 虛7 虛6 虛5 虛4 虛3 虛2 虛1 女11 女10 女9 女8 女7 女6 女5 女4 女3 女2",
    // 丑
    "女1 女0 牛6 牛5 牛4 牛3 牛2 牛1 牛0 斗23 斗22 斗21 斗20 斗19 斗18 斗17 斗16 斗15 斗14 斗13 斗12 斗11 斗10 斗9 斗8 斗7 斗6 斗5 斗4 斗3",
    // 寅
    "斗2 斗1 斗0 箕10 箕9 箕8 箕7 箕6 箕5 箕4 箕3 箕2 箕1 箕0 尾18 尾17 尾16 尾15 尾14 尾13 尾12 尾11 尾10 尾9 尾8 尾7 尾6 尾5 尾4 尾3",
    // 卯
    "尾2 尾1 心6 心5 心4 心3 心2 心1 房5 房4 房3 房2 房1 房0 氐16 氐15 氐14 氐13 氐12 氐11 氐10 氐9 氐8 氐7 氐6 氐5 氐4 氐3 氐2 氐1",
    // 辰
    "氐0 亢9 亢8 亢7 亢6 亢5 亢4 亢3 亢2 亢1 角12 角11 角10 角9 角8 角7 角6 角5 角4 角3 角2 角1 軫17 軫16 軫15 軫14 軫13 軫12 軫11 軫10",
    // 巳
    "軫9 軫8 軫7 軫6 軫5 軫4 軫3 軫2 軫1 翼19 翼18 翼17 翼16 翼15 翼14 翼13 翼12 翼11 翼10 翼9 翼8 翼7 翼6 翼5 翼4 翼3 翼2 翼1 張16 張15",
    // 午
    "張14 張13 張12 張11 張10 張9 張8 張7 張6 張5 張4 張3 張2 張1 星6 星5 星4 星3 星2 星1 柳13 柳12 柳11 柳10 柳9 柳8 柳7 柳6 柳5 柳4",
    // 未
    "柳3 柳2 柳1 柳0 鬼2 鬼1 鬼0 井30 井29 井28 井27 井26 井25 井24 井23 井22 井21 井20 井19 井18 井17 井16 井15 井14 井13 井12 井11 井10 井9 井8",
    // 申
    "井7 井6 井5 井4 井3 井2 井1 井0 參10 參9 參8 參7 參6 參5 參4 參3 參2 參1 觜0 畢16 畢15 畢14 畢13 畢12 畢11 畢10 畢9 畢8 畢7 畢6",
    // 酉
    "畢5 畢4 畢3 畢2 畢1 畢0 昴10 昴9 昴8 昴7 昴6 昴5 昴4 昴3 昴2 昴1 昴0 胃15 胃14 胃13 胃12 胃11 胃10 胃9 胃8 胃7 胃6 胃5 胃4 胃3",
    // 戌
    "胃2 胃1 婁12 婁11 婁10 婁9 婁8 婁7 婁6 婁5 婁4 婁3 婁2 婁1 奎16 奎15 奎14 奎13 奎12 奎11 奎10 奎9 奎8 奎7 奎6 奎5 奎4 奎3 奎2 奎1",
    // 亥
    "奎0 壁8 壁7 壁6 壁5 壁4 壁3 壁2 壁1 室17 室16 室15 室14 室13 室12 室11 室10 室9 室8 室7 室6 室5 室4 室3 室2 室1 危15 危14 危13 危12",
];

/// 读法略有疑之格：（宫，度列，说明）。皆宿名下小字，依同行相邻列逐度递减定之
pub const UNSURE: [(u8, u8, &str); 3] = [
    (1, 1, "女一或女二：女下小字只见一横，作一；子宫三十度亦作女二，作二则两宫重一度"),
    (9, 7, "昴十：昴下小字残，八度为九，依递减作十"),
    (0, 1, "危十一：危下小字「十」，其下一横与危字末笔相并，依二度为十作十一"),
];

/// 图中宿名（与 `SHOUSHI_*_XIU` 同序，角起）
const XIU: [&str; 28] = [
    "角", "亢", "氐", "房", "心", "尾", "箕", "斗", "牛", "女", "虛", "危", "室", "壁", "奎", "婁", "胃", "昴", "畢", "觜", "參", "井", "鬼",
    "柳", "星", "張", "翼", "軫",
];

/// `gong` 宫第 `col`（1—30）度所当之宿度（宿名，度）
pub fn guo_gong(gong: u8, col: u8) -> Option<(&'static str, u32)> {
    let row = ROWS.get(gong as usize)?;
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

/// 宿度表
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XiuBiao {
    /// 授时黄道宿度（《元史》卷五十四「黃道宿度」）
    HuangDao,
    /// 授时赤道宿度（《元史》卷五十四「赤道宿度」）
    ChiDao,
}

/// 推算之图：宫界取授时黄道十二次（[`GONG_JIE_SHOUSHI`]），宫宽三十等分，第 `col` 格之下沿（距宫首 (30 − col)／30 宫宽）
/// 所在宿度取整。第三十格即宫首宿度之整数，与图同。返回（宿名，度）
pub fn tui_suan(gong: u8, col: u8, biao: XiuBiao) -> (&'static str, u32) {
    let w = match biao {
        XiuBiao::HuangDao => &SHOUSHI_HUANGDAO_XIU,
        XiuBiao::ChiDao => &SHOUSHI_CHIDAO_XIU,
    };
    let mut acc = [real(0.0); 29];
    for i in 0..28 {
        acc[i + 1] = acc[i] + real(w[i]);
    }
    let tot = acc[28];
    let wrap = |x: Real| x - tot * (x / tot).floor();
    let pos = |z: usize| {
        let (x, d) = GONG_JIE_SHOUSHI[z % 12];
        acc[x] + real(d)
    };
    let s = pos(gong as usize);
    let e = pos(gong as usize + 11); // 逆支下一宫（亥之于子）宫首即本宫宫尾
    let width = wrap(e - s);
    let p = wrap(s + width * real(30.0 - col as f64) / real(30.0));
    let i = (0..28).rev().find(|&i| acc[i] <= p).unwrap_or(0);
    (XIU[i], (p - acc[i]).to_i32_floor().max(0) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_monotone() {
        for g in 0..12u8 {
            assert_eq!(ROWS[g as usize].split(' ').count(), 30, "{g}");
            // 同宿之内逐列递减一度
            for c in 1..30u8 {
                let (a, b) = (guo_gong(g, c).unwrap(), guo_gong(g, c + 1).unwrap());
                if a.0 == b.0 {
                    assert_eq!(a.1, b.1 + 1, "{g} {c}");
                }
            }
            // 跨宫首尾相接：本宫三十度（宫首）之前一度为顺支下一宫之一度（同宿时差一度）
            let (a, b) = (guo_gong(g, 30).unwrap(), guo_gong((g + 1) % 12, 1).unwrap());
            if a.0 == b.0 {
                assert_eq!(a.1, b.1 + 1, "{g}");
            }
        }
        // 全表宿之度数（各宿所占格数）之和为 360
        let n: usize = XIU
            .iter()
            .map(|x| (0..12u8).flat_map(|g| (1..=30u8).map(move |c| (g, c))).filter(|&(g, c)| guo_gong(g, c).unwrap().0 == *x).count())
            .sum();
        assert_eq!(n, 360);
    }

    /// 图之宫首（三十度）即授时黄道十二次宿度之整数，十二宫十一合（午宫柳四，授时柳 3.87）
    #[test]
    fn gong_shou_vs_shoushi() {
        let mut hit = 0;
        for g in 0..12u8 {
            let (x, d) = GONG_JIE_SHOUSHI[g as usize];
            let book = guo_gong(g, 30).unwrap();
            if book == (XIU[x], d.floor() as u32) {
                hit += 1;
            } else {
                assert_eq!((g, book), (6, ("柳", 4)));
            }
            // 一度在宫尾：逆支下一宫宫首之前一整度（未宫因午宫首作柳四而作柳三，余皆合）
        }
        assert_eq!(hit, 11);
    }

    /// 逐格比对推算（[`tui_suan`]）：黄道宿度 235／360 格合，赤道宿度 254／360 格合；不合者几乎皆为换宿处前后错一格
    #[test]
    fn compare_with_tui_suan() {
        for (biao, want, per) in [
            (XiuBiao::HuangDao, 235, [28, 27, 7, 28, 9, 6, 20, 10, 29, 30, 17, 24]),
            (XiuBiao::ChiDao, 254, [29, 19, 30, 25, 9, 4, 15, 20, 29, 18, 28, 28]),
        ] {
            let mut tot = 0;
            for g in 0..12u8 {
                let row: Vec<_> = (1..=30u8).map(|c| tui_suan(g, c, biao)).collect();
                let n = (1..=30u8).filter(|&c| guo_gong(g, c).unwrap() == row[c as usize - 1]).count();
                eprintln!("{biao:?} 宫{g} 合 {n}：{}", row.iter().map(|(x, d)| format!("{x}{d}")).collect::<Vec<_>>().join(" "));
                assert_eq!(n, per[g as usize], "{biao:?} {g}");
                tot += n;
            }
            assert_eq!(tot, want);
        }
    }

    /// 各宿在图中所占之度（最大、最小度数）与授时赤道、黄道宿度之整数比较：图中斗二十三、婁十二、畢十六、參十、虛九、牛六合黄道，
    /// 箕十、室十七、壁八、奎十六、軫十七合赤道，尾十八、昴十、井三十、張十六、翼十九皆不合（余十二宿二表整数同，皆合）
    #[test]
    fn xiu_du_range() {
        let range = |x: &str| {
            let v: Vec<u32> = (0..12u8).flat_map(|g| (1..=30u8).map(move |c| (g, c))).filter_map(|(g, c)| guo_gong(g, c).filter(|y| y.0 == x).map(|y| y.1)).collect();
            (*v.iter().min().unwrap(), *v.iter().max().unwrap())
        };
        let (mut huang, mut chi) = (vec![], vec![]);
        for (i, x) in XIU.iter().enumerate() {
            let (_, mx) = range(x);
            if mx == SHOUSHI_HUANGDAO_XIU[i].floor() as u32 {
                huang.push(*x);
            }
            if mx == SHOUSHI_CHIDAO_XIU[i].floor() as u32 {
                chi.push(*x);
            }
            eprintln!("{x}：图 {:?}，黄道 {}，赤道 {}", range(x), SHOUSHI_HUANGDAO_XIU[i], SHOUSHI_CHIDAO_XIU[i]);
        }
        eprintln!("合黄道 {huang:?}；合赤道 {chi:?}");
        for x in ["斗", "婁", "畢", "參", "虛", "牛"] {
            assert!(huang.contains(&x) && !chi.contains(&x), "{x}");
        }
        for x in ["箕", "室", "壁", "奎", "軫"] {
            assert!(chi.contains(&x) && !huang.contains(&x), "{x}");
        }
        for x in ["尾", "昴", "井", "張", "翼"] {
            assert!(!chi.contains(&x) && !huang.contains(&x), "{x}");
        }
    }

    /// 《張果星宗·定限度法》（集成卷五百六十七）午宫诸例与《星學大成》卷三翼十度例，按图之度列
    #[test]
    fn ding_xian_du_fa_by_table() {
        // 星五 → 十五歲：合
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "星", 5).unwrap()), 15);
        // 柳三四五 → 十一歲：柳四、柳五合；柳三在未宫一度（午宫三十度为柳四）
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "柳", 4).unwrap()), 11);
        assert_eq!(tong_xian_by_col(guo_gong_col(6, "柳", 5).unwrap()), 11);
        assert_eq!(guo_gong_col(6, "柳", 3), None);
        assert_eq!(guo_gong_col(7, "柳", 3), Some(1));
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
        // 图同列为星六、柳五，书皆取次一列（星五、柳四），二例一致（解见文档：书按连续同位推，午宫首柳 3.87 带大分）
        let c = guo_gong_col(0, "虛", 6).unwrap();
        assert_eq!((c, guo_gong(6, c)), (15, Some(("星", 6))));
        assert_eq!(guo_gong(6, c + 1), Some(("星", 5)));
        let c = guo_gong_col(0, "女", 3).unwrap();
        assert_eq!((c, guo_gong(6, c)), (29, Some(("柳", 5))));
        assert_eq!(guo_gong(6, c + 1), Some(("柳", 4)));
    }
}
