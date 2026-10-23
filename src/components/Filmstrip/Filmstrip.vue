<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { Check, X, Star } from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import type { PhotoGroup } from "../../types/photo";

const store = usePhotoStore();
const filmstripContainer = ref<HTMLElement | null>(null);

// 监听当前选中的照片索引，自动平移居中当前胶片
watch(
  () => store.selectedIndex,
  async (newIdx) => {
    await nextTick();
    if (!filmstripContainer.value) return;
    const cardEl = filmstripContainer.value.children[newIdx] as HTMLElement;
    if (cardEl) {
      cardEl.scrollIntoView({
        behavior: "auto",
        inline: "center",
        block: "nearest",
      });
    }
  }
);

const getBadgeLabel = (photo: PhotoGroup): string => {
  if (photo.std_image && photo.raw_image) return "RAW+JPG";
  if (photo.raw_image) return "RAW";
  return "JPG";
};

const isRawPlusJpg = (photo: PhotoGroup): boolean => {
  return !!(photo.std_image && photo.raw_image);
};

const isRawOnly = (photo: PhotoGroup): boolean => {
  return !!(photo.raw_image && !photo.std_image);
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
  <div
    class="h-[132px] flex-shrink-0 bg-[#16161A] border-t border-[#26262C] flex flex-col select-none overflow-hidden transition-colors"
  >
    <!-- 胶片栏顶部微信息条 -->
    <div
      class="h-6 px-3.5 bg-[#131316] border-b border-[#222228] flex items-center justify-between text-[11px] text-[#8E8E98] font-mono transition-colors"
    >
      <div class="flex items-center space-x-2.5">
        <span class="text-[#71717A]">序列:</span>
        <span class="font-bold text-[#F4F4F5]">
          {{ store.filteredGroups.length > 0 ? store.selectedIndex + 1 : 0 }} / {{ store.filteredGroups.length }}
        </span>
        <span v-if="store.currentPhoto" class="text-white/30">|</span>
        <span v-if="store.currentPhoto" class="text-[#F4F4F5] font-semibold truncate max-w-[240px]">
          {{ store.currentPhoto.base_name }}
        </span>
      </div>
      <div class="flex items-center space-x-3">
        <span
          v-if="store.currentPhoto"
          class="px-1.5 py-0.2 rounded text-[10px] font-bold"
          :class="isRawPlusJpg(store.currentPhoto) ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30' : 'bg-white/5 text-[#A1A1AA]'"
        >
          {{ getBadgeLabel(store.currentPhoto) }}
        </span>
      </div>
    </div>

    <!-- 横向胶片卡片流 -->
    <div
      ref="filmstripContainer"
      class="flex-1 flex items-center overflow-x-auto px-2.5 py-1.5 space-x-2 scroll-smooth"
    >
      <div
        v-for="(photo, index) in store.filteredGroups"
        :key="photo.id"
        @click="(e) => store.selectIndex(index, e.ctrlKey || e.metaKey)"
        :class="[
          'relative h-[94px] w-[94px] flex-shrink-0 rounded-lg bg-[#1A1A20] border cursor-pointer overflow-hidden transition-all duration-150 ease-out flex flex-col justify-between shadow-sm',
          index === store.selectedIndex
            ? (store.viewMode === 'compare' || store.compareIndex !== null)
              ? 'ring-2 ring-blue-500 border-transparent scale-[1.03] shadow-[0_0_16px_rgba(59,130,246,0.35)] z-10'
              : 'ring-2 ring-sky-400 border-transparent scale-[1.03] shadow-[0_0_16px_rgba(56,189,248,0.35)] z-10'
            : index === store.compareIndex
            ? 'ring-2 ring-emerald-400 border-transparent scale-[1.03] shadow-[0_0_16px_rgba(52,211,153,0.35)] z-10'
            : 'border-[#2A2A34] hover:border-[#4A4A58] opacity-80 hover:opacity-100 hover:scale-[1.01]',
          colorClassMap[photo.color_label] || '',
        ]"
      >
        <!-- 淘汰状态下的全卡半透明黑遮罩 -->
        <div
          v-if="photo.flag === -1"
          class="absolute inset-0 bg-black/65 z-10 flex items-center justify-center pointer-events-none backdrop-blur-[0.5px]"
        >
          <X class="w-8 h-8 text-rose-500/90 stroke-[2.5]" />
        </div>

        <!-- 缩略图主体 (极速 20KB 缩略图流，避免 45MP 解压内存爆炸) -->
        <img
          :src="store.getThumbnailUrl(photo)"
          loading="lazy"
          decoding="async"
          alt=""
          class="absolute inset-0 w-full h-full object-cover select-none pointer-events-none"
        />

        <!-- 卡片左上角序号与对比角色徽章 -->
        <div class="relative z-20 p-1 flex items-start justify-between">
          <div class="flex items-center space-x-0.5">
            <span class="text-[9px] font-mono font-bold text-white/90 bg-black/70 px-1 py-0.2 rounded backdrop-blur-sm">
              {{ index + 1 }}
            </span>
            <span
              v-if="index === store.selectedIndex && (store.viewMode === 'compare' || store.compareIndex !== null)"
              class="text-[8px] font-mono font-bold px-1 rounded bg-blue-600 text-white shadow-sm"
            >
              A
            </span>
            <span
              v-else-if="index === store.compareIndex"
              class="text-[8px] font-mono font-bold px-1 rounded bg-emerald-600 text-white shadow-sm"
            >
              B
            </span>
            <span
              v-if="photo.burst_group_id || photo.subject_group_id"
              :class="[
                'text-[8px] font-mono font-bold px-1 rounded shadow-sm',
                (photo.is_burst_best || photo.is_subject_best) ? 'bg-amber-400 text-black font-extrabold' : 'bg-purple-600/90 text-white'
              ]"
              :title="(photo.is_burst_best || photo.is_subject_best) ? '推荐最佳代表帧' : `组 ${photo.burst_group_id || photo.subject_group_id}`"
            >
              {{ (photo.is_burst_best || photo.is_subject_best) ? '★' : '⧉' }}
            </span>
            <span
              v-if="photo.is_duplicate"
              class="text-[8px] font-mono font-bold px-0.5 rounded bg-rose-600/90 text-white shadow-sm"
              title="重复雷同帧"
            >
              DUP
            </span>
          </div>

          <!-- 右上角旗标徽章 -->
          <div v-if="photo.flag === 1" class="bg-emerald-500 text-white rounded p-0.5 shadow-md">
            <Check class="w-2.5 h-2.5 stroke-[3]" />
          </div>
        </div>

        <!-- 卡片底部格式徽章与星级点阵 -->
        <div class="relative z-20 p-1 flex items-end justify-between bg-gradient-to-t from-black/85 via-black/45 to-transparent">
          <span
            :class="[
              'text-[8px] font-mono font-bold px-1 rounded',
              isRawPlusJpg(photo)
                ? 'bg-blue-600/90 text-white'
                : isRawOnly(photo)
                ? 'bg-amber-600/90 text-white'
                : 'bg-zinc-700/90 text-zinc-200',
            ]"
          >
            {{ getBadgeLabel(photo) }}
          </span>

          <!-- 底部微星级 -->
          <div v-if="photo.rating > 0" class="flex items-center space-x-0.5">
            <Star
              v-for="s in photo.rating"
              :key="s"
              class="w-2 h-2 text-amber-400 fill-amber-400"
            />
          </div>
        </div>
      </div>

      <div
        v-if="store.filteredGroups.length === 0"
        class="w-full text-center text-xs text-[#55555C] italic py-8 font-medium"
      >
        暂无符合当前筛选条件的照片
      </div>
    </div>
  </div>
</template>
