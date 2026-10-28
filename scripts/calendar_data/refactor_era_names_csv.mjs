#!/usr/bin/env node
/**
 * 从现有 年号表.csv 解析 era_simp，生成 政权/帝号/年号名/年数 四段及合并年号：
 * regime_simp, emperor_simp, era_name_simp, era_year_simp, era_only_simp（年号名+年数，兜底）
 * 展示格式：汉 / 武帝 / 元封 / 五年
 */

import fs from "fs";
import path from "path";

const PREFIXES = [
  "后周", "后汉", "后晋", "后唐", "后梁",
  "汉", "魏", "晋", "宋", "唐", "明", "清", "元", "新", "周", "吴", "蜀", "隋", "秦", "齐", "陈", "梁", "金", "辽",
  "武周", "南明", "西夏", "大理", "渤海",
];

// 从最后一个「年」或「载」往前扫，遇到 帝/祖/宗 视为帝号与年号分界（如 汉武帝元封五年 → 武帝|元封五年；唐玄宗天宝三载 → 玄宗|天宝三载）
function findEraStart(s) {
  const iYear = s.lastIndexOf("年");
  const iZai = s.lastIndexOf("载");
  const i = iYear >= 0 && iZai >= 0 ? Math.max(iYear, iZai) : iYear >= 0 ? iYear : iZai >= 0 ? iZai : -1;
  if (i < 0) return -1;
  for (let j = i - 1; j >= 0; j--) {
    const c = s[j];
    if (c === "帝" || c === "祖" || c === "宗") return j + 1;
  }
  return 0;
}

function parseEraSimp(era_simp) {
  let s = (era_simp || "").trim().replace(/^\(|\)$/g, "");
  if (!s) return { regime: "", emperor: "", era: "" };

  let regime = "";
  const bracket = s.match(/\[([^\]]+)\]/);
  if (bracket) {
    regime = bracket[1];
    s = s.replace(/\[[^\]]+\]/, "").trim();
  }

  let eraStart = -1;
  let idx = 0;
  for (const seg of s.split("/")) {
    const pos = findEraStart(seg);
    if (pos >= 0) {
      eraStart = idx + pos;
      break;
    }
    idx += seg.length + 1;
  }

  let emperor = "";
  let era = "";
  if (eraStart >= 0) {
    emperor = s.slice(0, eraStart).trimEnd();
    era = s.slice(eraStart).trim();
  } else {
    emperor = s;
  }

  if (!regime) {
    for (const p of PREFIXES) {
      if (emperor.startsWith(p)) {
        regime = p;
        break;
      }
    }
    if (!regime && emperor) regime = emperor.slice(0, 1);
    if (!regime && era) {
      for (const p of PREFIXES) {
        if (era.startsWith(p)) {
          regime = p;
          era = era.slice(p.length);
          break;
        }
      }
    }
  }
  if (regime && emperor.startsWith(regime)) {
    emperor = emperor.slice(regime.length);
  }

  const ERA_NUM_RE = /^(.+?)(元年|[一二三四五六七八九十百千〇零廿卅]+(?:年|载))$/;

  function parseEraPart(part) {
    const m = (part || "").trim().match(ERA_NUM_RE);
    if (m) return { name: m[1], year: m[2] };
    return { name: (part || "").trim(), year: "" };
  }

  let era_name = "";
  let era_year = "";
  let gaiyuan_prev_era_name = "";
  let gaiyuan_prev_era_year = "";
  if (era) {
    const segments = era.split("/").map((t) => t.trim()).filter(Boolean);
    if (segments.length > 1) {
      const mainPart = segments[segments.length - 1];
      const prevPart = segments[0];
      const main = parseEraPart(mainPart);
      const prev = parseEraPart(prevPart);
      era_name = main.name;
      era_year = main.year;
      gaiyuan_prev_era_name = prev.name;
      gaiyuan_prev_era_year = prev.year;
    } else {
      const main = parseEraPart(era);
      era_name = main.name;
      era_year = main.year;
    }
  }

  return { regime, emperor, era, era_name, era_year, gaiyuan_prev_era_name, gaiyuan_prev_era_year };
}

function quote(s) {
  if (s == null) return '""';
  const t = String(s);
  if (t.includes(",") || t.includes('"') || t.includes("\n")) return `"${t.replace(/"/g, '""')}"`;
  return t ? `"${t}"` : '""';
}

const dataDir = path.join(process.cwd(), "vendor", "calendar", "chinese_calendar");
const inputPath = path.join(dataDir, "年号表.csv");
const outputPath = path.join(dataDir, "年号表.csv.new");

const csv = fs.readFileSync(inputPath, "utf-8");
const lines = csv.split(/\r?\n/);
const header = lines[0];
if (!header.includes("lunar_year")) {
  console.error("Unexpected header:", header);
  process.exit(1);
}
// 支持输入：4 列(含 era_simp) 或 9 列(含 regime/emperor/era_name/era_year)，输出统一 6 列

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
      if (i < line.length && line[i] === ",") i++;
    } else {
      let s = "";
      while (i < line.length && line[i] !== ",") s += line[i++];
      fields.push(s.trim());
      if (i < line.length) i++;
    }
  }
  return fields;
}

const ERA_NUM_RE = /^(.+?)(元年|[一二三四五六七八九十百千〇零廿卅]+(?:年|载))$/;
function parseEraPart(part) {
  const m = (part || "").trim().match(ERA_NUM_RE);
  if (m) return { name: m[1], year: m[2] };
  return { name: (part || "").trim(), year: "" };
}

// 输出 8 列：基础 6 列 + 改元前年号名/年数（仅 has_gaiyuan=1 时非空）
const newHeader = "lunar_year,has_gaiyuan,regime_simp,emperor_simp,era_name_simp,era_year_simp,gaiyuan_prev_era_name_simp,gaiyuan_prev_era_year_simp";
const out = [newHeader];

for (let i = 1; i < lines.length; i++) {
  const line = lines[i];
  if (!line.trim()) continue;
  const fields = parseCsvLine(line);
  if (fields.length < 4) {
    out.push(line);
    continue;
  }
  const lunar_year = fields[0];
  const has_gaiyuan = fields.length >= 6 ? fields[1] : (fields.length >= 4 ? fields[3] : "0");
  let regime, emperor, era_name, era_year, gaiyuan_prev_era_name = "", gaiyuan_prev_era_year = "";
  if (fields.length >= 9) {
    regime = fields[4];
    emperor = fields[5];
    era_name = fields[6];
    era_year = fields[7];
    const era_only = fields[8] || "";
    if (has_gaiyuan === "1" && era_only.includes("/")) {
      const segs = era_only.split("/").map((t) => t.trim()).filter(Boolean);
      if (segs.length > 1) {
        const main = parseEraPart(segs[segs.length - 1]);
        const prev = parseEraPart(segs[0]);
        era_name = main.name;
        era_year = main.year;
        gaiyuan_prev_era_name = prev.name;
        gaiyuan_prev_era_year = prev.year;
      }
    }
  } else if (fields.length >= 6) {
    regime = fields[2];
    emperor = fields[3];
    era_name = fields[4];
    era_year = fields[5];
    if (has_gaiyuan === "1" && era_name.includes("/")) {
      const segs = era_name.split("/").map((t) => t.trim()).filter(Boolean);
      if (segs.length > 1) {
        const main = parseEraPart(segs[segs.length - 1] + era_year);
        const prev = parseEraPart(segs[0]);
        era_name = main.name;
        era_year = main.year;
        gaiyuan_prev_era_name = prev.name;
        gaiyuan_prev_era_year = prev.year;
      }
    }
  } else {
    const era_simp = fields[2] || "";
    const parsed = parseEraSimp(era_simp);
    regime = parsed.regime;
    emperor = parsed.emperor;
    era_name = parsed.era_name;
    era_year = parsed.era_year;
    gaiyuan_prev_era_name = parsed.gaiyuan_prev_era_name || "";
    gaiyuan_prev_era_year = parsed.gaiyuan_prev_era_year || "";
  }
  out.push(
    [lunar_year, has_gaiyuan, quote(regime), quote(emperor), quote(era_name), quote(era_year), quote(gaiyuan_prev_era_name), quote(gaiyuan_prev_era_year)].join(",")
  );
}

fs.writeFileSync(outputPath, out.join("\n") + "\n", "utf-8");
console.log("Wrote", outputPath);
console.log("Review then: mv vendor/calendar/chinese_calendar/年号表.csv.new vendor/calendar/chinese_calendar/年号表.csv");
