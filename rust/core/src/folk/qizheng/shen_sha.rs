//! 七政四余神煞：以生年干支、生月、生时、命宫等起例，或化为某曜（「化曜」，其宫即该曜所躔之宫），或落于某宫（地支）。
//!
//! 起例据：《古今圖書集成·藝術典》卷五百六十八《張果星宗二·諸星起例》（下称「集成」：各神之歌与末附
//! 「天干化曜星例」「天干吉凶星例」「地支吉凶星例一至四」诸表）、卷五百六十九「月建吉凶星例」；
//! 《星學大成》（四库本，下称「大成」）卷一（變曜、文星、魁星、官星、印星、催官、祿神、喜神、天厨、歲殿、
//! 地爵、紅鸞天喜、天德月德、解神）、卷二（驛馬十二位、天馬地驛、卦氣、三元星、三魁）、卷三（駕前駕後神煞、
//! 天哭披頭、的煞、大煞、孤辰寡宿、黄幡豹尾、直難）。表与歌、两书互异处并存为选项或记于 doc/folk/qizheng.md §4.9。
//! 只算神煞所在，不作吉凶。

use super::{gong_zhu, YaoPos, ZHI_CH};

/// 十干化曜之序（《星學大成》卷一變曜歌訣「火孛木金土月水炁計羅」）：甲化祿为火，乙孛……癸羅
pub const HUA_SEQ: [usize; 10] = [4, 9, 5, 3, 6, 1, 2, 10, 8, 7];
/// 十化之名（「禄暗福耗䕃貴刑印囚權」）
pub const HUA_NAME: [&str; 10] = ["天祿", "天暗", "天福", "天耗", "天廕", "天貴", "天刑", "天印", "天囚", "天權"];

/// 曜字 → 十一曜下标（日月水金火木土羅計孛炁；陽、隂、熒等异写亦收）
pub fn yao_of(c: char) -> Option<usize> {
    Some(match c {
        '日' | '陽' => 0,
        '月' | '隂' | '陰' => 1,
        '水' => 2,
        '金' => 3,
        '火' | '熒' => 4,
        '木' => 5,
        '土' => 6,
        '羅' => 7,
        '計' => 8,
        '孛' => 9,
        '炁' | '紫' => 10,
        _ => return None,
    })
}

/// 五行（木火土金水 0—4）→ 曜（木火土金水五星）
const XING_YAO: [usize; 5] = [5, 4, 6, 3, 2];
/// 天干五行（甲乙木、丙丁火、戊己土、庚辛金、壬癸水）之曜
pub fn gan_yao(gan: u8) -> usize {
    XING_YAO[(gan % 10 / 2) as usize]
}
/// 地支五行（寅卯木、巳午火、申酉金、亥子水、辰戌丑未土）之曜。集成《天經地緯》「其論支神所屬者，即寅卯屬木……
/// 切不可以寅亥二宮屬木論也」——是支五行，不是宫主
pub fn zhi_yao(zhi: u8) -> usize {
    const X: [u8; 12] = [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4];
    XING_YAO[X[(zhi % 12) as usize] as usize]
}
/// 十干化祿之曜（甲火乙孛丙木丁金戊土己月庚水辛炁壬計癸羅）
pub fn hua_lu(gan: u8) -> usize {
    HUA_SEQ[(gan % 10) as usize]
}
/// 十干祿位（甲寅乙卯丙戊巳丁己午庚申辛酉壬亥癸子；大成卷一論禄勲）
pub fn lu_wei(gan: u8) -> u8 {
    [2, 3, 5, 6, 5, 6, 8, 9, 11, 0][(gan % 10) as usize]
}
/// 驛馬（申子辰寅、寅午戌申、巳酉丑亥、亥卯未巳；集成馬前諸殺例）
pub fn yi_ma(zhi: u8) -> u8 {
    [2, 11, 8, 5][(zhi % 4) as usize]
}
/// 五虎遁：年干 `gan` 之年，`zhi` 宫所得之干（甲己之年丙作首……）
pub fn wu_hu_dun(gan: u8, zhi: u8) -> u8 {
    let yin = (2 + 2 * (gan % 5)) % 10;
    ((yin as i32 + (zhi as i32 - 2).rem_euclid(12)) % 10) as u8
}
/// 卦氣之宫（集成「壬甲從乾乙癸坤，戊坎丙艮震居庚，辛巽己離丁兌位」；大成卷二定卦氣同）。
/// 乾取亥、坤取申（集成例「甲生人從亥上起」「乙生人……從申上起乙」），坎子、震卯、離午、兌酉；
/// 艮、巽各兼二支，依乾坤之例取后一支（艮寅、巽巳）——本项目读法，未核
pub fn gua_qi(gan: u8) -> u8 {
    [11, 8, 2, 9, 0, 6, 3, 5, 11, 8][(gan % 10) as usize]
}
/// 自卦氣宫起年干，逆（`shun` = false）或顺数至 `zhi` 所得之干
pub fn gua_qi_gan(gan: u8, zhi: u8, shun: bool) -> u8 {
    let g = gua_qi(gan) as i32;
    let n = if shun { (zhi as i32 - g).rem_euclid(12) } else { (g - zhi as i32).rem_euclid(12) };
    ((gan as i32 + n) % 10) as u8
}

/// 一个神煞
#[derive(Clone, Debug, PartialEq)]
pub struct ShenSha {
    /// 神名（用原书字）
    pub name: &'static str,
    /// 起例所据：「年干」「年支」「年干支」「命宫」「月」「月時」「年干命宫」「年干日月」
    pub by: &'static str,
    /// 化曜神煞：所化之曜（十一曜下标）；宫位神煞为 None
    pub yao: Option<usize>,
    /// 所在宫（地支序）：宫位神煞即其宫；化曜神煞为该曜所躔之宫（该曜不在盘上则 None）
    pub gong: Option<u8>,
    /// 出处（简）
    pub src: &'static str,
}

/// 魁星（正魁）异说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KuiXingRule {
    /// 庚水：《星學大成》卷一論魁星詩「己金庚水辛逢孛」、集成天干吉凶星例表「月日羅計火金水孛炁水」（缺省）
    #[default]
    DaCheng,
    /// 庚木：集成《張果星宗二·魁星》歌「己金庚木辛逢孛」（《三辰通載》卷十四木星「庚生人以為魁刑星」与此合）
    XingZongGe,
}

/// 晝夜貴人异说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GuiRenRule {
    /// 《星學大成》卷一晝貴人詩「甲戊庚兮是牛鄉，乙己鼠兮丙猪方，丁猪壬癸居蛇位，穴辛逢馬」、夜貴人詩「甲戊庚兮居羊地，
    /// 乙己在猴丙丁雞，辛遇虎穴壬癸兎」，注「依臺厯推載」（缺省，四库本）
    #[default]
    DaCheng,
    /// 集成《天乙》（晝貴）「甲見未，戊庚在丑，乙申，己子，丙酉，辛寅，丁亥，壬兔，癸巳」、《玉堂》（夜貴）
    /// 「甲見丑，戊庚在未，丁酉，丙亥，乙子，己申，壬巳，癸卯，辛午」及天干吉凶星例表
    JiCheng,
}

/// 天解异说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TianJieRule {
    /// 子年在戌，逐年逆行：《星學大成》卷三總論駕後神煞歌「血刃浮沈及解神，戌上分明」、集成駕後神煞歌与地支吉凶星例四表同；
    /// 卷三流年十二變曜原表丑年「金:太隂天解」（丑年在酉，金宫）亦合（缺省）
    #[default]
    XuNi,
    /// 子年在辰，逐年顺行：《星學大成》卷一論解神「天解每從辰上起子年挨宫順輪」
    ChenShun,
}

/// 生年干支换年之界
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NianJie {
    /// 立春（太阳视黄经 315°）。星命书未见明文，取通例（《星學大成》卷三起大運以立春为岁首之节），未核（缺省）
    #[default]
    LiChun,
    /// 冬至（270°）：《星學大成》卷三流年變曜「並從冬至日為始」（流年之说，借作生年之界，选项）
    DongZhi,
}

/// 神煞起例所需：生年干支序（甲子 = 0）、节月（0 = 寅月正月）、生时支、晝生否、命宫、各曜位置
pub struct ShenShaInput<'a> {
    pub nian_gz: u8,
    pub month: u8,
    pub hour_zhi: u8,
    pub zhou: bool,
    pub ming: u8,
    pub yao: &'a [YaoPos],
}

/// 神煞选项
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ShenShaOptions {
    pub kui_xing: KuiXingRule,
    pub gui_ren: GuiRenRule,
    pub tian_jie: TianJieRule,
}

/// 按年干取曜之表（十字，甲……癸）：（名，表，出处）。表字为曜字
const GAN_YAO: [(&str, &str, &str); 7] = [
    // 集成天干化曜星例表「天官 炁水羅計孛火金木月土」；集成、大成卷一官星歌「甲炁乙水是官星……壬月（隂）癸土」同；
    // 大成卷八注「丁用金為天元禄主，即是庚之天元禄主，一名天官星」（庚金）合
    ("天官", "炁水羅計孛火金木月土", "集成卷568天干化曜星例；大成卷一論官星"),
    ("文星", "羅計金火金炁木土日月", "大成卷一論文星詩；集成卷568文星"),
    ("印星", "木日火月土羅金計水孛", "大成卷一論印星；集成卷568印星"),
    ("催官", "金水日羅木炁孛土月計", "大成卷一論催官星；集成卷568催官"),
    // 喜神：大成卷一歌作「辛見水」，其注「喜神之序羅計炁水月土金木孛火」及集成歌、表皆辛木，从注与集成
    ("喜神", "羅計炁水月土金木孛火", "大成卷一論喜神注；集成卷568喜神"),
    ("小魁", "羅炁金火水計水土日月", "大成卷二論三魁小魁例"),
    ("科名", "木木火火土土金金水水", "大成卷一定科名星；集成卷568科名"),
];

/// 按年干取宫之表（十字）
const GAN_GONG: [(&str, &str, &str); 4] = [
    ("文昌", "巳午申酉申酉亥戌寅卯", "大成卷一論文昌；集成卷568文昌"),
    ("天廚", "巳午子巳午申寅午酉亥", "大成卷一論天厨；集成卷568天廚"),
    ("國印", "戌亥丑寅丑寅辰巳未申", "集成卷568祿勳陽刃唐符國印（祿前第九）；大成卷一「國印祿前九位」"),
    ("唐符", "酉戌子丑子丑卯辰午未", "集成卷568（祿前第八，又名飛刃）"),
];

/// 晝夜貴人表（十字，甲……癸）
const GUI_DACHENG: [&str; 2] = ["丑子亥亥丑子午午巳巳", "未申酉酉未申寅寅卯卯"];
const GUI_JICHENG: [&str; 2] = ["未申酉亥丑子丑寅卯巳", "丑子亥酉未申未午巳卯"];

/// 按年支取宫：年支加 k 顺数（駕前）。（名，k，出处）
const JIA_QIAN: [(&str, u8, &str); 15] = [
    ("歲駕", 0, "大成卷一論歲駕；集成歲駕"),
    ("天空", 1, "大成卷三定天空「太嵗前一位」"),
    ("喪門", 2, "大成卷三定喪門白虎吊客病符；駕前神煞歌"),
    ("勾神", 3, "大成卷三定勾絞「勾以子年加卯逐年順數」"),
    ("官符", 4, "大成卷三定飛符（子年加辰順）；集成年符「年支順數第五位」"),
    ("月德", 5, "大成卷一「月德子年加巳順數」"),
    ("死符", 5, "駕前神煞歌（大成卷三、集成）"),
    ("歲破", 6, "駕前神煞歌"),
    ("暴敗", 7, "大成卷三定飛符暴敗「暴敗以子年加未順數」"),
    ("天厄", 7, "大成卷三定天厄「天空對位」"),
    ("白虎", 8, "大成卷三定喪門白虎吊客病符"),
    ("天德", 9, "大成卷一「天德子年加酉順數」"),
    ("絞殺", 9, "大成卷三定勾絞「絞以子年加酉逐年順數」"),
    ("弔客", 10, "大成卷三定喪門白虎吊客病符"),
    ("病符", 11, "大成卷三定喪門白虎吊客病符"),
];
/// 天狗：駕後二辰（大成卷三定天狗），即 +10
const TIAN_GOU: (&str, u8, &str) = ("天狗", 10, "大成卷三定天狗「駕後二辰」");

/// 按年支逆数（駕後）：子年在 `start`，逐年逆行一宫。（名，子年之宫，出处）
const JIA_HOU: [(&str, u8, &str); 5] = [
    ("紅鸞", 3, "大成卷一「紅鸞子年加卯逆數」；駕後神煞歌"),
    ("天喜", 9, "大成卷一「天喜子年加酉逆數」"),
    ("血刃", 10, "大成卷三定血刃「以子年加戌逆數」"),
    ("天哭", 6, "大成卷三定天哭披頭「天哭以子年加午逆數」；集成駕後神煞歌「天哭還從午上尋」"),
    ("披頭", 4, "大成卷三「披頭以子年加辰逆數」"),
];

/// 驛馬十二位（驛馬起，顺数）中取用者：（名，距驛馬之数）。大成卷二定驛馬十二位次第「驛馬、六害、華葢、刼煞、災煞、天煞、地煞、
/// 年煞、月煞、亡神、將星、攀鞍」，集成馬前諸殺例表同。馬前六害、月煞与集成按年支之「六害」、按月之「月殺」同名異法，此不取
const MA_QIAN: [(&str, u8); 10] = [
    ("驛馬", 0),
    ("華蓋", 2),
    ("劫殺", 3),
    ("災殺", 4),
    ("天殺", 5),
    ("地殺", 6),
    ("年殺", 7),
    ("亡神", 9),
    ("將星", 10),
    ("攀鞍", 11),
];

/// 按年支取宫之表（十二字，子……亥）
const ZHI_GONG: [(&str, &str, &str); 9] = [
    ("孤辰", "寅寅巳巳巳申申申亥亥亥寅", "大成卷三孤辰寡宿歌「寅卯辰人怕巳丑……亥子丑人寅戌嗔」；集成孤辰"),
    ("寡宿", "戌戌丑丑丑辰辰辰未未未戌", "同上"),
    ("的殺", "巳丑酉巳丑酉巳丑酉巳丑酉", "大成卷三論的煞「子午夘酉的在巳，寅申巳亥的在酉，辰戌丑未的在丑」；集成的殺"),
    ("咸池", "酉午卯子酉午卯子酉午卯子", "集成咸池「申子辰雞……亥卯未鼠」"),
    ("大殺", "申酉戌巳午未寅卯辰亥子丑", "大成卷三飛㢘煞一名大煞歌；集成大煞"),
    ("黃旛", "辰丑戌未辰丑戌未辰丑戌未", "大成卷三論黄幡豹尾"),
    ("豹尾", "戌未辰丑戌未辰丑戌未辰丑", "大成卷三論黄幡豹尾"),
    ("地解", "未未申申酉酉戌戌亥亥午午", "大成卷一論解神；集成地支吉凶星例四"),
    ("六害", "未午巳辰卯寅丑子亥戌酉申", "集成六害「子未不相親，丑害午兮寅巳嗔，卯害辰兮申害亥，酉戌相穿」"),
];

/// 爵星（按年支化曜）：大成卷一論地爵五言「鼠猴元是土，猪羊火裏生，馬牛均下水，逢兎炁孤神，虎蛇皆屬木，鷄犬便逢金，龍人惟見孛」；
/// 集成爵星歌同。集成地支吉凶星例一表申作火、亥作水，与两歌不合，从歌
const JUE_XING: &str = "土水木炁孛木水火土金金火";

/// 按节月取宫之表（十二字，正月……十二月）：集成卷五百六十九月建吉凶星例（各神歌见卷五百六十八）
const YUE_GONG: [(&str, &str, &str); 6] = [
    ("月廉", "申酉戌亥子丑寅卯辰巳午未", "集成卷569月建吉凶星例；卷568月廉「正月起於申，順數至本生」"),
    ("月殺", "戌巳午未寅卯辰亥子丑申酉", "集成卷569；卷568月煞歌"),
    ("天耗", "子寅辰午申戌子寅辰午申戌", "集成卷569；卷568天耗（雷霆殺）"),
    ("地耗", "酉亥丑卯巳未酉亥丑卯巳未", "集成卷569；卷568地耗"),
    ("月符", "午未申酉戌亥子丑寅卯辰巳", "集成卷569；卷568月符「以午起為首，順數」"),
    ("注受", "子亥戌酉戌亥子丑寅卯寅丑", "集成卷569；卷568注受「正月起子，逆數至酉，五月又轉戌，順數至卯，十一月又轉寅」"),
];
/// 值難（按节月化曜）：集成卷569表「日日月月火羅水孛木炁金金」；大成卷三直難星歌「正二太陽三四隂五六火星……十一十二是金星」
/// （「一說十一十二土星」）
const ZHI_NAN: &str = "日日月月火羅水孛木炁金金";

/// 血忌（按年支化曜）：大成卷三論血忌「丑寅土，子日，夘月，辰亥木，巳戌水，午酉火，未申金」；集成地支吉凶星例一表
/// 「日土土月木水火金金火水木」同
const XUE_JI: &str = "日土土月木水火金金火水木";

/// 擎天（按生年干支取宫）：集成地支吉凶星例四表原录（「已上皆是擎天之位，遊奕對宮」），十宫各列所值之干支。
/// 录文「丙戌」重出于子、未两格而缺丙辰：依表中六十花甲以子起、卯止之循环（丙戌为一轮之首当在子），未格之「丙戌」当作「丙辰」（本项目校改，未核）
const QING_TIAN: [(char, &str); 10] = [
    ('子', "甲子庚午乙亥辛巳丙戌壬辰丁酉戊申癸丑己未"),
    ('卯', "己巳甲戌庚辰乙酉辛卯丙申壬寅丁未戊午癸亥"),
    ('辰', "庚寅辛丑壬子"),
    ('巳', "戊辰癸酉己卯甲申乙未丙午丁巳"),
    ('午', "己丑庚子戊寅辛亥壬戌"),
    ('未', "乙巳丁卯甲午丙辰癸未"),
    ('申', "丙寅壬申丁丑戊子己亥庚戌辛酉"),
    ('酉', "癸巳甲辰乙卯"),
    ('戌', "乙丑辛未丙子壬午丁亥戊戌己酉甲寅庚申"),
    ('亥', "癸卯"),
];

/// 擎天之宫（地支序）
pub fn qing_tian(nian_gz: u8) -> Option<u8> {
    const G: &str = "甲乙丙丁戊己庚辛壬癸";
    let gc = G.chars().nth((nian_gz % 10) as usize)?;
    let zc = ZHI_CH.chars().nth((nian_gz % 12) as usize)?;
    QING_TIAN.iter().find_map(|(p, t)| {
        let cs: Vec<char> = t.chars().collect();
        cs.chunks(2).any(|w| w[0] == gc && w[1] == zc).then(|| ZHI_CH.chars().position(|x| x == *p).unwrap() as u8)
    })
}

/// 与已有神煞同位之別名（集成地支吉凶星例表中「同前」者、大成卷三總論駕前神煞歌同宫并举者）：（名，年支顺数，出处）
const JIA_QIAN_ALIAS: [(&str, u8, &str); 11] = [
    ("伏尸", 0, "大成卷三總論駕前神煞歌「一太嵗劍鋒伏尸寄」；集成表太歲行「伏屍」"),
    ("貫索", 3, "大成卷三駕前神煞歌「四太隂貫索勾絞具」；集成表貫索行（勾神同前）"),
    ("五鬼", 4, "集成表五鬼行（官符、飛符、年符同前）；大成卷十三注「五鬼者五官符是也，假如甲子年五鬼在辰」"),
    ("小耗", 5, "集成表月德行（死符、小耗同前）"),
    ("大耗", 6, "集成表歲破行（大耗、闌干同前）"),
    ("闌干", 6, "大成卷三駕前神煞歌「七嵗破月空欄干是」；集成表"),
    ("龍德", 7, "大成卷三駕前神煞歌「八龍徳暴敗天厄至」；卷三流年十二神"),
    ("天雄", 8, "集成表白虎行（天雄同前）"),
    ("福星", 9, "大成卷三駕前神煞歌「十天徳福星卷舌係」"),
    ("卷舌", 9, "大成卷三駕前神煞歌；集成表天德行（卷舌、絞殺同前）"),
    ("驀越", 11, "大成卷三定驀越「駕後一位」；集成表「病符驀越」"),
];

fn zhi_at(s: &str, k: usize) -> u8 {
    let c = s.chars().nth(k).unwrap();
    ZHI_CH.chars().position(|x| x == c).unwrap() as u8
}
fn yao_at(s: &str, k: usize) -> usize {
    yao_of(s.chars().nth(k).unwrap()).unwrap()
}

/// 推神煞
pub fn shen_sha(inp: &ShenShaInput, o: &ShenShaOptions) -> Vec<ShenSha> {
    let gan = inp.nian_gz % 10;
    let zhi = inp.nian_gz % 12;
    let g = gan as usize;
    let ming = inp.ming;
    let gong_of_yao = |k: usize| inp.yao.get(k).filter(|y| y.name == super::YAO[k]).map(|y| y.gong);
    let mut v: Vec<ShenSha> = Vec::new();
    let mut yao = |name: &'static str, by: &'static str, k: usize, src: &'static str| {
        v.push(ShenSha { name, by, yao: Some(k), gong: gong_of_yao(k), src });
    };
    // 一、年干化曜
    for (k, n) in HUA_NAME.iter().enumerate() {
        yao(n, "年干", HUA_SEQ[(g + k) % 10], "大成卷一變曜歌訣「禄暗福耗䕃貴刑印囚權，火孛木金土月水炁計羅」；集成卷568變曜、天干化曜星例");
    }
    for (n, t, s) in GAN_YAO {
        yao(n, "年干", yao_at(t, g), s);
    }
    let kui = match o.kui_xing {
        KuiXingRule::DaCheng => "月日羅計火金水孛炁水",
        KuiXingRule::XingZongGe => "月日羅計火金木孛炁水",
    };
    yao("魁星", "年干", yao_at(kui, g), "大成卷一論魁星詩；集成卷568魁星（庚字异说见 KuiXingRule）");
    // 祿神：甲兼木孛（大成卷一論祿神「甲兼木孛乙水星，丙計丁羅戊土居，己火庚金辛紫炁，壬日癸月是祿神」）
    let lu_shen = "木水計羅土火金炁日月";
    yao("祿神", "年干", yao_at(lu_shen, g), "大成卷一論祿神；集成卷568祿神");
    if gan == 0 {
        yao("祿神", "年干", 9, "大成卷一論祿神「甲兼木孛」");
    }
    yao("仁元", "年干", gan_yao(gan), "大成卷二論四元「仁元者以甲乙生人屬木即木星是也」；集成天干化曜星例");
    yao("祿元", "年干", gong_zhu(lu_wei(gan)), "大成卷二論四元注「禄元者如甲生人禄在寅，寅属木即木星」（祿位宫主）");
    yao("壽元", "年干支", XING_YAO[nayin_xing(inp.nian_gz)], "大成卷二論四元注「夀元者如甲子生人納音是金即金星」；集成壽元");
    // 二、年支化曜
    yao("馬元", "年支", gong_zhu(yi_ma(zhi)), "大成卷二論四元注「馬元如申子辰馬在寅，寅屬木即木星」（驛馬宫主）");
    yao("爵星", "年支", yao_at(JUE_XING, zhi as usize), "大成卷一論地爵五言；集成卷568爵星歌");
    // 天馬地驛（諸家）：驛馬宫遁祿干，干化祿為天馬，干屬為地驛。亥卯未馬在巳，丙戊皆祿巳，大成云「木土為天馬，丙火戊土為地驛」，
    // 集成表取丙（天馬木、地驛火），从之
    let ma_gan = [0u8, 8, 6, 2][(zhi % 4) as usize];
    yao("天馬", "年支", hua_lu(ma_gan), "大成卷二論天馬地驛；集成卷568天馬地驛、地支吉凶星例一");
    yao("地驛", "年支", gan_yao(ma_gan), "同上");
    // 產星：大成卷三論産星「申子辰年金，巳酉丑年水，寅午戌年木，亥卯未年火」；集成表「金水木火」循环同
    yao("產星", "年支", [3usize, 2, 5, 4][(zhi % 4) as usize], "大成卷三論産星；集成卷568地支吉凶星例一、卷二十四「産星用二」");
    // 血支：大成卷三論血支「丑寅土，子卯木，辰亥火，巳戌金，午酉水，未日，申月，即星盤所屬而移一位也」——前一支之宫主
    yao("血支", "年支", gong_zhu((zhi + 11) % 12), "大成卷三論血支（星盤宫主移一位）；集成卷568地支吉凶星例一");
    yao("血忌", "年支", yao_at(XUE_JI, zhi as usize), "大成卷三論血忌；集成卷568地支吉凶星例一");
    // 三、命宫起
    let ju = [4usize, 3, 1, 0][(ming % 4) as usize]; // 三合局五行：申子辰水、巳酉丑金、寅午戌火、亥卯未木
    yao("天元印", "命宫", XING_YAO[ju], "大成卷二定三元星「亥夘未為命宫者木為天元印火為地元禄」……；卷二十三印夀雙全「亥屬木故木為天元印，木生火故火為地元禄，火生土故土為人元夀」");
    yao("地元祿", "命宫", XING_YAO[(ju + 1) % 5], "大成卷二定三元星");
    yao("人元壽", "命宫", XING_YAO[(ju + 2) % 5], "大成卷二定三元星");
    yao("科甲", "命宫", gong_zhu((ming + 6) % 12), "集成卷568科甲「假令寅宮安命，與申相對，申宮屬水，即取水星為科甲」；大成卷一定科甲星");
    // 四、年干＋命宫（《張果星宗》，集成卷568、卷569諸吉星捷法例）
    let ming_gan = wu_hu_dun(gan, ming);
    yao("天元祿", "年干命宫", hua_lu(ming_gan), "集成卷568天元祿「以年干五虎遁，順數至命宮得何干，以此干化祿」");
    yao("地元祿", "年干命宫", gan_yao(gua_qi_gan(gan, ming, false)), "集成卷568地元祿「以年干所管卦氣，逆數至命宮，得何干，以此干屬某」");
    let guan = (ming + 3) % 12; // 官祿宫（命宫逆布第十）
    yao("人元祿", "年干命宫", XING_YAO[(gan_xing(wu_hu_dun(gan, guan)) + 3) % 5], "集成卷568人元祿「以年干起五虎遁，順數至官祿，得何干，以此干受剋」");
    yao("天經", "年干命宫", gan_yao(ming_gan), "集成卷568天經地緯");
    yao("地緯", "命宫", zhi_yao(ming), "集成卷568天經地緯「以地支屬某為地緯」");
    let zhi_gan = gua_qi_gan(gan, ming, true);
    yao("職元", "年干命宫", hua_lu(zhi_gan), "集成卷568職元「以年干起卦氣，順數至命宮，得何干，以此干化祿」（书例「在寅得丁，即以丁火為職元」作干屬，与「化祿」不合，从规则）");
    yao("局主", "年干命宫", hua_lu((zhi_gan + 5) % 10), "集成卷568局主「以職元干所合之干，以此干化祿」");
    yao("天馬", "年干命宫", zhi_yao(lu_wei(wu_hu_dun(gan, guan))), "集成卷568天馬地驛（果老）「取干之祿為天馬」（祿支之屬）");
    yao("地驛", "命宫", zhi_yao(yi_ma(guan)), "集成卷568天馬地驛（果老）「取官支馬為地驛」");
    // 五、按月
    let m = (inp.month % 12) as usize;
    yao("值難", "月", yao_at(ZHI_NAN, m), "集成卷569月建吉凶星例；大成卷三直難星歌");

    // 宫位神煞
    let mut gong = |name: &'static str, by: &'static str, z: u8, src: &'static str| {
        v.push(ShenSha { name, by, yao: None, gong: Some(z % 12), src });
    };
    gong("祿勳", "年干", lu_wei(gan), "大成卷一論禄勲「甲在寅乙在卯丙戊在巳……」；集成祿勳");
    gong("陽刃", "年干", lu_wei(gan) + 1, "大成卷三陽刃煞「禄前一位為陽刃」；集成羊刃");
    for (n, t, s) in GAN_GONG {
        gong(n, "年干", zhi_at(t, g), s);
    }
    let gui = match o.gui_ren {
        GuiRenRule::DaCheng => GUI_DACHENG,
        GuiRenRule::JiCheng => GUI_JICHENG,
    };
    gong("天乙", "年干", zhi_at(gui[0], g), "晝貴人：大成卷一晝貴人詩／集成天乙（见 GuiRenRule）");
    gong("玉堂", "年干", zhi_at(gui[1], g), "夜貴人：大成卷一夜貴人詩／集成玉堂（见 GuiRenRule）");
    gong("卦氣", "年干", gua_qi(gan), "大成卷二定卦氣；集成卷568卦氣");
    gong("歲殿", "年干支", zhi + gan, "大成卷一論歲殿「丙辰生人從辰上起甲數至午見丙便是歲殿」");
    // 空亡：旬空二支，陽年空陽、陰年空陰（大成卷三定空亡；集成空亡）；孤虛為空亡對宫（集成孤虛）
    let xun = (zhi as i32 - gan as i32).rem_euclid(12) as u8; // 旬首之支
    let kong = (xun + 10 + gan % 2) % 12;
    gong("空亡", "年干支", kong, "大成卷三定空亡「陽年空陽隂年空隂」；集成空亡");
    gong("孤虛", "年干支", kong + 6, "集成孤虛「空亡對宮，即是孤虛也」");
    // 天祿卦氣：卦氣逆數至晝日夜月之宫，得干之祿宫（集成卷568卦氣〈此為天祿卦氣與前不同〉）
    let rm = if inp.zhou { gong_of_yao(0) } else { gong_of_yao(1) };
    if let Some(rm) = rm {
        gong("天祿卦氣", "年干日月", lu_wei(gua_qi_gan(gan, rm, false)), "集成卷568卦氣「以年干起卦氣，逆數至晝日夜月之宮得何干，以此干祿宮為是」");
    }
    for &(n, k, s) in JIA_QIAN.iter().chain(core::iter::once(&TIAN_GOU)) {
        gong(n, "年支", zhi + k, s);
    }
    for (n, s0, s) in JIA_HOU {
        gong(n, "年支", (s0 as i32 - zhi as i32).rem_euclid(12) as u8, s);
    }
    for (n, k, s) in JIA_QIAN_ALIAS {
        gong(n, "年支", zhi + k, s);
    }
    gong("浮沉", "年支", (10 - zhi as i32).rem_euclid(12) as u8, "大成卷三總論駕後神煞歌「血刃浮沈及解神，戌上分明」；集成表浮沉");
    // 三刑：子卯相刑，丑戌未、寅巳申轉轉相刑，辰午酉亥自刑（大成卷三三刑歌注「寅刑巳巳刑申申刑寅」；卷十三注同）。
    // 集成表「卯戌巳午辰申午丑寅酉未亥」卯年作午，与通例子不合，不从
    gong("三刑", "年支", [3u8, 10, 5, 0, 4, 8, 6, 1, 2, 9, 7, 11][zhi as usize], "大成卷三三刑歌；集成卷568三刑、地支吉凶星例三");
    // 擎天、遊奕：生年干支查集成表；遊奕為擎天對宫（大成卷三論擎天遊奕「擎天屬火遊奕屬水，數以六十花甲隔節布於十宫，起於子終於夘」）
    if let Some(q) = qing_tian(inp.nian_gz) {
        gong("擎天", "年干支", q, "集成卷568地支吉凶星例四擎天表；大成卷三論擎天遊奕");
        gong("遊奕", "年干支", q + 6, "同上「遊奕對宮」");
    }
    let tj = match o.tian_jie {
        TianJieRule::XuNi => (10 - zhi as i32).rem_euclid(12) as u8,
        TianJieRule::ChenShun => (4 + zhi) % 12,
    };
    gong("天解", "年支", tj, "大成卷三駕後神煞歌、集成（戌逆）／大成卷一論解神（辰順），见 TianJieRule");
    let ma = yi_ma(zhi);
    for (n, k) in MA_QIAN {
        gong(n, "年支", ma + k, "大成卷二定驛馬十二位；集成卷568馬前諸殺例、劫殺、亡神");
    }
    for (n, t, s) in ZHI_GONG {
        gong(n, "年支", zhi_at(t, zhi as usize), s);
    }
    for (n, t, s) in YUE_GONG {
        gong(n, "月", zhi_at(t, m), s);
    }
    // 斗標：以戌時加月建，順數至生時（集成卷568斗標「且如正月卯時生，正月建寅，以戌時加在寅宮，順數至未」）
    let yue_jian = (m as u8 + 2) % 12;
    gong("斗標", "月時", (yue_jian as i32 + inp.hour_zhi as i32 - 10).rem_euclid(12) as u8, "集成卷568斗標、卷569");
    v
}

/// 天干五行序（木0 火1 土2 金3 水4）
fn gan_xing(gan: u8) -> usize {
    (gan % 10 / 2) as usize
}

/// 年干支之納音五行（木0 火1 土2 金3 水4）
pub fn nayin_xing(gz: u8) -> usize {
    use crate::folk::nayin::{nayin_wuxing, WuXing};
    match nayin_wuxing(gz) {
        WuXing::Mu => 0,
        WuXing::Huo => 1,
        WuXing::Tu => 2,
        WuXing::Jin => 3,
        WuXing::Shui => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(c: char) -> u8 {
        ZHI_CH.chars().position(|x| x == c).unwrap() as u8
    }
    const GAN: &str = "甲乙丙丁戊己庚辛壬癸";

    /// 化曜规则对集成卷568「天干化曜星例」十行原表（10 × 10 = 100 项）
    #[test]
    fn hua_yao_vs_jicheng_table() {
        let rows = [
            "火孛木金土月水炁計羅",
            "孛木金土月水炁計羅火",
            "木金土月水炁計羅火孛",
            "金土月水炁計羅火孛木",
            "土月水炁計羅火孛木金",
            "月水炁計羅火孛木金土",
            "水炁計羅火孛木金土月",
            "炁計羅火孛木金土月水",
            "計羅火孛木金土月水炁",
            "羅火孛木金土月水炁計",
        ];
        let mut ok = 0;
        for (k, r) in rows.iter().enumerate() {
            for g in 0..10 {
                if yao_at(r, g) == HUA_SEQ[(g + k) % 10] {
                    ok += 1;
                }
            }
        }
        assert_eq!(ok, 100);
    }

    /// 《星學大成》三辰通載各曜「某生人以為某星」（卷十四木、十五火、十六土金、十七水、十九太隂紫炁、二十月孛、
    /// 二十一羅㬋、二十二計都）对化曜规则；其中「魁」「催官」另对魁星三说与催官表
    #[test]
    fn hua_yao_vs_san_chen_tong_zai() {
        // （曜，原文；已去注记符号，「巳生人」即己，「戊主人」即戊）
        let texts: [(usize, &str); 10] = [
            (5, "丙禄主甲福乙暗丁權戊囚己䕃庚魁刑辛貴壬印癸正耗魁"),
            (4, "甲禄主乙權丙囚丁印戊正魁刑己貴庚䕃辛耗壬福癸暗"),
            (6, "戊禄元甲䕃乙耗丙福丁暗己權庚囚辛印壬刑癸貴"),
            (3, "丁禄主甲耗乙福丙暗戊魁權己囚庚印辛刑壬貴癸䕃"),
            (2, "庚禄主甲刑乙貴丙䕃丁耗戊福己暗辛權壬囚癸印"),
            (1, "己祿主甲貴正魁乙印丙耗丁福戊暗庚權辛囚壬䕃癸魁刑"),
            (10, "辛禄主甲䕃乙魁刑丙貴丁印戊耗己權庚暗壬正福魁癸囚"),
            (9, "乙祿主甲暗丙權丁囚戊印己刑庚催官貴辛䕃壬耗癸福"),
            (7, "癸禄主甲魁權乙囚丙印丁催官刑戊貴己䕃庚耗辛福壬暗"),
            (8, "壬禄主甲囚乙䕃丙刑丁正魁戊印己耗庚權辛暗癸催官福"),
        ];
        let hua_ch = ['禄', '暗', '福', '耗', '䕃', '貴', '刑', '印', '囚', '權'];
        let kui3 = ["月日羅計火金水孛炁水", "羅炁金火水計水土日月", "月日羅計火金木孛炁水"];
        let mut n = 0;
        let mut ok = 0;
        let mut bad = Vec::new();
        let mut kui = Vec::new();
        for (y, t) in texts {
            let cs: Vec<char> = t.chars().collect();
            let mut i = 0;
            while i < cs.len() {
                let g = GAN.chars().position(|c| c == cs[i]).unwrap();
                let mut j = i + 1;
                while j < cs.len() && !GAN.contains(cs[j]) {
                    j += 1;
                }
                let label: String = cs[i + 1..j].iter().collect();
                n += 1;
                let k = (HUA_SEQ.iter().position(|&x| x == y).unwrap() + 10 - g) % 10;
                if label.contains(hua_ch[k]) || (k == 0 && label.contains('祿')) {
                    ok += 1;
                } else {
                    bad.push(format!("{}{}{}(規則{})", super::super::YAO[y], &GAN[g * 3..g * 3 + 3], label, HUA_NAME[k]));
                }
                if label.contains('魁') {
                    let hit: Vec<&str> = ["正魁", "小魁", "星宗歌"].iter().zip(kui3).filter(|(_, t)| yao_at(t, g) == y).map(|(a, _)| *a).collect();
                    kui.push(format!("{}{}:{}", super::super::YAO[y], &GAN[g * 3..g * 3 + 3], if hit.is_empty() { "无".to_string() } else { hit.join("、") }));
                }
                if label.contains("催官") {
                    assert_eq!(yao_at("金水日羅木炁孛土月計", g), y);
                }
                i = j;
            }
        }
        eprintln!("三辰通載化曜：{ok}/{n}，不合 {:?}；魁：{:?}", bad, kui);
        assert_eq!(n, 100);
        // 不合 13：印、䕃互易 8 处（木己壬、月乙壬、炁甲丁、計乙戊），福、權互易 4 处（炁己壬、計庚癸），計丁只记「正魁」（规则化貴）。
        // 土、金、水、火、月孛、羅㬋六曜六十项全合
        assert_eq!(ok, 87, "{bad:?}");
    }

    /// 天干诸表：集成表、集成歌、大成卷一歌逐干对勘
    #[test]
    fn gan_tables_vs_songs() {
        // 集成卷568天干吉凶星例表（照录）与本模块所用之表
        let jc_table = [
            ("文星", "羅計金火金炁木土日月"),
            ("魁星", "月日羅計火金水孛炁水"),
            ("官星", "炁水羅計孛火金木月土"),
            ("印星", "木日火月土羅金計水孛"),
            ("催官", "金水日羅木炁孛土月計"),
            ("祿神", "木水計羅土火金炁日月"),
            ("喜神", "羅計炁水月土金木孛火"),
            ("科名", "木木火火土土金金水水"),
        ];
        for (n, t) in jc_table {
            let ours = GAN_YAO.iter().find(|x| x.0 == if n == "官星" { "天官" } else { n }).map(|x| x.1);
            match n {
                "魁星" | "祿神" => {}
                _ => assert_eq!(ours, Some(t), "{n}"),
            }
        }
        // 宫位表：祿勳、陽刃、國印、唐符、文昌、天廚
        let lu = "寅卯巳午巳午申酉亥子";
        let ren = "卯辰午未午未酉戌子丑";
        for g in 0..10u8 {
            assert_eq!(lu_wei(g), zhi_at(lu, g as usize));
            assert_eq!((lu_wei(g) + 1) % 12, zhi_at(ren, g as usize));
            assert_eq!((lu_wei(g) + 7) % 12, zhi_at(GAN_GONG[3].1, g as usize)); // 唐符：祿前第八（含本位）
            assert_eq!((lu_wei(g) + 8) % 12, zhi_at(GAN_GONG[2].1, g as usize)); // 國印：祿前第九
            assert_eq!(gan_yao(g), yao_at("木木火火土土金金水水", g as usize));
        }
        // 晝夜貴人：集成表与集成歌合；大成为另一系
        assert_eq!(GUI_JICHENG[0], "未申酉亥丑子丑寅卯巳");
    }

    /// 年支诸神：集成地支吉凶星例一至四原表对本模块规则（駕前顺数、駕後逆数、驛馬十二位等）
    #[test]
    fn zhi_tables_vs_rules() {
        let inp_yao: Vec<YaoPos> = Vec::new();
        let table: [(&str, &str); 25] = [
            ("天空", "丑寅卯辰巳午未申酉戌亥子"),
            ("喪門", "寅卯辰巳午未申酉戌亥子丑"),
            ("勾神", "卯辰巳午未申酉戌亥子丑寅"),
            ("官符", "辰巳午未申酉戌亥子丑寅卯"),
            ("月德", "巳午未申酉戌亥子丑寅卯辰"),
            ("歲破", "午未申酉戌亥子丑寅卯辰巳"),
            ("暴敗", "未申酉戌亥子丑寅卯辰巳午"),
            ("白虎", "申酉戌亥子丑寅卯辰巳午未"),
            ("天德", "酉戌亥子丑寅卯辰巳午未申"),
            ("天狗", "戌亥子丑寅卯辰巳午未申酉"),
            ("病符", "亥子丑寅卯辰巳午未申酉戌"),
            ("的殺", "巳丑酉巳丑酉巳丑酉巳丑酉"),
            ("咸池", "酉午卯子酉午卯子酉午卯子"),
            ("大殺", "申酉戌巳午未寅卯辰亥子丑"),
            ("孤辰", "寅寅巳巳巳申申申亥亥亥寅"),
            ("寡宿", "戌戌丑丑丑辰辰辰未未未戌"),
            ("六害", "未午巳辰卯寅丑子亥戌酉申"),
            ("劫殺", "巳寅亥申巳寅亥申巳寅亥申"),
            ("亡神", "亥申巳寅亥申巳寅亥申巳寅"),
            ("紅鸞", "卯寅丑子亥戌酉申未午巳辰"),
            ("天喜", "酉申未午巳辰卯寅丑子亥戌"),
            ("血刃", "戌酉申未午巳辰卯寅丑子亥"),
            ("天解", "戌酉申未午巳辰卯寅丑子亥"),
            ("天哭", "午巳辰卯寅丑子寅戌酉申未"),
            ("披頭", "辰卯寅丑子亥戌酉申未午巳"),
        ];
        let mut bad = Vec::new();
        let mut n = 0;
        for zhi in 0..12u8 {
            // 取一个年支为 zhi 的干支（干任取，与此表无关）
            let gz = (0..60u8).find(|g| g % 12 == zhi).unwrap();
            let inp = ShenShaInput { nian_gz: gz, month: 0, hour_zhi: 0, zhou: true, ming: 0, yao: &inp_yao };
            let v = shen_sha(&inp, &ShenShaOptions::default());
            for (name, t) in table {
                n += 1;
                let got = v.iter().find(|s| s.name == name && s.by == "年支").and_then(|s| s.gong).unwrap();
                if got != zhi_at(t, zhi as usize) {
                    bad.push(format!("{name}{}年:表{}規則{}", &ZHI_CH[zhi as usize * 3..zhi as usize * 3 + 3], t.chars().nth(zhi as usize).unwrap(), &ZHI_CH[got as usize * 3..got as usize * 3 + 3]));
                }
            }
        }
        eprintln!("地支表对规则：{}/{n} 合，不合 {:?}", n - bad.len(), bad);
        // 唯天哭未年一格（表寅，规则亥）不合
        assert_eq!(bad.len(), 1, "{bad:?}");
        // 爵星：集成表「土水木炁孛木水火火金金水」对两歌，申、亥二格不合
        let jc = "土水木炁孛木水火火金金水";
        let diff: Vec<usize> = (0..12).filter(|&k| yao_at(jc, k) != yao_at(JUE_XING, k)).collect();
        assert_eq!(diff, vec![8, 11]);
        // 天馬、地驛（諸家）对集成地支吉凶星例一表
        for zhi in 0..12u8 {
            let mg = [0u8, 8, 6, 2][(zhi % 4) as usize];
            assert_eq!(hua_lu(mg), yao_at("火計水木火計水木火計水木", zhi as usize));
            assert_eq!(gan_yao(mg), yao_at("木水金火木水金火木水金火", zhi as usize));
        }
        // 月建表（集成卷569）：歌（卷568）推出者与表同
        for m in 0..12usize {
            assert_eq!(zhi_at(YUE_GONG[0].1, m), ((8 + m) % 12) as u8); // 月廉：正月申順
            assert_eq!(zhi_at(YUE_GONG[4].1, m), ((6 + m) % 12) as u8); // 月符：正月午順
        }
    }

    /// 年干支：壽元納音（集成壽元表五组各十二干支）、空亡（集成空亡表）
    #[test]
    fn nian_gz_tables() {
        let shou = [
            ("甲子乙丑壬申癸酉庚辰辛巳甲午乙未壬寅癸卯庚戌辛亥", 3usize),
            ("丙寅丁卯甲戌乙亥戊子己丑丙申丁酉甲辰乙巳戊午己未", 1),
            ("戊辰己巳壬午癸未庚寅辛卯戊戌己亥壬子癸丑庚申辛酉", 0),
            ("庚午辛未戊寅己卯丙戌丁亥庚子辛丑戊申己酉丙辰丁巳", 2),
            ("丙子丁丑甲申乙酉壬辰癸巳丙午丁未甲寅乙卯壬戌癸亥", 4),
        ];
        let gz_of = |a: char, b: char| {
            let g = GAN.chars().position(|c| c == a).unwrap() as u8;
            let zz = z(b);
            (0..60u8).find(|k| k % 10 == g && k % 12 == zz).unwrap()
        };
        for (s, x) in shou {
            let cs: Vec<char> = s.chars().collect();
            for p in cs.chunks(2) {
                assert_eq!(nayin_xing(gz_of(p[0], p[1])), x, "{}{}", p[0], p[1]);
            }
        }
        // 空亡：甲子丙寅戊辰庚午壬申戌、乙丑……亥；甲戌旬申酉；甲申旬午未；甲午旬辰巳；甲辰旬寅卯；甲寅旬子丑
        let yao: Vec<YaoPos> = Vec::new();
        for (gz, k) in [("甲子", '戌'), ("乙丑", '亥'), ("壬申", '戌'), ("甲戌", '申'), ("乙亥", '酉'), ("甲午", '辰'), ("癸亥", '丑'), ("甲寅", '子')] {
            let c: Vec<char> = gz.chars().collect();
            let inp = ShenShaInput { nian_gz: gz_of(c[0], c[1]), month: 0, hour_zhi: 0, zhou: true, ming: 0, yao: &yao };
            let v = shen_sha(&inp, &ShenShaOptions::default());
            assert_eq!(v.iter().find(|s| s.name == "空亡").unwrap().gong, Some(z(k)), "{gz}");
        }
    }

    /// 书中举例：三元（大成卷二表、卷二十三规则、三辰通載各曜所言）、《張果星宗》諸吉星例（集成卷568）
    #[test]
    fn book_examples() {
        let yao: Vec<YaoPos> = Vec::new();
        let gz_of = |a: char, b: char| {
            let g = GAN.chars().position(|c| c == a).unwrap() as u8;
            let zz = z(b);
            (0..60u8).find(|k| k % 10 == g && k % 12 == zz).unwrap()
        };
        let get = |v: &[ShenSha], n: &str, by: &str| v.iter().find(|s| s.name == n && s.by == by).unwrap().clone();
        // 三元：大成卷二定三元星四组
        for (m, t) in [('亥', "木火土"), ('寅', "火土金"), ('申', "水木火"), ('酉', "金水木")] {
            let inp = ShenShaInput { nian_gz: 0, month: 0, hour_zhi: 0, zhou: true, ming: z(m), yao: &yao };
            let v = shen_sha(&inp, &ShenShaOptions::default());
            let c: Vec<char> = t.chars().collect();
            assert_eq!(get(&v, "天元印", "命宫").yao, yao_of(c[0]));
            assert_eq!(get(&v, "地元祿", "命宫").yao, yao_of(c[1]));
            assert_eq!(get(&v, "人元壽", "命宫").yao, yao_of(c[2]));
        }
        // 三辰通載：木（卷十四）亥卯未天元印、申子辰地元禄、巳酉丑人元綬；火（卷十五）亥卯未地元禄、寅午戌天元印、申子辰人元壽
        let san = |m: char| {
            let inp = ShenShaInput { nian_gz: 0, month: 0, hour_zhi: 0, zhou: true, ming: z(m), yao: &yao };
            let v = shen_sha(&inp, &ShenShaOptions::default());
            [get(&v, "天元印", "命宫").yao.unwrap(), get(&v, "地元祿", "命宫").yao.unwrap(), get(&v, "人元壽", "命宫").yao.unwrap()]
        };
        assert_eq!(san('卯')[0], 5);
        assert_eq!(san('子')[1], 5);
        assert_eq!(san('酉')[2], 5);
        assert_eq!(san('未')[1], 4);
        assert_eq!(san('午')[0], 4);
        assert_eq!(san('辰')[2], 4);
        // 卷十六土「申子辰安命……以此為天元印」：本表申子辰天元印为水，不合（土为寅午戌地元禄）
        assert_ne!(san('子')[0], 6);
        // 《張果星宗》（集成卷568）甲生人寅上立命：天元祿木、地元祿水、人元祿木、天經火、地緯木、局主計、天馬（果老）火、地驛（果老）水
        let inp = ShenShaInput { nian_gz: gz_of('甲', '子'), month: 0, hour_zhi: 3, zhou: true, ming: z('寅'), yao: &yao };
        let v = shen_sha(&inp, &ShenShaOptions::default());
        assert_eq!(get(&v, "天元祿", "年干命宫").yao, Some(5));
        assert_eq!(get(&v, "地元祿", "年干命宫").yao, Some(2));
        assert_eq!(get(&v, "人元祿", "年干命宫").yao, Some(5));
        assert_eq!(get(&v, "天經", "年干命宫").yao, Some(4));
        assert_eq!(get(&v, "地緯", "命宫").yao, Some(5));
        assert_eq!(get(&v, "局主", "年干命宫").yao, Some(8));
        assert_eq!(get(&v, "天馬", "年干命宫").yao, Some(4));
        assert_eq!(get(&v, "地驛", "命宫").yao, Some(2));
        // 職元：顺数得丁（书例合），丁化祿为金；书作「丁火」（干屬），不合
        assert_eq!(gua_qi_gan(0, z('寅'), true), 3);
        assert_eq!(get(&v, "職元", "年干命宫").yao, Some(3));
        // 科甲：寅宫安命，申宫屬水
        assert_eq!(get(&v, "科甲", "命宫").yao, Some(2));
        // 斗標：正月卯時生 → 未
        assert_eq!(get(&v, "斗標", "月時").gong, Some(z('未')));
        // 天祿卦氣：甲生人太陽在酉 → 巳；乙生人太陰在卯 → 申
        assert_eq!(lu_wei(gua_qi_gan(0, z('酉'), false)), z('巳'));
        assert_eq!(lu_wei(gua_qi_gan(1, z('卯'), false)), z('申'));
        // 歲殿：丙辰生人 → 午
        let inp = ShenShaInput { nian_gz: gz_of('丙', '辰'), ..inp };
        assert_eq!(get(&shen_sha(&inp, &ShenShaOptions::default()), "歲殿", "年干支").gong, Some(z('午')));
        // 定小限例之甲子生：驛馬寅；劫殺巳、亡神亥、華蓋辰、將星子（申子辰）
        let inp = ShenShaInput { nian_gz: 0, ..inp };
        let v = shen_sha(&inp, &ShenShaOptions::default());
        for (n, c) in [("驛馬", '寅'), ("劫殺", '巳'), ("亡神", '亥'), ("華蓋", '辰'), ("將星", '子'), ("攀鞍", '丑')] {
            assert_eq!(get(&v, n, "年支").gong, Some(z(c)), "{n}");
        }
    }

    /// 宫主所命之星：《三辰通載》各曜「坐命在某者以此為科甲／官星」「安身在某者以此為學堂」「坐命在某以此星為天厨禄主」
    #[test]
    fn gong_zhu_named_stars() {
        // 科甲 = 命对宫之主
        for (m, y) in [('巳', 5), ('申', 5), ('午', 6), ('未', 6), ('卯', 3), ('戌', 3), ('寅', 2), ('亥', 2), ('子', 0), ('丑', 1)] {
            assert_eq!(gong_zhu((z(m) + 6) % 12), y, "科甲 {m}");
        }
        // 官星（三辰通載之义）= 官祿宫（命宫逆布第十，支 = 命 + 3）之主
        for (m, y) in [('酉', 6), ('戌', 6), ('午', 3), ('丑', 3), ('卯', 0), ('辰', 1)] {
            assert_eq!(gong_zhu((z(m) + 3) % 12), y, "官星 {m}");
        }
        // 學堂 = 身宫主
        for (s, y) in [('寅', 5), ('亥', 5), ('酉', 3), ('辰', 3), ('巳', 2), ('申', 2), ('午', 0)] {
            assert_eq!(gong_zhu(z(s)), y, "學堂 {s}");
        }
        // 天厨禄主 = 第八宫（疾厄，支 = 命 − 7）之主：卷十七水「坐命在夘子者以此星為天厨禄主」
        for m in ['卯', '子'] {
            assert_eq!(gong_zhu((z(m) + 5) % 12), 2);
        }
    }

    /// 產星、血支、血忌：大成卷三规则对集成地支吉凶星例一表；擎天、遊奕表之覆盖；三刑；別名同位
    #[test]
    fn chan_xue_qing_tian() {
        let none: Vec<YaoPos> = Vec::new();
        let run = |gz: u8| shen_sha(&ShenShaInput { nian_gz: gz, month: 0, hour_zhi: 0, zhou: true, ming: 0, yao: &none }, &ShenShaOptions::default());
        let chan = "金水木火金水木火金水木火";
        let xue_zhi = "木土土木火金水日月水金火";
        let xue_ji = "日土土月木水火金金火水木";
        let mut sanxing_bad = Vec::new();
        for zhi in 0..12u8 {
            let gz = (0..60u8).find(|g| g % 12 == zhi).unwrap();
            let v = run(gz);
            let y = |n: &str| v.iter().find(|s| s.name == n).and_then(|s| s.yao).unwrap();
            assert_eq!(y("產星"), yao_at(chan, zhi as usize), "產星 {zhi}");
            assert_eq!(y("血支"), yao_at(xue_zhi, zhi as usize), "血支 {zhi}");
            assert_eq!(y("血忌"), yao_at(xue_ji, zhi as usize), "血忌 {zhi}");
            let g = |n: &str| v.iter().find(|s| s.name == n).and_then(|s| s.gong).unwrap();
            if g("三刑") != zhi_at("卯戌巳午辰申午丑寅酉未亥", zhi as usize) {
                sanxing_bad.push(zhi);
            }
            assert_eq!(g("貫索"), g("勾神"));
            assert_eq!(g("五鬼"), g("官符"));
            assert_eq!(g("浮沉"), g("血刃"));
        }
        assert_eq!(sanxing_bad, vec![3]); // 集成表卯年作午
        // 擎天：六十甲子各得一宫，不入丑寅；遊奕不入未申
        let mut cnt = [0u32; 12];
        for gz in 0..60u8 {
            let q = qing_tian(gz).expect("擎天表缺");
            assert!(q != 1 && q != 2);
            cnt[q as usize] += 1;
            let v = run(gz);
            let g = |n: &str| v.iter().find(|s| s.name == n).and_then(|s| s.gong).unwrap();
            assert_eq!(g("遊奕"), (q + 6) % 12);
        }
        assert_eq!(cnt.iter().sum::<u32>(), 60);
        assert_eq!(cnt, [10, 0, 0, 10, 3, 7, 5, 5, 7, 3, 9, 1]);
    }
}
