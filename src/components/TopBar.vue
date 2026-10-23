<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import {
  FolderOpen,
  LayoutGrid,
  Maximize2,
  Columns,
  Zap,
  Download,
  Star,
  Check,
  X,
  Settings,
  Layers,
  ZoomIn,
} from "lucide-vue-next";
import { usePhotoStore } from "../stores/photoStore";
import { useI18n } from "../i18n";
import type { FlagStatus } from "../types/photo";

const store = usePhotoStore();
const { t } = useI18n();

const handleOpenFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择包含照片或相机 RAW 的文件夹",
    });
    if (selected && typeof selected === "string") {
      await store.loadFolder(selected);
    }
  } catch (err) {
    console.error("打开文件夹失败:", err);
  }
};

const toggleFlagFilter = (flag: FlagStatus) => {
  const idx = store.filter.flags.indexOf(flag);
  if (idx >= 0) {
    store.filter.flags.splice(idx, 1);
  } else {
    store.filter.flags.push(flag);
  }
};
</script>

<template>
  <header
    class="h-10 px-3 flex items-center justify-between border-b border-[#26262C] bg-[#16161A]/95 backdrop-blur-md text-xs select-none z-30 transition-colors"
  >
    <!-- 左区: 文件夹入口与路径 -->
    <div class="flex items-center space-x-2 min-w-0 max-w-[34%]">
      <button
        @click="handleOpenFolder"
        class="flex items-center space-x-1.5 px-2.5 py-1 rounded-lg bg-[#222228] hover:bg-[#2C2C34] text-[#F4F4F5] transition-all font-medium border border-[#2E2E38] hover:border-sky-500/40 text-[11px] shadow-sm active:scale-95 cursor-pointer flex-shrink-0"
        :title="t.topbar.openFolder"
      >
        <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
        <span>{{ t.topbar.openFolder }}</span>
      </button>

      <div
        class="truncate px-2.5 py-0.5 rounded-md bg-[#1B1B20] border border-[#26262E] text-[#9E9EA8] font-mono text-[11px]"
        :title="store.currentFolderPath"
      >
        {{ store.currentFolderPath || t.topbar.noFolder }}
      </div>

      <span
        v-if="store.isScanning"
        class="inline-flex items-center space-x-1 text-[10px] text-amber-300 font-medium bg-amber-500/10 border border-amber-500/25 px-2 py-0.5 rounded-full animate-pulse flex-shrink-0"
      >
        <span>⚡</span>
        <span>{{ t.topbar.scanning }}</span>
      </span>
    </div>

    <!-- 中区: 视图模式切换 (Loupe [E] / Grid [G] / Compare [C] / Group Folded) -->
    <div class="flex items-center bg-[#121214] p-0.5 rounded-lg border border-[#26262C] shadow-inner">
      <button
        @click="store.viewMode = 'loupe'"
        :class="[
          'flex items-center space-x-1 px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer',
          store.viewMode === 'loupe'
            ? 'bg-[#24242B] text-white shadow-sm font-semibold'
            : 'text-[#8E8E98] hover:text-[#E4E4E8] hover:bg-white/5',
        ]"
        :title="t.topbar.loupe"
      >
        <Maximize2 class="w-3 h-3" />
        <span>{{ t.topbar.loupe }}</span>
      </button>
      <button
        @click="store.viewMode = 'grid'"
        :class="[
          'flex items-center space-x-1 px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer',
          store.viewMode === 'grid'
            ? 'bg-[#24242B] text-white shadow-sm font-semibold'
            : 'text-[#8E8E98] hover:text-[#E4E4E8] hover:bg-white/5',
        ]"
        :title="t.topbar.grid"
      >
        <LayoutGrid class="w-3 h-3" />
        <span>{{ t.topbar.grid }}</span>
      </button>
      <button
        @click="store.viewMode = 'compare'"
        :class="[
          'flex items-center space-x-1 px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer',
          store.viewMode === 'compare'
            ? 'bg-[#24242B] text-white shadow-sm font-semibold'
            : 'text-[#8E8E98] hover:text-[#E4E4E8] hover:bg-white/5',
        ]"
        :title="t.topbar.compare"
      >
        <Columns class="w-3 h-3" />
        <span>{{ t.topbar.compare }}</span>
      </button>

      <div class="h-3 w-[1px] bg-[#2A2A30] mx-0.5"></div>

      <!-- 分组折叠选片视图切换 -->
      <button
        @click="store.toggleGroupFoldedMode()"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer',
          store.isGroupFoldedMode
            ? 'bg-purple-600/30 text-purple-300 border border-purple-500/40 shadow-sm'
            : 'text-[#8E8E98] hover:text-[#E4E4E8] hover:bg-white/5',
        ]"
        :title="store.isGroupFoldedMode ? '已开启分组折叠 (同组仅显示最佳帧，点击平铺全部)' : '开启分组折叠选片 (同主体/连拍叠放)'"
      >
        <Layers class="w-3 h-3" />
        <span>分组折叠</span>
      </button>
    </div>

    <!-- 右区: 过滤胶囊、自动推进与快捷导出入口 -->
    <div class="flex items-center space-x-2">
      <!-- 自动推进 Auto-Advance 开关 -->
      <button
        @click="store.autoAdvance = !store.autoAdvance"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded-lg text-[11px] font-medium border transition-all cursor-pointer',
          store.autoAdvance
            ? 'bg-emerald-500/15 border-emerald-500/35 text-emerald-300 shadow-sm'
            : 'bg-[#18181C] border-[#26262C] text-[#8E8E98] hover:text-[#E4E4E8] hover:bg-[#202026]',
        ]"
        :title="t.topbar.autoAdvanceTip"
      >
        <Zap class="w-3 h-3" />
        <span>{{ t.topbar.autoAdvance }}</span>
      </button>

      <div class="h-4 w-[1px] bg-[#26262C]"></div>

      <!-- 星级过滤 (>= ★) -->
      <div class="flex items-center space-x-0.5 bg-[#121214] px-1 py-0.5 rounded-lg border border-[#26262C]">
        <button
          v-for="star in 5"
          :key="star"
          @click="store.filter.min_rating = store.filter.min_rating === star ? 0 : star"
          :class="[
            'p-0.5 rounded transition-colors cursor-pointer',
            store.filter.min_rating >= star ? 'text-amber-400' : 'text-[#4D4D4D] hover:text-[#8C8C8C]',
          ]"
          :title="t.topbar.filterMinRating.replace('{0}', star.toString())"
        >
          <Star class="w-3 h-3 fill-current" />
        </button>
      </div>

      <!-- 旗标快捷过滤 -->
      <div class="flex items-center space-x-0.5 bg-[#121214] p-0.5 rounded-lg border border-[#26262C]">
        <button
          @click="toggleFlagFilter(1)"
          :class="[
            'p-1 rounded text-[10px] transition-all cursor-pointer',
            store.filter.flags.includes(1)
              ? 'bg-emerald-500/25 text-emerald-300 font-bold shadow-sm'
              : 'text-[#8E8E98] hover:text-[#E4E4E8]',
          ]"
          :title="t.topbar.pickFilter"
        >
          <Check class="w-3 h-3" />
        </button>
        <button
          @click="toggleFlagFilter(-1)"
          :class="[
            'p-1 rounded text-[10px] transition-all cursor-pointer',
            store.filter.flags.includes(-1)
              ? 'bg-rose-500/25 text-rose-300 font-bold shadow-sm'
              : 'text-[#8E8E98] hover:text-[#E4E4E8]',
          ]"
          :title="t.topbar.rejectFilter"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- 仅 RAW+JPG 联动照片切换 -->
      <button
        @click="store.filter.raw_plus_jpg_only = !store.filter.raw_plus_jpg_only"
        :class="[
          'px-2 py-0.5 rounded-md text-[10px] font-mono font-bold border transition-all cursor-pointer',
          store.filter.raw_plus_jpg_only
            ? 'bg-blue-500/20 border-blue-500/40 text-blue-400 shadow-sm'
            : 'border-[#26262C] text-[#8E8E98] hover:text-[#E4E4E8]',
        ]"
        :title="t.topbar.rawPlusJpg"
      >
        RAW+JPG
      </button>

      <button
        v-if="store.currentPhoto?.std_image && store.currentPhoto?.raw_image"
        @click="store.setRawPreviewMode(!store.rawPreviewMode)"
        :class="[
          'px-2 py-0.5 rounded-md text-[10px] font-mono font-bold border transition-all cursor-pointer',
          store.rawPreviewMode
            ? 'bg-amber-500/20 border-amber-500/40 text-amber-300 shadow-sm'
            : 'border-[#26262C] text-[#8E8E98] hover:text-[#E4E4E8]',
        ]"
        :title="t.topbar.rawPreview"
        :aria-pressed="store.rawPreviewMode"
      >
        {{ store.rawPreviewMode ? 'RAW' : 'JPG' }}
      </button>

      <div class="h-4 w-[1px] bg-[#26262C]"></div>

      <!-- 快捷导出面板触发器 (Professional Action Button) -->
      <button
        @click="store.exportDrawerOpen = !store.exportDrawerOpen"
        class="flex items-center space-x-1.5 px-3 py-1 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-medium text-xs shadow-sm border border-sky-400/30 active:scale-95 transition-all cursor-pointer"
        :title="t.topbar.export"
      >
        <Download class="w-3.5 h-3.5" />
        <span>{{ t.topbar.export }}</span>
        <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-black/25">
          {{ store.exportCounts.picked || store.filteredGroups.length }}
        </span>
      </button>

      <div class="h-4 w-[1px] bg-[#26262C]"></div>

      <!-- 页面与字体缩放：紧凑级循环切换 (100% -> 125% -> 150% -> 200%) -->
      <button
        @click="store.zoomInUiScale()"
        @contextmenu.prevent="store.resetUiScale()"
        class="flex items-center space-x-1 px-2 py-1 rounded-lg bg-[#18181C] hover:bg-[#222228] border border-[#26262C] text-[11px] font-mono text-zinc-300 hover:text-white transition-all cursor-pointer shadow-sm flex-shrink-0"
        :title="`${t.settings.uiScale}: ${store.uiScale}% (点击步进放大 / 右键复位 100% / 快捷键 Ctrl + = / -)`"
      >
        <ZoomIn class="w-3 h-3 text-zinc-400" />
        <span>{{ store.uiScale }}%</span>
      </button>

      <!-- 系统设置对话框触发器 -->
      <button
        @click="store.settingsModalOpen = true"
        class="p-1.5 rounded-lg bg-[#202026] hover:bg-[#2A2A32] text-[#8E8E98] hover:text-white transition-all border border-[#2E2E38] hover:border-[#3E3E48] cursor-pointer hover:rotate-45 duration-200"
        :title="t.topbar.settings"
      >
        <Settings class="w-3.5 h-3.5" />
      </button>
    </div>
  </header>
</template>
