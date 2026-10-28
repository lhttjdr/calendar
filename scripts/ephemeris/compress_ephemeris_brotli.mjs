#!/usr/bin/env node
/**
 * 将 vendor/vsop87、vendor/elpmpp02、vendor/IAU2000 下的 .bin 压成 .br（Brotli），写回同目录。
 * 前端会优先请求 .bin.br 并用 DecompressionStream 解压，可显著减带宽。
 * 用法（项目根）：node scripts/ephemeris/compress_ephemeris_brotli.mjs [vendor 目录] [数据目录]
 * 默认 vendor 目录：./vendor；默认数据目录（fit/）：./data
 */

import fs from 'node:fs'
import path from 'node:path'
import zlib from 'node:zlib'

const repoRoot = path.resolve(path.dirname(process.argv[1]), '..', '..')
const vendorDir = path.resolve(repoRoot, process.argv[2] || 'vendor')
const dataDir = path.resolve(repoRoot, process.argv[3] || 'data')

const files = [
  path.join(vendorDir, 'vsop87', 'VSOP87B.ear.bin'),
  path.join(vendorDir, 'elpmpp02', 'ELP_MAIN.S1.bin'),
  path.join(vendorDir, 'elpmpp02', 'ELP_MAIN.S2.bin'),
  path.join(vendorDir, 'elpmpp02', 'ELP_MAIN.S3.bin'),
  path.join(vendorDir, 'elpmpp02', 'ELP_PERT.S1.bin'),
  path.join(vendorDir, 'elpmpp02', 'ELP_PERT.S2.bin'),
  path.join(vendorDir, 'elpmpp02', 'ELP_PERT.S3.bin'),
  path.join(vendorDir, 'IAU2000', 'tab5.3a.bin'),
  path.join(dataDir, 'fit', 'vsop87-de406-earth.bin'),
  path.join(dataDir, 'fit', 'vsop87-de441-earth.bin'),
]

for (const fp of files) {
  if (!fs.existsSync(fp)) {
    console.warn('skip (missing):', fp)
    continue
  }
  const buf = fs.readFileSync(fp)
  const br = zlib.brotliCompressSync(buf, { params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11 } })
  const outPath = fp + '.br'
  fs.writeFileSync(outPath, br)
  const pct = ((1 - br.length / buf.length) * 100).toFixed(1)
  console.log(path.relative(repoRoot, fp), '->', path.basename(outPath), `(${buf.length} -> ${br.length}, -${pct}%)`)
}
console.log('done.')
