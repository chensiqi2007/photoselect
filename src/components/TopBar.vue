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
    class="h-9 px-3 flex items-center justify-between border-b border-[#2A2A2A] bg-[#181818] text-xs select-none z-30 transition-colors"
  >
    <!-- 左区: 文件夹入口与路径 -->
    <div class="flex items-center space-x-2 min-w-0 max-w-[32%]">
      <button
        @click="handleOpenFolder"
        class="flex items-center space-x-1.5 px-2 py-1 rounded bg-[#262626] hover:bg-[#333333] text-[#E5E5E5] transition-colors font-medium border border-[#333333] cursor-pointer"
        :title="t.topbar.openFolder"
      >
        <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
        <span>{{ t.topbar.openFolder }}</span>
      </button>

      <div class="truncate text-[#8C8C8C] font-mono text-[11px]" :title="store.currentFolderPath">
        {{ store.currentFolderPath || t.topbar.noFolder }}
      </div>

      <span
        v-if="store.isScanning"
        class="inline-flex items-center text-[10px] text-amber-400 animate-pulse bg-amber-400/10 px-1.5 py-0.5 rounded border border-amber-400/20"
      >
        {{ t.topbar.scanning }}
      </span>
    </div>

    <!-- 中区: 视图模式切换 (Loupe [E] / Grid [G] / Compare [C]) -->
    <div class="flex items-center bg-[#121212] p-0.5 rounded border border-[#2A2A2A]">
      <button
        @click="store.viewMode = 'loupe'"
        :class="[
          'flex items-center space-x-1 px-2.5 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          store.viewMode === 'loupe'
            ? 'bg-[#262626] text-[#E5E5E5] shadow'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.topbar.loupe"
      >
        <Maximize2 class="w-3 h-3" />
        <span>{{ t.topbar.loupe }}</span>
      </button>
      <button
        @click="store.viewMode = 'grid'"
        :class="[
          'flex items-center space-x-1 px-2.5 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          store.viewMode === 'grid'
            ? 'bg-[#262626] text-[#E5E5E5] shadow'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.topbar.grid"
      >
        <LayoutGrid class="w-3 h-3" />
        <span>{{ t.topbar.grid }}</span>
      </button>
      <button
        @click="store.viewMode = 'compare'"
        :class="[
          'flex items-center space-x-1 px-2.5 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          store.viewMode === 'compare'
            ? 'bg-[#262626] text-[#E5E5E5] shadow'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.topbar.compare"
      >
        <Columns class="w-3 h-3" />
        <span>{{ t.topbar.compare }}</span>
      </button>
    </div>

    <!-- 右区: 过滤胶囊、自动推进与快捷导出入口 -->
    <div class="flex items-center space-x-2">
      <!-- 自动推进 Auto-Advance 开关 -->
      <button
        @click="store.autoAdvance = !store.autoAdvance"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded text-[11px] font-medium border transition-colors cursor-pointer',
          store.autoAdvance
            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
            : 'bg-[#181818] border-[#2A2A2A] text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.topbar.autoAdvanceTip"
      >
        <Zap class="w-3 h-3" />
        <span>{{ t.topbar.autoAdvance }}</span>
      </button>

      <div class="h-4 w-[1px] bg-[#2A2A2A]"></div>

      <!-- 星级过滤 (>= ★) -->
      <div class="flex items-center space-x-0.5 bg-[#121212] px-1 py-0.5 rounded border border-[#2A2A2A]">
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
      <div class="flex items-center space-x-0.5 bg-[#121212] p-0.5 rounded border border-[#2A2A2A]">
        <button
          @click="toggleFlagFilter(1)"
          :class="[
            'p-1 rounded text-[10px] transition-colors cursor-pointer',
            store.filter.flags.includes(1)
              ? 'bg-emerald-500/20 text-emerald-400'
              : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
          ]"
          :title="t.topbar.pickFilter"
        >
          <Check class="w-3 h-3" />
        </button>
        <button
          @click="toggleFlagFilter(-1)"
          :class="[
            'p-1 rounded text-[10px] transition-colors cursor-pointer',
            store.filter.flags.includes(-1)
              ? 'bg-rose-500/20 text-rose-400'
              : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
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
          'px-1.5 py-1 rounded text-[10px] font-mono font-bold border transition-colors cursor-pointer',
          store.filter.raw_plus_jpg_only
            ? 'bg-blue-500/20 border-blue-500/40 text-blue-400'
            : 'border-[#2A2A2A] text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.topbar.rawPlusJpg"
      >
        RAW+JPG
      </button>

      <button
        v-if="store.currentPhoto?.std_image && store.currentPhoto?.raw_image"
        @click="store.setRawPreviewMode(!store.rawPreviewMode)"
        :class="[
          'px-1.5 py-1 rounded text-[10px] font-mono font-bold border transition-colors cursor-pointer',
          store.rawPreviewMode
            ? 'bg-amber-500/20 border-amber-500/40 text-amber-300'
            : 'border-[#2A2A2A] text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.topbar.rawPreview"
        :aria-pressed="store.rawPreviewMode"
      >
        {{ store.rawPreviewMode ? 'RAW' : 'JPG' }}
      </button>

      <div class="h-4 w-[1px] bg-[#2A2A2A]"></div>

      <!-- 快捷导出面板触发器 -->
      <button
        @click="store.exportDrawerOpen = !store.exportDrawerOpen"
        class="flex items-center space-x-1.5 px-2.5 py-1 rounded bg-[#E5E5E5] hover:bg-[#FFFFFF] text-[#121212] font-semibold transition-all shadow-md active:scale-95 cursor-pointer"
        :title="t.topbar.export"
      >
        <Download class="w-3.5 h-3.5" />
        <span>{{ t.topbar.export }}</span>
        <span class="text-[10px] opacity-75">({{ store.exportCounts.picked || store.filteredGroups.length }})</span>
      </button>

      <!-- 系统设置对话框触发器 -->
      <button
        @click="store.settingsModalOpen = true"
        class="p-1.5 rounded bg-[#262626] hover:bg-[#333333] text-[#8C8C8C] hover:text-[#E5E5E5] transition-colors border border-[#333333] cursor-pointer"
        :title="t.topbar.settings"
      >
        <Settings class="w-3.5 h-3.5" />
      </button>
    </div>
  </header>
</template>
