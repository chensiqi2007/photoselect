<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import {
  Check,
  X,
  Star,
  ZoomIn,
  ZoomOut,
  RotateCcw,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type { PhotoGroup } from "../../types/photo";

const store = usePhotoStore();
const { t } = useI18n();
const gridContainerRef = ref<HTMLElement | null>(null);

// 缩放滚轮监听 (非 passive，拦截默认网页缩放并缩放网格卡片)
const handleWheel = (e: WheelEvent) => {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault();
    const delta = e.deltaY < 0 ? 16 : -16;
    store.setGridItemSize(store.gridItemSize + delta);
  }
};

onMounted(() => {
  if (gridContainerRef.value) {
    gridContainerRef.value.addEventListener("wheel", handleWheel, {
      passive: false,
    });
  }
});

onUnmounted(() => {
  if (gridContainerRef.value) {
    gridContainerRef.value.removeEventListener("wheel", handleWheel);
  }
});

const getBadgeLabel = (photo: PhotoGroup): string => {
  if (photo.std_image && photo.raw_image) return "RAW+JPG";
  if (photo.raw_image) return "RAW";
  return "JPG";
};

// 单击照片卡片 (支持按住 Ctrl 选择第 2 张对比图)
const handleCardClick = (index: number, e: MouseEvent) => {
  const isCtrl = e.ctrlKey || e.metaKey;
  store.selectIndex(index, isCtrl);
};

const handleDoubleClick = (index: number) => {
  store.selectIndex(index, false);
  store.viewMode = "loupe";
};

const colorClassMap: Record<string, string> = {
  red: "border-b-4 border-b-[#EF4444]",
  yellow: "border-b-4 border-b-[#F59E0B]",
  green: "border-b-4 border-b-[#10B981]",
  blue: "border-b-4 border-b-[#3B82F6]",
  purple: "border-b-4 border-b-[#8B5CF6]",
  none: "border-b-4 border-b-transparent",
};
</script>

<template>
  <div class="w-full h-full flex flex-col bg-[#1E1E1E] select-none overflow-hidden transition-colors">
    <!-- 主网格滚动容器 -->
    <div
      ref="gridContainerRef"
      class="flex-1 overflow-y-auto p-4 scroll-smooth"
    >
      <div
        class="grid gap-3 transition-all duration-150 ease-linear"
        :style="{
          gridTemplateColumns: `repeat(auto-fill, minmax(${store.gridItemSize}px, 1fr))`,
        }"
      >
        <div
          v-for="(photo, index) in store.filteredGroups"
          :key="photo.id"
          @click="(e) => handleCardClick(index, e)"
          @dblclick="handleDoubleClick(index)"
          :class="[
            'group relative aspect-4/3 rounded bg-[#181818] cursor-pointer overflow-hidden transition-all duration-150 ease-linear flex flex-col justify-between border',
            // 选定高亮逻辑
            index === store.selectedIndex
              ? 'border-blue-500 ring-2 ring-blue-500 shadow-2xl scale-[1.01] z-10'
              : index === store.compareIndex
              ? 'border-emerald-500 ring-2 ring-emerald-500 shadow-2xl scale-[1.01] z-10'
              : 'border-[#2A2A2A] hover:border-[#4A4A4A] opacity-90 hover:opacity-100',
            colorClassMap[photo.color_label] || '',
          ]"
        >
          <!-- 淘汰半透明遮罩 -->
          <div
            v-if="photo.flag === -1"
            class="absolute inset-0 bg-black/60 z-10 flex items-center justify-center pointer-events-none"
          >
            <div class="w-8 h-8 rounded-full bg-rose-500/80 flex items-center justify-center text-white shadow-lg">
              <X class="w-5 h-5 stroke-[3]" />
            </div>
          </div>

          <!-- 缩略图图片 (极速 20KB 原生轻量流) -->
          <img
            :src="store.getThumbnailUrl(photo)"
            loading="lazy"
            decoding="async"
            alt=""
            class="absolute inset-0 w-full h-full object-cover select-none pointer-events-none"
          />

          <!-- 卡片顶部悬浮条: 编号 + 留用徽章 + 对比身份徽章 -->
          <div class="relative z-20 p-2 flex items-center justify-between bg-gradient-to-b from-black/70 to-transparent">
            <div class="flex items-center space-x-1">
              <span class="text-[10px] font-mono font-bold text-white/90 bg-black/60 px-1.5 py-0.5 rounded">
                #{{ index + 1 }}
              </span>

              <!-- 对比角色徽章 -->
              <span
                v-if="index === store.selectedIndex && (store.viewMode === 'compare' || store.compareIndex !== null)"
                class="text-[9px] font-mono font-bold px-1 rounded bg-blue-600 text-white shadow"
              >
                {{ t.grid.refBadge }}
              </span>
              <span
                v-else-if="index === store.compareIndex"
                class="text-[9px] font-mono font-bold px-1 rounded bg-emerald-600 text-white shadow"
              >
                {{ t.grid.candBadge }}
              </span>
            </div>

            <!-- 留用标记 -->
            <div v-if="photo.flag === 1" class="bg-emerald-500 text-white rounded p-0.5 shadow">
              <Check class="w-3 h-3 stroke-[3]" />
            </div>
          </div>

          <!-- 卡片底部文件名与参数 -->
          <div class="relative z-20 p-2 flex items-center justify-between bg-gradient-to-t from-black/80 via-black/40 to-transparent">
            <div class="truncate text-[11px] font-mono text-white/90 font-medium max-w-[65%]">
              {{ photo.base_name }}
            </div>

            <div class="flex items-center space-x-1.5">
              <span class="text-[9px] font-mono font-bold px-1 rounded bg-black/60 text-white/90">
                {{ getBadgeLabel(photo) }}
              </span>
              <div v-if="photo.rating > 0" class="flex items-center space-x-0.5">
                <Star
                  v-for="s in photo.rating"
                  :key="s"
                  class="w-2.5 h-2.5 text-amber-400 fill-amber-400"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <div
        v-if="store.filteredGroups.length === 0"
        class="w-full h-full flex items-center justify-center text-[#8C8C8C] text-sm italic py-20"
      >
        {{ t.grid.noPhotos }}
      </div>
    </div>

    <!-- 底部网格控制与状态工具条 (34px) -->
    <div
      class="h-8 px-4 bg-[#161616] border-t border-[#2A2A2A] flex items-center justify-between text-xs text-[#8C8C8C] flex-shrink-0 z-20 transition-colors"
    >
      <!-- 左侧统计 -->
      <div class="flex items-center space-x-3 font-mono text-[11px]">
        <span>{{ t.grid.totalPhotos.replace('{0}', store.filteredGroups.length.toString()) }}</span>
        <span class="text-[#CCCCCC]">{{ t.grid.selectedIndex.replace('{0}', (store.filteredGroups.length > 0 ? store.selectedIndex + 1 : 0).toString()) }}</span>
        <span v-if="store.compareIndex !== null" class="text-emerald-400">
          {{ t.grid.compareTarget.replace('{0}', (store.compareIndex + 1).toString()) }}
        </span>
      </div>

      <!-- 中间操作提示 -->
      <div class="hidden md:flex items-center space-x-2 text-[11px] text-[#777777]">
        <span>{{ t.grid.hints }}</span>
      </div>

      <!-- 右侧网格缩放滑块与微调 -->
      <div class="flex items-center space-x-2">
        <button
          @click="store.setGridItemSize(store.gridItemSize - 20)"
          class="p-1 hover:text-white transition-colors cursor-pointer"
          title="缩小缩略图 [Ctrl + 滚轮向下]"
        >
          <ZoomOut class="w-3.5 h-3.5" />
        </button>

        <input
          type="range"
          min="110"
          max="480"
          step="10"
          :value="store.gridItemSize"
          @input="(e) => store.setGridItemSize(parseInt((e.target as HTMLInputElement).value, 10))"
          class="w-24 h-1 bg-[#2A2A2A] rounded-lg appearance-none cursor-pointer accent-sky-400"
          title="调节缩略图尺寸"
        />

        <button
          @click="store.setGridItemSize(store.gridItemSize + 20)"
          class="p-1 hover:text-white transition-colors cursor-pointer"
          title="放大缩略图 [Ctrl + 滚轮向上]"
        >
          <ZoomIn class="w-3.5 h-3.5" />
        </button>

        <span class="font-mono text-[11px] text-[#AAAAAA] w-10 text-right">
          {{ store.gridItemSize }}px
        </span>

        <!-- 重置尺寸为 220px -->
        <button
          @click="store.setGridItemSize(220)"
          class="p-1 text-[#666666] hover:text-[#CCCCCC] transition-colors cursor-pointer ml-1"
          :title="t.grid.resetGrid"
        >
          <RotateCcw class="w-3 h-3" />
        </button>
      </div>
    </div>
  </div>
</template>
