//! 六爻纳甲筮法：京房八宫、纳甲纳支、六亲、世应、六神、伏神、旬空、卦身、动变；起卦（掷钱、梅花易数年月日时与数字）。
//!
//! 只排卦，不断吉凶。规则逐条依原典（见 doc/folk/liuyao.md）：
//! - 八宫世应：《京氏易傳》；《增刪卜易》卷一「八宮圖第三」「世應章第六」。
//! - 纳甲：《京氏易傳》卷下「分天地乾坤之象，益之以甲乙壬癸，震巽之象配庚辛，坎離之象配戊己，艮兌之象配丙丁」；
//!   纳支：《增刪卜易》卷一「渾天甲子章第四」（「乾在內：子水寅木辰土，乾在外：午火申金戌土」等）。
//! - 六亲：《增刪卜易》卷一「六親歌章第五」，以本宫五行为我；变爻六亲「仍照正卦而推」（同书动变章）。
//! - 六神：《增刪卜易》卷一「六神章第十八」表（甲乙日初爻青龙……壬癸日初爻玄武，自初而上青龙、朱雀、勾陈、螣蛇、白虎、玄武）。
//! - 伏神：《火珠林》「本宫之六亲在飞象之下，为之亲王，为之伏神」——本卦所缺之六亲，取本宫首卦同位之爻为伏神。
//! - 卦身：《火珠林》「阳世则从子月起，阴世还当午月生」；《增刪卜易》「古以身爲卦身……予試之不驗」，不用（选项）。
//!
//! 卦以六位表示：`u8` 低 6 位，第 0 位为初爻，阳为 1。

/// 十二支
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
/// 十干
pub const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 五行：0 木 1 火 2 土 3 金 4 水
pub const WU_XING: [&str; 5] = ["木", "火", "土", "金", "水"];
/// 六亲
pub const LIU_QIN: [&str; 5] = ["兄弟", "子孫", "妻財", "官鬼", "父母"];
/// 六神（自初爻起之序）
pub const LIU_SHEN: [&str; 6] = ["青龍", "朱雀", "勾陳", "螣蛇", "白虎", "玄武"];

/// 八卦（三爻，第 0 位为初爻）：名、卦画、先天数（梅花易数卷一「周易卦數」）
pub const BA_GUA: [(&str, u8, u8); 8] = [
    ("乾", 0b111, 1),
    ("兌", 0b011, 2),
    ("離", 0b101, 3),
    ("震", 0b001, 4),
    ("巽", 0b110, 5),
    ("坎", 0b010, 6),
    ("艮", 0b100, 7),
    ("坤", 0b000, 8),
];

fn tri_index(t: u8) -> usize {
    BA_GUA.iter().position(|(_, b, _)| *b == t & 7).unwrap()
}

/// 支之五行
pub fn zhi_wx(z: u8) -> u8 {
    [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4][z as usize % 12]
}

/// 宫（八纯卦）之五行：乾兌金、坎水、艮坤土、震巽木、離火（《增刪卜易》六親歌、《梅花易數》八宮所屬五行）
pub fn gong_wx(tri: u8) -> u8 {
    match BA_GUA[tri_index(tri)].0 {
        "乾" | "兌" => 3,
        "坎" => 4,
        "艮" | "坤" => 2,
        "震" | "巽" => 0,
        _ => 1,
    }
}

/// 纳甲：三爻卦在内、在外所纳之干（《京氏易傳》）
pub fn na_gan(tri: u8, outer: bool) -> u8 {
    match BA_GUA[tri_index(tri)].0 {
        "乾" => if outer { 8 } else { 0 }, // 内甲外壬
        "坤" => if outer { 9 } else { 1 }, // 内乙外癸
        "震" => 6,
        "巽" => 7,
        "坎" => 4,
        "離" => 5,
        "艮" => 2,
        _ => 3, // 兌丁
    }
}

/// 纳支：三爻卦在内、在外三爻之支（自下而上），《增刪卜易》渾天甲子章
pub fn na_zhi(tri: u8, outer: bool) -> [u8; 3] {
    let (inner, outer_z): ([u8; 3], [u8; 3]) = match BA_GUA[tri_index(tri)].0 {
        "乾" => ([0, 2, 4], [6, 8, 10]),
        "坎" => ([2, 4, 6], [8, 10, 0]),
        "艮" => ([4, 6, 8], [10, 0, 2]),
        "震" => ([0, 2, 4], [6, 8, 10]),
        "巽" => ([1, 11, 9], [7, 5, 3]),
        "離" => ([3, 1, 11], [9, 7, 5]),
        "坤" => ([7, 5, 3], [1, 11, 9]),
        _ => ([5, 3, 1], [11, 9, 7]), // 兌
    };
    if outer { outer_z } else { inner }
}

/// 一卦在八宫中的位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GongWei {
    /// 本宫（八纯卦之三爻画）
    pub gong: u8,
    /// 宫中次序 0 本宫、1—5 一世至五世、6 游魂、7 归魂
    pub order: u8,
    /// 世爻（1—6）、应爻
    pub shi: u8,
    pub ying: u8,
}

/// 八宫：本宫首卦依次变初、二、三、四、五爻得一世至五世；五世复变四爻为游魂；游魂内卦复本宫为归魂（《京氏易傳》）
fn gong_seq(tri: u8) -> [u8; 8] {
    let pure = tri | (tri << 3);
    let mut s = [0u8; 8];
    s[0] = pure;
    let mut g = pure;
    for k in 0..5 {
        g ^= 1 << k;
        s[k + 1] = g;
    }
    let you = g ^ (1 << 3);
    s[6] = you;
    s[7] = (you & 0b111000) | tri;
    s
}

/// 卦之宫与世应。世：本宫六、一世至五世依次、游魂四、归魂三（《增刪卜易》世應章：「乾爲天世在六……火地晉世在四，火天大有世退在三」）；
/// 应隔世两位。
pub fn gong_wei(gua: u8) -> GongWei {
    for &(_, tri, _) in BA_GUA.iter() {
        let s = gong_seq(tri);
        if let Some(o) = s.iter().position(|&g| g == gua & 63) {
            let shi = [6, 1, 2, 3, 4, 5, 4, 3][o];
            let ying = if shi > 3 { shi - 3 } else { shi + 3 };
            return GongWei { gong: tri, order: o as u8, shi, ying };
        }
    }
    unreachable!()
}

/// 一爻之装卦
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Yao {
    /// 阳爻
    pub yang: bool,
    /// 动爻（老阳九、老阴六）
    pub dong: bool,
    /// 纳干、纳支、五行
    pub gan: u8,
    pub zhi: u8,
    pub wx: u8,
    /// 六亲序（[`LIU_QIN`]）
    pub qin: u8,
    /// 六神序（[`LIU_SHEN`]）；未给日干时为 None
    pub shen: Option<u8>,
    /// 伏神：(六亲序, 干, 支)
    pub fu: Option<(u8, u8, u8)>,
    /// 变爻（动爻所化）：(干, 支, 五行, 六亲序——仍以本卦之宫论)
    pub bian: Option<(u8, u8, u8, u8)>,
}

/// 卦身之法
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GuaShenRule {
    /// 《火珠林》：阳世从子月起、阴世从午月起，自初爻数至世爻（缺省）
    #[default]
    HuoZhuLin,
    /// 不用卦身（《增刪卜易》「予試之不驗」）
    None,
}

/// 装卦选项
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LiuyaoOptions {
    pub gua_shen: GuaShenRule,
}

/// 一卦之装排
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paigua {
    pub ben: u8,
    /// 之卦（无动爻则同本卦）
    pub zhi: u8,
    pub ben_name: &'static str,
    pub zhi_name: &'static str,
    pub wei: GongWei,
    /// 宫名、宫五行
    pub gong_name: &'static str,
    pub gong_wx: u8,
    /// 六爻（初至上）
    pub yao: [Yao; 6],
    /// 旬空二支（给日干支时）
    pub xun_kong: Option<[u8; 2]>,
    /// 卦身之支（按 [`GuaShenRule`]）
    pub gua_shen: Option<u8>,
    /// 月建、日辰支（调用方所给，月建以交节为界）
    pub yue_jian: Option<u8>,
    pub ri_chen: Option<(u8, u8)>,
}

/// 六十四卦名（通行本），由经文表取
pub fn gua_name(g: u8) -> &'static str {
    super::zhouyi::text::GUA.iter().find(|r| r.2 == g & 63).map(|r| r.1).unwrap()
}

fn qin_of(gong_wx: u8, wx: u8) -> u8 {
    // 同我兄弟、我生子孙、我克妻财、克我官鬼、生我父母（木0 火1 土2 金3 水4，相生 +1，相克 +2）
    ((wx + 5 - gong_wx) % 5) as u8
}

fn zhuang(g: u8, k: usize) -> (u8, u8, u8) {
    let outer = k >= 3;
    let tri = if outer { (g >> 3) & 7 } else { g & 7 };
    let gan = na_gan(tri, outer);
    let zhi = na_zhi(tri, outer)[k % 3];
    (gan, zhi, zhi_wx(zhi))
}

/// 装卦。`values`：初至上六爻之数（6 老阴、7 少阳、8 少阴、9 老阳）；`yue_jian` 月建支（以节换月）；`ri` 日干支序（0 甲子）。
pub fn pai_gua(values: [u8; 6], yue_jian: Option<u8>, ri: Option<u8>, o: LiuyaoOptions) -> Paigua {
    let mut ben = 0u8;
    let mut zhi = 0u8;
    for (k, &v) in values.iter().enumerate() {
        let yang = v == 7 || v == 9;
        let dong = v == 6 || v == 9;
        if yang {
            ben |= 1 << k;
        }
        if yang != dong {
            zhi |= 1 << k;
        }
    }
    let wei = gong_wei(ben);
    let gwx = gong_wx(wei.gong);
    let pure = wei.gong | (wei.gong << 3);
    let shen0 = ri.map(|r| match r % 10 {
        0 | 1 => 0,
        2 | 3 => 1,
        4 => 2,
        5 => 3,
        6 | 7 => 4,
        _ => 5,
    });
    let qins: Vec<u8> = (0..6).map(|k| qin_of(gwx, zhuang(ben, k).2)).collect();
    let yao: Vec<Yao> = (0..6)
        .map(|k| {
            let (gan, z, wx) = zhuang(ben, k);
            let qin = qin_of(gwx, wx);
            let dong = values[k] == 6 || values[k] == 9;
            // 伏神：本卦所缺六亲，取本宫首卦同位之爻
            let (pg, pz, pw) = zhuang(pure, k);
            let pq = qin_of(gwx, pw);
            let fu = (!qins.contains(&pq) && pure != ben).then_some((pq, pg, pz));
            let bian = dong.then(|| {
                let (bg, bz, bw) = zhuang(zhi, k);
                (bg, bz, bw, qin_of(gwx, bw))
            });
            Yao { yang: ben >> k & 1 == 1, dong, gan, zhi: z, wx, qin, shen: shen0.map(|s| ((s + k) % 6) as u8), fu, bian }
        })
        .collect();
    let xun_kong = ri.map(|r| {
        let xs = (r / 10) as u8; // 旬首序：0 甲子旬 … 5 甲寅旬
        let a = (10 + 12 - 2 * xs) % 12; // 甲子旬空戌亥
        [a, (a + 1) % 12]
    });
    let gua_shen = match o.gua_shen {
        GuaShenRule::None => None,
        GuaShenRule::HuoZhuLin => {
            let shi_yang = ben >> (wei.shi - 1) & 1 == 1;
            Some(((if shi_yang { 0 } else { 6 }) + wei.shi - 1) % 12)
        }
    };
    Paigua {
        ben,
        zhi,
        ben_name: gua_name(ben),
        zhi_name: gua_name(zhi),
        gong_name: BA_GUA[tri_index(wei.gong)].0,
        gong_wx: gwx,
        wei,
        yao: yao.try_into().unwrap(),
        xun_kong,
        gua_shen,
        yue_jian,
        ri_chen: ri.map(|r| (r % 10, r % 12)),
    }
}

/// 掷钱：每爻三钱，背（字）为阳三、面为阴二（或反之，各家约定不一，调用方径给 6—9 之数即可）；此处由三钱阳面个数得爻数：
/// 三阳 9（老阳）、两阳 8（少阴）、一阳 7（少阳）、零阳 6（老阴）——「單拆重交」之法（《火珠林》以钱代蓍之通法，详 doc）。
pub fn qian_to_value(yang_faces: u8) -> u8 {
    match yang_faces {
        3 => 9,
        2 => 8,
        1 => 7,
        _ => 6,
    }
}

/// 梅花易数「年月日時起例」（卷一）：年支数（子一……亥十二）＋农历月数＋农历日数 → 以八除得上卦；
/// 加时支数（子一……亥十二）→ 以八除得下卦；总数以六除得动爻。余零作八、作六。返回 (上卦, 下卦, 动爻 1—6)。
pub fn meihua_time(year_zhi: u8, lunar_month: u8, lunar_day: u8, hour_zhi: u8) -> (u8, u8, u8) {
    let a = (year_zhi as u32 % 12 + 1) + lunar_month as u32 + lunar_day as u32;
    let b = a + (hour_zhi as u32 % 12 + 1);
    meihua_numbers(a, b, b)
}

/// 梅花易数以数起卦：上卦之数、下卦之数各以八除（余零为坤八），动爻以 `yao_sum` 除六（余零为六）（卷一「卦以八除」「爻以六除」）。
/// 物数、声音等占：上卦数为所见之数，下卦为该数加时数，动爻以两数并（卷一「物數占例」：「卦數並時數總除六取動爻」）。
pub fn meihua_numbers(upper_n: u32, lower_n: u32, yao_sum: u32) -> (u8, u8, u8) {
    let r8 = |n: u32| -> u8 {
        let r = (n % 8) as u8;
        if r == 0 { 8 } else { r }
    };
    let up = BA_GUA.iter().find(|x| x.2 == r8(upper_n)).unwrap().1;
    let lo = BA_GUA.iter().find(|x| x.2 == r8(lower_n)).unwrap().1;
    let y = (yao_sum % 6) as u8;
    (up, lo, if y == 0 { 6 } else { y })
}

/// 梅花之卦转为六爻之数（动爻作老，余作少），以便装卦或取辞
pub fn meihua_values(upper: u8, lower: u8, dong: u8) -> [u8; 6] {
    let g = lower | (upper << 3);
    core::array::from_fn(|k| {
        let yang = g >> k & 1 == 1;
        let d = k + 1 == dong as usize;
        match (yang, d) {
            (true, true) => 9,
            (true, false) => 7,
            (false, true) => 6,
            (false, false) => 8,
        }
    })
}

/// 互卦（梅花卷一「互卦起例」）：去初、上爻，以二三四爻为下卦、三四五爻为上卦
pub fn hu_gua(g: u8) -> u8 {
    ((g >> 1) & 7) | (((g >> 2) & 7) << 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(name: &str) -> u8 {
        super::super::zhouyi::text::GUA.iter().find(|r| r.1 == name).unwrap().2
    }

    #[test]
    fn eight_palaces() {
        // 《增刪卜易》世應章：乾宫八卦之世
        let seq = ["乾", "姤", "遯", "否", "觀", "剝", "晉", "大有"];
        let shi = [6, 1, 2, 3, 4, 5, 4, 3];
        for (i, n) in seq.iter().enumerate() {
            let w = gong_wei(g(n));
            assert_eq!((BA_GUA[tri_index(w.gong)].0, w.order as usize, w.shi), ("乾", i, shi[i]), "{n}");
        }
        // 六十四卦恰分属八宫各八
        let mut cnt = [0; 8];
        for x in 0..64u8 {
            cnt[tri_index(gong_wei(x).gong)] += 1;
        }
        assert_eq!(cnt, [8; 8]);
        // 六冲卦：八纯与无妄、大壮（増刪卜易八宮圖第三）
        let chong = |x: u8| (0..3).all(|k| {
            let a = zhuang(x, k).1;
            let b = zhuang(x, k + 3).1;
            (a + 6) % 12 == b
        });
        let mut liu_chong: Vec<&str> = (0..64u8).filter(|&x| chong(x)).map(gua_name).collect();
        liu_chong.sort();
        let mut want = vec!["乾", "坎", "艮", "震", "巽", "離", "坤", "兌", "无妄", "大壯"];
        want.sort();
        assert_eq!(liu_chong, want);
    }

    #[test]
    fn qian_wei_tian_example() {
        // 《增刪卜易》八宮圖第三、六親歌章：乾爲天 父母戌土世、兄弟申金、官鬼午火、父母辰土應、妻財寅木、子孫子水
        let p = pai_gua([7; 6], None, None, LiuyaoOptions::default());
        let line = |k: usize| (LIU_QIN[p.yao[k].qin as usize], ZHI[p.yao[k].zhi as usize], WU_XING[p.yao[k].wx as usize]);
        assert_eq!(line(5), ("父母", "戌", "土"));
        assert_eq!(line(4), ("兄弟", "申", "金"));
        assert_eq!(line(3), ("官鬼", "午", "火"));
        assert_eq!(line(2), ("父母", "辰", "土"));
        assert_eq!(line(1), ("妻財", "寅", "木"));
        assert_eq!(line(0), ("子孫", "子", "水"));
        assert_eq!((p.wei.shi, p.wei.ying), (6, 3));
        // 纳甲：乾内甲外壬
        assert_eq!((GAN[p.yao[0].gan as usize], GAN[p.yao[5].gan as usize]), ("甲", "壬"));
    }

    #[test]
    fn gou_na_zhi_example() {
        // 渾天甲子章：天風姤自下而上 丑土 亥水 酉金 午火 申金 戌土
        let p = pai_gua([8, 7, 7, 7, 7, 7], None, None, LiuyaoOptions::default());
        let z: Vec<&str> = p.yao.iter().map(|y| ZHI[y.zhi as usize]).collect();
        assert_eq!(z, ["丑", "亥", "酉", "午", "申", "戌"]);
    }

    #[test]
    fn bo_zhi_guan_example() {
        // 六神章：戊子日占生產，得山地剝變風地觀（五爻子孫子水動，化官鬼巳火）
        // 書列：朱雀 妻財寅木 / 靑龍 子孫子水ㄨ世 / 元武 父母戌土 / 白虎 妻財卯木 / 勾陳 官鬼巳火應 / 滕蛇 父母未土
        let ri = 24; // 戊子
        let p = pai_gua([8, 8, 8, 8, 6, 7], None, Some(ri), LiuyaoOptions::default());
        assert_eq!((p.ben_name, p.zhi_name), ("剝", "觀"));
        let q = |k: usize| (LIU_QIN[p.yao[k].qin as usize], ZHI[p.yao[k].zhi as usize]);
        assert_eq!([q(5), q(4), q(3), q(2), q(1), q(0)], [("妻財", "寅"), ("子孫", "子"), ("父母", "戌"), ("妻財", "卯"), ("官鬼", "巳"), ("父母", "未")]);
        assert_eq!((p.wei.shi, p.wei.ying), (5, 2));
        let b = p.yao[4].bian.unwrap();
        assert_eq!((ZHI[b.1 as usize], LIU_QIN[b.3 as usize]), ("巳", "官鬼"));
        // 六神：書中上三爻 朱雀、青龍、玄武，三爻白虎，與表（戊日初勾陳、二螣蛇、三白虎、四玄武、五青龍、上朱雀）合；
        // 書例初、二爻作「滕蛇」「勾陳」，與同章表互倒，从表（见 doc）
        let s: Vec<&str> = p.yao.iter().map(|y| LIU_SHEN[y.shen.unwrap() as usize]).collect();
        assert_eq!(s, ["勾陳", "螣蛇", "白虎", "玄武", "青龍", "朱雀"]);
    }

    #[test]
    fn xun_kong_and_fu_shen() {
        // 旬空：甲子旬戌亥、甲戌旬申酉、甲申旬午未、甲午旬辰巳、甲辰旬寅卯、甲寅旬子丑
        let want = [[10, 11], [8, 9], [6, 7], [4, 5], [2, 3], [0, 1]];
        for xs in 0..6u8 {
            assert_eq!(pai_gua([7; 6], None, Some(xs * 10 + 3), LiuyaoOptions::default()).xun_kong.unwrap(), want[xs as usize]);
        }
        // 天風姤（乾宫一世）缺妻財：伏乾宫首卦二爻妻財寅木
        let p = pai_gua([8, 7, 7, 7, 7, 7], None, None, LiuyaoOptions::default());
        let fu: Vec<_> = p.yao.iter().enumerate().filter_map(|(k, y)| y.fu.map(|f| (k, LIU_QIN[f.0 as usize], ZHI[f.2 as usize]))).collect();
        assert_eq!(fu, [(1, "妻財", "寅")]);
    }

    #[test]
    fn gua_shen() {
        // 火珠林：阳世子月起、阴世午月起。乾为天世在六（阳）→ 子起初爻数至六爻得巳；姤世在初（阴爻）→ 午
        assert_eq!(pai_gua([7; 6], None, None, LiuyaoOptions::default()).gua_shen, Some(5));
        assert_eq!(pai_gua([8, 7, 7, 7, 7, 7], None, None, LiuyaoOptions::default()).gua_shen, Some(6));
        assert_eq!(pai_gua([7; 6], None, None, LiuyaoOptions { gua_shen: GuaShenRule::None }).gua_shen, None);
    }

    #[test]
    fn meihua() {
        // 卷一年月日時起例：子年1 + 正月1 + 初一1 = 3 → 上卦離；加子时 1 → 4 震；动爻 4
        let (up, lo, d) = meihua_time(0, 1, 1, 0);
        assert_eq!((BA_GUA[tri_index(up)].0, BA_GUA[tri_index(lo)].0, d), ("離", "震", 4));
        // 余零：八为坤、六为六爻
        let (up, lo, d) = meihua_numbers(16, 8, 12);
        assert_eq!((BA_GUA[tri_index(up)].0, BA_GUA[tri_index(lo)].0, d), ("坤", "坤", 6));
        // 互卦：乾坤无互（互仍乾坤）
        assert_eq!(hu_gua(0b111111), 0b111111);
        assert_eq!(gua_name(hu_gua(g("觀"))), "剝"); // 觀 二三四坤、三四五艮 → 山地剝
        // 上離下震为噬嗑，四爻（阳）动作老阳九
        let v = meihua_values(0b101, 0b001, 4);
        assert_eq!(v, [7, 8, 8, 9, 8, 7]);
        assert_eq!(pai_gua(v, None, None, LiuyaoOptions::default()).ben_name, "噬嗑");
    }
}
