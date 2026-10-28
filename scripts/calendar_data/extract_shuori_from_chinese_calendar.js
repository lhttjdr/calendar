#!/usr/bin/env node
/**
 * 从 ytliu0/ChineseCalendar 的 calendarData.js 提取实历朔日数据（历史朔日）。
 * 年号单独成表（年号表.csv），朔日表不包含年号列。
 * 数据来源: https://github.com/ytliu0/ChineseCalendar
 *
 * 每行格式: [农历年, 正月朔日偏移, 二月..., 十二月, 闰月朔偏移, 闰月月份号, 该年总日数]
 * 日偏移: 相对于前一年12月31日的天数，1=1月1日。>1000 表示减5000为实际偏移。
 */

const fs = require("fs");
const path = require("path");

// 供 eras.js 在 Node 下使用（无 document 时用 li 参数，此处 mock 避免报错）
const document = {
  getElementById: () => ({ style: { display: "none" }, classList: { contains: () => false } }),
};

// 儒略历/格里高利历某年天数（1582 年前为儒略历，之后为格里高利历）
function NdaysGregJul(y) {
  let ndays = (y === 1582 ? 355 : 365) + (Math.abs(y) % 4 === 0 ? 1 : 0);
  if (y > 1582) {
    ndays += (y % 100 === 0 ? -1 : 0) + (y % 400 === 0 ? 1 : 0);
  }
  return ndays;
}

// 日偏移 -> 儒略历/格里高利历 (年, 月, 日)。day 为相对前一年12月31日的天数
function dayOffsetToDate(gregorianYear, dayIn) {
  let day = dayIn > 1000 ? dayIn - 5000 : dayIn;
  if (day === 9999 || day === 0) return null;

  let y = gregorianYear;
  let ndays = NdaysGregJul(y);

  if (day < 1) {
    y--;
    ndays = NdaysGregJul(y);
    day += ndays;
  } else if (day > ndays) {
    day -= ndays;
    y++;
    ndays = NdaysGregJul(y);
  }

  const leap = ndays === 366 ? 1 : 0;
  let mday = [
    0, 31, 59 + leap, 90 + leap, 120 + leap, 151 + leap, 181 + leap,
    212 + leap, 243 + leap, 273 + leap, 304 + leap, 334 + leap, 365 + leap,
  ];
  if (y === 1582 && day > 277) day += 10;

  let m = 0,
    d = 0;
  for (let i = 0; i < 13; i++) {
    if (day - mday[i] < 1) {
      m = i;
      d = day - (mday[i - 1] || 0);
      break;
    }
  }
  if (m === 13) {
    m = 1;
    d = day - mday[12];
    y++;
  }
  return { y, m, d };
}

// 从 "return [[..." 开始提取直到匹配的 "];"
function extractChineseToGregorianArray(code) {
  const start = code.indexOf("function ChineseToGregorian() {return ");
  if (start === -1) throw new Error("未找到 ChineseToGregorian");
  const arrayStart = code.indexOf("[[", start);
  if (arrayStart === -1) throw new Error("未找到 return 数组");
  let depth = 0;
  let i = arrayStart;
  while (i < code.length) {
    if (code[i] === "[" && code[i + 1] !== "/") depth++;
    else if (code[i] === "]") {
      depth--;
      if (depth === 0 && code[i + 1] === ";") return code.slice(arrayStart, i + 1);
    }
    i++;
  }
  throw new Error("数组括号不匹配");
}

// 加载 eras.js，返回 (lunarYear) => { era_trad, era_simp }；若文件不存在则返回 () => ({ era_trad: '', era_simp: '' })
function loadEraLookup(calendarDataPath) {
  const erasPath = path.join(path.dirname(calendarDataPath), "eras.js");
  if (!fs.existsSync(erasPath)) return () => ({ era_trad: "", era_simp: "" });
  const code = fs.readFileSync(erasPath, "utf8") + "\nreturn { eraName, eraNameSim };";
  let eraName, eraNameSim;
  try {
    const fn = new Function("document", code);
    const out = fn(document);
    eraName = out.eraName;
    eraNameSim = out.eraNameSim;
  } catch (e) {
    return () => ({ era_trad: "", era_simp: "" });
  }
  const cache = {};
  return function getEra(lunarYear) {
    if (cache[lunarYear]) return cache[lunarYear];
    const li =
      lunarYear >= -722 && lunarYear <= -480
        ? "Chunqiu"
        : lunarYear >= -479 && lunarYear <= -221
          ? "Zhuanxu"
          : null;
    const era_trad = (typeof eraName === "function" ? eraName(lunarYear, "default", li) : "") || "";
    const era_simp = (typeof eraNameSim === "function" ? eraNameSim(lunarYear, "default", li) : "") || "";
    cache[lunarYear] = { era_trad, era_simp };
    return cache[lunarYear];
  };
}

function extractShuoriData(calendarDataPath, outDir, getEra) {
  const code = fs.readFileSync(calendarDataPath, "utf8");
  const arrayStr = extractChineseToGregorianArray(code);
  let data;
  try {
    data = new Function("return " + arrayStr)();
  } catch (e) {
    throw new Error("解析数组失败: " + e.message);
  }

  getEra = getEra || (() => ({ era_trad: "", era_simp: "" }));
  const rows = [];
  const byWestern = [];

  for (let i = 0; i < data.length; i++) {
    const row = data[i];
    const lunarYear = row[0];
    const monthOffsets = row.slice(1, 13);
    const leapOffset = row[13];
    const leapMonthNum = row[14];

    for (let j = 0; j < 12; j++) {
      const off = monthOffsets[j];
      if (off === 9999) continue;
      const date = dayOffsetToDate(lunarYear, off);
      if (date) {
        rows.push({
          gregorian_year: date.y,
          gregorian_month: date.m,
          gregorian_day: date.d,
          lunar_year: lunarYear,
          lunar_month: j + 1,
          is_leap: 0,
        });
        byWestern.push({
          ...date,
          lunarYear,
          lunarMonth: j + 1,
          isLeap: false,
        });
      }
    }
    if (leapOffset > 0 && leapMonthNum > 0) {
      const date = dayOffsetToDate(lunarYear, leapOffset);
      if (date) {
        rows.push({
          gregorian_year: date.y,
          gregorian_month: date.m,
          gregorian_day: date.d,
          lunar_year: lunarYear,
          lunar_month: leapMonthNum,
          is_leap: 1,
        });
        byWestern.push({
          ...date,
          lunarYear,
          lunarMonth: leapMonthNum,
          isLeap: true,
        });
      }
    }
  }

  const csvHeader = "gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap";
  const csvLines = [
    csvHeader,
    ...rows.map((r) =>
      [r.gregorian_year, r.gregorian_month, r.gregorian_day, r.lunar_year, r.lunar_month, r.is_leap].join(
        ","
      )
    ),
  ];

  fs.mkdirSync(outDir, { recursive: true });
  const csvPath = path.join(outDir, "实历朔日表.csv");
  const jsonPath = path.join(outDir, "实历朔日表.json");
  fs.writeFileSync(csvPath, csvLines.join("\n") + "\n", "utf8");
  fs.writeFileSync(jsonPath, JSON.stringify(rows, null, 0), "utf8");

  const quote = (s) => (s == null ? "" : '"' + String(s).replace(/"/g, '""') + '"');
  // 年号单独成表：按农历年一行，含 lunar_year, era_trad, era_simp, has_gaiyuan（该年是否改元，即年号串中含 "/"）
  const eraRows = data.map((row) => {
    const lunarYear = row[0];
    const { era_trad, era_simp } = getEra(lunarYear);
    const has_gaiyuan = (era_trad && era_trad.indexOf("/") !== -1) ? 1 : 0;
    return { lunar_year: lunarYear, era_trad, era_simp, has_gaiyuan };
  });
  const eraCsvPath = path.join(outDir, "年号表.csv");
  const eraJsonPath = path.join(outDir, "年号表.json");
  const eraCsvLines = [
    "lunar_year,era_trad,era_simp,has_gaiyuan",
    ...eraRows.map((r) =>
      [r.lunar_year, quote(r.era_trad), quote(r.era_simp), r.has_gaiyuan].join(",")
    ),
  ];
  fs.writeFileSync(eraCsvPath, eraCsvLines.join("\n") + "\n", "utf8");
  fs.writeFileSync(eraJsonPath, JSON.stringify(eraRows, null, 0), "utf8");

  // 改元候选表：仅含 has_gaiyuan=1 的年份，便于查「一年多号」；本数据源无改元具体日期，需查历日/史书
  const gaiyuanRows = eraRows.filter((r) => r.has_gaiyuan === 1);
  const gaiyuanCsvPath = path.join(outDir, "改元候选表.csv");
  const gaiyuanJsonPath = path.join(outDir, "改元候选表.json");
  const gaiyuanCsvLines = [
    "lunar_year,era_trad,era_simp",
    ...gaiyuanRows.map((r) => [r.lunar_year, quote(r.era_trad), quote(r.era_simp)].join(",")),
  ];
  fs.writeFileSync(gaiyuanCsvPath, gaiyuanCsvLines.join("\n") + "\n", "utf8");
  fs.writeFileSync(gaiyuanJsonPath, JSON.stringify(gaiyuanRows, null, 0), "utf8");

  const yearRange = data.length
    ? { first: data[0][0], last: data[data.length - 1][0] }
    : {};
  return {
    csvPath,
    jsonPath,
    eraCsvPath,
    eraJsonPath,
    gaiyuanCsvPath,
    gaiyuanJsonPath,
    totalShuori: rows.length,
    totalEraYears: eraRows.length,
    totalGaiyuanYears: gaiyuanRows.length,
    yearRange,
  };
}

// 默认路径：先找克隆的 ChineseCalendar，否则用当前目录下的 clone
const defaultCalendarData =
  process.env.CALENDAR_DATA_PATH ||
  path.join(__dirname, "../../../ChineseCalendar/src/calendarData.js");

if (!fs.existsSync(defaultCalendarData)) {
  console.error("未找到 calendarData.js。请先克隆仓库：");
  console.error("  git clone https://github.com/ytliu0/ChineseCalendar.git");
  console.error("或将 ChineseCalendar 放在项目旁，或设置 CALENDAR_DATA_PATH");
  process.exit(1);
}

const outDir = path.resolve(__dirname, "..", "..", "vendor", "calendar", "chinese_calendar");
try {
  const getEra = loadEraLookup(defaultCalendarData);
  const result = extractShuoriData(defaultCalendarData, outDir, getEra);
  console.log("朔日数据已提取（实历，来自 ytliu0/ChineseCalendar）：");
  console.log("   CSV:", result.csvPath);
  console.log("  JSON:", result.jsonPath);
  console.log("  条数:", result.totalShuori);
  console.log("  农历年范围:", result.yearRange.first, "~", result.yearRange.last);
  console.log("年号单独成表：");
  console.log("   CSV:", result.eraCsvPath);
  console.log("  JSON:", result.eraJsonPath);
  console.log("  条数:", result.totalEraYears, "（每年一行），含 has_gaiyuan 列");
  console.log("改元候选表（该年有多年号，无具体改元日）：");
  console.log("   CSV:", result.gaiyuanCsvPath);
  console.log("  JSON:", result.gaiyuanJsonPath);
  console.log("  条数:", result.totalGaiyuanYears);
} catch (err) {
  console.error(err.message);
  process.exit(1);
}
