# astronomy 包：Decimal / 元组 / 裸数值 → 物理量审计

本文档为**历史审计**：针对原实现中 `lunar.astronomy` 下使用 `Decimal`、`Vector`、元组处的排查，标注**有对应物理概念、建议改为物理量**的项。路径与文件名为当时代码结构，仅供参考。  
（“物理量”指：`PlaneAngle`、`Length`/`Distance`、`Duration`/`TimeInterval`、`Position`、`Velocity`、以及可扩展的 `Pressure`/`Temperature` 等。）

---

## 一、建议改为物理量的用法

### 1. 大气折射 `atmospheric_refraction/AtmosphericRefraction`

| 当前 | 物理含义 | 建议类型 |
|------|----------|----------|
| `ha` / `h` / `altitude`: `Decimal` | 地平高度角（弧度） | `PlaneAngle` |
| 返回值 `Decimal` | 折射量（弧度） | `PlaneAngle` |
| `P`: `Decimal` | 气压 (kPa) | 保持 `Decimal` 或日后引入 `Pressure` |
| `T`: `Decimal` | 气温 (°C) | 保持 `Decimal` 或日后引入 `Temperature` |

**建议**：公开 API 的“角度入参 + 角度返回值”改为 `PlaneAngle`，内部再按需用 `.rad`。P/T 暂无对应量纲类型时可保留 `Decimal` 并注释单位。

---

### 2. 章动 nutation：`(Decimal, Decimal)` → Δψ、Δε

**涉及文件**：  
`NutationSOFA`、`MHB2000Truncated`、`Nutation`、`MHB2000`、`apparent/Apparent`

- `nutation(t): (Decimal, Decimal)`  
- `nutationDerivative(t): (Decimal, Decimal)`  
- `nutationAt(t): (Decimal, Decimal)`  
- 内部 `dpsi`、`deps` / `eps` 等均为弧度。

**建议**（二选一或组合）：

- 定义小类型，例如：  
  `case class Nutation(psi: PlaneAngle, epsilon: PlaneAngle)`  
  或  
  `case class NutationRad(psiRad: Decimal, epsilonRad: Decimal)`（若希望保留弧度标量）。
- 或对外 API 使用 `(PlaneAngle, PlaneAngle)`，避免“两个裸 Decimal 谁是谁”的歧义。

---

### 3. 坐标点 `coordinate/Point`（及 Ecliptic/FirstEquatorial/SecondEquatorial/Horizontal）

| 类型 | 当前字段 | 物理含义 | 建议类型 |
|------|----------|----------|----------|
| `EclipticPoint` | `longitude`, `latitude`: `Decimal` | 黄经、黄纬 | `PlaneAngle` |
| | `distance`: `Option[Decimal]` | 距离（米） | `Option[Length]` 或 `Option[Distance]` |
| `SecondEquatorialPoint` | `rightAscension`, `declination`: `Decimal` | 赤经、赤纬 | `PlaneAngle` |
| | `distance`: `Option[Decimal]` | 距离 | `Option[Length]` |
| `FirstEquatorialPoint` | `hourAngle`, `declination`: `Decimal` | 时角、赤纬 | `PlaneAngle` |
| | `distance`: `Option[Decimal]` | 距离 | `Option[Length]` |
| `HorizontalPoint` | `azimuth`, `altitude`: `Decimal` | 方位、高度 | `PlaneAngle` |
| | `distance`: `Option[Decimal]` | 距离 | `Option[Length]` |

`AstroPoint` 与 `Ecliptic`/`FirstEquatorial`/`SecondEquatorial`/`Horizontal` 的工厂方法参数也应一并改为上述物理量；`eclipticToPosition` 等已用 `PlaneAngle`/`Length` 的可以继续基于新点类型用 `.longitude`/`.distance` 等。

---

### 4. 光行时 / 视位置：`initialDistanceMeters: Option[Decimal]`

**涉及**：`apparent/Apparent`、`aspects/SunMoon`、`LightTime.withLightTimeFast(_, initialDistanceMeters.map(Length.inSI))`

- 语义：地心距（米），用于光行时初值。
- **建议**：改为 `initialDistance: Option[Length]`（或 `Option[Distance]`），调用方传 `Length.inSI(m)` 或已有 `Length`，避免“米”散落各处。

---

### 5. 合朔/节气中的 `prevSunDist` / `prevMoonDist`: `Option[Decimal]`

**涉及**：`aspects/SolarTerm`、`aspects/SunMoon`

- 语义：上一步太阳/月球地心距（米）。
- **建议**：改为 `Option[Length]` 或 `Option[Distance]`，与 `initialDistance` 统一。

---

### 6. `coordinate/ToPosition` 内部

- 公有 API 已使用 `PlaneAngle`、`Option[Length]`，合理。
- 私有 `sphericalToCartesian(lon: Decimal, lat: Decimal, r: Decimal)` 为纯数值实现细节，可保留 `Decimal`，或改为 `(PlaneAngle, PlaneAngle, Length)` 再在内部取 `.rad`/`.valueInSI`。

---

### 7. `TimeScaleContext.deltaT(jdTT: Decimal): Decimal`

**涉及**：`time/TimePoint`、实现类

- 返回值：ΔT 秒（标量）。
- **建议**：可改为返回 `Duration` 或 `TimeInterval`（如 `TimeInterval.inSeconds(deltaTSeconds)`），在 `TimePoint` 换算处再取 `.seconds` 或 `.inDays`，与“时间量用 Duration/TimeInterval”一致。

---

## 二、可保留或仅做轻量改进的用法

### 1. 时间与归一化时间

- `TimePoint.jd: Decimal`：儒略日本身是标量日数，保留 `Decimal` 合理。
- `NormalizedTime.JulianCentury`：已为 opaque type 包装 Decimal，表示无量纲 t，合理。
- `julianCentury(jd: Decimal)`、`julianMillennium(jd: Decimal)`：输入为 JD，合理。

### 2. 历表 / 级数内部

- **Elpmpp02Parse / Elpmpp02**：`Elpmpp02Term(Ci, Fi, ...)`、`Elpmpp02ParseConstants` 中 `ra0`、`ratioMeanMotion`、`deltaNU` 等为系数或弧度/无量纲参数，与现有量纲层不完全对应，可保留 `Decimal`，仅在对外暴露的“角度/长度”处用物理量。
- **VSOP87 / 其它历表**：内部多项式系数、T 为无量纲，保留 `Decimal` 无问题。

### 3. 矩阵 / 向量运算

- **Apparent** 等中的 `Vec[3]`、`Matrix`、`(x,y,z)`：多为中间直角坐标或与 `Position`/`Velocity` 的 `.x.value` 等混用，若已通过 `Position`/`Velocity` 与坐标系交互，可保留；若某处“裸 Vec + 单位约定”可改为 `Displacement`/`Velocity`，可单独标注再改。
- **Fk5Icrs**：`rotateEquatorialComponents(x,y,z: Decimal)` 为内部数值旋转，保留可接受。

### 4. 无量纲或约定单位

- **vsopBarycentricPlanets: Option[Seq[(Vsop87, Decimal)]]**：Decimal 为质量比（无量纲），保留或另起类型别名即可。
- **PipelineContext.aberrationVelocityJd: Option[Decimal]**：若表示“某速度对应的 JD 偏移”等，属内部/可选，可保留。
- **SolarTerm 的 onIteration / externalLongitudeFromGCRF**：回调参数中既有角度也有 JD 等，可逐步把“角度”改为 `PlaneAngle`，“距离”改为 `Length`，不必一次全改。

### 5. 容差 / 配置

- **coarseToleranceRad: Option[Decimal]**、**refForContinuity: Option[Decimal]**：弧度标量或“参考角”，保留 `Decimal` 或改为 `PlaneAngle` 均可，视 API 风格而定。

---

## 三、元组替代建议汇总

| 当前签名/类型 | 含义 | 建议 |
|---------------|------|------|
| `(Decimal, Decimal)` 章动 | Δψ, Δε（弧度） | `Nutation(psi, epsilon: PlaneAngle)` 或 `(PlaneAngle, PlaneAngle)` |
| `(TimePoint, Position, Velocity)` | 推迟时 + 位置速度 | 已为物理量，可保留；若多处使用可考虑 `case class RetardedResult(tr: TimePoint, position: Position, velocity: Velocity)` |
| `(Decimal, Decimal, Decimal, Decimal)` 等长列表 | 多为 (λ, β, …) 或 (x,y,z,vx,vy,vz) 等 | 角度用 `PlaneAngle`，长度/速度用 `Length`/`Velocity`，或 small case class 命名各分量 |
| `(Vsop87, Decimal)` | 历表 + 质量比 | 保留或 `case class BarycentricPlanet(vsop: Vsop87, massRatio: Decimal)` |

---

## 四、建议实施顺序

1. **低风险、高一致**：大气折射的“角度入参 + 角度返回”改为 `PlaneAngle`；`initialDistanceMeters` → `Option[Length]`；`prevSunDist`/`prevMoonDist` → `Option[Length]`。
2. **类型清晰**：章动 `(Decimal, Decimal)` → `(PlaneAngle, PlaneAngle)` 或 `Nutation`；坐标点 `Point` 各字段改为 `PlaneAngle` / `Option[Length]`。
3. **可选**：`TimeScaleContext.deltaT` 返回 `TimeInterval`；内部 `sphericalToCartesian` 等用物理量入参；为质量比或回调参数引入小类型。

完成上述后，astronomy 对外 API 将统一为“有物理概念的用物理量，仅无量纲或内部数值的用 Decimal”，便于维护和避免单位误用。

---

## 五、Rust core：裸 f64 → 物理量审计

以下为 `rust/core/src` 中**可用物理量替代**的裸 f64，按模块与类型分类。

### 5.1 角度（→ `PlaneAngle`）

| 位置 | 当前 | 建议 |
|------|------|------|
| **atmospheric_refraction** | `bennett_refraction_rad(altitude_rad: f64, ...) -> f64` | 入参、返回值改为 `PlaneAngle` |
| | `saemundsson_refraction_rad(h_rad, ...) -> f64` | 同上 |
| | `smart_refraction_rad(ha_rad, ...) -> f64`、`meeus_refraction_rad(altitude_rad, ...) -> f64` | 同上 |
| **solar_term** | `solar_longitude_jd(..., target_longitude_rad: f64, tolerance_rad: f64)` | `PlaneAngle` |
| | `solar_term_longitude_rad(term_index) -> f64` | 返回 `PlaneAngle` |
| | `solar_term_jd(..., tolerance_rad: f64)`、`solar_term_jds_in_range(..., tolerance_rad)` | `tolerance_rad` → `PlaneAngle` |
| | `approximate_solar_longitude_jd(..., longitude_ref_rad, target_longitude_rad)` | 内部；可改为 `PlaneAngle` |
| **synodic** | `expected_new_moon_longitude_difference(jd) -> f64` | 返回 `PlaneAngle` |
| | `NewMoonOptions.coarse_tolerance_rad: Option<f64>`、各 `tolerance_rad: f64` | `Option<PlaneAngle>` / `PlaneAngle` |
| **axial_precession** | `zeta(t) -> f64`、`theta(t) -> f64`、`z(t) -> f64` | 内部；可返回 `PlaneAngle` |
| | `rotation_z(angle: f64)`、`rotation_y(angle)`、`rotation_x(angle)` | 入参 `PlaneAngle`（内部仍用 `.rad()`） |
| **apparent** | `rotation_x(angle: f64)`、`rotation_z(angle: f64)` | 同上 |
| | `nutation_matrix(eps_mean: f64, dpsi: f64, deps: f64)` | 三个角 → `PlaneAngle` |
| | `spherical_to_cartesian(l, b, r: f64)` | l,b → `PlaneAngle`，r 见长度 |
| | `spherical_to_cartesian_with_velocity(..., dl, db, dr: f64)` | dl,db → `AngularRate`，dr 见速度 |
| **nutation** | `fundamental_arguments_rad(t) -> [f64; 5]` | 可返回 `[PlaneAngle; 5]`（或保留弧度给级数用） |
| **vsop87_de406_icrs_patch** | `apply_patch_to_equatorial_for_geocentric_sun(x,y,z)` 内 `ra`/`dec` 用 `atan2`/`asin` 得 f64 | 中间量；可封装为 `PlaneAngle` 再参与运算 |
| **calendar/chinese_lunar**、**calendar/mod** | `tolerance_rad: f64` 透传 | 改为 `PlaneAngle` 与 solar_term/synodic 一致 |

### 5.2 长度 / 距离（→ `Length`）

| 位置 | 当前 | 建议 |
|------|------|------|
| **apparent** | `sun_position_icrs_meters(...) -> (f64, f64, f64)` | 返回 `Position` 或 `(Length, Length, Length)` / `Displacement` |
| **vsop87_de406_icrs_patch** | `apply_patch_to_equatorial_for_geocentric_sun(x, y, z: f64) -> (f64, f64, f64)` | 入参/返回可为 `Position` 或 AU 约定下保留 f64 并注释；内部 `d_r` 已为 `Length` |
| **elpmpp02** | 部分 `vel_km_per_century` / 86400 等得到速度 | 用 `Speed` 构造（若尚未） |
| **axial_precession** | `precession_apply(t, v: [f64; 3]) -> [f64; 3]` | `v` 可为 `Displacement`，返回亦然（或保持与矩阵 API 一致用 f64） |

### 5.3 角速度 / 速度（→ `AngularRate` / `Speed`）

| 位置 | 当前 | 建议 |
|------|------|------|
| **solar_term** | `MEAN_SOLAR_LONGITUDE_VELOCITY_RAD_PER_DAY: f64` | 改为 `AngularRate` 常量（如 `from_rad_per_day(...)`） |
| **synodic** | `MEAN_SYNODIC_VELOCITY_RAD_PER_DAY: f64` | 同上 |

### 5.4 时间（→ `Duration` / 保持 JD）

| 位置 | 当前 | 建议 |
|------|------|------|
| **time** | `delta_t_seconds(jd_tt: f64) -> f64` | 可返回 `Duration`（调用方用 `.seconds()`） |
| | `TimeInterval::from_days/from_seconds(f64)`、`.in_days()/.in_seconds()` | 已是物理量接口，保留 f64 标量合理 |
| **solar_term** | `NUMERICAL_DELTA_JD`、`EARLY_DELTA_JD` 等 | 内部常数，可保留或 `Duration` |

### 5.5 保留或低优先级

- **JD / 儒略世纪 t**：`jd: f64`、`t: f64`（儒略世纪）作为时间自变量广泛使用，保留 f64 可接受；`TimePoint` 已包装 JD。
- **历表/级数内部**：VSOP87、ELP、章动等系数、`t_pow`、`vel_scalar`、多项式自变量等为无量纲或约定单位，保留 f64。
- **math/series**：`arcsec_to_rad`、`power_series_at` 等为通用数学，入参/返回值 f64 合理。
- **quantity 自身**：`from_rad`、`meters()` 等为量纲边界，保留 f64。
- **calendar/gregorian**：`from_julian_day(jd: f64)` 等为日期↔JD 换算，保留 f64。

### 5.6 建议实施顺序（Rust）

1. ~~**高可见度、低风险**：大气折射 API 改为 `PlaneAngle` 入参/返回；`solar_term_longitude_rad` 返回 `PlaneAngle`；`expected_new_moon_longitude_difference` 返回 `PlaneAngle`。~~ ✅ 已做（且已去掉兼容接口）。
2. ~~**定气/定朔容差**：`tolerance_rad` 全部改为 `PlaneAngle`（solar_term、synodic、calendar 透传）。~~ ✅ 已做。
3. ~~**常量角速度**：`MEAN_SOLAR_LONGITUDE_VELOCITY_RAD_PER_DAY`、`MEAN_SYNODIC_VELOCITY_RAD_PER_DAY` 改为 `AngularRate`。~~ ✅ 已做：`mean_solar_longitude_velocity()`、`mean_synodic_velocity()` 返回 `AngularRate`。
4. ~~**可选**：`delta_t_seconds` 返回 `Duration`；`fundamental_arguments_rad` → `fundamental_arguments` 返回 `[PlaneAngle; 5]`；`sun_position_icrs_meters` → `sun_position_icrs` 返回 `Position`。~~ ✅ 已做。
5. ~~**仍可选**：岁差/章动 `rotation_*` 入参改为 `PlaneAngle`；`nutation_matrix` 三参数改为 `PlaneAngle`；`apply_patch_to_equatorial_for_geocentric_sun` 接受/返回 `Position`；`stephenson_morrison_delta_t_*` 返回 `Duration`。~~ ✅ 已做。
