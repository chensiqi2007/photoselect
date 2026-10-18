<script setup lang="ts">
import { onMounted } from "vue";
import {
  HardDrive,
  Clock,
  Images,
  BookmarkCheck,
  Star,
  Trash2,
  CircleHelp,
  RefreshCw,
  Folder,
} from "lucide-vue-next";
import { usePhotoStore } from "../stores/photoStore";
import { useI18n } from "../i18n";
import type { ColorLabel } from "../types/photo";

const store = usePhotoStore();
const { t } = useI18n();

onMounted(async () => {
  await store.loadMountPoints();
  await store.loadRecentWorkspaces();
});

const selectSmartCollection = (
  type: "all" | "picked" | "rejected" | "unflagged" | "five_star"
) => {
  store.filter.smart_collection = type;
};

const toggleColorLabel = (color: ColorLabel) => {
  const idx = store.filter.colors.indexOf(color);
  if (idx >= 0) {
    store.filter.colors.splice(idx, 1);
  } else {
    store.filter.colors.push(color);
  }
};
</script>

<template>
  <aside
    class="w-60 flex-shrink-0 bg-[#181818] border-r border-[#2A2A2A] flex flex-col justify-between text-xs select-none overflow-hidden transition-colors"
  >
    <!-- 上半区: 物理挂载磁盘与 SD 卡直达 -->
    <div class="flex-1 overflow-y-auto px-3 py-3 space-y-4">
      <div>
        <div class="flex items-center justify-between text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider mb-2">
          <span>{{ t.sidebar.storageDrives }}</span>
          <button
            @click="store.loadMountPoints"
            class="p-0.5 hover:text-[#E5E5E5] transition-colors cursor-pointer"
            title="刷新系统挂载设备"
          >
            <RefreshCw class="w-3 h-3" />
          </button>
        </div>

        <div class="space-y-1">
          <button
            v-for="mount in store.mountPoints"
            :key="mount"
            @click="store.loadFolder(mount)"
            :class="[
              'w-full flex items-center space-x-2 px-2 py-1.5 rounded transition-colors text-left font-mono cursor-pointer',
              store.currentFolderPath.startsWith(mount)
                ? 'bg-[#262626] text-[#E5E5E5] border border-[#333333]'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
            :title="`打开存储卷: ${mount}`"
          >
            <HardDrive class="w-3.5 h-3.5 text-blue-400 flex-shrink-0" />
            <span class="truncate font-medium">{{ mount }}</span>
          </button>
          <div v-if="store.mountPoints.length === 0" class="text-[11px] text-[#4D4D4D] italic px-2">
            未检测到可移动设备
          </div>
        </div>
      </div>

      <!-- 智能分类集合 (Smart Collections) -->
      <div>
        <div class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider mb-2">
          {{ t.sidebar.smartCollections }}
        </div>
        <div class="space-y-0.5">
          <!-- 全部照片 -->
          <button
            @click="selectSmartCollection('all')"
            :class="[
              'w-full flex items-center justify-between px-2.5 py-1.5 rounded transition-colors cursor-pointer',
              store.filter.smart_collection === 'all'
                ? 'bg-[#262626] text-[#E5E5E5] font-semibold'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
          >
            <div class="flex items-center space-x-2">
              <Images class="w-3.5 h-3.5 text-sky-400" />
              <span>{{ t.sidebar.allPhotos }}</span>
            </div>
            <span class="tabular-nums font-mono text-[10px] px-1.5 py-0.2 rounded bg-[#121212] text-[#8C8C8C]">
              {{ store.smartCounts.all }}
            </span>
          </button>

          <!-- 已留用 Picked -->
          <button
            @click="selectSmartCollection('picked')"
            :class="[
              'w-full flex items-center justify-between px-2.5 py-1.5 rounded transition-colors cursor-pointer',
              store.filter.smart_collection === 'picked'
                ? 'bg-[#262626] text-emerald-400 font-semibold'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
          >
            <div class="flex items-center space-x-2">
              <BookmarkCheck class="w-3.5 h-3.5 text-emerald-400" />
              <span>{{ t.sidebar.picked }}</span>
            </div>
            <span class="tabular-nums font-mono text-[10px] px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 font-bold">
              {{ store.smartCounts.picked }}
            </span>
          </button>

          <!-- 五星精选 5-Star -->
          <button
            @click="selectSmartCollection('five_star')"
            :class="[
              'w-full flex items-center justify-between px-2.5 py-1.5 rounded transition-colors cursor-pointer',
              store.filter.smart_collection === 'five_star'
                ? 'bg-[#262626] text-amber-400 font-semibold'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
          >
            <div class="flex items-center space-x-2">
              <Star class="w-3.5 h-3.5 text-amber-400 fill-amber-400" />
              <span>{{ t.sidebar.fiveStar }}</span>
            </div>
            <span class="tabular-nums font-mono text-[10px] px-1.5 py-0.2 rounded bg-amber-400/10 text-amber-400 font-bold">
              {{ store.smartCounts.five_star }}
            </span>
          </button>

          <!-- 待淘汰 Rejected -->
          <button
            @click="selectSmartCollection('rejected')"
            :class="[
              'w-full flex items-center justify-between px-2.5 py-1.5 rounded transition-colors cursor-pointer',
              store.filter.smart_collection === 'rejected'
                ? 'bg-[#262626] text-rose-400 font-semibold'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
          >
            <div class="flex items-center space-x-2">
              <Trash2 class="w-3.5 h-3.5 text-rose-400" />
              <span>{{ t.sidebar.rejected }}</span>
            </div>
            <span class="tabular-nums font-mono text-[10px] px-1.5 py-0.2 rounded bg-rose-500/10 text-rose-400 font-bold">
              {{ store.smartCounts.rejected }}
            </span>
          </button>

          <!-- 未标记 Unflagged -->
          <button
            @click="selectSmartCollection('unflagged')"
            :class="[
              'w-full flex items-center justify-between px-2.5 py-1.5 rounded transition-colors cursor-pointer',
              store.filter.smart_collection === 'unflagged'
                ? 'bg-[#262626] text-[#E5E5E5] font-semibold'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
          >
            <div class="flex items-center space-x-2">
              <CircleHelp class="w-3.5 h-3.5 text-[#8C8C8C]" />
              <span>{{ t.sidebar.unflagged }}</span>
            </div>
            <span class="tabular-nums font-mono text-[10px] px-1.5 py-0.2 rounded bg-[#121212] text-[#8C8C8C]">
              {{ store.smartCounts.unflagged }}
            </span>
          </button>
        </div>
      </div>

      <!-- 色彩标签选择器 (Color Labels) -->
      <div>
        <div class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider mb-2">
          {{ t.sidebar.colorLabels }}
        </div>
        <div class="flex items-center justify-between px-1 py-1 bg-[#121212] rounded border border-[#2A2A2A]">
          <button
            v-for="color in (['red', 'yellow', 'green', 'blue', 'purple'] as ColorLabel[])"
            :key="color"
            @click="toggleColorLabel(color)"
            :class="[
              'w-5 h-5 rounded-full flex items-center justify-center transition-transform cursor-pointer',
              store.filter.colors.includes(color) ? 'scale-110 ring-2 ring-white/50' : 'opacity-80 hover:opacity-100',
              color === 'red' ? 'bg-[#EF4444]' : '',
              color === 'yellow' ? 'bg-[#F59E0B]' : '',
              color === 'green' ? 'bg-[#10B981]' : '',
              color === 'blue' ? 'bg-[#3B82F6]' : '',
              color === 'purple' ? 'bg-[#8B5CF6]' : '',
            ]"
            :title="`${t.sidebar.colorLabels}: ${color}`"
          />
        </div>
      </div>

      <!-- 最近访问目录 -->
      <div v-if="store.recentWorkspaces.length > 0">
        <div class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider mb-2 flex items-center space-x-1">
          <Clock class="w-3 h-3 text-[#8C8C8C]" />
          <span>{{ t.sidebar.recentWorkspaces }}</span>
        </div>
        <div class="space-y-1">
          <button
            v-for="ws in store.recentWorkspaces.slice(0, 5)"
            :key="ws.id"
            @click="store.loadFolder(ws.folder_path)"
            :class="[
              'w-full flex items-center space-x-1.5 px-2 py-1 rounded text-left transition-colors truncate cursor-pointer',
              store.currentFolderPath === ws.folder_path
                ? 'bg-[#262626] text-[#E5E5E5]'
                : 'text-[#8C8C8C] hover:bg-[#202020] hover:text-[#E5E5E5]',
            ]"
            :title="ws.folder_path"
          >
            <Folder class="w-3 h-3 flex-shrink-0 text-amber-500/70" />
            <span class="truncate text-[11px] font-mono">{{ ws.folder_path }}</span>
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>
