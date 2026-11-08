# 网页：择日页、排盘页与设备传感器

日历页只放历日与宜忌摘要；择日、排盘各为独立页面（顶栏「择日」「排盘」），与日历共用所选日期时刻（`App.tsx` 的 `shiftSelected` 调日、时辰），日柱、时柱取日历所示四柱（四柱预设、日界、当地太阳时皆随之）。

| 页 | 文件 | 内容 | wasm |
|---|---|---|---|
| 择日 | `web/src/ZeRiPage.tsx` | 年神方位二十四山圆图（协纪卷三）；当日建除、日游神鹤神太岁出游、神煞宜忌（月表 + 卷十铺注 + 时宪书体例、時憲書宜、時憲書朱字） | `nian_shen_json`、`shensha_day_wasm`、`jianchu_day_wasm` |
| 排盘 | `web/src/PaiPanPage.tsx` | 奇门九宫盘（转盘／飞盘、寄宫三说、八神名目、定局法、节气来源皆可选）；大六壬天地盘与四课三传（换将法可选）；太乙盘（外圈十六神、内圈九宫；岁月日时四计、积年两家）；三元九运；紫微（开发中） | `qimen_json`、`liuren_json`、`taiyi_json`、`paipan_wasm` |

**图式**：一律南上北下、东左西右（古图惯例）。二十四山圆图为罗盘式：每字字头朝外，绕圆心一周统一旋转（不为「便读」而翻转）；诸神每位一竖行，沿半径自外端起读，一山之内由内向外依次排，放不下时分列。

**传感器**（`web/src/sensors.ts`）：
- 定位：Geolocation API，取经纬度、海拔、精度；写回观测地（时柱当地时、更点、磁偏角共用），存于本机 localStorage。日历设置「观测地」与择日页均有按钮。
- 罗盘：DeviceOrientation。iOS 用 `webkitCompassHeading`（须在点击中 `requestPermission`），Android 用 `deviceorientationabsolute` 的 `alpha`（朝向 = 360 − alpha），加屏幕旋转角；角度圆周指数平滑。所得为磁北方位，加 WMM2025 磁偏角（`magnetic_declination_wasm`，见 `doc/astronomy/geomag.md`）为真北。二十四山图北在下，故整盘转 −真北方位 − 180°，手机平放时图上方位与实地相合。
- 平放检测：取 beta、gamma 较大者为倾角，超过 15° 提示放平（立起时 alpha 不再是平面朝向）；iOS 报 `webkitCompassAccuracy` 不佳（负或大于 15°）时提示远离金属、画「8」字校准。
- 接口只在安全上下文（https 或 localhost）可用。

**检验**：无头 Chromium（授权 accelerometer、gyroscope、magnetometer）派发 `deviceorientationabsolute`（alpha 90，即顶端指西）：读数真北 262°（磁北 270°，北京磁偏角西 7.6°），盘上酉转至上方、北在右，合「面西而北在右」。实机（iOS、Android）未测。
