<script setup lang="ts">
// 文档动画：src 为 public/media/ 下不含扩展名的路径（如 celestial-sphere/precession），
// 同名 .mp4 与封面 .jpg 由 scripts/render_manim.sh 生成；withBase 处理部署子路径。
import { withBase } from 'vitepress'

const props = defineProps<{ src: string; caption?: string }>()
const mp4 = withBase(`/media/${props.src}.mp4`)
const poster = withBase(`/media/${props.src}.jpg`)
</script>

<template>
  <figure class="anim">
    <video controls loop muted playsinline preload="metadata" :poster="poster">
      <source :src="mp4" type="video/mp4" />
    </video>
    <figcaption v-if="caption">{{ caption }}</figcaption>
  </figure>
</template>
