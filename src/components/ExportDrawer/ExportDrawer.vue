<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  X,
  FolderOpen,
  Download,
  CheckCircle2,
  AlertCircle,
  FileText,
  Layers,
  Settings,
  ShieldCheck,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type {
  ExportConfig,
  ExportOp,
  ExportProgress,
  ExportStrategy,
  PhotoGroup,
} from "../../types/photo";

const store = usePhotoStore();
const { t } = useI18n();

// 导出范围: 'picked' | 'filtered' | 'all'
const exportScope = ref<"picked" | "filtered" | "all">("picked");

// 目标目录路径
const targetDir = ref<string>("");

// 常用预设路径
const targetPresets = [
  { name: "本地精修临时库", path: "D:/PhotoSelect_Exports/精修库" },
  { name: "客户直出交付夹", path: "D:/PhotoSelect_Exports/客户交付" },
];

// 操作模式: 'copy' | 'move'
const operation = ref<ExportOp>("copy");

// RAW/JPG 策略: 'raw_only' | 'both' | 'std_only'
const exportStrategy = ref<ExportStrategy>("raw_only");
const smartFallback = ref<boolean>(true);
const splitFolders = ref<boolean>(false);
const generateXmp = ref<boolean>(true);

// 重命名模板
const renamePattern = ref<string>("{original}");
const renamePresets = [
  { label: "保留原文件名", pattern: "{original}" },
  { label: "日期_4位序列号 (20261017_0001)", pattern: "{date}_{seq4}" },
  { label: "机型_日期_序列号", pattern: "{camera}_{date}_{seq4}" },
];

// 导出运行状态
const isExporting = ref<boolean>(false);
const exportProgress = ref<ExportProgress | null>(null);
const exportDoneMessage = ref<string>("");
let unlistenProgress: UnlistenFn | null = null;

// 计算当前实际待导出的照片集合
const targetItems = computed<PhotoGroup[]>(() => {
  if (exportScope.value === "picked") {
    return store.groups.filter((g) => g.flag === 1);
  }
  if (exportScope.value === "filtered") {
    return store.filteredGroups;
  }
  return store.groups;
});

const handleBrowseTargetDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择导出目标文件夹",
    });
    if (selected && typeof selected === "string") {
      targetDir.value = selected;
    }
  } catch (err) {
    console.error("浏览文件夹失败:", err);
  }
};

const handleStartExport = async () => {
  if (!targetDir.value) {
    alert("请先选择或输入导出目标硬盘目录！");
    return;
  }

  const items = targetItems.value;
  if (items.length === 0) {
    alert("当前待导出的照片数量为 0！请先标记留用照片 [P] 或更改导出范围。");
    return;
  }

  isExporting.value = true;
  exportDoneMessage.value = "";
  exportProgress.value = {
    total: items.length,
    current: 0,
    current_file: "准备多线程流式流转中...",
    success_count: 0,
    fail_count: 0,
    errors: [],
    is_finished: false,
  };

  const config: ExportConfig = {
    target_dir: targetDir.value,
    operation: operation.value,
    export_strategy: exportStrategy.value,
    smart_fallback: smartFallback.value,
    split_folders: splitFolders.value,
    generate_xmp: generateXmp.value,
    rename_pattern: renamePattern.value === "{original}" ? null : renamePattern.value,
    items,
  };

  try {
    const finalResult = await invoke<ExportProgress>("execute_export", { config });
    exportProgress.value = finalResult;
    exportDoneMessage.value = `导出完成！成功: ${finalResult.success_count} 张, 失败: ${finalResult.fail_count} 张`;
  } catch (err) {
    console.error("导出执行失败:", err);
    exportDoneMessage.value = `导出异常中止: ${String(err)}`;
  } finally {
    isExporting.value = false;
  }
};

onMounted(async () => {
  try {
    // 监听 Tauri 后端实时流转进度事件
    unlistenProgress = await listen<ExportProgress>("export-progress", (event) => {
      exportProgress.value = event.payload;
    });
  } catch (err) {
    // 浏览器预览环境下忽略
  }
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
});
</script>

<template>
  <div
    v-if="store.exportDrawerOpen"
    class="fixed inset-0 z-50 flex justify-end bg-black/60 backdrop-blur-xs select-none transition-opacity"
  >
    <!-- 抽屉主体面板 -->
    <div
      class="w-[440px] h-full bg-[#181818] border-l border-[#2A2A2A] shadow-2xl flex flex-col justify-between overflow-hidden text-xs"
    >
      <!-- 抽屉头部 -->
      <div class="h-12 px-4 border-b border-[#2A2A2A] flex items-center justify-between bg-[#1C1C1C] transition-colors">
        <div class="flex items-center space-x-2">
          <Download class="w-4 h-4 text-emerald-400" />
          <h2 class="font-bold text-sm text-[#E5E5E5] tracking-wide">
            {{ t.export.title }}
          </h2>
        </div>
        <button
          @click="store.exportDrawerOpen = false"
          class="p-1 rounded text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-[#262626] transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 抽屉滚动配置区 -->
      <div class="flex-1 overflow-y-auto p-4 space-y-5">
        <!-- 1. 导出范围 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider flex items-center space-x-1">
            <Layers class="w-3.5 h-3.5 text-blue-400" />
            <span>导出范围</span>
          </label>
          <div class="grid grid-cols-3 gap-2">
            <button
              @click="exportScope = 'picked'"
              :class="[
                'p-2 rounded border text-center transition-all cursor-pointer',
                exportScope === 'picked'
                  ? 'border-emerald-500 bg-emerald-500/10 text-emerald-400 font-bold'
                  : 'border-[#2A2A2A] bg-[#141414] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <div>{{ t.export.pickedOnly }}</div>
              <div class="text-[10px] opacity-75 tabular-nums">({{ store.smartCounts.picked }} 张)</div>
            </button>
            <button
              @click="exportScope = 'filtered'"
              :class="[
                'p-2 rounded border text-center transition-all cursor-pointer',
                exportScope === 'filtered'
                  ? 'border-blue-500 bg-blue-500/10 text-blue-400 font-bold'
                  : 'border-[#2A2A2A] bg-[#141414] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <div>{{ t.export.filteredOnly }}</div>
              <div class="text-[10px] opacity-75 tabular-nums">({{ store.filteredGroups.length }} 张)</div>
            </button>
            <button
              @click="exportScope = 'all'"
              :class="[
                'p-2 rounded border text-center transition-all cursor-pointer',
                exportScope === 'all'
                  ? 'border-amber-500 bg-amber-500/10 text-amber-400 font-bold'
                  : 'border-[#2A2A2A] bg-[#141414] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <div>{{ t.export.allPhotos }}</div>
              <div class="text-[10px] opacity-75 tabular-nums">({{ store.groups.length }} 张)</div>
            </button>
          </div>
        </div>

        <!-- 2. 导出目标文件夹 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider flex items-center space-x-1">
            <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
            <span>目标硬盘目录</span>
          </label>
          <div class="flex items-center space-x-2">
            <input
              v-model="targetDir"
              type="text"
              placeholder="选择目标文件夹..."
              class="flex-1 bg-[#121212] border border-[#2A2A2A] rounded px-2.5 py-1.5 text-xs text-[#E5E5E5] font-mono focus:outline-hidden focus:border-[#4D4D4D]"
            />
            <button
              @click="handleBrowseTargetDir"
              class="px-3 py-1.5 rounded bg-[#262626] hover:bg-[#333333] border border-[#333333] text-[#E5E5E5] font-medium"
            >
              浏览...
            </button>
          </div>
          <!-- 预设快速填入 -->
          <div class="flex items-center space-x-1.5">
            <span class="text-[10px] text-[#8C8C8C]">预设:</span>
            <button
              v-for="p in targetPresets"
              :key="p.name"
              @click="targetDir = p.path"
              class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5] border border-[#2A2A2A]"
            >
              {{ p.name }}
            </button>
          </div>
        </div>

        <!-- 3. 文件操作方式: 复制 vs 移动 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider">
            流转操作方式
          </label>
          <div class="grid grid-cols-2 gap-2">
            <label
              :class="[
                'flex items-center space-x-2 p-2 rounded border cursor-pointer',
                operation === 'copy'
                  ? 'border-emerald-500/50 bg-emerald-500/10 text-[#E5E5E5]'
                  : 'border-[#2A2A2A] bg-[#141414] text-[#8C8C8C]',
              ]"
            >
              <input type="radio" value="copy" v-model="operation" class="text-emerald-500" />
              <div>
                <div class="font-bold">复制 (Copy)</div>
                <div class="text-[10px] opacity-75">保留相机原卡数据 (推荐)</div>
              </div>
            </label>
            <label
              :class="[
                'flex items-center space-x-2 p-2 rounded border cursor-pointer',
                operation === 'move'
                  ? 'border-rose-500/50 bg-rose-500/10 text-[#E5E5E5]'
                  : 'border-[#2A2A2A] bg-[#141414] text-[#8C8C8C]',
              ]"
            >
              <input type="radio" value="move" v-model="operation" class="text-rose-500" />
              <div>
                <div class="font-bold">移动 (Move)</div>
                <div class="text-[10px] opacity-75">安全落盘后删除源文件</div>
              </div>
            </label>
          </div>
        </div>

        <!-- 4. 同名 RAW 联动与文件分类核心策略 -->
        <div class="space-y-2 p-3 bg-[#141414] rounded-lg border border-[#2A2A2A]">
          <label class="text-[11px] font-bold text-amber-400 uppercase tracking-wider flex items-center space-x-1">
            <ShieldCheck class="w-3.5 h-3.5" />
            <span>RAW+JPG 智能同名流转策略</span>
          </label>

          <div class="space-y-1.5 pt-1">
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input
                type="radio"
                value="raw_only"
                v-model="exportStrategy"
                class="text-blue-500"
              />
              <span class="font-medium">仅导出同名 RAW <span class="text-[#8C8C8C] text-[10px]">(看 JPG 挑选，只把同名 RAW 拷入修图盘)</span></span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input
                type="radio"
                value="both"
                v-model="exportStrategy"
                class="text-blue-500"
              />
              <span class="font-medium">同时导出同名 RAW + JPG</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input
                type="radio"
                value="std_only"
                v-model="exportStrategy"
                class="text-blue-500"
              />
              <span class="font-medium">仅导出标准图 (JPG/PNG)</span>
            </label>
          </div>

          <div class="h-[1px] bg-[#222222] my-2"></div>

          <!-- 附加复选框 -->
          <div class="space-y-2 pt-1">
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input type="checkbox" v-model="smartFallback" class="rounded text-blue-500" />
              <span>智能回退防漏片: 选仅导 RAW 时，若单张照片无 RAW 则导出 JPG</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input type="checkbox" v-model="splitFolders" class="rounded text-blue-500" />
              <span>分设独立子目录: 分别存放至 /RAW 与 /JPG 文件夹</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input type="checkbox" v-model="generateXmp" class="rounded text-emerald-500" />
              <span class="text-emerald-400 font-semibold flex items-center space-x-1">
                <FileText class="w-3 h-3" />
                <span>生成 Lightroom Classic 兼容的 .xmp 侧车文件 (同步星级与旗标)</span>
              </span>
            </label>
          </div>
        </div>

        <!-- 5. 批量重命名 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#8C8C8C] uppercase tracking-wider flex items-center space-x-1">
            <Settings class="w-3.5 h-3.5" />
            <span>导出批量重命名</span>
          </label>
          <div class="space-y-1.5">
            <select
              v-model="renamePattern"
              class="w-full bg-[#121212] border border-[#2A2A2A] rounded px-2.5 py-1.5 text-xs text-[#E5E5E5] font-mono focus:outline-hidden"
            >
              <option v-for="r in renamePresets" :key="r.pattern" :value="r.pattern">
                {{ r.label }}
              </option>
            </select>
          </div>
        </div>

        <!-- 6. 实时导出进度条 -->
        <div v-if="isExporting || exportProgress" class="p-3 bg-[#121212] rounded border border-[#2A2A2A] space-y-2">
          <div class="flex items-center justify-between text-[11px] font-mono">
            <span class="text-[#8C8C8C]">传输进度:</span>
            <span class="text-[#E5E5E5] font-bold">
              {{ exportProgress?.current || 0 }} / {{ exportProgress?.total || 0 }}
            </span>
          </div>
          <!-- 进度条 -->
          <div class="w-full h-2 bg-[#222222] rounded-full overflow-hidden">
            <div
              class="h-full bg-emerald-500 transition-all duration-150"
              :style="{
                width: `${exportProgress?.total ? ((exportProgress.current / exportProgress.total) * 100) : 0}%`,
              }"
            ></div>
          </div>
          <div class="text-[10px] font-mono text-[#8C8C8C] truncate">
            {{ exportProgress?.current_file }}
          </div>

          <div v-if="exportDoneMessage" class="flex items-center space-x-1.5 text-emerald-400 text-xs font-semibold pt-1">
            <CheckCircle2 class="w-3.5 h-3.5" />
            <span>{{ exportDoneMessage }}</span>
          </div>

          <!-- 错误日志 -->
          <div v-if="exportProgress?.errors && exportProgress.errors.length > 0" class="text-rose-400 text-[10px] font-mono space-y-1">
            <div v-for="err in exportProgress.errors" :key="err" class="flex items-center space-x-1">
              <AlertCircle class="w-3 h-3 flex-shrink-0" />
              <span class="truncate">{{ err }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 抽屉底部大按钮 -->
      <div class="p-4 border-t border-[#2A2A2A] bg-[#1C1C1C]">
        <button
          @click="handleStartExport"
          :disabled="isExporting || targetItems.length === 0"
          :class="[
            'w-full py-2.5 rounded font-bold text-sm tracking-wide transition-all shadow-lg flex items-center justify-center space-x-2',
            isExporting || targetItems.length === 0
              ? 'bg-[#333333] text-[#666666] cursor-not-allowed'
              : 'bg-emerald-500 hover:bg-emerald-400 text-black active:scale-[0.98]',
          ]"
        >
          <Download class="w-4 h-4" />
          <span>
            {{ isExporting ? '多线程流式导出中...' : `立即快捷导出 (共 ${targetItems.length} 张)` }}
          </span>
        </button>
      </div>
    </div>
  </div>
</template>
