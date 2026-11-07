//! 杂节气：依节气与日干支推出的节令日（三伏、社日、寒食、腊日、入梅出梅、土王用事、数九、七十二候）。
//!
//! 输入是节气所在的民用日（儒略日数，按所用地方时取日），本模块只做"某气之后第几个某干日"之类的计数，
//! 不碰天文。各项规则与边界（节气当日是否计入）都有出处，见 `doc/folk/zajie.md`：
//! - 三伏：《欽定協紀辨方書》卷十二「夏至後三庚為初伏，四庚為中伏，立秋後初庚為末伏。如庚日夏至即為初庚，
//!   庚日立秋即為末伏」；《推步捷例》同，并记道光戊申（1848）后立秋值庚「時刻勿拘午前午後」。
//! - 社日两家：「立春後五戊」（《夢粱錄》卷一、《田家五行》，《歲序總考》注「如戊日立春則不筭」）；
//!   「二分前後近戊」（《協紀》卷十二引《歷例》，《禮記·月令》注「元日近春分前後戊日」）。
//! - 寒食：《荆楚歲時記》「去冬節一百五日」，注「亦有去冬至一百六日者」；《推步捷例》「不計冬至日」。
//! - 腊日：《說文》「冬至後三戌，臘祭百神」；《推步捷例》朝鲜用「冬至後第三未」，「冬至値未，則自冬至為三未之數」。
//! - 入梅出梅两家（《協紀》卷十二「霉天」）：《神樞經》芒種後逢丙入、小暑後逢未出；《碎金》芒種後逢壬入、
//!   夏至後逢庚出；《歲序總考》「芒種後逢壬日……若本日是壬則不筭」。
//! - 土王用事：《協紀》卷一引《神樞經》「於四立之前各旺一十八日」；《推步捷例》「自四立節前一辰，逆數至十八日」。
//! - 数九：《帝京景物略》「日冬至，畫素梅一枝，為瓣八十有一，日染一瓣，瓣盡而九九出」。
//! - 七十二候：名目逐字据《協紀》卷十二「氣候」（原书「𤣥」避讳缺笔、「䑕」为鼠的异体，改作玄、鼠；
//!   小寒「雉雊」、大寒「雞乳」「征鳥厲疾」与《逸周書·時訓》等本不同，照协纪）；每气三候，以候策（气长的三分之一）累加（《宋史·律曆志》应天历
//!   「各因諸氣大小餘秒命之，即初候日也；各以候策加之，得次候日」）。

/// 日干序（甲 = 0）
pub fn day_gan(jdn: i64) -> u8 {
    (jdn + 49).rem_euclid(10) as u8
}

/// 日支序（子 = 0）
pub fn day_zhi(jdn: i64) -> u8 {
    (jdn + 49).rem_euclid(12) as u8
}

/// 自 `start` 起（`include_start` 决定 start 当日是否计入）第 n 个满足条件的日子（n 从 1 起）。
fn nth_after(start: i64, n: u32, include_start: bool, pred: impl Fn(i64) -> bool) -> i64 {
    let mut j = if include_start { start } else { start + 1 };
    let mut k = 0;
    loop {
        if pred(j) {
            k += 1;
            if k == n {
                return j;
            }
        }
        j += 1;
    }
}

/// 三伏：(初伏, 中伏, 末伏) 起日。夏至、立秋当日值庚均计入。
pub fn san_fu(xia_zhi: i64, li_qiu: i64) -> (i64, i64, i64) {
    let geng = |j: i64| day_gan(j) == 6;
    (
        nth_after(xia_zhi, 3, true, geng),
        nth_after(xia_zhi, 4, true, geng),
        nth_after(li_qiu, 1, true, geng),
    )
}

/// 社日（立春/立秋后第五戊）。立春、立秋当日值戊不计入。
pub fn she_wu_wu(li_chun_or_qiu: i64) -> i64 {
    nth_after(li_chun_or_qiu, 5, false, |j| day_gan(j) == 4)
}

/// 社日（春分/秋分前后最近的戊日）。前后各五日相等时取前一日（原典未言，待考）。
pub fn she_jin_wu(fen: i64) -> i64 {
    let d = (4 - day_gan(fen) as i64).rem_euclid(10); // 距下一个戊（含当日）
    if d <= 5 && d != 5 { fen + d } else { fen + d - 10 }
}

/// 寒食：冬至后 n 日（不计冬至日），n 通常 105，亦有 106。
pub fn han_shi(dong_zhi_prev: i64, n: i64) -> i64 {
    dong_zhi_prev + n
}

/// 腊日：冬至后第三个某支日，冬至当日计入。中国用戌（支序 10），朝鲜用未（7）。
pub fn la_ri(dong_zhi: i64, zhi: u8) -> i64 {
    nth_after(dong_zhi, 3, true, |j| day_zhi(j) == zhi)
}

/// 入梅出梅两家：(入, 出)。节气当日值该干支不计入。
pub fn mei_shen_shu(mang_zhong: i64, xiao_shu: i64) -> (i64, i64) {
    (nth_after(mang_zhong, 1, false, |j| day_gan(j) == 2), nth_after(xiao_shu, 1, false, |j| day_zhi(j) == 7))
}
pub fn mei_sui_jin(mang_zhong: i64, xia_zhi: i64) -> (i64, i64) {
    (nth_after(mang_zhong, 1, false, |j| day_gan(j) == 8), nth_after(xia_zhi, 1, false, |j| day_gan(j) == 6))
}

/// 土王用事：四立前十八日。
pub fn tu_wang(si_li: i64) -> i64 {
    si_li - 18
}

/// 数九：冬至当日为一九第一日；返回 (第几九 1—9, 第几日 1—9)，九九之外返回 None。
pub fn shu_jiu(dong_zhi: i64, jdn: i64) -> Option<(u8, u8)> {
    let k = jdn - dong_zhi;
    if (0..81).contains(&k) { Some(((k / 9 + 1) as u8, (k % 9 + 1) as u8)) } else { None }
}

/// 七十二候（《協紀》卷十二），每气三候，从立春起。
pub const QI_HOU: [(&str, [&str; 3]); 24] = [
    ("立春", ["東風解凍", "蟄蟲始振", "魚陟負冰"]),
    ("雨水", ["獺祭魚", "候鴈北", "草木萌動"]),
    ("驚蟄", ["桃始華", "倉庚鳴", "鷹化為鳩"]),
    ("春分", ["玄鳥至", "雷乃發聲", "始電"]),
    ("清明", ["桐始華", "田鼠化為鴽", "虹始見"]),
    ("穀雨", ["萍始生", "鳴鳩拂其羽", "戴勝降於桑"]),
    ("立夏", ["螻蟈鳴", "蚯蚓出", "王瓜生"]),
    ("小滿", ["苦菜秀", "靡草死", "麥秋至"]),
    ("芒種", ["螳螂生", "鵙始鳴", "反舌無聲"]),
    ("夏至", ["鹿角解", "蜩始鳴", "半夏生"]),
    ("小暑", ["溫風至", "蟋蟀居壁", "鷹始摰"]),
    ("大暑", ["腐草為螢", "土潤溽暑", "大雨時行"]),
    ("立秋", ["凉風至", "白露降", "寒蟬鳴"]),
    ("處暑", ["鷹乃祭鳥", "天地始肅", "禾乃登"]),
    ("白露", ["鴻鴈來", "玄鳥歸", "羣鳥養羞"]),
    ("秋分", ["雷始收聲", "蟄蟲坯戸", "水始涸"]),
    ("寒露", ["鴻鴈來賔", "雀入大水為蛤", "菊有黄華"]),
    ("霜降", ["豺乃祭獸", "草木黄落", "蟄蟲咸俯"]),
    ("立冬", ["水始冰", "地始凍", "雉入大水為蜃"]),
    ("小雪", ["虹藏不見", "天氣上升地氣下降", "閉塞而成冬"]),
    ("大雪", ["鶡鴠不鳴", "虎始交", "荔挺出"]),
    ("冬至", ["蚯蚓結", "麋角解", "水泉動"]),
    ("小寒", ["鴈北鄉", "鵲始巢", "雉雊"]),
    ("大寒", ["雞乳", "征鳥厲疾", "水澤腹堅"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn find_gan(from: i64, g: u8) -> i64 {
        (from..from + 10).find(|&j| day_gan(j) == g).unwrap()
    }

    #[test]
    fn san_fu_counts_xia_zhi_when_geng() {
        let xz = find_gan(2460000, 6); // 夏至值庚
        let (a, b, _) = san_fu(xz, xz + 46);
        assert_eq!(a, xz + 20);
        assert_eq!(b, xz + 30);
        let xz2 = xz + 1; // 夏至为辛日
        assert_eq!(san_fu(xz2, xz2 + 46).0, xz + 30);
        // 立秋值庚即末伏
        let lq = find_gan(2460050, 6);
        assert_eq!(san_fu(lq - 46, lq).2, lq);
    }

    #[test]
    fn she_rules() {
        let lc = find_gan(2460000, 4); // 立春值戊不计
        assert_eq!(she_wu_wu(lc), lc + 50);
        assert_eq!(she_wu_wu(lc + 1), lc + 50);
        let f = find_gan(2460000, 4);
        assert_eq!(she_jin_wu(f), f);
        assert_eq!(she_jin_wu(f + 3), f);
        assert_eq!(she_jin_wu(f - 3), f);
        assert_eq!(she_jin_wu(f + 5), f); // 前后各五日取前
    }

    #[test]
    fn la_and_mei() {
        let dz = (2460000..2460012).find(|&j| day_zhi(j) == 10).unwrap(); // 冬至值戌，计入
        assert_eq!(la_ri(dz, 10), dz + 24);
        let mz = find_gan(2460000, 8); // 芒种值壬不计
        assert_eq!(mei_sui_jin(mz, mz + 15).0, mz + 10);
    }

    /// 以真实定气（东八区日）核对：2023 年初伏 7-11、中伏 7-21、末伏 8-10；2024 年初伏 7-15、中伏 7-25、末伏 8-14
    /// （通行万年历与气象部门公布的入伏日期）。
    #[test]
    fn san_fu_2023_2024() {
        use crate::astronomy::aspects::solar_term_jds_for_year_cached;
        use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;
        use crate::calendar::chinese_lunar::tt_to_day_number_utc8;
        use crate::calendar::gregorian::Gregorian;
        use crate::math::real::RealOps;
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let day = |y, m, d| (Gregorian::to_julian_day(y, m, d).as_f64() + 0.5).floor() as i64;
        for (y, a, b, c) in [(2023, (7, 11), (7, 21), (8, 10)), (2024, (7, 15), (7, 25), (8, 14))] {
            let t = solar_term_jds_for_year_cached(&vsop, y);
            let got = san_fu(tt_to_day_number_utc8(t[6]), tt_to_day_number_utc8(t[9]));
            assert_eq!(got, (day(y, a.0, a.1), day(y, b.0, b.1), day(y, c.0, c.1)), "{y}");
        }
    }

    #[test]
    fn shu_jiu_range() {
        assert_eq!(shu_jiu(100, 100), Some((1, 1)));
        assert_eq!(shu_jiu(100, 118), Some((3, 1)));
        assert_eq!(shu_jiu(100, 180), Some((9, 9)));
        assert_eq!(shu_jiu(100, 181), None);
    }
}
