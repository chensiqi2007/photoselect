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
    class="w-full h-full flex flex-col bg-[#0E0E11] select-none overflow-hidden transition-colors"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <!-- 顶部对比控制条 (40px) -->
    <div
      class="h-10 px-4 bg-[#141418]/95 backdrop-blur-md border-b border-white/10 flex items-center justify-between text-xs flex-shrink-0 z-20 transition-colors"
    >
      <!-- 左侧: 图 A 简标与互换 -->
      <div class="flex items-center space-x-2.5">
        <span class="px-2.5 py-1 rounded-md bg-blue-500/15 border border-blue-500/30 text-blue-400 font-mono text-[11px] font-semibold tracking-wide flex items-center gap-1.5 shadow-sm">
          <span class="w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse"></span>
          <span>[A] {{ t.compare.refPhoto }}: {{ photoA ? photoA.base_name : "--" }}</span>
        </span>

        <!-- 互换位置按钮 [S] -->
        <button
          @click="store.swapComparePhotos"
          class="flex items-center space-x-1 px-2.5 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-[#E5E5E5] border border-white/10 hover:border-white/20 transition-all duration-150 ease-out cursor-pointer font-medium text-[11px] active:scale-95 shadow-sm"
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
          class="flex items-center space-x-1.5 px-3 py-1 rounded-lg bg-emerald-500/15 hover:bg-emerald-500/25 text-emerald-400 border border-emerald-500/30 hover:border-emerald-500/50 transition-all duration-150 ease-out cursor-pointer font-medium text-[11px] shadow-sm hover:shadow-emerald-500/10 active:scale-95"
          :title="t.compare.pickARejectB"
        >
          <Check class="w-3.5 h-3.5 stroke-[2.5]" />
          <span>{{ t.compare.pickARejectB }}</span>
        </button>

        <!-- 留用 B · 淘汰 A -->
        <button
          @click="pickBAndRejectA"
          class="flex items-center space-x-1.5 px-3 py-1 rounded-lg bg-teal-500/15 hover:bg-teal-500/25 text-teal-300 border border-teal-500/30 hover:border-teal-500/50 transition-all duration-150 ease-out cursor-pointer font-medium text-[11px] shadow-sm hover:shadow-teal-500/10 active:scale-95"
          :title="t.compare.pickBRejectA"
        >
          <Check class="w-3.5 h-3.5 stroke-[2.5]" />
          <span>{{ t.compare.pickBRejectA }}</span>
        </button>

        <div class="h-4 w-[1px] bg-white/10 mx-1"></div>

        <!-- 双图同步缩放联动切换 -->
        <button
          @click="store.syncCompareZoom = !store.syncCompareZoom"
          :class="[
            'flex items-center space-x-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium border transition-all cursor-pointer shadow-sm active:scale-95',
            store.syncCompareZoom
              ? 'bg-amber-500/15 border-amber-500/35 text-amber-300 shadow-amber-500/10'
              : 'bg-white/5 border-white/10 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10',
          ]"
          :title="store.syncCompareZoom ? t.compare.syncZoom : t.compare.indepZoom"
        >
          <component :is="store.syncCompareZoom ? Lock : Unlock" class="w-3 h-3" />
          <span>{{ store.syncCompareZoom ? t.compare.syncZoom : t.compare.indepZoom }}</span>
        </button>

        <!-- 100% 锐度检查切换 -->
        <button
          @click="toggle100Percent"
          class="flex items-center space-x-1.5 px-2.5 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-[#CCCCCC] hover:text-white border border-white/10 hover:border-white/20 text-[11px] transition-all cursor-pointer shadow-sm active:scale-95"
          :title="is100Percent ? t.compare.fitScreen : t.compare.sharpness100"
        >
          <component :is="is100Percent ? Minimize2 : Maximize2" class="w-3 h-3 text-sky-400" />
          <span>{{ is100Percent ? t.compare.fitScreen : t.compare.sharpness100 }}</span>
        </button>
      </div>

      <!-- 右侧: 图 B 简标与操作提示 -->
      <div class="flex items-center space-x-2.5">
        <span class="px-2.5 py-1 rounded-md bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 font-mono text-[11px] font-semibold tracking-wide flex items-center gap-1.5 shadow-sm">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>[B] {{ t.compare.candPhoto }}: {{ photoB ? photoB.base_name : "--" }}</span>
        </span>
      </div>
    </div>

    <!-- 中间: 双图左右并排视口 -->
    <div class="flex-1 flex overflow-hidden p-2.5 gap-2.5">
      <!-- 1. 左侧面板: 图 A (参考基准) -->
      <div
        class="flex-1 relative rounded-2xl bg-[#111114] border border-white/10 shadow-2xl overflow-hidden flex flex-col justify-between"
      >
        <!-- 顶部身份浮标 -->
        <div class="absolute top-3 left-3 z-20 flex items-center space-x-2">
          <span class="px-2.5 py-1 rounded-lg bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-mono font-bold text-xs shadow-lg shadow-blue-600/30 tracking-wide">
            {{ t.compare.refPhoto }}
          </span>
          <span v-if="photoA" class="px-2.5 py-1 rounded-lg bg-black/75 backdrop-blur-xl text-[#E5E5E5] font-mono text-xs border border-white/10 shadow-md">
            {{ photoA.base_name }}
          </span>
          <span v-if="photoA" class="px-2 py-0.5 rounded-md bg-blue-500/15 text-[10px] font-mono font-semibold text-blue-300 border border-blue-500/30">
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
              class="max-w-[85vw] max-h-[80vh] object-contain select-none pointer-events-none drop-shadow-[0_20px_50px_rgba(0,0,0,0.8)]"
              draggable="false"
            />
          </div>
          <div v-else class="text-xs text-[#555555] italic">{{ t.compare.noRef }}</div>
        </div>

        <!-- 淘汰黑遮罩 -->
        <div
          v-if="photoA && photoA.flag === -1"
          class="absolute inset-0 bg-black/70 backdrop-blur-xs z-10 flex items-center justify-center pointer-events-none transition-opacity"
        >
          <div class="px-5 py-3.5 bg-black/85 rounded-2xl border border-rose-500/50 backdrop-blur-xl flex items-center space-x-2.5 text-rose-400 shadow-2xl shadow-rose-500/20">
            <X class="w-7 h-7 stroke-[2.5]" />
            <span class="text-xs font-bold uppercase tracking-wider">{{ t.compare.rejectedBadge }}</span>
          </div>
        </div>

        <!-- 底部照片控制工具栏 -->
        <div
          v-if="photoA"
          class="absolute bottom-3 left-3 right-3 z-20 flex items-center justify-between px-3.5 py-2 rounded-xl bg-black/80 backdrop-blur-xl border border-white/10 shadow-2xl"
        >
          <!-- 旗标 -->
          <div class="flex items-center space-x-1.5">
            <button
              @click="store.setFlagForPhoto(photoA, 1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2.5 py-1 rounded-lg text-[11px] font-semibold transition-all cursor-pointer shadow-sm active:scale-95',
                photoA.flag === 1
                  ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-emerald-600/30'
                  : 'bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10 border border-white/5',
              ]"
            >
              <Check class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.pick }}</span>
            </button>
            <button
              @click="store.setFlagForPhoto(photoA, -1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2.5 py-1 rounded-lg text-[11px] font-semibold transition-all cursor-pointer shadow-sm active:scale-95',
                photoA.flag === -1
                  ? 'bg-gradient-to-r from-rose-600 to-red-600 text-white shadow-rose-600/30'
                  : 'bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10 border border-white/5',
              ]"
            >
              <X class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.reject }}</span>
            </button>
          </div>

          <!-- 星级评分 -->
          <div class="flex items-center space-x-1">
            <button
              v-for="star in 5"
              :key="star"
              @click="store.setRatingForPhoto(photoA, photoA.rating === star ? 0 : star)"
              class="p-0.5 hover:scale-125 transition-transform cursor-pointer"
            >
              <Star
                :class="[
                  'w-4 h-4 transition-colors',
                  photoA.rating >= star ? 'text-amber-400 fill-amber-400 filter drop-shadow-[0_0_6px_rgba(251,191,36,0.5)]' : 'text-[#444444] hover:text-[#888888]',
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
                'w-3.5 h-3.5 rounded-full transition-transform cursor-pointer shadow-sm',
                photoA.color_label === color ? 'ring-2 ring-white scale-125 shadow-lg' : 'opacity-70 hover:opacity-100 hover:scale-110',
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
        class="flex-1 relative rounded-2xl bg-[#111114] border border-white/10 shadow-2xl overflow-hidden flex flex-col justify-between"
      >
        <!-- 顶部身份浮标 -->
        <div class="absolute top-3 left-3 z-20 flex items-center space-x-2">
          <span class="px-2.5 py-1 rounded-lg bg-gradient-to-r from-emerald-600 to-teal-600 text-white font-mono font-bold text-xs shadow-lg shadow-emerald-600/30 tracking-wide">
            {{ t.compare.candPhoto }}
          </span>
          <span v-if="photoB" class="px-2.5 py-1 rounded-lg bg-black/75 backdrop-blur-xl text-[#E5E5E5] font-mono text-xs border border-white/10 shadow-md">
            {{ photoB.base_name }}
          </span>
          <span v-if="photoB" class="px-2 py-0.5 rounded-md bg-emerald-500/15 text-[10px] font-mono font-semibold text-emerald-300 border border-emerald-500/30">
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
              class="max-w-[85vw] max-h-[80vh] object-contain select-none pointer-events-none drop-shadow-[0_20px_50px_rgba(0,0,0,0.8)]"
              draggable="false"
            />
          </div>
          <div v-else class="text-xs text-[#555555] italic">{{ t.compare.noCand }}</div>
        </div>

        <!-- 淘汰黑遮罩 -->
        <div
          v-if="photoB && photoB.flag === -1"
          class="absolute inset-0 bg-black/70 backdrop-blur-xs z-10 flex items-center justify-center pointer-events-none transition-opacity"
        >
          <div class="px-5 py-3.5 bg-black/85 rounded-2xl border border-rose-500/50 backdrop-blur-xl flex items-center space-x-2.5 text-rose-400 shadow-2xl shadow-rose-500/20">
            <X class="w-7 h-7 stroke-[2.5]" />
            <span class="text-xs font-bold uppercase tracking-wider">{{ t.compare.rejectedBadge }}</span>
          </div>
        </div>

        <!-- 底部照片控制工具栏 -->
        <div
          v-if="photoB"
          class="absolute bottom-3 left-3 right-3 z-20 flex items-center justify-between px-3.5 py-2 rounded-xl bg-black/80 backdrop-blur-xl border border-white/10 shadow-2xl"
        >
          <!-- 旗标 -->
          <div class="flex items-center space-x-1.5">
            <button
              @click="store.setFlagForPhoto(photoB, 1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2.5 py-1 rounded-lg text-[11px] font-semibold transition-all cursor-pointer shadow-sm active:scale-95',
                photoB.flag === 1
                  ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-emerald-600/30'
                  : 'bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10 border border-white/5',
              ]"
            >
              <Check class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.pick }}</span>
            </button>
            <button
              @click="store.setFlagForPhoto(photoB, -1 as FlagStatus)"
              :class="[
                'flex items-center space-x-1 px-2.5 py-1 rounded-lg text-[11px] font-semibold transition-all cursor-pointer shadow-sm active:scale-95',
                photoB.flag === -1
                  ? 'bg-gradient-to-r from-rose-600 to-red-600 text-white shadow-rose-600/30'
                  : 'bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10 border border-white/5',
              ]"
            >
              <X class="w-3 h-3 stroke-[3]" />
              <span>{{ t.compare.reject }}</span>
            </button>
          </div>

          <!-- 星级评分 -->
          <div class="flex items-center space-x-1">
            <button
              v-for="star in 5"
              :key="star"
              @click="store.setRatingForPhoto(photoB, photoB.rating === star ? 0 : star)"
              class="p-0.5 hover:scale-125 transition-transform cursor-pointer"
            >
              <Star
                :class="[
                  'w-4 h-4 transition-colors',
                  photoB.rating >= star ? 'text-amber-400 fill-amber-400 filter drop-shadow-[0_0_6px_rgba(251,191,36,0.5)]' : 'text-[#444444] hover:text-[#888888]',
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
                'w-3.5 h-3.5 rounded-full transition-transform cursor-pointer shadow-sm',
                photoB.color_label === color ? 'ring-2 ring-white scale-125 shadow-lg' : 'opacity-70 hover:opacity-100 hover:scale-110',
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
      class="h-6 px-4 bg-[#121215] border-t border-white/5 flex items-center justify-between text-[11px] text-[#71717A] font-mono transition-colors"
    >
      <div class="flex items-center space-x-1.5">
        <Info class="w-3.5 h-3.5 text-sky-400" />
        <span>{{ t.compare.compareTip }}</span>
      </div>
      <div>
        <span>{{ t.compare.availablePhotos.replace('{0}', store.filteredGroups.length.toString()) }}</span>
      </div>
    </div>
  </div>
</template>
