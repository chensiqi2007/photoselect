<script setup lang="ts">
import { computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Camera,
  Aperture,
  Timer,
  Zap,
  Layers,
  Keyboard,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type { ExifSummary } from "../../types/photo";

const store = usePhotoStore();
const { t } = useI18n();
const photo = computed(() => store.currentPhoto);

// 自动懒加载当前照片缺失的 EXIF 参数 (双重保险机制)
watch(
  () => store.currentPhoto,
  async (cur) => {
    if (cur && !cur.exif) {
      const targetPath = cur.std_image || cur.raw_image;
      if (targetPath) {
        try {
          const res = await invoke<ExifSummary | null>("get_file_exif", {
            path: targetPath,
          });
          if (res && store.currentPhoto?.id === cur.id) {
            cur.exif = res;
            if (res.capture_time && !cur.capture_time) {
              cur.capture_time = res.capture_time;
            }
          }
        } catch (err) {
          console.warn("EXIF 补充提取失败:", err);
        }
      }
    }
  },
  { immediate: true }
);

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return "--";
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
};

const getBadgeLabel = (photo: any): string => {
  if (!photo) return "";
  if (photo.std_image && photo.raw_image) return t.value.inspector.rawPlusJpgBinding;
  if (photo.raw_image) return t.value.inspector.rawOnly;
  return t.value.inspector.stdOnly;
};
</script>

<template>
  <aside
    class="w-70 flex-shrink-0 bg-[#181818] border-l border-[#2A2A2A] flex flex-col justify-between text-xs select-none overflow-hidden transition-colors"
  >
    <div class="flex-1 overflow-y-auto p-3 space-y-4">
      <!-- 模块 1: 相机核心曝光四要素矩阵 -->
      <div>
        <div class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider mb-2 flex items-center space-x-1">
          <Camera class="w-3.5 h-3.5 text-sky-400" />
          <span>{{ t.inspector.exifTitle }}</span>
        </div>

        <div class="grid grid-cols-2 gap-1.5 bg-[#121212] p-2 rounded border border-[#2A2A2A] font-mono">
          <!-- 光圈 -->
          <div class="flex items-center space-x-1.5">
            <Aperture class="w-3 h-3 text-[#8C8C8C]" />
            <span class="text-[#8C8C8C] text-[10px]">{{ t.inspector.aperture }}</span>
            <span class="text-[#E5E5E5] font-semibold tabular-nums">
              {{ photo?.exif?.aperture || "f/--" }}
            </span>
          </div>

          <!-- 快门 -->
          <div class="flex items-center space-x-1.5">
            <Timer class="w-3 h-3 text-[#8C8C8C]" />
            <span class="text-[#8C8C8C] text-[10px]">{{ t.inspector.shutter }}</span>
            <span class="text-[#E5E5E5] font-semibold tabular-nums">
              {{ photo?.exif?.shutter_speed || "1/--s" }}
            </span>
          </div>

          <!-- 感光度 ISO -->
          <div class="flex items-center space-x-1.5">
            <Zap class="w-3 h-3 text-[#8C8C8C]" />
            <span class="text-[#8C8C8C] text-[10px]">{{ t.inspector.iso }}</span>
            <span class="text-[#E5E5E5] font-semibold tabular-nums">
              {{ photo?.exif?.iso || "--" }}
            </span>
          </div>

          <!-- 焦距 -->
          <div class="flex items-center space-x-1.5">
            <Layers class="w-3 h-3 text-[#8C8C8C]" />
            <span class="text-[#8C8C8C] text-[10px]">{{ t.inspector.focal }}</span>
            <span class="text-[#E5E5E5] font-semibold tabular-nums">
              {{ photo?.exif?.focal_length || "-- mm" }}
            </span>
          </div>
        </div>
      </div>

      <!-- 模块 2: 相机与镜头型号 -->
      <div class="space-y-1.5 text-[11px] font-mono">
        <div class="flex justify-between py-1 border-b border-[#222222]">
          <span class="text-[#8C8C8C]">{{ t.inspector.cameraModel }}</span>
          <span class="text-[#E5E5E5] truncate max-w-[150px]" :title="photo?.exif?.camera_model || '未知机身'">
            {{ photo?.exif?.camera_model || photo?.exif?.camera_make || "--" }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222222]">
          <span class="text-[#8C8C8C]">{{ t.inspector.lensModel }}</span>
          <span class="text-[#E5E5E5] truncate max-w-[150px]" :title="photo?.exif?.lens_model || '标准镜头'">
            {{ photo?.exif?.lens_model || "--" }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222222]">
          <span class="text-[#8C8C8C]">{{ t.inspector.captureTime }}</span>
          <span class="text-[#E5E5E5] tabular-nums">
            {{ photo?.capture_time || "--" }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222222]">
          <span class="text-[#8C8C8C]">{{ t.inspector.fileSize }}</span>
          <span class="text-[#E5E5E5] tabular-nums font-semibold">
            {{ formatFileSize(photo?.file_size_bytes || 0) }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222222]">
          <span class="text-[#8C8C8C]">{{ t.inspector.bindingStatus }}</span>
          <span class="text-blue-400 font-semibold font-mono text-[10px]">
            {{ getBadgeLabel(photo) }}
          </span>
        </div>
      </div>

      <!-- 模块 3: 文件系统物理路径详情 -->
      <div v-if="photo" class="space-y-1 bg-[#121212] p-2 rounded border border-[#2A2A2A] text-[10px] font-mono break-all text-[#8C8C8C]">
        <div v-if="photo.raw_image" class="text-amber-400/90">
          <span class="font-bold text-white">[RAW]</span> {{ photo.raw_image }}
        </div>
        <div v-if="photo.std_image" class="text-sky-400/90">
          <span class="font-bold text-white">[JPG]</span> {{ photo.std_image }}
        </div>
        <div v-if="photo.xmp_sidecar" class="text-emerald-400/90">
          <span class="font-bold text-white">[XMP]</span> {{ photo.xmp_sidecar }}
        </div>
      </div>

      <!-- 模块 4: 单手暗房快捷键速查表 -->
      <div class="bg-[#141414] p-2.5 rounded border border-[#222222] space-y-2">
        <div class="flex items-center space-x-1 text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider">
          <Keyboard class="w-3.5 h-3.5 text-amber-400" />
          <span>{{ t.inspector.hotkeysTitle }}</span>
        </div>
        <div class="grid grid-cols-2 gap-y-1 text-[10px] font-mono text-[#8C8C8C]">
          <div><kbd class="px-1 rounded bg-[#222222] text-[#E5E5E5]">A / D</kbd> {{ t.inspector.keysPrevNext }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-[#E5E5E5]">0 - 5</kbd> {{ t.inspector.keysRate }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-emerald-400 font-bold">P</kbd> {{ t.inspector.keysPick }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-rose-400 font-bold">X</kbd> {{ t.inspector.keysReject }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-[#E5E5E5]">6 - 9</kbd> {{ t.inspector.keysColor }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-[#E5E5E5]">E/G/C</kbd> {{ t.inspector.keysMode }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-[#E5E5E5]">F</kbd> {{ t.inspector.keysPeaking }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-[#E5E5E5]">B</kbd> {{ t.inspector.keysBg }}</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-sky-400 font-bold">O</kbd> 人眼追踪</div>
          <div><kbd class="px-1 rounded bg-[#222222] text-amber-400 font-bold">Z</kbd> 吸附对焦</div>
        </div>
      </div>
    </div>
  </aside>
</template>
