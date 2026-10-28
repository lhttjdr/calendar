#!/usr/bin/env node
/**
 * 将 年号表.csv 转为基础字段：6 列或 8 列（含改元时 gaiyuan_prev_era_name_simp, gaiyuan_prev_era_year_simp）。
 * 若输入已有 8 列则保留；否则输出 6 列。
 */

import fs from "fs";
import path from "path";

function quote(s) {
  if (s == null) return '""';
  const t = String(s);
  if (t.includes(",") || t.includes('"') || t.includes("\n")) return `"${t.replace(/"/g, '""')}"`;
  return t ? `"${t}"` : '""';
}

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

const dataDir = path.join(process.cwd(), "vendor", "calendar", "chinese_calendar");
const inputPath = path.join(dataDir, "年号表.csv");
const outputPath = path.join(dataDir, "年号表.csv.new");

const csv = fs.readFileSync(inputPath, "utf-8");
const lines = csv.split(/\r?\n/);
const header = lines[0];

const hasGaiyuanCols = header.includes("gaiyuan_prev");
const out = hasGaiyuanCols
  ? ["lunar_year,has_gaiyuan,regime_simp,emperor_simp,era_name_simp,era_year_simp,gaiyuan_prev_era_name_simp,gaiyuan_prev_era_year_simp"]
  : ["lunar_year,has_gaiyuan,regime_simp,emperor_simp,era_name_simp,era_year_simp"];

for (let i = 1; i < lines.length; i++) {
  const line = lines[i];
  if (!line.trim()) continue;
  const fields = parseCsvLine(line);
  if (fields.length >= 6) {
    const lunar_year = fields[0];
    const has_gaiyuan = fields[1];
    const regime = fields[2] ?? "";
    const emperor = fields[3] ?? "";
    const era_name = fields[4] ?? "";
    const era_year = fields[5] ?? "";
    if (hasGaiyuanCols && fields.length >= 8) {
      out.push(
        [lunar_year, has_gaiyuan, quote(regime), quote(emperor), quote(era_name), quote(era_year), quote(fields[6]), quote(fields[7])].join(",")
      );
    } else {
      out.push(
        [lunar_year, has_gaiyuan, quote(regime), quote(emperor), quote(era_name), quote(era_year)].join(",")
      );
    }
  }
}

fs.writeFileSync(outputPath, out.join("\n") + "\n", "utf-8");
console.log("Wrote", outputPath);
console.log("Review then: mv vendor/calendar/chinese_calendar/年号表.csv.new vendor/calendar/chinese_calendar/年号表.csv");
