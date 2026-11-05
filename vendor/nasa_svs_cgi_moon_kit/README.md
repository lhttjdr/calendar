# NASA SVS CGI Moon Kit

原样保留，不改动。来源：<https://svs.gsfc.nasa.gov/4720>（CGI Moon Kit，Ernie Wright；月面彩色图据 LRO LROC WAC 拼图，两极用 LOLA 补齐）。

- `lroc_color_poles_2k.tif`（2048×1024，sRGB，3.3 MB，下载于 2026-11-05）：等距圆柱投影，中央经线为月面经度 0°，东经在右，上为月球北极；
  月面经纬度为 IAU 平地球/极轴（ME）坐标。由 `scripts/phenomena/moon_texture.py` 转成 `data/sky/moon_lroc_color_2k.jpg`（前端用）。
- `lroc_color_poles_1k.jpg`（1024×512）：官方 1K 预览图，供核对。

署名（NASA SVS 页面要求）：NASA's Scientific Visualization Studio。用途见 `doc/astronomy/celestial-phenomena.md`「月食的月面实况」。
