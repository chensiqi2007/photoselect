<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import {
  Check,
  X,
  Star,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Layers,
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

const isStackedCollapsed = (photo: PhotoGroup) => {
  if (!store.isGroupFoldedMode) return false;
  const gid = photo.burst_group_id || photo.subject_group_id;
  if (!gid) return false;
  const info = store.groupInfoMap.get(gid);
  return info && info.count > 1 && !info.isExpanded;
};

const getGroupSummary = (photo: PhotoGroup) => {
  const gid = photo.burst_group_id || photo.subject_group_id;
  if (!gid) return null;
  const info = store.groupInfoMap.get(gid);
  if (!info || info.count <= 1) return null;
  return {
    gid,
    count: info.count,
    isExpanded: info.isExpanded,
  };
};
</script>

<template>
  <div class="w-full h-full flex flex-col bg-[#141418] select-none overflow-hidden transition-colors">
    <!-- 主网格滚动容器 -->
    <div
      ref="gridContainerRef"
      class="flex-1 overflow-y-auto p-4 md:p-6 scroll-smooth"
    >
      <div
        class="grid gap-3.5 transition-all duration-150 ease-out"
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
            'group relative aspect-4/3 rounded-xl bg-[#1A1A20] cursor-pointer overflow-hidden transition-all duration-150 ease-out flex flex-col justify-between border shadow-sm',
            // 折叠叠放视觉层次阴影
            isStackedCollapsed(photo)
              ? 'shadow-[4px_4px_0px_#2E2E38,8px_8px_0px_#1C1C22] ring-1 ring-purple-500/80'
              : '',
            // 选定高亮逻辑
            index === store.selectedIndex
              ? 'border-sky-400 ring-2 ring-sky-400 shadow-[0_0_20px_rgba(56,189,248,0.35)] scale-[1.015] z-10'
              : index === store.compareIndex
              ? 'border-emerald-400 ring-2 ring-emerald-400 shadow-[0_0_20px_rgba(52,211,153,0.35)] scale-[1.015] z-10'
              : 'border-[#26262C] hover:border-[#4A4A58] hover:-translate-y-0.5 hover:shadow-lg opacity-90 hover:opacity-100',
            colorClassMap[photo.color_label] || '',
          ]"
        >
          <!-- 淘汰半透明遮罩 -->
          <div
            v-if="photo.flag === -1"
            class="absolute inset-0 bg-black/65 z-10 flex items-center justify-center pointer-events-none backdrop-blur-[0.5px]"
          >
            <div class="w-9 h-9 rounded-full bg-rose-500/90 flex items-center justify-center text-white shadow-xl">
              <X class="w-5 h-5 stroke-[3]" />
            </div>
          </div>

          <!-- 缩略图图片 (极速 20KB 原生轻量流) -->
          <img
            :src="store.getThumbnailUrl(photo)"
            loading="lazy"
            decoding="async"
            alt=""
            class="absolute inset-0 w-full h-full object-cover select-none pointer-events-none transition-transform duration-200 group-hover:scale-[1.02]"
          />

          <!-- 卡片顶部悬浮条: 编号 + 留用徽章 + 对比身份徽章 + 叠放操作 -->
          <div class="relative z-20 p-2 flex items-center justify-between bg-gradient-to-b from-black/80 via-black/40 to-transparent">
            <div class="flex items-center space-x-1">
              <span class="text-[10px] font-mono font-bold text-white/90 bg-black/70 px-1.5 py-0.5 rounded backdrop-blur-sm shadow-sm">
                #{{ index + 1 }}
              </span>

              <!-- 对比角色徽章 -->
              <span
                v-if="index === store.selectedIndex && (store.viewMode === 'compare' || store.compareIndex !== null)"
                class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded bg-blue-600 text-white shadow-sm"
              >
                {{ t.grid.refBadge }}
              </span>
              <span
                v-else-if="index === store.compareIndex"
                class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded bg-emerald-600 text-white shadow-sm"
              >
                {{ t.grid.candBadge }}
              </span>

              <!-- 连拍/同主体动作与最佳代表帧徽章 -->
              <span
                v-if="photo.burst_group_id || photo.subject_group_id"
                :class="[
                  'text-[9px] font-mono font-bold px-1.5 py-0.2 rounded flex items-center space-x-0.5 shadow-sm',
                  (photo.is_burst_best || photo.is_subject_best) ? 'bg-amber-400 text-black font-extrabold' : 'bg-purple-600/95 text-white'
                ]"
                :title="(photo.is_burst_best || photo.is_subject_best) ? '推荐最佳代表帧 (Best Shot)' : `分组 ${photo.burst_group_id || photo.subject_group_id}`"
              >
                <span>{{ (photo.is_burst_best || photo.is_subject_best) ? '★ BEST' : (photo.burst_group_id ? 'BURST' : 'SUBJ') }}</span>
              </span>

              <!-- 严格重复帧警示 -->
              <span
                v-if="photo.is_duplicate"
                class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded bg-rose-600/90 text-white shadow-sm"
                title="与前帧内容极度雷同/重复"
              >
                DUP
              </span>
            </div>

            <!-- 右侧操作栏: 折叠展开控件 + 留优汰次 + 留用旗标 -->
            <div class="flex items-center space-x-1">
              <!-- 折叠展开按钮 -->
              <button
                v-if="getGroupSummary(photo)"
                @click.stop="store.toggleGroupExpanded(getGroupSummary(photo)!.gid)"
                :class="[
                  'text-[9px] font-mono font-bold px-2 py-0.5 rounded shadow-sm flex items-center space-x-1 cursor-pointer transition-all active:scale-95',
                  getGroupSummary(photo)!.isExpanded
                    ? 'bg-[#222228]/90 hover:bg-[#32323A] text-purple-300 border border-purple-500/40'
                    : 'bg-purple-600 hover:bg-purple-500 text-white'
                ]"
                :title="getGroupSummary(photo)!.isExpanded ? '点击折叠此组' : `该组共叠放 ${getGroupSummary(photo)!.count} 张，点击展开`"
              >
                <Layers class="w-2.5 h-2.5" />
                <span>{{ getGroupSummary(photo)!.isExpanded ? '折叠' : `${getGroupSummary(photo)!.count}张` }}</span>
              </button>

              <!-- 留优汰次一键高能动作 (仅折叠状态或组代表帧展示) -->
              <button
                v-if="getGroupSummary(photo) && !getGroupSummary(photo)!.isExpanded"
                @click.stop="store.pickBestAndRejectRest(getGroupSummary(photo)!.gid)"
                class="text-[9px] font-mono font-bold px-2 py-0.5 rounded bg-amber-400 hover:bg-amber-300 text-black shadow-sm cursor-pointer transition-all active:scale-95"
                :title="`一键选优：保留最佳帧(Pick)，其余 ${getGroupSummary(photo)!.count - 1} 张全部淘汰(Reject)`"
              >
                ★ 留优汰次
              </button>

              <!-- 留用标记 -->
              <div v-if="photo.flag === 1" class="bg-emerald-500 text-white rounded p-0.5 shadow-md">
                <Check class="w-3 h-3 stroke-[3]" />
              </div>
            </div>
          </div>

          <!-- 卡片底部文件名与参数 -->
          <div class="relative z-20 p-2 flex items-center justify-between bg-gradient-to-t from-black/85 via-black/45 to-transparent">
            <div class="truncate text-[11px] font-mono text-white/95 font-medium max-w-[65%] flex items-center space-x-1">
              <span class="truncate">{{ photo.base_name }}</span>
              <span
                v-if="photo.tags && photo.tags.length > 0"
                class="text-[9px] text-sky-300 bg-sky-500/20 px-1 rounded font-normal shrink-0 border border-sky-500/30"
                :title="photo.tags.join(', ')"
              >
                {{ photo.tags.length }}
              </span>
            </div>

            <div class="flex items-center space-x-1.5">
              <span class="text-[9px] font-mono font-bold px-1.5 py-0.2 rounded bg-black/60 text-white/90 border border-white/10">
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
        class="w-full h-full flex items-center justify-center text-[#71717A] text-sm italic py-20"
      >
        {{ t.grid.noPhotos }}
      </div>
    </div>

    <!-- 底部网格控制与状态工具条 (36px) -->
    <div
      class="h-9 px-4 bg-[#16161A] border-t border-[#26262C] flex items-center justify-between text-xs text-[#8E8E98] flex-shrink-0 z-20 transition-colors"
    >
      <!-- 左侧统计 -->
      <div class="flex items-center space-x-3 font-mono text-[11px]">
        <span class="text-[#F4F4F5] font-semibold">{{ t.grid.totalPhotos.replace('{0}', store.filteredGroups.length.toString()) }}</span>
        <span class="text-white/20">|</span>
        <span class="text-[#A1A1AA]">{{ t.grid.selectedIndex.replace('{0}', (store.filteredGroups.length > 0 ? store.selectedIndex + 1 : 0).toString()) }}</span>
        <span v-if="store.compareIndex !== null" class="text-emerald-400 font-medium">
          {{ t.grid.compareTarget.replace('{0}', (store.compareIndex + 1).toString()) }}
        </span>
      </div>

      <!-- 中间操作提示 -->
      <div class="hidden md:flex items-center space-x-2 text-[11px] text-[#71717A]">
        <span>{{ t.grid.hints }}</span>
      </div>

      <!-- 右侧网格缩放滑块与微调 -->
      <div class="flex items-center space-x-2">
        <button
          @click="store.setGridItemSize(store.gridItemSize - 20)"
          class="p-1 text-[#8E8E98] hover:text-white transition-colors cursor-pointer"
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
          class="w-24 h-1 bg-[#26262C] rounded-lg appearance-none cursor-pointer accent-sky-400"
          title="调节缩略图尺寸"
        />

        <button
          @click="store.setGridItemSize(store.gridItemSize + 20)"
          class="p-1 text-[#8E8E98] hover:text-white transition-colors cursor-pointer"
          title="放大缩略图 [Ctrl + 滚轮向上]"
        >
          <ZoomIn class="w-3.5 h-3.5" />
        </button>

        <span class="font-mono text-[11px] text-[#A1A1AA] w-12 text-right">
          {{ store.gridItemSize }}px
        </span>

        <!-- 重置尺寸为 220px -->
        <button
          @click="store.setGridItemSize(220)"
          class="p-1 text-[#71717A] hover:text-[#F4F4F5] transition-colors cursor-pointer ml-1"
          :title="t.grid.resetGrid"
        >
          <RotateCcw class="w-3 h-3" />
        </button>
      </div>
    </div>
  </div>
</template>
