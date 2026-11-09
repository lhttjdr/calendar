//! 婚嫁择吉：协纪所改、删之旧说与类编、鳌头、董公诸书独有之神（起例）。
//!
//! 清点与出处见 `doc/folk/jiaqu_shensha_inventory.md` 第三至六节、第十一节「待实现清单」；判定之接入在
//! [`super::jiaqu::judge_day`]：缺省一律只作附注（协纪为准），网页「取舍规则」中逐神可选否决（[`TONGSHU_SHEN`] 位）。
//! 本模块只给规则：某月（月建序，寅月 0，以节分）某日是否值某神。
//!
//! 原典：协纪据维基文库四库本录文（`vendor/folk/wikisource/欽定協紀辨方書/`）；董公据维基文库录文
//!（`vendor/folk/wikisource/董公選擇日要覽/全文.wikitext`，未以书影校）；类编、鳌头据书影读图（页序见清点篇凡例）。

use super::jianchu::{huang_hei, is_huang_dao, jian_chu, HUANG_HEI_SHEN};
use super::zhiri::{xiu_index, XIU_28};

const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
const GAN: &str = "甲乙丙丁戊己庚辛壬癸";

/// 干支名 → 序（甲子 0）
pub fn gz(name: &str) -> u8 {
    let cs: Vec<char> = name.chars().collect();
    let g = GAN.chars().position(|c| c == cs[0]).expect("干") as u8;
    let z = ZHI.chars().position(|c| c == cs[1]).expect("支") as u8;
    (0..60u8).find(|k| k % 10 == g && k % 12 == z).expect("干支")
}

/// 支名串（每月一支，正月起）→ 某月之支
fn col(s: &str, month_ord: u8) -> u8 {
    let c = s.chars().nth(month_ord as usize % 12).unwrap();
    ZHI.chars().position(|x| x == c).unwrap() as u8
}

fn in_gz(list: &[&str], day_gz: u8) -> bool {
    list.iter().any(|g| gz(g) == day_gz % 60)
}

// ───────────────────────── 一、协纪所改、删之旧说 ─────────────────────────

/// 大煞（协纪卷六引《神樞經》「所值之日忌出軍征討嫁娶納財豎柱上樑移徙置室」；李鼎祚：正戌二巳三午四未五寅六夘七辰
/// 八亥九子十丑十一申十二酉）。卷十止忌軍事，「舊本並忌進人口納財豎柱上梁其義不倫故刪」
pub fn da_sha(month_ord: u8, day_zhi: u8) -> bool {
    col("戌巳午未寅卯辰亥子丑申酉", month_ord) == day_zhi % 12
}

/// 类编之天火（类编卷十五 `02094410.cn` 第 59 页，即协纪卷六所引《歷例》旧文）：正月子顺行四仲（子卯午酉）。
/// 《玉帳經》「忌……㑹親娶婦」。协纪以天火为災煞异名（逆行），災煞已在婚事忌神中，此为类编之位
pub fn tian_huo_leibian(month_ord: u8, day_zhi: u8) -> bool {
    (3 * month_ord as u32 % 12) as u8 == day_zhi % 12
}

/// 当日黄黑道十二神（协纪卷七引《神樞經》：青龍正月起子……）：神名与是否黄道。以月建起
pub fn huang_hei_dao(month_ord: u8, day_zhi: u8) -> (&'static str, bool) {
    let s = huang_hei(day_zhi % 12, (month_ord + 2) % 12);
    (HUANG_HEI_SHEN[s as usize], is_huang_dao(s))
}

/// 九醜（协纪卷七引《金匱經》「五干臨此四辰其日不可出軍嫁娶移徙築室」）：乙戊己辛壬配子午卯酉之十日。
/// 协纪按语「必大吉臨日辰乃為九醜」，作时神；此取日之旧说
pub const JIU_CHOU: [&str; 10] = ["戊子", "戊午", "壬子", "壬午", "乙卯", "己卯", "辛卯", "乙酉", "己酉", "辛酉"];
pub fn jiu_chou(day_gz: u8) -> bool {
    in_gz(&JIU_CHOU, day_gz)
}

/// 旧本宜婚之神（协纪皆载于月表，而卷十改其所宜，不及婚事）：(名, 旧说, 协纪之断)
pub fn jiu_ben_yi_shen(month_ord: u8, day_zhi: u8) -> Vec<(&'static str, &'static str, &'static str)> {
    let m = month_ord % 12;
    let z = day_zhi % 12;
    let jc = jian_chu(z, (m + 2) % 12);
    let mut v = Vec::new();
    // 時陽：月建後二辰，即开日（卷四引總要歷「所值之日宜叙婚姻行宴樂」）
    if jc == 10 {
        v.push(("時陽", "總要歷：宜叙婚姻", "卷十開日按语：不及婚事"));
    }
    // 陽德：正月起戌順行六陽辰（卷六引總要歷「宜交易開市結親姻」）
    if (10 + 2 * m) % 12 == z {
        v.push(("陽德", "總要歷：宜結親姻", "卷十：無義例可推，删"));
    }
    // 吉期：除日（卷十除日「又為吉期」，舊謂宜結婚姻）
    if jc == 1 {
        v.push(("吉期", "舊本：宜結婚姻", "卷十：於義無取，删"));
    }
    if col("卯酉辰戌巳亥午子未丑申寅", m) == z {
        v.push(("玉宇", "樞要歷：宜結婚姻㑹賓客", "卷十：宜修祠宇"));
    }
    let yi_hou = col("子午丑未寅申卯酉辰戌巳亥", m);
    if yi_hou == z {
        v.push(("益後", "樞要歷：宜行嫁娶", "卷十：宜祭祀祈福求嗣"));
    }
    if (yi_hou + 1) % 12 == z {
        v.push(("續世", "樞要歷：宜結婚姻", "卷十：宜祭祀祈福求嗣"));
    }
    // 母倉：春亥子、夏寅卯、秋辰戌丑未、冬申酉（土王後巳午，铺注之事，此不计）
    let mc: &[u8] = match m / 3 {
        0 => &[11, 0],
        1 => &[2, 3],
        2 => &[4, 10, 1, 7],
        _ => &[8, 9],
    };
    if mc.contains(&z) {
        v.push(("母倉", "卷三十五附录：嫁娶吉日宜母倉", "卷十：宜納財栽種，不及婚事"));
    }
    // 六儀：正辰逆行（与厭對同位）
    if (16 - m as i32).rem_euclid(12) as u8 == z {
        v.push(("六儀", "神樞經：宜結親納禮", "卷四：定為厭對忌嫁娶"));
    }
    v
}

/// 冰消瓦解两说（协纪卷三十六「冰消瓦解滅門大禍」斥为「即年支之平收也」；类编卷十五、董公「百事皆忌」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BingXiao {
    /// 类编卷十五（`02094410.cn` 第 60 页）：巳子丑申卯戌亥午未寅酉辰
    LeiBian,
    /// 董公「冰消瓦解日」：巳子丑寅卯戌亥午未申酉辰（四月、十月与类编互易，未核）
    DongGong,
}

pub fn bing_xiao_wa_jie(rule: BingXiao, month_ord: u8, day_zhi: u8) -> bool {
    let s = match rule {
        BingXiao::LeiBian => "巳子丑申卯戌亥午未寅酉辰",
        BingXiao::DongGong => "巳子丑寅卯戌亥午未申酉辰",
    };
    col(s, month_ord) == day_zhi % 12
}

// ───────────────────────── 二、类编求婚表所避 ─────────────────────────

/// 九土鬼两说（类编卷七求婚表尾「九土鬼」，起例见鳌头卷二出行凶神、董公「九土鬼日」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiuTuGui {
    /// 鳌头卷二出行凶神（第 3 册第 28 页）：乙酉癸巳辛丑庚戌丁巳甲午壬寅己酉戊申
    AoTou,
    /// 董公：乙酉癸巳甲午辛丑壬寅己酉庚戌丁巳戊午（末日作戊午，未核）；「與建、破、平、收日相併……則無妨」
    DongGong,
}

pub const JIU_TU_GUI_AOTOU: [&str; 9] = ["乙酉", "癸巳", "辛丑", "庚戌", "丁巳", "甲午", "壬寅", "己酉", "戊申"];
pub const JIU_TU_GUI_DONGGONG: [&str; 9] = ["乙酉", "癸巳", "甲午", "辛丑", "壬寅", "己酉", "庚戌", "丁巳", "戊午"];

/// 是否值九土鬼（董公说之「與建破平收日相併則無妨」不在此，见 [`jiu_tu_gui_wu_fang`]）
pub fn jiu_tu_gui(rule: JiuTuGui, day_gz: u8) -> bool {
    match rule {
        JiuTuGui::AoTou => in_gz(&JIU_TU_GUI_AOTOU, day_gz),
        JiuTuGui::DongGong => in_gz(&JIU_TU_GUI_DONGGONG, day_gz),
    }
}

/// 董公：九土鬼「此星與建、破、平、收日相併，凶吉星相併，則無妨」——日值建破平收者无妨
pub fn jiu_tu_gui_wu_fang(month_ord: u8, day_zhi: u8) -> bool {
    matches!(jian_chu(day_zhi % 12, (month_ord + 2) % 12), 0 | 6 | 3 | 9)
}

/// 类编九空（求婚表尾所避）：辰丑戌未卯子酉午寅亥申巳（与焦坎同位；起例据 `data/folk/leibian_juan11_vs_xieji.csv`）。
/// 协纪九空正辰逆行四季，所忌進人口等，不及婚事
pub fn jiu_kong_leibian(month_ord: u8, day_zhi: u8) -> bool {
    col("辰丑戌未卯子酉午寅亥申巳", month_ord) == day_zhi % 12
}

// ───────────────────────── 三、鳌头卷三嫁娶门 ─────────────────────────

/// 紅嘴朱雀（鳌头第 4 册第 13 页，读图）：壬申辛巳庚寅己亥戊申丁巳（每九日一位，书只列六日），「損宅長」
pub const HONG_ZUI_ZHU_QUE: [&str; 6] = ["壬申", "辛巳", "庚寅", "己亥", "戊申", "丁巳"];
pub fn hong_zui_zhu_que(day_gz: u8) -> bool {
    in_gz(&HONG_ZUI_ZHU_QUE, day_gz)
}

/// 婚娶忌宿（鳌头第 4 册第 13 页）：「心翼箕參鬼，亢氐及昴牛，婚姻不犯此，夫婦樂無憂」
pub const HUN_QU_JI_XIU: [&str; 9] = ["心", "翼", "箕", "參", "鬼", "亢", "氐", "昴", "牛"];
pub fn hun_qu_ji_xiu(jdn: i64) -> Option<&'static str> {
    let x = XIU_28[xiu_index(jdn) as usize];
    HUN_QU_JI_XIU.contains(&x).then_some(x)
}

/// 離窠（鳌头第 4 册第 14 页；卷二出行凶神第 3 册第 28 页同）
pub const LI_KE: [&str; 13] = ["丁卯", "戊辰", "己巳", "戊寅", "辛巳", "戊子", "己丑", "戊戌", "己亥", "辛丑", "戊午", "壬戌", "癸亥"];
pub fn li_ke(day_gz: u8) -> bool {
    in_gz(&LI_KE, day_gz)
}

/// 女嫁厄年（鳌头第 4 册第 12 页「女嫁 生命 犯此厄年多疾患」）：女命子丑寅……亥，厄年卯寅丑子亥戌酉申未午巳辰。
/// `year_zhi` 为嫁娶之年（立春分年）之支
pub fn nv_jia_e_nian(nv_zhi: u8, year_zhi: u8) -> bool {
    (3 - nv_zhi as i32 % 12).rem_euclid(12) as u8 == year_zhi % 12
}

/// 荒蕪鳌头序（鳌头第 4 册第 13 页）：巳酉丑申子辰亥卯未寅午戌，即孟平仲破季收（类编卷十五新增之序）。
/// 类编卷七序（巳酉丑辰申子卯未亥寅戌午）在 [`super::jiaqu::LEIBIAN_SHEN`]「荒蕪」
pub fn huang_wu_aotou(month_ord: u8, day_zhi: u8) -> bool {
    col("巳酉丑申子辰亥卯未寅午戌", month_ord) == day_zhi % 12
}

/// 十二星斷（鳌头卷一第 1 册第 77—80 页，建除逐星论嫁娶）：(建除名, 宜忌, 原句)。
/// 除日一条两说（「出行嫁娶亦吉」「又曰最忌嫁娶」），宜忌记作「两说」
pub const SHI_ER_XING_DUAN: [(&str, &str, &str); 12] = [
    ("建", "宜", "只宜行嫁"),
    ("除", "两说", "出行嫁娶亦吉；又曰最忌嫁娶埋葬"),
    ("滿", "宜", "婚姻結義完全好"),
    ("平", "宜", "又宜行嫁及教畜"),
    ("定", "宜", "只宜冠帶安牀嫁娶"),
    ("執", "宜", "此執日宜逢造葬嫁娶"),
    ("破", "忌", "破日造葬嫁娶凶，婚姻不諧"),
    ("危", "忌", "惟造葬嫁娶俱凶"),
    ("成", "宜", "娶妻必定生貴子"),
    ("收", "宜", "收日娶妻埋葬吉"),
    ("開", "宜", "嫁娶移徙出行吉"),
    ("閉", "宜", "大宜娶婦不妄動"),
];

pub fn shi_er_xing_duan(month_ord: u8, day_zhi: u8) -> (&'static str, &'static str, &'static str) {
    SHI_ER_XING_DUAN[jian_chu(day_zhi % 12, (month_ord + 2) % 12) as usize]
}

/// 滅沒（鳌头第 4 册第 13 页；卷二第 3 册第 41 页）：弦日值虛、晦日值婁、朔日值角、望日值亢；「虛日值鬼、盈日值牛」之虛盈所指未核，不编。
/// `phase` 为当日所值朔、上弦、望、下弦；`hui` 为农历月末日
pub fn mie_mo(phase: Option<&str>, hui: bool, jdn: i64) -> Option<&'static str> {
    let x = XIU_28[xiu_index(jdn) as usize];
    match (phase, x) {
        (Some("朔"), "角") => Some("朔日值角"),
        (Some("望"), "亢") => Some("望日值亢"),
        (Some("上弦" | "下弦"), "虚") => Some("弦日值虛"),
        _ if hui && x == "婁" => Some("晦日值婁"),
        _ => None,
    }
}

// ───────────────────────── 四、董公 ─────────────────────────

/// 金神七煞（董公「金神七煞歌」：「角亢奎婁牛鬼星……起造婚姻逢此日，不出三年見哭聲」；歌后注列六宿缺鬼，从歌取七宿）。
/// 董公：煞貢、直星、人專「能解凶星，唯不能解金神七煞」
pub const JIN_SHEN_QI_SHA: [&str; 7] = ["角", "亢", "奎", "婁", "牛", "鬼", "星"];
pub fn jin_shen_qi_sha(jdn: i64) -> Option<&'static str> {
    let x = XIU_28[xiu_index(jdn) as usize];
    JIN_SHEN_QI_SHA.contains(&x).then_some(x)
}

/// 十惡大敗（董公年月表，「百事皆忌」）：年干 × 节月。甲己年三月戊戌、七月癸亥、十月丙申、十一月丁亥；乙庚年四月壬申、九月乙巳；
/// 丙辛年三月辛巳、九月庚辰、十月甲戌；戊癸年六月己丑；丁壬年无。年以立春分、月以节分（董公以月建分节；未核）
pub fn shi_e_da_bai_nian_yue(year_gan: u8, month_ord: u8, day_gz: u8) -> bool {
    let t: &[(u8, &str)] = match year_gan % 5 {
        0 => &[(3, "戊戌"), (7, "癸亥"), (10, "丙申"), (11, "丁亥")],
        1 => &[(4, "壬申"), (9, "乙巳")],
        2 => &[(3, "辛巳"), (9, "庚辰"), (10, "甲戌")],
        4 => &[(6, "己丑")],
        _ => &[],
    };
    t.iter().any(|(mo, g)| *mo == month_ord % 12 + 1 && gz(g) == day_gz % 60)
}

/// 煞貢、直星、人專（董公「煞貢、直星、人專吉日」：「最吉之三星，遇其一星值日，用之可獲吉，能解凶星，唯不能解金神七煞」）。
/// 按孟仲季月取干支，照录文：季月直星「辛卯」按每九日一位当作「辛亥」、季月人專「己亥」当作「丁亥」，皆未核，先照录文。
/// 董公略记又云白虎入中宫之日「縱有人專、直星、煞貢……可以化解……莫若不用為上」。返回所值之名
pub fn sha_gong_zhi_xing_ren_zhuan(month_ord: u8, day_gz: u8) -> Vec<&'static str> {
    const T: [[&[&str]; 3]; 3] = [
        [
            &["丁卯", "丙子", "乙酉", "甲午", "癸卯", "壬子", "辛酉"],
            &["戊辰", "丁丑", "丙戌", "乙未", "甲辰", "癸丑", "壬戌"],
            &["辛未", "庚辰", "己丑", "戊戌", "丁未", "丙辰"],
        ],
        [
            &["丙寅", "乙亥", "甲申", "癸巳", "壬寅", "辛亥", "庚申"],
            &["丁卯", "丙子", "乙酉", "甲午", "癸卯", "壬子", "辛酉"],
            &["庚午", "己卯", "戊子", "丁酉", "丙午", "乙卯"],
        ],
        [
            &["己丑", "甲戌", "癸未", "壬辰", "辛丑", "庚戌", "己未"],
            &["丙寅", "乙亥", "甲申", "癸巳", "壬寅", "辛卯", "庚申"],
            &["己巳", "戊寅", "己亥", "丙申", "乙巳", "甲寅", "癸亥"],
        ],
    ];
    let row = &T[(month_ord % 3) as usize];
    ["煞貢", "直星", "人專"].iter().zip(row.iter()).filter(|(_, l)| in_gz(l, day_gz)).map(|(n, _)| *n).collect()
}

// ───────────────────────── 逐神取舍之表 ─────────────────────────

/// 可选否决诸神：(名, 组, 所主与协纪之断)。下标即 [`super::jiaqu::JiaquOptions::tongshu_mask`] 位（置位者否决，余只作附注）
pub const TONGSHU_SHEN: [(&str, &str, &str); 19] = [
    ("大煞", "协纪改删", "神樞經忌嫁娶；协纪卷十止忌軍事，删其餘"),
    ("天火（类编）", "协纪改删", "玉帳經忌㑹親娶婦；类编顺行四仲，协纪天火即災煞（已在忌神）"),
    ("黑道", "协纪改删", "神樞經「所值之日皆不可……嫁娶」；协纪卷十「六黑道亦無專忌之事」"),
    ("九醜", "协纪改删", "金匱經「不可出軍嫁娶」；协纪作时神"),
    ("冰消瓦解（类编）", "协纪改删", "类编卷十五百事忌；协纪卷三十六斥「即年支之平收」"),
    ("冰消瓦解（董公）", "协纪改删", "董公「百事皆忌」；四、十月与类编互易"),
    ("九土鬼（鳌头）", "类编", "类编求婚表所避；鳌头卷二起例"),
    ("九土鬼（董公）", "类编", "董公起例（末日戊午），與建破平收日相併則無妨"),
    ("九空（类编）", "类编", "类编求婚表所避；协纪九空起例异、不及婚事"),
    ("紅嘴朱雀", "鳌头", "鳌头嫁娶表尾所避，「損宅長」"),
    ("婚娶忌宿", "鳌头", "「心翼箕參鬼，亢氐及昴牛，婚姻不犯此」"),
    ("離窠", "鳌头", "鳌头嫁娶凶神"),
    ("女嫁厄年", "鳌头", "女命犯此厄年多疾患（择年）"),
    ("荒蕪（鳌头序）", "鳌头", "鳌头嫁娶凶神表之序（孟平仲破季收）"),
    ("十二星斷", "鳌头", "鳌头卷一：破「婚姻不諧」、危「造葬嫁娶俱凶」"),
    ("十二星斷除日又說", "鳌头", "鳌头卷一除日「又曰最忌嫁娶」"),
    ("滅沒", "鳌头", "朔角、望亢、晦婁、弦虛（虛日、盈日未核不编）"),
    ("金神七煞", "董公", "「起造婚姻逢此日，不出三年見哭聲」；煞貢直星人專不能解"),
    ("十惡大敗（年月）", "董公", "董公年月表「百事皆忌」；协纪以無祿論，不取此表"),
];

/// 名 → 位
pub fn tongshu_bit(name: &str) -> u64 {
    TONGSHU_SHEN.iter().position(|(n, _, _)| *n == name).map_or(0, |i| 1 << i)
}

/// 一日所值诸神之输入
#[derive(Clone, Copy, Debug)]
pub struct TongshuInput<'a> {
    pub month_ord: u8,
    pub day_gz: u8,
    pub jdn: i64,
    /// 嫁娶之年（立春分）干支序
    pub year_gz: u8,
    /// 女命生年干支序
    pub nv_year_gz: u8,
    /// 当日所值朔、上弦、望、下弦
    pub phase: Option<&'a str>,
    /// 农历月末日
    pub hui: bool,
}

/// 一日所值 [`TONGSHU_SHEN`] 诸神：(下标, 说明)
pub fn tongshu_hits(t: &TongshuInput) -> Vec<(usize, String)> {
    let (m, g) = (t.month_ord % 12, t.day_gz % 60);
    let z = g % 12;
    let mut v: Vec<(usize, String)> = Vec::new();
    let mut push = |name: &str, s: String| {
        let i = TONGSHU_SHEN.iter().position(|(n, _, _)| *n == name).unwrap();
        v.push((i, s));
    };
    if da_sha(m, z) {
        push("大煞", "大煞".into());
    }
    if tian_huo_leibian(m, z) {
        push("天火（类编）", "天火（类编顺行四仲）".into());
    }
    let (hh, huang) = huang_hei_dao(m, z);
    if !huang {
        push("黑道", format!("黑道{hh}"));
    }
    if jiu_chou(g) {
        push("九醜", "九醜".into());
    }
    if bing_xiao_wa_jie(BingXiao::LeiBian, m, z) {
        push("冰消瓦解（类编）", "冰消瓦解（类编）".into());
    }
    if bing_xiao_wa_jie(BingXiao::DongGong, m, z) {
        push("冰消瓦解（董公）", "冰消瓦解（董公）".into());
    }
    if jiu_tu_gui(JiuTuGui::AoTou, g) {
        push("九土鬼（鳌头）", "九土鬼（鳌头起例）".into());
    }
    if jiu_tu_gui(JiuTuGui::DongGong, g) && !jiu_tu_gui_wu_fang(m, z) {
        push("九土鬼（董公）", "九土鬼（董公起例）".into());
    }
    if jiu_kong_leibian(m, z) {
        push("九空（类编）", "九空（类编）".into());
    }
    if hong_zui_zhu_que(g) {
        push("紅嘴朱雀", "紅嘴朱雀".into());
    }
    if let Some(x) = hun_qu_ji_xiu(t.jdn) {
        push("婚娶忌宿", format!("婚娶忌宿（值{x}）"));
    }
    if li_ke(g) {
        push("離窠", "離窠".into());
    }
    if nv_jia_e_nian(t.nv_year_gz % 12, t.year_gz % 12) {
        push("女嫁厄年", "女嫁厄年".into());
    }
    if huang_wu_aotou(m, z) {
        push("荒蕪（鳌头序）", "荒蕪（鳌头序）".into());
    }
    let (jc, yj, _) = shi_er_xing_duan(m, z);
    if yj == "忌" {
        push("十二星斷", format!("十二星斷：{jc}日忌嫁娶"));
    }
    if jc == "除" {
        push("十二星斷除日又說", "十二星斷：除日又說最忌嫁娶".into());
    }
    if let Some(s) = mie_mo(t.phase, t.hui, t.jdn) {
        push("滅沒", format!("滅沒（{s}）"));
    }
    if let Some(x) = jin_shen_qi_sha(t.jdn) {
        push("金神七煞", format!("金神七煞（值{x}）"));
    }
    if shi_e_da_bai_nian_yue(t.year_gz % 10, m, g) {
        push("十惡大敗（年月）", "十惡大敗（董公年月表）".into());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zi(c: char) -> u8 {
        ZHI.chars().position(|x| x == c).unwrap() as u8
    }

    /// 诸神起例逐条与原文核
    #[test]
    fn qi_li() {
        // 大煞：正戌、五寅、十二酉
        assert!(da_sha(0, zi('戌')) && da_sha(4, zi('寅')) && da_sha(11, zi('酉')));
        // 类编天火：正子、二卯、三午、四酉、五子
        assert!(tian_huo_leibian(0, 0) && tian_huo_leibian(1, 3) && tian_huo_leibian(3, 9) && tian_huo_leibian(4, 0));
        // 黄黑道：正月青龍在子、明堂丑、天刑寅
        assert_eq!(huang_hei_dao(0, 0), ("青龍", true));
        assert_eq!(huang_hei_dao(0, 2), ("天刑", false));
        assert!(jiu_chou(gz("戊子")) && jiu_chou(gz("辛酉")) && !jiu_chou(gz("甲子")));
        // 冰消瓦解两说：四月类编申、董公寅；十月类编寅、董公申；余月同
        assert!(bing_xiao_wa_jie(BingXiao::LeiBian, 3, zi('申')) && bing_xiao_wa_jie(BingXiao::DongGong, 3, zi('寅')));
        assert!(bing_xiao_wa_jie(BingXiao::LeiBian, 9, zi('寅')) && bing_xiao_wa_jie(BingXiao::DongGong, 9, zi('申')));
        assert_eq!((0..12u8).filter(|&m| (0..12u8).any(|z| bing_xiao_wa_jie(BingXiao::LeiBian, m, z) != bing_xiao_wa_jie(BingXiao::DongGong, m, z))).count(), 2);
        // 九土鬼两说：只末日异（戊申／戊午）
        let a: Vec<u8> = (0..60).filter(|&g| jiu_tu_gui(JiuTuGui::AoTou, g)).collect();
        let b: Vec<u8> = (0..60).filter(|&g| jiu_tu_gui(JiuTuGui::DongGong, g)).collect();
        assert_eq!(a.iter().filter(|g| !b.contains(g)).copied().collect::<Vec<_>>(), vec![gz("戊申")]);
        assert_eq!(b.iter().filter(|g| !a.contains(g)).copied().collect::<Vec<_>>(), vec![gz("戊午")]);
        // 紅嘴朱雀每九日一位
        for w in HONG_ZUI_ZHU_QUE.windows(2) {
            assert_eq!((gz(w[1]) + 60 - gz(w[0])) % 60, 9);
        }
        // 女嫁厄年：子卯、丑寅、寅丑、卯子、辰亥……亥辰
        for (nz, ez) in "子丑寅卯辰巳午未申酉戌亥".chars().zip("卯寅丑子亥戌酉申未午巳辰".chars()) {
            assert!(nv_jia_e_nian(zi(nz), zi(ez)), "{nz}");
        }
        // 荒蕪鳌头序与类编卷七序：正二三、十（亥）月同，四至九、十一、十二月异
        let lb = "巳酉丑辰申子卯未亥寅戌午";
        let same: Vec<u8> = (0..12u8).filter(|&m| huang_wu_aotou(m, col(lb, m))).collect();
        assert_eq!(same, vec![0, 1, 2, 9]);
        // 十二星斷：正月寅为建（宜）、申为破（忌）、酉为危（忌）、卯为除（两说）
        assert_eq!(shi_er_xing_duan(0, zi('寅')).1, "宜");
        assert_eq!(shi_er_xing_duan(0, zi('申')).1, "忌");
        assert_eq!(shi_er_xing_duan(0, zi('酉')).1, "忌");
        assert_eq!(shi_er_xing_duan(0, zi('卯')).1, "两说");
        // 十惡大敗（年月）：甲年三月戊戌、己年十一月丁亥、戊年六月己丑；丁壬年无
        assert!(shi_e_da_bai_nian_yue(0, 2, gz("戊戌")) && shi_e_da_bai_nian_yue(5, 10, gz("丁亥")) && shi_e_da_bai_nian_yue(4, 5, gz("己丑")));
        assert!((0..12u8).all(|m| (0..60u8).all(|g| !shi_e_da_bai_nian_yue(3, m, g) && !shi_e_da_bai_nian_yue(8, m, g))));
        // 煞貢直星人專：孟月直星即大殺白虎入中宮七日；季月照录文「辛卯」「己亥」
        assert_eq!(sha_gong_zhi_xing_ren_zhuan(0, gz("戊辰")), vec!["直星"]);
        assert_eq!(sha_gong_zhi_xing_ren_zhuan(1, gz("丁卯")), vec!["直星"]);
        assert_eq!(sha_gong_zhi_xing_ren_zhuan(2, gz("辛卯")), vec!["直星"]);
        assert_eq!(sha_gong_zhi_xing_ren_zhuan(2, gz("己亥")), vec!["人專"]);
        // 仲月煞貢即孟月之后一位；孟月煞貢与仲月直星同
        assert!((0..60u8).all(|g| sha_gong_zhi_xing_ren_zhuan(0, g).contains(&"煞貢") == sha_gong_zhi_xing_ren_zhuan(1, g).contains(&"直星")));
        // 值宿：jdn 2461354（2026-11-09）
        assert_eq!(TONGSHU_SHEN.len(), 19);
        assert!(TONGSHU_SHEN.len() <= 32);
    }

    /// 协纪月表所列之神与本模块起例比对：大煞、时阳等旧本宜神、黄黑道 720 日全同
    #[test]
    fn vs_yuebiao() {
        use super::super::yiji::yuebiao_day;
        let mut n = [0usize; 3];
        for m in 0..12u8 {
            for g in 0..60u8 {
                let d = yuebiao_day(m, g);
                // 月表黄道天德作「寶光」、玄武作「元武」
                let has = |x: &str| d.ji_shen.iter().chain(d.xiong_shen.iter()).any(|s| s == x || (x == "玄武" && s == "元武") || (x == "天德" && s == "寶光"));
                assert_eq!(da_sha(m, g % 12), has("大煞"), "大煞 {m} {g}");
                for (name, _, _) in jiu_ben_yi_shen(m, g % 12) {
                    assert!(has(name), "{name} {m} {g}");
                    n[0] += 1;
                }
                for name in ["時陽", "陽德", "吉期", "玉宇", "益後", "續世", "六儀"] {
                    assert_eq!(has(name), jiu_ben_yi_shen(m, g % 12).iter().any(|x| x.0 == name), "{name} {m} {g}");
                }
                // 母倉：月表四季之月土王后另有巳午，此不计，只验此所推者皆在月表
                let (hh, _) = huang_hei_dao(m, g % 12);
                assert!(has(hh), "{hh} {m} {g}");
                n[1] += 1;
                n[2] += da_sha(m, g % 12) as usize;
            }
        }
        assert_eq!(n[1], 720);
        assert_eq!(n[2], 60);
    }
}

#[cfg(test)]
mod tests_book {
    use super::*;

    const GZ: [&str; 60] = [
        "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥", "丙子", "丁丑", "戊寅", "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳", "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申", "己酉", "庚戌", "辛亥", "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
    ];

    /// 鳌头卷三婚姻四表、类编卷七逐月吉日表（录文）所列之日，逐神检其值否——书例与表尾所称之对照。
    /// `cargo test -p lunar-core --lib book_examples -- --nocapture` 打印逐条（写入 doc/folk/jiaqu.md）
    #[test]
    fn book_examples() {
        use super::super::aotou::jiri_for_day;
        use super::super::jiaqu::{jiri_entries, JiriBiao};
        // (书, 表, 月 1—12, 干支, 等)
        let mut rows: Vec<(&str, String, u8, u8, String)> = Vec::new();
        for m in 0..12u8 {
            for g in 0..60u8 {
                for r in jiri_for_day(m, g) {
                    let short = match r.table.as_str() {
                        "逐月求婚吉日" => "求婚",
                        "逐月下定結婚吉日" => "下定結婚",
                        "逐月嫁娶吉日" => "嫁娶",
                        "逐月嫁娶結婚吉日備急用" => "嫁娶備急",
                        _ => continue,
                    };
                    rows.push(("鳌头", short.to_string(), m + 1, g, r.deng.clone()));
                }
            }
        }
        for e in jiri_entries() {
            rows.push(("类编", e.biao.name().to_string(), e.month, e.gz, e.zhu.to_string()));
        }
        let hit = |f: &dyn Fn(u8, u8) -> bool, book: &str, table: &str| -> Vec<String> {
            rows.iter()
                .filter(|r| r.0 == book && r.1 == table && f(r.2 - 1, r.3))
                .map(|r| format!("{}月{}{}", r.2, GZ[r.3 as usize], if r.4 == "外" { "（外）" } else { "" }))
                .collect()
        };
        let tg = |_m: u8, g: u8| jiu_tu_gui(JiuTuGui::AoTou, g);
        // 九土鬼：鳌头求婚表三月己酉、六月壬寅、十一月壬寅；类编求婚表三月己酉（外）——表尾称不犯九土鬼，与起例不合，照录。
        //（清点篇原记「正月己酉」，据录文为三月）
        assert_eq!(hit(&tg, "鳌头", "求婚"), ["3月己酉", "6月壬寅", "11月壬寅"]);
        let lb_qh: Vec<String> = hit(&tg, "类编", JiriBiao::QiuHun.name());
        assert_eq!(lb_qh.len(), 1, "{lb_qh:?}");
        assert!(lb_qh[0].starts_with("3月己酉"));
        // 紅嘴朱雀：鳌头、类编嫁娶表六月庚寅、十二月庚寅
        let hz = |_m: u8, g: u8| hong_zui_zhu_que(g);
        assert_eq!(hit(&hz, "鳌头", "嫁娶"), ["6月庚寅", "12月庚寅"]);
        assert_eq!(hit(&hz, "类编", JiriBiao::JiaQu.name()), ["6月庚寅", "12月庚寅"]);
        // 離窠：鳌头嫁娶表 10 日值之（表尾未称避離窠）
        let lk = |_m: u8, g: u8| li_ke(g);
        let v = hit(&lk, "鳌头", "嫁娶");
        assert_eq!(v.len(), 10, "{v:?}");
        // 冰消瓦解（类编）、荒蕪（鳌头序）：两书八表无一日值之
        for f in [(|m: u8, g: u8| bing_xiao_wa_jie(BingXiao::LeiBian, m, g % 12)) as fn(u8, u8) -> bool, |m, g| huang_wu_aotou(m, g % 12)] {
            assert!(rows.iter().all(|r| !f(r.2 - 1, r.3)));
        }
        // 逐神逐表统计（打印）
        type F = fn(u8, u8) -> bool;
        let gods: [(&str, F); 14] = [
            ("大煞", |m, g| da_sha(m, g % 12)),
            ("天火（类编）", |m, g| tian_huo_leibian(m, g % 12)),
            ("黑道", |m, g| !huang_hei_dao(m, g % 12).1),
            ("九醜", |_, g| jiu_chou(g)),
            ("冰消瓦解（类编）", |m, g| bing_xiao_wa_jie(BingXiao::LeiBian, m, g % 12)),
            ("冰消瓦解（董公）", |m, g| bing_xiao_wa_jie(BingXiao::DongGong, m, g % 12)),
            ("九土鬼（鳌头）", |_, g| jiu_tu_gui(JiuTuGui::AoTou, g)),
            ("九土鬼（董公，除建破平收）", |m, g| jiu_tu_gui(JiuTuGui::DongGong, g) && !jiu_tu_gui_wu_fang(m, g % 12)),
            ("九空（类编）", |m, g| jiu_kong_leibian(m, g % 12)),
            ("紅嘴朱雀", |_, g| hong_zui_zhu_que(g)),
            ("離窠", |_, g| li_ke(g)),
            ("荒蕪（鳌头序）", |m, g| huang_wu_aotou(m, g % 12)),
            ("十二星斷破危", |m, g| shi_er_xing_duan(m, g % 12).1 == "忌"),
            ("十二星斷除日", |m, g| shi_er_xing_duan(m, g % 12).1 == "两说"),
        ];
        let tables: [(&str, &str); 8] = [
            ("鳌头", "求婚"),
            ("鳌头", "下定結婚"),
            ("鳌头", "嫁娶"),
            ("鳌头", "嫁娶備急"),
            ("类编", "求婚"),
            ("类编", "下定"),
            ("类编", "嫁娶"),
            ("类编", "嫁娶備急"),
        ];
        println!("| 神 | {} |", tables.iter().map(|(b, t)| format!("{b}{t}")).collect::<Vec<_>>().join(" | "));
        println!("|---|{}", "---|".repeat(tables.len()));
        println!("| （表日数） | {} |", tables.iter().map(|(b, t)| rows.iter().filter(|r| r.0 == *b && r.1 == *t).count().to_string()).collect::<Vec<_>>().join(" | "));
        for (name, f) in gods {
            println!("| {name} | {} |", tables.iter().map(|(b, t)| rows.iter().filter(|r| r.0 == *b && r.1 == *t && f(r.2 - 1, r.3)).count().to_string()).collect::<Vec<_>>().join(" | "));
        }
    }
}
