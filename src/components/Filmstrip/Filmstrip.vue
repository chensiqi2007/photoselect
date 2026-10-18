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
    class="h-[130px] flex-shrink-0 bg-[#141414] border-t border-[#2A2A2A] flex flex-col select-none overflow-hidden transition-colors"
  >
    <!-- 胶片栏顶部微信息条 -->
    <div
      class="h-5 px-3 bg-[#181818] border-b border-[#222222] flex items-center justify-between text-[10px] text-[#8C8C8C] font-mono transition-colors"
    >
      <div class="flex items-center space-x-2">
        <span>当前序列: {{ store.filteredGroups.length > 0 ? store.selectedIndex + 1 : 0 }} / {{ store.filteredGroups.length }}</span>
        <span v-if="store.currentPhoto" class="text-[#E5E5E5] font-semibold">
          {{ store.currentPhoto.base_name }}
        </span>
      </div>
      <div class="flex items-center space-x-3">
        <span v-if="store.currentPhoto">
          {{ getBadgeLabel(store.currentPhoto) }}
        </span>
      </div>
    </div>

    <!-- 横向胶片卡片流 -->
    <div
      ref="filmstripContainer"
      class="flex-1 flex items-center overflow-x-auto px-2 py-1 space-x-2 scroll-smooth"
    >
      <div
        v-for="(photo, index) in store.filteredGroups"
        :key="photo.id"
        @click="(e) => store.selectIndex(index, e.ctrlKey || e.metaKey)"
        :class="[
          'relative h-[96px] w-[96px] flex-shrink-0 rounded bg-[#1C1C1C] cursor-pointer overflow-hidden transition-all duration-150 ease-linear flex flex-col justify-between',
          index === store.selectedIndex
            ? (store.viewMode === 'compare' || store.compareIndex !== null)
              ? 'ring-2 ring-blue-500 scale-[1.02] shadow-xl z-10'
              : 'ring-2 ring-[#E5E5E5] scale-[1.02] shadow-xl z-10'
            : index === store.compareIndex
            ? 'ring-2 ring-emerald-500 scale-[1.02] shadow-xl z-10'
            : 'hover:ring-1 hover:ring-[#444444] opacity-85 hover:opacity-100',
          colorClassMap[photo.color_label] || '',
        ]"
      >
        <!-- 淘汰状态下的全卡半透明黑遮罩 -->
        <div
          v-if="photo.flag === -1"
          class="absolute inset-0 bg-black/60 z-10 flex items-center justify-center pointer-events-none"
        >
          <X class="w-8 h-8 text-rose-500/80 stroke-[2.5]" />
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
          <div class="flex items-center space-x-1">
            <span class="text-[9px] font-mono font-bold text-white/90 bg-black/60 px-1 rounded backdrop-blur-xs">
              {{ index + 1 }}
            </span>
            <span
              v-if="index === store.selectedIndex && (store.viewMode === 'compare' || store.compareIndex !== null)"
              class="text-[8px] font-mono font-bold px-1 rounded bg-blue-600 text-white shadow"
            >
              A
            </span>
            <span
              v-else-if="index === store.compareIndex"
              class="text-[8px] font-mono font-bold px-1 rounded bg-emerald-600 text-white shadow"
            >
              B
            </span>
          </div>

          <!-- 右上角旗标徽章 -->
          <div v-if="photo.flag === 1" class="bg-emerald-500 text-white rounded p-0.5 shadow">
            <Check class="w-2.5 h-2.5 stroke-[3]" />
          </div>
        </div>

        <!-- 卡片底部格式徽章与星级点阵 -->
        <div class="relative z-20 p-1 flex items-end justify-between bg-gradient-to-t from-black/80 via-black/40 to-transparent">
          <span
            :class="[
              'text-[8px] font-mono font-bold px-1 rounded',
              isRawPlusJpg(photo)
                ? 'bg-blue-600/80 text-white'
                : isRawOnly(photo)
                ? 'bg-amber-600/80 text-white'
                : 'bg-zinc-700/80 text-zinc-200',
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
        class="w-full text-center text-xs text-[#4D4D4D] italic py-8"
      >
        暂无符合筛选条件的照片
      </div>
    </div>
  </div>
</template>
