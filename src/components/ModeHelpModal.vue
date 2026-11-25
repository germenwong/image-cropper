<script setup lang="ts">
import { NModal, NTag } from "naive-ui";

defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "update:show", v: boolean): void }>();
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    title="两种裁切模式的区别"
    class="mode-help-modal"
    :style="{ width: '660px', maxWidth: '92vw' }"
    :bordered="false"
    @update:show="(v: boolean) => emit('update:show', v)"
  >
    <div class="sub">
      同一张 <b>1600 × 2133</b> 的原图，目标尺寸都设为 <b>800 × 800</b>，结果完全不同。
    </div>

    <!-- 示例图素材：只定义一次，下面用 use 复用 -->
    <svg width="0" height="0" style="position: absolute" aria-hidden="true">
      <defs>
        <g id="mode-demo-art">
          <rect width="1600" height="2133" fill="#e8ecf4" />
          <rect width="1600" height="1180" fill="#cfdae8" />
          <circle cx="1170" cy="470" r="180" fill="#f6d9a0" />
          <path d="M0 1180 L420 790 L860 1180 Z" fill="#93a6bd" />
          <path d="M600 1180 L1060 730 L1560 1180 Z" fill="#7589a3" />
          <rect y="1180" width="1600" height="953" fill="#f4f1ea" />
          <!-- 瓶身 -->
          <rect x="700" y="770" width="200" height="120" rx="24" fill="#2f3b52" />
          <rect x="640" y="880" width="320" height="820" rx="46" fill="#3c4a63" />
          <rect x="600" y="1650" width="400" height="26" rx="13" fill="#ded9cd" />
          <!-- 标签：中心 100×100 区域落在条形码上，放大后能看出是局部细节 -->
          <rect x="676" y="975" width="248" height="230" rx="12" fill="#eef2f9" />
          <text
            x="800"
            y="1035"
            text-anchor="middle"
            font-family="Arial, Helvetica, sans-serif"
            font-size="54"
            font-weight="700"
            fill="#2f3b52"
          >
            LBR
          </text>
          <g fill="#2f3b52">
            <rect x="700" y="1052" width="10" height="120" />
            <rect x="718" y="1052" width="6" height="120" />
            <rect x="734" y="1052" width="14" height="120" />
            <rect x="758" y="1052" width="6" height="120" />
            <rect x="774" y="1052" width="10" height="120" />
            <rect x="794" y="1052" width="18" height="120" />
            <rect x="822" y="1052" width="6" height="120" />
            <rect x="838" y="1052" width="12" height="120" />
            <rect x="860" y="1052" width="6" height="120" />
            <rect x="876" y="1052" width="14" height="120" />
          </g>
        </g>
      </defs>
    </svg>

    <div class="cols">
      <!-- 规格输出 -->
      <div class="col">
        <div class="col-head">
          <n-tag type="primary" size="small" :bordered="false">规格输出</n-tag>
          <span class="col-tag">推荐</span>
        </div>

        <div class="shots">
          <figure class="shot">
            <div class="src">
              <svg viewBox="0 0 1600 2133"><use href="#mode-demo-art" /></svg>
              <div class="box spec" />
            </div>
            <figcaption>框 = 1600 × 1600</figcaption>
          </figure>

          <span class="arrow">→</span>

          <figure class="shot">
            <div class="out">
              <div class="out-inner">
                <svg viewBox="0 0 1600 2133"><use href="#mode-demo-art" /></svg>
              </div>
            </div>
            <figcaption>输出 800 × 800<span class="note">整图缩略</span></figcaption>
          </figure>
        </div>

        <p class="desc">
          按目标比例在原图里取最大的一块（1600 × 1600），再整体缩放到 800 × 800。
          <b>内容完整</b>，适合把一堆不同尺寸的图统一成同一规格。
        </p>
      </div>

      <!-- 像素裁切 -->
      <div class="col">
        <div class="col-head">
          <n-tag size="small" :bordered="false">像素裁切</n-tag>
        </div>

        <div class="shots">
          <figure class="shot">
            <div class="src">
              <svg viewBox="0 0 1600 2133"><use href="#mode-demo-art" /></svg>
              <div class="box pixel" />
            </div>
            <figcaption>框 = 中心 800 × 800</figcaption>
          </figure>

          <span class="arrow">→</span>

          <figure class="shot">
            <div class="out">
              <div class="out-inner zoom">
                <svg viewBox="0 0 1600 2133"><use href="#mode-demo-art" /></svg>
              </div>
            </div>
            <figcaption>输出 800 × 800<span class="note">只取中心局部</span></figcaption>
          </figure>
        </div>

        <p class="desc">
          直接从原图中心抠出 800 × 800，<b>原尺寸输出、不缩放</b>。
          只保留局部画面，但细节一点不损失，适合精确抠图，不适合批量统一规格。
        </p>
      </div>
    </div>

    <div class="foot">不确定选哪个就用「规格输出」；需要保留原始像素尺寸就用「像素裁切」。</div>
  </n-modal>
</template>

<style scoped>
.sub {
  font-size: 13px;
  color: var(--text-3);
  margin-bottom: 14px;
}
.cols {
  display: flex;
  gap: 18px;
}
.col {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  background: var(--surface-2);
}
.col-head {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 10px;
}
.col-tag {
  font-size: 11px;
  color: var(--text-4);
}
.shots {
  display: flex;
  align-items: center;
  gap: 8px;
}
.shot {
  margin: 0;
  flex: 0 0 auto;
  width: 108px;
}
figure figcaption {
  margin-top: 6px;
  font-size: 10.5px;
  line-height: 1.5;
  color: var(--text-4);
  text-align: center;
  white-space: normal;
  overflow-wrap: anywhere;
}
figcaption .note {
  display: block;
  font-size: 10px;
  color: var(--text-3);
}
.src {
  position: relative;
  width: 108px;
  aspect-ratio: 1600 / 2133;
  border-radius: 4px;
  overflow: hidden;
  background: var(--surface);
  box-shadow: 0 1px 6px var(--shadow);
}
.src svg {
  display: block;
  width: 100%;
  height: 100%;
}
.box {
  position: absolute;
  border: 1.5px solid #4c6ef5;
  box-shadow: 0 0 0 9999px rgba(15, 22, 45, 0.3);
}
.box.spec {
  left: 0;
  width: 100%;
  top: 12.5%;
  height: 75%;
}
.box.pixel {
  left: 50%;
  top: 50%;
  width: 50%;
  aspect-ratio: 1;
  transform: translate(-50%, -50%);
  border-width: 2px;
  box-shadow: 0 0 0 2px rgba(76, 110, 245, 0.45), 0 0 0 9999px rgba(15, 22, 45, 0.42);
}
.out {
  position: relative;
  width: 108px;
  aspect-ratio: 1;
  border-radius: 4px;
  overflow: hidden;
  background: var(--surface);
  box-shadow: 0 1px 6px var(--shadow);
}
.out-inner {
  position: absolute;
  left: 50%;
  top: 50%;
  width: 100%;
  transform: translate(-50%, -50%);
}
.out-inner svg {
  display: block;
  width: 100%;
  height: auto;
}
/* 像素裁切：只显示中心 800/1600 区域，视觉上等于放大 2 倍 */
.out-inner.zoom {
  width: 200%;
}
.arrow {
  color: var(--text-4);
  font-size: 14px;
  margin-top: -14px;
}
.desc {
  margin: 12px 0 0;
  font-size: 12px;
  line-height: 1.7;
  color: var(--text-2);
}
.foot {
  margin-top: 14px;
  font-size: 12px;
  color: var(--text-3);
  text-align: center;
}
</style>
