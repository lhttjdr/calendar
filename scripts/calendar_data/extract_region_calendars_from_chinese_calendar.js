#!/usr/bin/env node
/**
 * 从 ytliu0/ChineseCalendar 导出主表以外各政权的朔闰（推步）与年号，写入 vendor/calendar/chinese_calendar/：
 *   分政权朔日表.csv：region, gregorian_year, gregorian_month, gregorian_day, lunar_year, lunar_month, is_leap
 *   分政权年号表.csv：region, lunar_year, era_trad, era_simp
 * 只运行上游代码导出数据（朔日为史实推算结果），不复制上游代码（GPL-3.0）。导出方式同 extract_shuori_from_chinese_calendar.js。
 *
 * 政权（上游 region 名）与年份范围取上游页面的显示范围：
 *   Shu 蜀 221–263（四分历）；Wu 吴 222–280（乾象历等）；LaterQin 后秦 384–417；NorthernLiang 北凉 412–439；
 *   WeiZhouSui 北魏—西魏—北周—隋 386–590；WeiQi 东魏—北齐 534–577。
 * 上游 setup_region_calendar(region, y) 返回：[0] 岁 y，[1..12] 正月至十二月朔（距 y−1 年 12 月 31 日正午 JDN 的日数），
 *   [13] 闰月朔、[14] 闰月月序（闰在该月之后；0 为无闰）、[15] 岁日数。日期 1582 年前为儒略历。
 *
 * 用法：CALENDAR_SRC=<ChineseCalendar/src 目录> node scripts/calendar_data/extract_region_calendars_from_chinese_calendar.js
 */
const fs = require("fs");
const path = require("path");

const src = process.env.CALENDAR_SRC || path.join(__dirname, "../../../ChineseCalendar/src");
const outDir = path.join(__dirname, "../../vendor/calendar/chinese_calendar");

const REGIONS = [
  ["Shu", 221, 263],
  ["Wu", 222, 280],
  ["LaterQin", 384, 417],
  ["NorthernLiang", 412, 439],
  ["WeiZhouSui", 386, 590],
  ["WeiQi", 534, 577],
];

// 上游代码只在页面处理函数里用 document；导出用不到，给一个空壳
const documentStub = {
  getElementById: () => ({ style: {}, classList: { contains: () => false, add() {}, remove() {} }, innerHTML: "" }),
};

function load(file, names) {
  const code = fs.readFileSync(path.join(src, file), "utf8") + `\nreturn { ${names.join(", ")} };`;
  return new Function("document", code)(documentStub);
}

// JDN → 儒略历 (y, m, d)（本表只到 590 年，均在 1582 年前）
function jdnToJulian(j) {
  const c = j + 32082;
  const d = Math.floor((4 * c + 3) / 1461);
  const e = c - Math.floor((1461 * d) / 4);
  const m = Math.floor((5 * e + 2) / 153);
  return [d - 4800 + Math.floor(m / 10), m + 3 - 12 * Math.floor(m / 10), e - Math.floor((153 * m + 2) / 5) + 1];
}

function main() {
  const { setup_region_calendar } = load("split.js", ["setup_region_calendar"]);
  const { eraName, eraNameSim } = load("eras.js", ["eraName", "eraNameSim"]);
  const rows = ["region,gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap"];
  const eras = ["region,lunar_year,era_trad,era_simp"];
  for (const [region, y0, y1] of REGIONS) {
    for (let y = y0; y <= y1; y++) {
      const accleap = Math.floor(0.25 * (y + 799) + 1e-5);
      const jdc = 1429223 + accleap + 365 * (y + 799); // y−1 年 12 月 31 日正午的 JDN（同上游约定）
      const cm = setup_region_calendar(region, y, false);
      if (cm[0] !== y) throw new Error(`上游返回格式不符：${region} ${y}`);
      const months = [];
      for (let i = 1; i <= 12; i++) months.push([cm[i], i, 0]);
      if (cm[14] > 0) months.push([cm[13], cm[14], 1]);
      months.sort((a, b) => a[0] - b[0]);
      for (const [off, m, leap] of months) {
        const [gy, gm, gd] = jdnToJulian(jdc + off);
        rows.push([region, gy, gm, gd, y, m, leap].join(","));
      }
      const clean = (s) => (s || "").replace(/<[^>]*>/g, "").replace(/,/g, "，").trim();
      eras.push([region, y, clean(eraName(y, region)), clean(eraNameSim(y, region))].join(","));
    }
  }
  fs.writeFileSync(path.join(outDir, "分政权朔日表.csv"), rows.join("\n") + "\n");
  fs.writeFileSync(path.join(outDir, "分政权年号表.csv"), eras.join("\n") + "\n");
  console.log(`写出 分政权朔日表.csv ${rows.length - 1} 行、分政权年号表.csv ${eras.length - 1} 行`);
}

main();
