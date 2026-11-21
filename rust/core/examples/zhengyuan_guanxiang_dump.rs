//! 输出正元历（代推观象）807—821 逐月朔 JDN（年,月,闰,JDN），供 scripts/calendar_data/tongjian_mulu_guanxiang.py 对照
fn main() {
    for y in 807..=821 {
        for (j, m, l) in lunar_core::calendar::dingshuo::reckon_year_any("正元历（代推观象）", y).unwrap() {
            println!("{},{},{},{}", y, m, l as u8, j);
        }
    }
}
