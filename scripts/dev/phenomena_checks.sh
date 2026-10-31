#!/usr/bin/env bash
# 重算天象模块的全部逐条校验，写 data/phenomena/checks/*.csv，并生成 doc/astronomy/phenomena-checks.md。
#
# 用法（仓库根）：
#   scripts/dev/phenomena_checks.sh            # 全量重算（缺省 f64 构建，约 20 倍快；见 doc/engineering/project-and-implementation.md §11.2.2）
#   PHENOMENA_REAL=dd scripts/dev/phenomena_checks.sh   # 改用 double-double（库缺省构建，参考精度，约慢 20 倍）
#   scripts/dev/phenomena_checks.sh compare    # 两种构建各跑一组较快的校验（NASA 日月食典抽样 1/20 等），逐格比对（# 说明行除外；
#                                              # 极值时刻列允许搜索容差，见 scripts/dev/compare_checks.py），
#                                              # 不一致则列出并以非零退出；不改仓库文件
#
# 需 DE406（vendor/jpl/de406）、DE441（vendor/jpl/de441，可缺；缺则相应校验跳过）。
# 线程数：PHENOMENA_THREADS（缺省为 CPU 数）；NASA 日月食典抽样：NASA_ECLIPSE_STRIDE（缺省全量 1）。
set -euo pipefail
cd "$(dirname "$0")/../.."
export PHENOMENA_CHECKS=1

TESTS=(
  solar_eclipses_vs_nasa_canon
  lunar_eclipses_vs_nasa_canon
  planet_events_vs_skyfield
  close_approaches_vs_skyfield
  close_approaches_vsop_elp_vs_de406
  occultations_vs_skyfield
  telescope_model_vs_schaefer1992
  matches_ap_mag_v3_official_test_data
  planet_magnitude_vs_skyfield
  star_apparent_vs_skyfield
  air_mass_matches_table_3
  moonlight_matches_krisciunas_schaefer_table_2
  zenith_limit_matches_1990_eq18
  schaefer1998_worked_example
  glare_galilean_satellites_schaefer1991
  glare_near_moon_schaefer1991_fig1
  schaefer1987_table1_observations
  schaefer1987_table7_venus_yucatan
  schaefer2000_sirius_egypt
  heliacal_brackets_conjunctions
  widths_vs_yuanshi_table
  winter_solstice_lodge_positions_vs_yuanshi
  qing_determinatives_vs_pannai_table_9_1_4
  shishi_catalog_epoch
  houhanshu_star_records_check
  hanshu_star_records_check
  jinshu_star_records_check
  songshu_star_records_check
  nanqishu_star_records_check
  weishu_star_records_check
  suishu_star_records_check
  xintangshu_star_records_check
  jiutangshu_star_records_check
  jiuwudaishi_star_records_check
  xinwudaishi_star_records_check
  songshi_star_records_check
  jinshi_star_records_check
  yuanshi_star_records_check
  mingshi_star_records_check
  qingshigao_star_records_check
)
# 两种构建比对用：覆盖日月食、行星、近距、掩星、恒星、伏见、宿度与史书凌犯，double-double 一侧约十余分钟
COMPARE_TESTS=(
  solar_eclipses_vs_nasa_canon
  lunar_eclipses_vs_nasa_canon
  planet_events_vs_skyfield
  close_approaches_vs_skyfield
  occultations_vs_skyfield
  star_apparent_vs_skyfield
  schaefer1987_table1_observations
  widths_vs_yuanshi_table
  qing_determinatives_vs_pannai_table_9_1_4
  hanshu_star_records_check
)

features_of() {
  case "$1" in
    f64) echo "--no-default-features --features real-f64" ;;
    dd) echo "" ;;
    *) echo "PHENOMENA_REAL 只能是 f64 或 dd：$1" >&2; exit 2 ;;
  esac
}

# run_tests <后端> <测试名…>：逐个运行，打印结果行；有失败则返回非零
run_tests() {
  local real="$1"; shift
  local feats; feats=$(features_of "$real")
  local failed=0
  # shellcheck disable=SC2086
  (cd rust && cargo test --release -p lunar-core $feats --no-run >/dev/null 2>&1) || { echo "编译失败（$real）" >&2; return 1; }
  for t in "$@"; do
    echo "== [$real] $t"
    local out
    # shellcheck disable=SC2086
    out=$(cd rust && cargo test --release -p lunar-core $feats "$t" -- --include-ignored --test-threads=1 2>&1 || true)
    echo "$out" | grep -E "^test |test result: .* [1-9][0-9]* (passed|failed)" || true
    if echo "$out" | grep -qE "FAILED|panicked|error(\[|:)"; then
      failed=1
      echo "$out" | grep -E "panicked|FAILED|^error" | head -5
    fi
  done
  return $failed
}

if [[ "${1:-}" == "compare" ]]; then
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  export NASA_ECLIPSE_STRIDE="${NASA_ECLIPSE_STRIDE:-20}"
  status=0
  for real in f64 dd; do
    PHENOMENA_CHECKS_DIR="$tmp/$real" run_tests "$real" "${COMPARE_TESTS[@]}" || status=1
  done
  echo "== 两种构建逐格比对（# 说明行除外；极值时刻列允许搜索容差，见 scripts/dev/compare_checks.py）"
  python3 scripts/dev/compare_checks.py "$tmp/dd" "$tmp/f64" || status=1
  exit $status
fi

REAL="${PHENOMENA_REAL:-f64}"
export NASA_ECLIPSE_STRIDE="${NASA_ECLIPSE_STRIDE:-1}"
status=0
run_tests "$REAL" "${TESTS[@]}" || status=1
echo "== 史书日月食（historical_eclipses，f64；有 site_lon/site_lat 列者逐行取都城，余按书给定）"
for f in data/phenomena/*_solar_eclipses.csv data/phenomena/*_lunar_eclipses.csv; do
  b=$(basename "$f" .csv); mode=${b%_eclipses}; mode=${mode##*_}
  case $b in hanshu_*) ll="108.94 34.27" ;; houhanshu_*) ll="112.62 34.73" ;; qingshigao_*) ll="116.39 39.91" ;; *) ll="0 0" ;; esac
  # shellcheck disable=SC2086
  (cd rust && cargo run --release -p lunar-core --example historical_eclipses --no-default-features --features real-f64 -- \
    "$mode" "../$f" "../data/phenomena/${b}_check.csv" $ll >/dev/null) || { echo "失败：$b" >&2; status=1; }
done
echo "== record_date_shift"
uv run --no-project --with skyfield python -I scripts/phenomena/record_date_shift.py
uv run --project scripts python scripts/phenomena/recorded_distances.py
uv run --project scripts python scripts/phenomena/lodge_degrees.py
uv run --no-project --with skyfield python -I scripts/phenomena/qingshigao_moon_fullday.py
uv run --project scripts python scripts/phenomena/qingshigao_eclipse_compare.py
uv run --project scripts python scripts/phenomena/checks_to_md.py
if [[ $status -ne 0 ]]; then
  echo "有校验失败，见上。" >&2
fi
exit $status
