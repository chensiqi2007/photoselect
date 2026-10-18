<script setup lang="ts">
import { ref, computed } from "vue";
import {
  ArrowLeftRight,
  Check,
  X,
  Star,
  Lock,
  Unlock,
  Maximize2,
  Minimize2,
  Info,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type { ColorLabel, FlagStatus, PhotoGroup } from "../../types/photo";
import EyeInspectionPanel from "../EyeInspection/EyeInspectionPanel.vue";

const store = usePhotoStore();
const { t } = useI18n();

const photoA = computed(() => store.comparePhotoA);
const photoB = computed(() => store.comparePhotoB);

// 缩放平移状态 (支持同步联动与独立控制)
const scaleA = ref(1.0);
const panA = ref({ x: 0, y: 0 });
const scaleB = ref(1.0);
const panB = ref({ x: 0, y: 0 });
const isDragging = ref<"A" | "B" | null>(null);
const dragStart = ref({ x: 0, y: 0 });

// 双图缩放联动
const handleWheel = (e: WheelEvent, side: "A" | "B") => {
  e.preventDefault();
  const zoomFactor = e.deltaY < 0 ? 1.15 : 0.85;

  if (store.syncCompareZoom) {
    const newScale = Math.max(0.5, Math.min(6.0, scaleA.value * zoomFactor));
    scaleA.value = newScale;
    scaleB.value = newScale;
  } else {
    if (side === "A") {
      scaleA.value = Math.max(0.5, Math.min(6.0, scaleA.value * zoomFactor));
    } else {
      scaleB.value = Math.max(0.5, Math.min(6.0, scaleB.value * zoomFactor));
    }
  }
};

const handleMouseDown = (e: MouseEvent, side: "A" | "B") => {
  isDragging.value = side;
  const currentPan = side === "A" ? panA.value : panB.value;
  dragStart.value = { x: e.clientX - currentPan.x, y: e.clientY - currentPan.y };
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return;
  const newX = e.clientX - dragStart.value.x;
  const newY = e.clientY - dragStart.value.y;

  if (store.syncCompareZoom) {
    panA.value = { x: newX, y: newY };
    panB.value = { x: newX, y: newY };
  } else {
    if (isDragging.value === "A") {
      panA.value = { x: newX, y: newY };
    } else {
      panB.value = { x: newX, y: newY };
    }
  }
};

const handleMouseUp = () => {
  isDragging.value = null;
};

// 一键重置 / 100% 切换
const is100Percent = computed(() => scaleA.value > 1.2 || scaleB.value > 1.2);
const toggle100Percent = () => {
  if (is100Percent.value) {
    scaleA.value = 1.0;
    scaleB.value = 1.0;
    panA.value = { x: 0, y: 0 };
    panB.value = { x: 0, y: 0 };
  } else {
    scaleA.value = 2.5;
    scaleB.value = 2.5;
  }
};

// 快捷裁决动作
const pickAAndRejectB = async () => {
  if (photoA.value) await store.setFlagForPhoto(photoA.value, 1 as FlagStatus);
  if (photoB.value) await store.setFlagForPhoto(photoB.value, -1 as FlagStatus);
};

const pickBAndRejectA = async () => {
  if (photoB.value) await store.setFlagForPhoto(photoB.value, 1 as FlagStatus);
  if (photoA.value) await store.setFlagForPhoto(photoA.value, -1 as FlagStatus);
};

const getBadgeLabel = (photo: PhotoGroup | null): string => {
  if (!photo) return "";
  if (photo.std_image && photo.raw_image) return "RAW+JPG";
  if (photo.raw_image) return "RAW";
  return "JPG";
};
</script>

<template>
  <div
    class="w-full h-full flex flex-col bg-[#161616] select-none overflow-hidden transition-colors"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <!-- 顶部对比控制条 (38px) -->
    <div
      class="h-9 px-4 bg-[#1C1C1C] border-b border-[#2A2A2A] flex items-center justify-between text-xs flex-shrink-0 z-20 transition-colors"
    >
      <!-- 左侧: 图 A 简标与互换 -->
      <div class="flex items-center space-x-2">
        <span class="px-2 py-0.5 rounded bg-blue-600/30 border border-blue-500/40 text-blue-300 font-mono text-[11px] font-bold">
          [A] {{ t.compare.refPhoto }}: {{ photoA ? photoA.base_name : "--" }}
        </span>

        <!-- 互换位置按钮 [S] -->
        <button
          @click="store.swapComparePhotos"
          class="flex items-center space-x-1 px-2.5 py-1 rounded bg-[#262626] hover:bg-[#333333] text-[#E5E5E5] border border-[#3A3A3A] transition-all duration-150 ease-linear cursor-pointer font-medium text-[11px] active:scale-95 shadow-sm"
          :title="t.compare.swap"
        >
          <ArrowLeftRight class="w-3.5 h-3.5 text-sky-400" />
          <span>{{ t.compare.swap }}</span>
        </button>
      </div>

      <!-- 中间: 极速裁决按钮 + 联动控制 -->
      <div class="flex items-center space-x-2">
        <!-- 留用 A · 淘汰 B -->
        <button
          @click="pickAAndRejectB"
          class="flex items-center space-x-1 px-2.5 py-1 rounded bg-emerald-500/15 hover:bg-emerald-500/25 text-emerald-400 border border-emerald-500/30 transition-all duration-150 ease-linear cursor-pointer font-medium text-[11px]"
          :title="t.compare.pickARejectB"
        >
          <Check class="w-3.5 h-3.5 stroke-[2.5]" />
          <span>{{ t.compare.pickARejectB }}</span>
        </button>

        <!-- 留用 B · 淘汰 A -->
        <button
          @click="pickBAndRejectA"
          class="flex items-center space-x-1 px-2.5 py-1 rounded bg-teal-500/15 hover:bg-teal-500/25 text-teal-300 border border-teal-500/30 transition-all duration-150 ease-linear cursor-pointer font-medium text-[11px]"
          :title="t.compare.pickBRejectA"
        >
          <Check class="w-3.5 h-3.5 stroke-[2.5]" />
          <span>{{ t.compare.pickBRejectA }}</span>
        </button>

        <div class="h-4 w-[1px] bg-[#333333]"></div>

        <!-- 双图同步缩放联动切换 -->
        <button
          @click="store.syncCompareZoom = !store.syncCompareZoom"
          :class="[
            'flex items-center space-x-1 px-2 py-1 rounded text-[11px] font-medium border transition-colors cursor-pointer',
            store.syncCompareZoom
              ? 'bg-amber-500/15 border-amber-500/30 text-amber-300'
              : 'bg-[#222222] border-[#333333] text-[#8C8C8C] hover:text-[#E5E5E5]',
          ]"
          :title="store.syncCompareZoom ? t.compare.syncZoom : t.compare.indepZoom"
        >
          <component :is="store.syncCompareZoom ? Lock : Unlock" class="w-3 h-3" />
          <span>{{ store.syncCompareZoom ? t.compare.syncZoom : t.compare.indepZoom }}</span>
        </button>

        <!-- 100% 锐度检查切换 -->
        <button
          @click="toggle100Percent"
          class="flex items-center space-x-1 px-2 py-1 rounded bg-[#262626] hover:bg-[#333333] text-[#CCCCCC] border border-[#3A3A3A] text-[11px] transition-colors cursor-pointer"
          :title="is100Percent ? t.compare.fitScreen : t.compare.sharpness100"
        >
          <component :is="is100Percent ? Minimize2 : Maximize2" class="w-3 h-3" />
          <span>{{ is100Percent ? t.compare.fitScreen : t.compare.sharpness100 }}</span>
        </button>
      </div>

      <!-- 右侧: 图 B 简标与操作提示 -->
      <div class="flex items-center space-x-2">
        <span class="px-2 py-0.5 rounded bg-emerald-600/30 border border-emerald-500/40 text-emerald-300 font-mono text-[11px] font-bold">
          [B] {{ t.compare.candPhoto }}: {{ photoB ? photoB.base_name : "--" }}
        </span>
      </div>
    </div>

    <!-- 中间: 双图左右并排视口 -->
    <div class="flex-1 flex overflow-hidden p-2 space-x-2">
      <!-- 1. 左侧面板: 图 A (参考基准) -->
      <div
        class="flex-1 relative rounded-xl bg-[#111111] border border-[#2A2A2A] overflow-hidden flex flex-col justify-between"
      >
        <!-- 顶部身份浮标 -->
        <div class="absolute top-3 left-3 z-20 flex items-center space-x-2">
          <span class="px-2 py-1 rounded bg-blue-600 text-white font-mono font-bold text-xs shadow-md">
            {{ t.compare.refPhoto }}
          </span>
          <span v-if="photoA" class="px-2 py-0.5 rounded bg-black/70 backdrop-blur-md text-[#E5E5E5] font-mono text-xs border border-[#333333]">
            {{ photoA.base_name }}
          </span>
          <span v-if="photoA" class="px-1.5 py-0.5 rounded bg-black/60 text-[10px] font-mono text-blue-300 border border-blue-500/30">
            {{ getBadgeLabel(photoA) }}
          </span>
        </div>

        <!-- 画布视口区域 -->
        <div
          class="w-full h-full flex items-center justify-center cursor-grab active:cursor-grabbing overflow-hidden"
          @wheel="(e) => handleWheel(e, 'A')"
          @mousedown="(e) => handleMouseDown(e, 'A')"
          @dblclick="toggle100Percent"
        >
          <div
            v-if="photoA"
            class="relative transition-transform duration-75 ease-linear"
            :style="{
              transform: `translate(${panA.x}px, ${panA.y}px) scale(${scaleA})`,
            }"
          >
            <img
              :src="store.getPreviewUrl(photoA)"
              alt=""
              class="max-w-[85vw] max-h-[80vh] object-contain select-none pointer-events-none drop-shadow-2xl"
              draggable="false"
            />
          </div>
          <div v-else class="text-xs text-[#555555] italic">{{ t.compare.noRef }}</div>
        </div>

        <!-- 淘汰黑遮罩 -->
        <div
          v-if="photoA && photoA.flag === -1"
          class="absolute inset-0 bg-black/65 z-10 flex items-center justify-center pointer-events-none"
        >
          <div class="p-4 bg-black/80 rounded-xl border border-rose-500/50 backdrop-blur-md flex items-center space-x-2 text-rose-400">
            <X class="w-8 h-8 stroke-[2.5]" />
            <span class="text-xs font-bold uppercase tracking-wider">{{ t.compare.rejectedBadge }}</span>
          </div>
        </div>

        <!-- 底部照片控制工具栏 -->
        <div
          v-if="photoA"
          class="absolute bottom-3 left-3 right-3 z-20 flex items-center justify-between px-3 py-2 rounded-lg bg-black/75 backdrop-blur-md border border-[#333333]"
        >
          <!-- 旗标 -->
          <div class="flex items-center space-x-1">
            <button
              @click="store.setFlagForPhoto(photoA, 1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2 py-0.5 rounded text-[11px] font-semibold transition-all cursor-pointer',
                photoA.flag === 1
                  ? 'bg-emerald-500 text-white shadow'
                  : 'bg-[#262626] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <Check class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.pick }}</span>
            </button>
            <button
              @click="store.setFlagForPhoto(photoA, -1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2 py-0.5 rounded text-[11px] font-semibold transition-all cursor-pointer',
                photoA.flag === -1
                  ? 'bg-rose-500 text-white shadow'
                  : 'bg-[#262626] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <X class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.reject }}</span>
            </button>
          </div>

          <!-- 星级评分 -->
          <div class="flex items-center space-x-0.5">
            <button
              v-for="star in 5"
              :key="star"
              @click="store.setRatingForPhoto(photoA, photoA.rating === star ? 0 : star)"
              class="p-0.5 hover:scale-120 transition-transform cursor-pointer"
            >
              <Star
                :class="[
                  'w-3.5 h-3.5',
                  photoA.rating >= star ? 'text-amber-400 fill-amber-400' : 'text-[#444444] hover:text-[#888888]',
                ]"
              />
            </button>
          </div>

          <!-- 快捷色标 -->
          <div class="flex items-center space-x-1.5">
            <button
              v-for="color in (['red', 'yellow', 'green', 'blue', 'purple'] as ColorLabel[])"
              :key="color"
              @click="store.setColorLabelForPhoto(photoA, color)"
              :class="[
                'w-3.5 h-3.5 rounded-full transition-transform cursor-pointer',
                photoA.color_label === color ? 'ring-2 ring-white scale-125' : 'opacity-70 hover:opacity-100',
                color === 'red' ? 'bg-[#EF4444]' : '',
                color === 'yellow' ? 'bg-[#F59E0B]' : '',
                color === 'green' ? 'bg-[#10B981]' : '',
                color === 'blue' ? 'bg-[#3B82F6]' : '',
                color === 'purple' ? 'bg-[#8B5CF6]' : '',
              ]"
            />
          </div>
        </div>
      </div>

      <!-- 2. 右侧面板: 图 B (候选对比) -->
      <div
        class="flex-1 relative rounded-xl bg-[#111111] border border-[#2A2A2A] overflow-hidden flex flex-col justify-between"
      >
        <!-- 顶部身份浮标 -->
        <div class="absolute top-3 left-3 z-20 flex items-center space-x-2">
          <span class="px-2 py-1 rounded bg-emerald-600 text-white font-mono font-bold text-xs shadow-md">
            {{ t.compare.candPhoto }}
          </span>
          <span v-if="photoB" class="px-2 py-0.5 rounded bg-black/70 backdrop-blur-md text-[#E5E5E5] font-mono text-xs border border-[#333333]">
            {{ photoB.base_name }}
          </span>
          <span v-if="photoB" class="px-1.5 py-0.5 rounded bg-black/60 text-[10px] font-mono text-emerald-300 border border-emerald-500/30">
            {{ getBadgeLabel(photoB) }}
          </span>
        </div>

        <!-- 画布视口区域 -->
        <div
          class="w-full h-full flex items-center justify-center cursor-grab active:cursor-grabbing overflow-hidden"
          @wheel="(e) => handleWheel(e, 'B')"
          @mousedown="(e) => handleMouseDown(e, 'B')"
          @dblclick="toggle100Percent"
        >
          <div
            v-if="photoB"
            class="relative transition-transform duration-75 ease-linear"
            :style="{
              transform: `translate(${panB.x}px, ${panB.y}px) scale(${scaleB})`,
            }"
          >
            <img
              :src="store.getPreviewUrl(photoB)"
              alt=""
              class="max-w-[85vw] max-h-[80vh] object-contain select-none pointer-events-none drop-shadow-2xl"
              draggable="false"
            />
          </div>
          <div v-else class="text-xs text-[#555555] italic">{{ t.compare.noCand }}</div>
        </div>

        <!-- 淘汰黑遮罩 -->
        <div
          v-if="photoB && photoB.flag === -1"
          class="absolute inset-0 bg-black/65 z-10 flex items-center justify-center pointer-events-none"
        >
          <div class="p-4 bg-black/80 rounded-xl border border-rose-500/50 backdrop-blur-md flex items-center space-x-2 text-rose-400">
            <X class="w-8 h-8 stroke-[2.5]" />
            <span class="text-xs font-bold uppercase tracking-wider">{{ t.compare.rejectedBadge }}</span>
          </div>
        </div>

        <!-- 底部照片控制工具栏 -->
        <div
          v-if="photoB"
          class="absolute bottom-3 left-3 right-3 z-20 flex items-center justify-between px-3 py-2 rounded-lg bg-black/75 backdrop-blur-md border border-[#333333]"
        >
          <!-- 旗标 -->
          <div class="flex items-center space-x-1">
            <button
              @click="store.setFlagForPhoto(photoB, 1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2 py-0.5 rounded text-[11px] font-semibold transition-all cursor-pointer',
                photoB.flag === 1
                  ? 'bg-emerald-500 text-white shadow'
                  : 'bg-[#262626] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <Check class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.pick }}</span>
            </button>
            <button
              @click="store.setFlagForPhoto(photoB, -1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2 py-0.5 rounded text-[11px] font-semibold transition-all cursor-pointer',
                photoB.flag === -1
                  ? 'bg-rose-500 text-white shadow'
                  : 'bg-[#262626] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <X class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.reject }}</span>
            </button>
          </div>

          <!-- 星级评分 -->
          <div class="flex items-center space-x-0.5">
            <button
              v-for="star in 5"
              :key="star"
              @click="store.setRatingForPhoto(photoB, photoB.rating === star ? 0 : star)"
              class="p-0.5 hover:scale-120 transition-transform cursor-pointer"
            >
              <Star
                :class="[
                  'w-3.5 h-3.5',
                  photoB.rating >= star ? 'text-amber-400 fill-amber-400' : 'text-[#444444] hover:text-[#888888]',
                ]"
              />
            </button>
          </div>

          <!-- 快捷色标 -->
          <div class="flex items-center space-x-1.5">
            <button
              v-for="color in (['red', 'yellow', 'green', 'blue', 'purple'] as ColorLabel[])"
              :key="color"
              @click="store.setColorLabelForPhoto(photoB, color)"
              :class="[
                'w-3.5 h-3.5 rounded-full transition-transform cursor-pointer',
                photoB.color_label === color ? 'ring-2 ring-white scale-125' : 'opacity-70 hover:opacity-100',
                color === 'red' ? 'bg-[#EF4444]' : '',
                color === 'yellow' ? 'bg-[#F59E0B]' : '',
                color === 'green' ? 'bg-[#10B981]' : '',
                color === 'blue' ? 'bg-[#3B82F6]' : '',
                color === 'purple' ? 'bg-[#8B5CF6]' : '',
              ]"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Narrative Select 风格人眼特写检视浮动面板 -->
    <EyeInspectionPanel v-if="store.eyeAfEnabled && store.currentPhoto" />

    <!-- 底部选片提示微条 -->
    <div
      class="h-6 px-4 bg-[#141414] border-t border-[#222222] flex items-center justify-between text-[11px] text-[#777777] font-mono transition-colors"
    >
      <div class="flex items-center space-x-1.5">
        <Info class="w-3 h-3 text-sky-400" />
        <span>{{ t.compare.compareTip }}</span>
      </div>
      <div>
        <span>{{ t.compare.availablePhotos.replace('{0}', store.filteredGroups.length.toString()) }}</span>
      </div>
    </div>
  </div>
</template>
