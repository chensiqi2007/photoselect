<script setup lang="ts">
import { computed, watch, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Camera,
  Aperture,
  Timer,
  Zap,
  Layers,
  Keyboard,
  Tag,
  FileCheck,
  Award,
  Plus,
  X as CloseIcon,
  Flame,
  Sparkles,
  Copy,
  AlertTriangle,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type { ExifSummary } from "../../types/photo";

const store = usePhotoStore();
const { t } = useI18n();
const photo = computed(() => store.currentPhoto);

const newTagInput = ref("");
const isWritingXmp = ref(false);
const xmpNotice = ref("");

const handleAddTag = async () => {
  if (!newTagInput.value.trim() || !store.currentPhoto) return;
  await store.addPhotoTag(store.currentPhoto, newTagInput.value.trim());
  newTagInput.value = "";
};

const handleRemoveTag = async (tag: string) => {
  if (!store.currentPhoto) return;
  await store.removePhotoTag(store.currentPhoto, tag);
};

const handleWriteXmp = async () => {
  if (!store.currentPhoto || isWritingXmp.value) return;
  isWritingXmp.value = true;
  xmpNotice.value = "";
  try {
    await store.writePhotoXmp(store.currentPhoto);
    xmpNotice.value = "XMP 写入成功";
    setTimeout(() => {
      xmpNotice.value = "";
    }, 2500);
  } catch (err) {
    xmpNotice.value = `失败: ${err}`;
  } finally {
    isWritingXmp.value = false;
  }
};

const handleRunBurstDetection = async () => {
  try {
    const clusters = await store.clusterBursts({ threshold_ms: 500, min_burst_count: 2, auto_flag_best: true });
    xmpNotice.value = `连拍聚合: ${clusters?.length || 0} 组`;
    setTimeout(() => {
      xmpNotice.value = "";
    }, 2500);
  } catch (err) {
    xmpNotice.value = `聚合失败: ${err}`;
  }
};

const handleRunSimilarityDetection = async () => {
  try {
    const clusters = await store.detectSimilarityAndDuplicates({
      dedup_threshold: 3,
      subject_threshold: 12,
      time_window_secs: 60,
      min_group_count: 2,
      auto_reject_duplicates: false,
      auto_flag_best: true,
    });
    xmpNotice.value = `同主体聚类: ${clusters?.length || 0} 组`;
    setTimeout(() => {
      xmpNotice.value = "";
    }, 2500);
  } catch (err) {
    xmpNotice.value = `分析失败: ${err}`;
  }
};

const handlePickBestAndRejectRest = async (gid: string) => {
  try {
    await store.pickBestAndRejectRest(gid);
    xmpNotice.value = "已选优并淘汰其余";
    setTimeout(() => {
      xmpNotice.value = "";
    }, 2500);
  } catch (err) {
    xmpNotice.value = `操作失败: ${err}`;
  }
};

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
    class="w-72 flex-shrink-0 bg-[#16161A] border-l border-[#26262C] flex flex-col justify-between text-xs select-none overflow-hidden transition-colors"
  >
    <div class="flex-1 overflow-y-auto p-3.5 space-y-4">
      <!-- 模块 1: 相机核心曝光四要素矩阵 (Studio Exposure HUD) -->
      <div>
        <div class="text-[10px] font-bold text-[#71717A] uppercase tracking-wider mb-2 flex items-center space-x-1.5">
          <Camera class="w-3.5 h-3.5 text-sky-400" />
          <span>{{ t.inspector.exifTitle }}</span>
        </div>

        <div class="grid grid-cols-2 gap-2 bg-[#121214] p-2.5 rounded-xl border border-[#26262C]">
          <!-- 光圈 -->
          <div class="bg-[#18181C] p-2 rounded-lg border border-[#222228] flex flex-col justify-between">
            <div class="flex items-center space-x-1 text-[#71717A] text-[10px]">
              <Aperture class="w-3 h-3 text-sky-400" />
              <span>{{ t.inspector.aperture }}</span>
            </div>
            <span class="text-[#F4F4F5] font-bold text-sm font-mono mt-1 tabular-nums">
              {{ photo?.exif?.aperture || "f/--" }}
            </span>
          </div>

          <!-- 快门 -->
          <div class="bg-[#18181C] p-2 rounded-lg border border-[#222228] flex flex-col justify-between">
            <div class="flex items-center space-x-1 text-[#71717A] text-[10px]">
              <Timer class="w-3 h-3 text-amber-400" />
              <span>{{ t.inspector.shutter }}</span>
            </div>
            <span class="text-[#F4F4F5] font-bold text-sm font-mono mt-1 tabular-nums">
              {{ photo?.exif?.shutter_speed || "1/--s" }}
            </span>
          </div>

          <!-- 感光度 ISO -->
          <div class="bg-[#18181C] p-2 rounded-lg border border-[#222228] flex flex-col justify-between">
            <div class="flex items-center space-x-1 text-[#71717A] text-[10px]">
              <Zap class="w-3 h-3 text-emerald-400" />
              <span>{{ t.inspector.iso }}</span>
            </div>
            <span class="text-[#F4F4F5] font-bold text-sm font-mono mt-1 tabular-nums">
              {{ photo?.exif?.iso || "--" }}
            </span>
          </div>

          <!-- 焦距 -->
          <div class="bg-[#18181C] p-2 rounded-lg border border-[#222228] flex flex-col justify-between">
            <div class="flex items-center space-x-1 text-[#71717A] text-[10px]">
              <Layers class="w-3 h-3 text-purple-400" />
              <span>{{ t.inspector.focal }}</span>
            </div>
            <span class="text-[#F4F4F5] font-bold text-sm font-mono mt-1 tabular-nums">
              {{ photo?.exif?.focal_length || "-- mm" }}
            </span>
          </div>
        </div>
      </div>

      <!-- 模块 2: 相机与镜头型号 -->
      <div class="space-y-1.5 text-[11px] font-mono bg-[#18181C] p-3 rounded-xl border border-[#26262C]">
        <div class="flex justify-between py-1 border-b border-[#222228]">
          <span class="text-[#71717A]">{{ t.inspector.cameraModel }}</span>
          <span class="text-[#F4F4F5] font-medium truncate max-w-[155px]" :title="photo?.exif?.camera_model || '未知机身'">
            {{ photo?.exif?.camera_model || photo?.exif?.camera_make || "--" }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222228]">
          <span class="text-[#71717A]">{{ t.inspector.lensModel }}</span>
          <span class="text-[#F4F4F5] font-medium truncate max-w-[155px]" :title="photo?.exif?.lens_model || '标准镜头'">
            {{ photo?.exif?.lens_model || "--" }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222228]">
          <span class="text-[#71717A]">{{ t.inspector.captureTime }}</span>
          <span class="text-[#F4F4F5] tabular-nums">
            {{ photo?.capture_time || "--" }}
          </span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#222228]">
          <span class="text-[#71717A]">{{ t.inspector.fileSize }}</span>
          <span class="text-[#F4F4F5] tabular-nums font-semibold">
            {{ formatFileSize(photo?.file_size_bytes || 0) }}
          </span>
        </div>
        <div class="flex justify-between py-1">
          <span class="text-[#71717A]">{{ t.inspector.bindingStatus }}</span>
          <span class="text-blue-400 font-semibold font-mono text-[10px]">
            {{ getBadgeLabel(photo) }}
          </span>
        </div>
      </div>

      <!-- 模块 3: 文件系统物理路径详情 -->
      <div v-if="photo" class="space-y-1 bg-[#121214] p-2.5 rounded-xl border border-[#26262C] text-[10px] font-mono break-all text-[#8E8E98]">
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

      <!-- 模块 4: 高速连拍动作聚合分析 -->
      <div class="bg-[#18181C] p-3 rounded-xl border border-[#26262C] space-y-2.5 shadow-sm">
        <div class="flex items-center justify-between text-[10px] font-bold text-[#71717A] uppercase tracking-wider">
          <div class="flex items-center space-x-1">
            <Flame class="w-3.5 h-3.5 text-amber-400" />
            <span>高速连拍聚合</span>
          </div>
          <button
            @click="handleRunBurstDetection"
            :disabled="store.isBurstClustering"
            class="flex items-center space-x-1 px-2 py-0.5 rounded-md bg-[#24242C] hover:bg-[#2E2E38] text-[10px] text-amber-400 border border-[#2E2E38] cursor-pointer disabled:opacity-50 transition-all active:scale-95 shadow-sm"
            title="毫秒级时间戳与文件名序列混合判定连拍组并推荐最佳帧"
          >
            <Zap class="w-3 h-3" />
            <span>{{ store.isBurstClustering ? '正在聚合...' : '一键聚合' }}</span>
          </button>
        </div>

        <div v-if="photo?.burst_group_id" class="space-y-1.5">
          <div class="flex items-center justify-between text-[11px] font-mono">
            <span class="text-[#71717A]">连拍组 ID:</span>
            <span class="text-purple-300 font-bold bg-purple-500/15 px-1.5 py-0.2 rounded border border-purple-500/30">
              {{ photo.burst_group_id }}
            </span>
          </div>
          <div class="flex items-center justify-between text-[10px] font-mono">
            <span class="text-[#71717A]">微秒时间戳:</span>
            <span class="text-[#F4F4F5] tabular-nums">
              {{ photo.exif?.sub_sec_time ? `${photo.exif.sub_sec_time} ms` : '--' }}
            </span>
          </div>
          <div
            v-if="photo.is_burst_best"
            class="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg bg-amber-400/15 border border-amber-400/35 text-amber-300 font-semibold text-[11px]"
          >
            <Award class="w-3.5 h-3.5 text-amber-400 flex-shrink-0" />
            <span>连拍最佳推荐帧 (Best Shot)</span>
          </div>
          <div v-else class="text-[10px] text-[#55555C] italic">
            连拍连续动作候选帧
          </div>
        </div>
        <div v-else class="text-[10px] text-[#55555C] italic">
          {{ store.burstClusters.length > 0 ? '该照片为独立单张拍摄' : '点击上方一键聚合分析全组连拍' }}
        </div>
      </div>

      <!-- 模块 5: 同主体视觉聚类与严格去重 -->
      <div class="bg-[#18181C] p-3 rounded-xl border border-[#26262C] space-y-2.5 shadow-sm">
        <div class="flex items-center justify-between text-[10px] font-bold text-[#71717A] uppercase tracking-wider">
          <div class="flex items-center space-x-1">
            <Sparkles class="w-3.5 h-3.5 text-purple-400" />
            <span>同主体筛选 & 去重</span>
          </div>
          <button
            @click="handleRunSimilarityDetection"
            :disabled="store.isAnalyzingSimilarity"
            class="flex items-center space-x-1 px-2 py-0.5 rounded-md bg-[#24242C] hover:bg-[#2E2E38] text-[10px] text-purple-300 border border-[#2E2E38] cursor-pointer disabled:opacity-50 transition-all active:scale-95 shadow-sm"
            title="基于 64-bit dHash 感知哈希与时序滑动窗口聚类同主体并识别雷同帧"
          >
            <Copy class="w-3 h-3" />
            <span>{{ store.isAnalyzingSimilarity ? '正在分析...' : '一键聚类' }}</span>
          </button>
        </div>

        <div v-if="photo?.subject_group_id || photo?.is_duplicate" class="space-y-2">
          <div v-if="photo?.subject_group_id" class="flex items-center justify-between text-[11px] font-mono">
            <span class="text-[#71717A]">同主体组:</span>
            <span class="text-blue-300 font-bold bg-blue-500/15 px-1.5 py-0.2 rounded border border-blue-500/30">
              {{ photo.subject_group_id }}
            </span>
          </div>

          <!-- 严格重复警示 -->
          <div
            v-if="photo?.is_duplicate"
            class="flex items-center space-x-1 px-2 py-1.5 rounded-lg bg-rose-500/15 border border-rose-500/35 text-rose-300 text-[10px]"
          >
            <AlertTriangle class="w-3.5 h-3.5 text-rose-400 shrink-0" />
            <span>判定为极度雷同/重复帧 (参考: {{ photo.duplicate_of || '前帧' }})</span>
          </div>

          <!-- 推荐最佳代表帧 -->
          <div
            v-if="photo?.is_subject_best"
            class="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg bg-amber-400/15 border border-amber-400/35 text-amber-300 font-semibold text-[11px]"
          >
            <Award class="w-3.5 h-3.5 text-amber-400 shrink-0" />
            <span>同主体推荐最佳帧 (Best Shot)</span>
          </div>

          <!-- 一键留优汰次动作 -->
          <button
            v-if="photo?.subject_group_id"
            @click="handlePickBestAndRejectRest(photo.subject_group_id)"
            class="w-full flex items-center justify-center space-x-1 py-1.5 rounded-lg bg-amber-500/15 hover:bg-amber-500/25 text-amber-300 border border-amber-500/30 text-[10px] font-medium cursor-pointer transition-all active:scale-95 shadow-sm"
            title="将当前组的最佳帧标记为 Pick，其余过渡雷同帧全部标记为 Reject"
          >
            <span>★ 留用此组最佳帧并淘汰其余</span>
          </button>
        </div>
        <div v-else class="text-[10px] text-[#55555C] italic">
          {{ store.subjectClusters.length > 0 ? '该照片为独立场景散片' : '点击上方一键聚类识别同场景同主体' }}
        </div>
      </div>

      <!-- 模块 6: 生态关键字与 XMP 侧车同步 -->
      <div class="bg-[#18181C] p-3 rounded-xl border border-[#26262C] space-y-2.5 shadow-sm">
        <div class="flex items-center justify-between text-[10px] font-bold text-[#71717A] uppercase tracking-wider">
          <div class="flex items-center space-x-1">
            <Tag class="w-3.5 h-3.5 text-sky-400" />
            <span>生态关键字 & XMP</span>
          </div>
          <span v-if="xmpNotice" class="text-[10px] text-emerald-400 font-normal">
            {{ xmpNotice }}
          </span>
        </div>

        <!-- 既有标签芯片列表 -->
        <div class="flex flex-wrap gap-1 min-h-[24px]">
          <span
            v-for="tag in (photo?.tags || [])"
            :key="tag"
            class="inline-flex items-center space-x-1 px-2 py-0.5 rounded-md bg-[#222228] text-[#F4F4F5] border border-[#2E2E38] text-[10px]"
          >
            <span :title="tag">{{ tag }}</span>
            <button
              @click="handleRemoveTag(tag)"
              class="text-[#71717A] hover:text-rose-400 transition-colors cursor-pointer"
            >
              <CloseIcon class="w-2.5 h-2.5" />
            </button>
          </span>
          <span v-if="!photo?.tags?.length" class="text-[10px] text-[#55555C] italic py-0.5">
            暂无关键字
          </span>
        </div>

        <!-- 标签录入输入框 -->
        <div class="flex items-center space-x-1">
          <input
            v-model="newTagInput"
            @keydown.enter.prevent="handleAddTag"
            placeholder="输入关键字或层级(如 鸟类|红嘴相思鸟)..."
            class="flex-1 bg-[#121214] border border-[#26262C] rounded-lg px-2.5 py-1 text-[11px] text-[#F4F4F5] placeholder-[#55555C] focus:outline-none focus:border-sky-500 transition-colors"
          />
          <button
            @click="handleAddTag"
            class="p-1 rounded-lg bg-[#222228] hover:bg-[#2C2C34] text-[#F4F4F5] border border-[#2E2E38] cursor-pointer transition-colors active:scale-95"
            title="添加标签"
          >
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- 常用生态快速预设 -->
        <div class="flex flex-wrap gap-1 text-[9px]">
          <button
            v-for="preset in ['鸟类', '生态特写', '展翅飞行', '动物|鸟类|红嘴相思鸟']"
            :key="preset"
            @click="store.addPhotoTag(photo, preset)"
            class="px-2 py-0.5 rounded-md bg-[#202026] hover:bg-[#2A2A32] text-[#8E8E98] hover:text-[#F4F4F5] border border-[#26262E] transition-all cursor-pointer"
          >
            + {{ preset }}
          </button>
        </div>

        <!-- 直接写入/更新 XMP 侧车文件 -->
        <button
          @click="handleWriteXmp"
          :disabled="isWritingXmp || !photo"
          class="w-full flex items-center justify-center space-x-1.5 py-1.5 rounded-lg bg-emerald-600/15 hover:bg-emerald-600/25 text-emerald-400 border border-emerald-500/30 transition-all font-medium text-[11px] cursor-pointer disabled:opacity-50 active:scale-95 shadow-sm"
          title="生成或更新原图同目录下的标准 .xmp 侧车文件 (包含星级、旗标、色标、平铺关键字与层级分类)"
        >
          <FileCheck class="w-3.5 h-3.5" />
          <span>{{ isWritingXmp ? '正在写入...' : '写入 XMP 侧车 (同步 LrC)' }}</span>
        </button>
      </div>

      <!-- 模块 7: 单手暗房快捷键速查表 -->
      <div class="bg-[#18181C] p-3 rounded-xl border border-[#26262C] space-y-2.5 shadow-sm">
        <div class="flex items-center space-x-1 text-[10px] font-bold text-[#71717A] uppercase tracking-wider">
          <Keyboard class="w-3.5 h-3.5 text-amber-400" />
          <span>{{ t.inspector.hotkeysTitle }}</span>
        </div>
        <div class="grid grid-cols-2 gap-y-1.5 text-[10px] font-mono text-[#8E8E98]">
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">A / D</kbd> <span>{{ t.inspector.keysPrevNext }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">0 - 5</kbd> <span>{{ t.inspector.keysRate }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-emerald-400 font-bold shadow-xs">P</kbd> <span>{{ t.inspector.keysPick }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-rose-400 font-bold shadow-xs">X</kbd> <span>{{ t.inspector.keysReject }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">6 - 9</kbd> <span>{{ t.inspector.keysColor }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">E/G/C</kbd> <span>{{ t.inspector.keysMode }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">F</kbd> <span>{{ t.inspector.keysPeaking }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">B</kbd> <span>{{ t.inspector.keysBg }}</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-sky-400 font-bold shadow-xs">O</kbd> <span>人眼追踪</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-amber-400 font-bold shadow-xs">Z</kbd> <span>吸附对焦</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-emerald-400 font-bold shadow-xs">Ctrl +/-</kbd> <span>界面缩放</span></div>
          <div class="flex items-center space-x-1"><kbd class="px-1.5 py-0.2 rounded-md bg-[#222228] border border-[#2E2E38] text-[#F4F4F5] shadow-xs">Ctrl 0</kbd> <span>重置100%</span></div>
        </div>
      </div>
    </div>
  </aside>
</template>
