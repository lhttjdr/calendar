#!/usr/bin/env node
/**
 * 生成 data/calendar/汇总/regime_era_calendar_matrix.xlsx（主输出）及同内容的 .csv 备查。
 * 第一列：农历年；主表各政权分列；**五胡十六国** 十六列（存续年内占位「未知-年号待补（历法）」）；
 * 再后为蜀/吴等并列政权列。格内格式「帝-年号（历法）」、改元用顿号。
 *
 * **南朝**：年号表.csv 中 regime_simp「南北朝」拆为南朝宋、南齐、南梁、南陈（按农历年分段）。
 * **十六国**：列名依常见史表；农历年存续区间取通说约数，年号未逐项录入，与后秦/北凉列合并入十六国块（不再单独重复后秦、北凉列）。
 *
 * 依赖：`uv sync --project scripts`（含 openpyxl）；Node 写 CSV 后调用 `uv run --project scripts python` 写 xlsx。
 * 用法：node scripts/calendar_data/build_regime_era_calendar_matrix.mjs
 * （工作目录为仓库根，勿 cd）
 */

import fs from "fs";
import path from "path";
import { spawnSync } from "child_process";

const eraDir = path.join(process.cwd(), "vendor", "calendar", "chinese_calendar");
const outDir = path.join(process.cwd(), "data", "calendar", "汇总");
const eraPath = path.join(eraDir, "年号表.csv");
const outCsvPath = path.join(outDir, "regime_era_calendar_matrix.csv");
const outXlsxPath = path.join(outDir, "regime_era_calendar_matrix.xlsx");

/**
 * 主表政权列（史序）。「南北朝」在源 CSV 中仍为 regime_simp 一字段，此处拆成南朝四列；赵宋列名仍为「宋」。
 */
const MAIN_TABLE_REGIMES = [
  "汉",
  "新",
  "魏",
  "晋",
  "南朝宋",
  "南齐",
  "南梁",
  "南陈",
  "隋",
  "唐",
  "武周",
  "五代",
  "宋",
  "元",
  "明",
  "清",
];
const MAIN_TABLE_REGIME_SET = new Set(MAIN_TABLE_REGIMES);

const ERA_NAMES_NANBEICHAO = "南北朝";

const VALID_ERA_NAMES_REGIME_SIMP = new Set([
  ...MAIN_TABLE_REGIME_SET,
  ERA_NAMES_NANBEICHAO,
]);

function southDynastyColumnForYear(y) {
  if (y < 420 || y > 589) return null;
  if (y <= 478) return "南朝宋";
  if (y <= 501) return "南齐";
  if (y <= 556) return "南梁";
  return "南陈";
}

/**
 * 五胡十六国：列序与农历年存续区间（与通说公元约对齐；年号待补）。
 * 历法栏为简述（各族所行不一，主表未逐政权导出朔闰）。
 */
const SIXTEEN_KINGDOMS_META = [
  { col: "成汉", start: 304, end: 347, cal: "未知" },
  { col: "前赵", start: 304, end: 329, cal: "未知" },
  { col: "后赵", start: 319, end: 351, cal: "未知" },
  { col: "前凉", start: 320, end: 376, cal: "未知" },
  { col: "前燕", start: 337, end: 370, cal: "未知" },
  { col: "前秦", start: 351, end: 394, cal: "未知" },
  { col: "后燕", start: 384, end: 407, cal: "未知" },
  { col: "后秦", start: 384, end: 417, cal: "三纪历" },
  { col: "西燕", start: 384, end: 394, cal: "未知" },
  { col: "西秦", start: 385, end: 431, cal: "未知" },
  { col: "后凉", start: 386, end: 403, cal: "未知" },
  { col: "南凉", start: 397, end: 414, cal: "未知" },
  { col: "北凉", start: 397, end: 439, cal: "玄始历" },
  { col: "南燕", start: 398, end: 410, cal: "未知" },
  { col: "北燕", start: 407, end: 436, cal: "未知" },
  { col: "胡夏", start: 407, end: 431, cal: "未知" },
];

const SIXTEEN_KINGDOMS_COLUMNS = SIXTEEN_KINGDOMS_META.map((m) => m.col);
const SIXTEEN_KINGDOM_SET = new Set(SIXTEEN_KINGDOMS_COLUMNS);

const PARALLEL_REGIMES = [
  ...SIXTEEN_KINGDOMS_COLUMNS,
  "蜀",
  "吴",
  "北朝西魏周隋",
  "东魏北齐",
  "辽金元",
  "南明",
];

const REGIME_COLUMNS = [...MAIN_TABLE_REGIMES, ...PARALLEL_REGIMES];

function parseCsvLine(line) {
  const fields = [];
  let i = 0;
  while (i < line.length) {
    if (line[i] === '"') {
      let s = "";
      i++;
      while (i < line.length) {
        if (line[i] === '"' && line[i + 1] === '"') {
          s += '"';
          i += 2;
        } else if (line[i] === '"') {
          i++;
          break;
        } else {
          s += line[i++];
        }
      }
      fields.push(s);
      if (line[i] === ",") i++;
    } else {
      let s = "";
      while (i < line.length && line[i] !== ",") s += line[i++];
      fields.push(s.trim());
      if (i < line.length) i++;
    }
  }
  return fields;
}

function quote(s) {
  const t = String(s ?? "");
  if (t.includes(",") || t.includes('"') || t.includes("\n")) return `"${t.replace(/"/g, '""')}"`;
  return t;
}

function mainTableCalendar(y) {
  if (y < -721) return "春秋历";
  if (y < -222) return "古六历";
  if (y < -104) return "颛顼历";
  return "主表（据历还原）";
}

function formatMainTableRegimeCell(row) {
  const [, hasGaiyuan, , emperor, eraName, eraYear, prevName, prevYear] = row;
  const cal = mainTableCalendar(parseInt(row[0], 10));
  const emp = emperor || "未知";
  let era = "";
  if (hasGaiyuan === "1" && prevName && prevYear) {
    era = `${prevName}${prevYear}、${eraName}${eraYear}`;
  } else {
    era = `${eraName || "未知"}${eraYear || ""}`;
  }
  return `${emp}-${era}（${cal}）`;
}

const SHU = {
  221: "昭烈帝-章武元年",
  222: "昭烈帝-章武二年",
};
for (let y = 223; y <= 237; y++) SHU[y] = `后主-建兴${numberToChineseYear(y - 222)}`;
for (let y = 238; y <= 257; y++) SHU[y] = `后主-延熙${numberToChineseYear(y - 237)}`;
for (let y = 258; y <= 262; y++) SHU[y] = `后主-景耀${numberToChineseYear(y - 257)}`;
SHU[263] = "后主-景耀六年、炎兴元年";

function numberToChineseYear(n) {
  const d = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
  if (n === 1) return "元年";
  if (n <= 10) return `${d[n]}年`;
  if (n < 20) return `十${d[n - 10]}年`.replace("十一", "十一").replace("十元", "十");
  if (n === 20) return "二十年";
  if (n < 30) return `二十${d[n - 20]}年`;
  return `${n}年`;
}

const WU_CAL = (y) => (y <= 222 ? "四分历" : "乾象历");
const WU = {};
function setWu(y, s) {
  WU[y] = `${s}（${WU_CAL(y)}）`;
}
for (let y = 222; y <= 228; y++) setWu(y, `大帝-黄武${numberToChineseYear(y - 221)}`);
for (let y = 229; y <= 231; y++) setWu(y, `大帝-黄龙${numberToChineseYear(y - 228)}`);
for (let y = 232; y <= 238; y++) setWu(y, `大帝-嘉禾${numberToChineseYear(y - 231)}`);
for (let y = 239; y <= 251; y++) setWu(y, `大帝-赤乌${numberToChineseYear(y - 238)}`);
setWu(252, "大帝-太元元年、神凤元年、会稽王-建兴元年");
for (let y = 253; y <= 254; y++) setWu(y, `会稽王-建兴${numberToChineseYear(y - 251)}`);
for (let y = 255; y <= 256; y++) setWu(y, `会稽王-五凤${numberToChineseYear(y - 254)}`);
setWu(257, "会稽王-太平元年");
setWu(258, "景帝-永安元年");
for (let y = 259; y <= 263; y++) setWu(y, `景帝-永安${numberToChineseYear(y - 257)}`);
setWu(264, "乌程侯-元兴元年");
setWu(265, "乌程侯-甘露元年");
setWu(266, "乌程侯-甘露二年");
for (let y = 267; y <= 269; y++) setWu(y, `乌程侯-宝鼎${numberToChineseYear(y - 266)}`);
for (let y = 270; y <= 271; y++) setWu(y, `乌程侯-建衡${numberToChineseYear(y - 269)}`);
for (let y = 272; y <= 274; y++) setWu(y, `乌程侯-凤凰${numberToChineseYear(y - 271)}`);
setWu(275, "乌程侯-天册元年");
setWu(276, "乌程侯-天玺元年");
for (let y = 277; y <= 280; y++) setWu(y, `乌程侯-天纪${numberToChineseYear(y - 276)}`);

const REGIME_META = [
  { col: "蜀", start: 221, end: 263, fallbackCal: "四分历" },
  { col: "吴", start: 222, end: 280, fallbackCal: null },
  { col: "北朝西魏周隋", start: 398, end: 590, fallbackCal: "景初→玄始→正光→明克让→天和→大象→开皇" },
  { col: "东魏北齐", start: 534, end: 577, fallbackCal: "正光→兴和→天保" },
  { col: "辽金元", start: 947, end: 1279, fallbackCal: "主表+辽金元修正、宣明/重修大明平气" },
  { col: "南明", start: 1645, end: 1683, fallbackCal: "大统历平气、南明朔闰表" },
];

function cellForRegime(col, lunarYear, eraRow) {
  if (MAIN_TABLE_REGIME_SET.has(col)) {
    const r = eraRow[2];
    if (r === ERA_NAMES_NANBEICHAO) {
      return southDynastyColumnForYear(lunarYear) === col
        ? formatMainTableRegimeCell(eraRow)
        : "";
    }
    return r === col ? formatMainTableRegimeCell(eraRow) : "";
  }
  if (SIXTEEN_KINGDOM_SET.has(col)) {
    const meta = SIXTEEN_KINGDOMS_META.find((m) => m.col === col);
    if (!meta || lunarYear < meta.start || lunarYear > meta.end) return "";
    const cal = meta.cal || "未知";
    return `未知-年号待补（${cal}）`;
  }
  if (col === "蜀") {
    if (lunarYear < 221 || lunarYear > 263) return "";
    return SHU[lunarYear]
      ? `${SHU[lunarYear]}（四分历）`
      : `未知（四分历）`;
  }
  if (col === "吴") {
    if (lunarYear < 222 || lunarYear > 280) return "";
    return WU[lunarYear] || `未知（${WU_CAL(lunarYear)}）`;
  }
  const meta = REGIME_META.find((m) => m.col === col);
  if (!meta || lunarYear < meta.start || lunarYear > meta.end) return "";
  const cal = meta.fallbackCal || "未知";
  return `未知-年号待补（${cal}）`;
}

function buildRows() {
  const csv = fs.readFileSync(eraPath, "utf-8");
  const lines = csv.split(/\r?\n/).filter(Boolean);
  const header = lines[0];
  if (!header.startsWith("lunar_year")) {
    console.error("Unexpected 年号表.csv header");
    process.exit(1);
  }

  const dataRows = [];
  for (let li = 1; li < lines.length; li++) {
    const row = parseCsvLine(lines[li]);
    const lunarYear = parseInt(row[0], 10);
    if (Number.isNaN(lunarYear)) continue;
    const regimeSimp = row[2];
    if (regimeSimp && !VALID_ERA_NAMES_REGIME_SIMP.has(regimeSimp)) {
      console.error("年号表.csv unknown regime_simp:", regimeSimp, "lunar_year", lunarYear);
      process.exit(1);
    }
    const cells = REGIME_COLUMNS.map((col) => cellForRegime(col, lunarYear, row));
    dataRows.push({ lunarYear, cells });
  }
  return dataRows;
}

function writeCsv(dataRows) {
  const outLines = [];
  outLines.push(["年份", ...REGIME_COLUMNS].join(","));
  for (const { lunarYear, cells } of dataRows) {
    outLines.push([lunarYear, ...cells.map(quote)].join(","));
  }
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(outCsvPath, `${outLines.join("\n")}\n`, "utf-8");
}

function writeXlsxViaPython() {
  const r = spawnSync(
    "uv",
    ["run", "--project", "scripts", "python", "scripts/calendar_data/csv_matrix_to_xlsx.py", outCsvPath, outXlsxPath],
    {
      cwd: process.cwd(),
      encoding: "utf-8",
      stdio: ["ignore", "inherit", "pipe"],
    },
  );
  if (r.status !== 0) {
    console.error(r.stderr || "uv run csv_matrix_to_xlsx.py failed");
    process.exit(r.status ?? 1);
  }
}

function main() {
  const dataRows = buildRows();
  writeCsv(dataRows);
  writeXlsxViaPython();
  console.log("Wrote", outCsvPath, "and", outXlsxPath, "rows", dataRows.length);
}

main();
