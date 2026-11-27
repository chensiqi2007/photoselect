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
  AlertTriangle,
  Trash2,
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

// 删除/移动模式二次确认弹窗控制
const showMoveConfirmModal = ref<boolean>(false);
const showExecuteConfirmModal = ref<boolean>(false);

const requestSelectOperation = (op: ExportOp) => {
  if (op === "move") {
    if (operation.value !== "move") {
      showMoveConfirmModal.value = true;
    }
  } else {
    operation.value = "copy";
  }
};

const confirmEnableMoveMode = () => {
  operation.value = "move";
  showMoveConfirmModal.value = false;
};

const cancelEnableMoveMode = () => {
  showMoveConfirmModal.value = false;
};

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

const handleStartExport = () => {
  if (!targetDir.value) {
    alert("请先选择或输入导出目标硬盘目录！");
    return;
  }

  const items = targetItems.value;
  if (items.length === 0) {
    alert("当前待导出的照片数量为 0！请先标记留用照片 [P] 或更改导出范围。");
    return;
  }

  // 若处于移动/删除模式，触发二次确认弹窗
  if (operation.value === "move") {
    showExecuteConfirmModal.value = true;
    return;
  }

  doExport();
};

const confirmAndExecuteMove = () => {
  showExecuteConfirmModal.value = false;
  doExport();
};

const doExport = async () => {
  const items = targetItems.value;
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

const handleEscapeCapture = (e: KeyboardEvent) => {
  if (e.key === "Escape") {
    if (showExecuteConfirmModal.value) {
      e.stopPropagation();
      e.stopImmediatePropagation();
      showExecuteConfirmModal.value = false;
    } else if (showMoveConfirmModal.value) {
      e.stopPropagation();
      e.stopImmediatePropagation();
      showMoveConfirmModal.value = false;
    }
  }
};

onMounted(async () => {
  window.addEventListener("keydown", handleEscapeCapture, true);
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
  window.removeEventListener("keydown", handleEscapeCapture, true);
  if (unlistenProgress) {
    unlistenProgress();
  }
});
</script>

<template>
  <div
    v-if="store.exportDrawerOpen"
    class="fixed inset-0 z-50 flex justify-end bg-black/60 backdrop-blur-sm select-none transition-opacity"
  >
    <!-- 抽屉主体面板 -->
    <div
      class="w-[460px] h-full bg-[#141418] border-l border-white/10 shadow-[0_0_60px_rgba(0,0,0,0.8)] flex flex-col justify-between overflow-hidden text-xs"
    >
      <!-- 抽屉头部 -->
      <div class="h-12 px-5 border-b border-white/10 flex items-center justify-between bg-white/5 shrink-0">
        <div class="flex items-center space-x-2">
          <Download class="w-4 h-4 text-emerald-400" />
          <h2 class="font-bold text-sm text-[#F4F4F5] tracking-wide">
            {{ t.export.title }}
          </h2>
        </div>
        <button
          @click="store.exportDrawerOpen = false"
          class="p-1.5 rounded-lg text-[#71717A] hover:text-[#F4F4F5] hover:bg-white/10 transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 抽屉滚动配置区 -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5">
        <!-- 1. 导出范围 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#A1A1AA] uppercase tracking-wider flex items-center space-x-1.5">
            <Layers class="w-3.5 h-3.5 text-blue-400" />
            <span>导出范围</span>
          </label>
          <div class="grid grid-cols-3 gap-2">
            <button
              @click="exportScope = 'picked'"
              :class="[
                'p-2.5 rounded-xl border text-center transition-all cursor-pointer shadow-sm',
                exportScope === 'picked'
                  ? 'border-emerald-500/50 bg-emerald-500/15 text-emerald-400 font-bold shadow-emerald-500/10'
                  : 'border-white/8 bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10',
              ]"
            >
              <div>{{ t.export.pickedOnly }}</div>
              <div class="text-[10px] opacity-75 tabular-nums font-mono">({{ store.smartCounts.picked }} 张)</div>
            </button>
            <button
              @click="exportScope = 'filtered'"
              :class="[
                'p-2.5 rounded-xl border text-center transition-all cursor-pointer shadow-sm',
                exportScope === 'filtered'
                  ? 'border-blue-500/50 bg-blue-500/15 text-blue-400 font-bold shadow-blue-500/10'
                  : 'border-white/8 bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10',
              ]"
            >
              <div>{{ t.export.filteredOnly }}</div>
              <div class="text-[10px] opacity-75 tabular-nums font-mono">({{ store.filteredGroups.length }} 张)</div>
            </button>
            <button
              @click="exportScope = 'all'"
              :class="[
                'p-2.5 rounded-xl border text-center transition-all cursor-pointer shadow-sm',
                exportScope === 'all'
                  ? 'border-amber-500/50 bg-amber-500/15 text-amber-400 font-bold shadow-amber-500/10'
                  : 'border-white/8 bg-white/5 text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-white/10',
              ]"
            >
              <div>{{ t.export.allPhotos }}</div>
              <div class="text-[10px] opacity-75 tabular-nums font-mono">({{ store.groups.length }} 张)</div>
            </button>
          </div>
        </div>

        <!-- 2. 导出目标文件夹 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#A1A1AA] uppercase tracking-wider flex items-center space-x-1.5">
            <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
            <span>目标硬盘目录</span>
          </label>
          <div class="flex items-center space-x-2">
            <input
              v-model="targetDir"
              type="text"
              placeholder="选择目标文件夹..."
              class="flex-1 bg-[#0E0E12] border border-white/10 rounded-xl px-3 py-2 text-xs text-[#E5E5E5] font-mono focus:outline-hidden focus:border-sky-500/60 shadow-inner"
            />
            <button
              @click="handleBrowseTargetDir"
              class="px-3.5 py-2 rounded-xl bg-white/10 hover:bg-white/15 border border-white/10 text-[#F4F4F5] font-medium transition-all cursor-pointer shadow-sm active:scale-95"
            >
              浏览...
            </button>
          </div>
          <!-- 预设快速填入 -->
          <div class="flex items-center space-x-1.5 pt-0.5">
            <span class="text-[10px] text-[#71717A]">预设:</span>
            <button
              v-for="p in targetPresets"
              :key="p.name"
              @click="targetDir = p.path"
              class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-white/5 text-[#A1A1AA] hover:text-[#F4F4F5] hover:bg-white/10 border border-white/8 transition-colors cursor-pointer"
            >
              {{ p.name }}
            </button>
          </div>
        </div>

        <!-- 3. 文件操作方式: 复制 vs 移动 (删除模式) -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#A1A1AA] uppercase tracking-wider flex items-center justify-between">
            <span>流转操作方式</span>
            <span v-if="operation === 'move'" class="text-[10px] text-rose-400 font-semibold flex items-center gap-1">
              <AlertTriangle class="w-3 h-3" />
              <span>已开启删除源卡模式</span>
            </span>
          </label>
          <div class="grid grid-cols-2 gap-2">
            <div
              @click="requestSelectOperation('copy')"
              :class="[
                'flex items-center space-x-2.5 p-2.5 rounded-xl border cursor-pointer transition-all',
                operation === 'copy'
                  ? 'border-emerald-500/50 bg-emerald-500/10 text-[#F4F4F5]'
                  : 'border-white/8 bg-white/5 text-[#8C8C8C] hover:bg-white/8',
              ]"
            >
              <input
                type="radio"
                name="export_op"
                value="copy"
                :checked="operation === 'copy'"
                class="accent-emerald-500 cursor-pointer pointer-events-none"
              />
              <div>
                <div class="font-bold text-xs">复制 (Copy)</div>
                <div class="text-[10px] opacity-75">保留相机原卡数据 (推荐)</div>
              </div>
            </div>
            <div
              @click="requestSelectOperation('move')"
              :class="[
                'flex items-center space-x-2.5 p-2.5 rounded-xl border cursor-pointer transition-all',
                operation === 'move'
                  ? 'border-rose-500/50 bg-rose-500/10 text-[#F4F4F5]'
                  : 'border-white/8 bg-white/5 text-[#8C8C8C] hover:bg-white/8',
              ]"
            >
              <input
                type="radio"
                name="export_op"
                value="move"
                :checked="operation === 'move'"
                class="accent-rose-500 cursor-pointer pointer-events-none"
              />
              <div>
                <div class="font-bold text-xs flex items-center space-x-1">
                  <span>移动 / 删除</span>
                  <span class="px-1 py-0.2 rounded text-[9px] bg-rose-500/20 text-rose-300 font-normal">删原卡</span>
                </div>
                <div class="text-[10px] opacity-75">安全落盘后删除源文件</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 4. 同名 RAW 联动与文件分类核心策略 -->
        <div class="space-y-2.5 p-3.5 bg-white/5 rounded-xl border border-white/8">
          <label class="text-[11px] font-bold text-amber-400 uppercase tracking-wider flex items-center space-x-1.5">
            <ShieldCheck class="w-3.5 h-3.5" />
            <span>RAW+JPG 智能同名流转策略</span>
          </label>

          <div class="space-y-2 pt-1">
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input
                type="radio"
                value="raw_only"
                v-model="exportStrategy"
                class="accent-blue-500 cursor-pointer"
              />
              <span class="font-medium">仅导出同名 RAW <span class="text-[#8C8C8C] text-[10px]">(看 JPG 挑选，只把同名 RAW 拷入修图盘)</span></span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input
                type="radio"
                value="both"
                v-model="exportStrategy"
                class="accent-blue-500 cursor-pointer"
              />
              <span class="font-medium">同时导出同名 RAW + JPG</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input
                type="radio"
                value="std_only"
                v-model="exportStrategy"
                class="accent-blue-500 cursor-pointer"
              />
              <span class="font-medium">仅导出标准图 (JPG/PNG)</span>
            </label>
          </div>

          <div class="h-[1px] bg-white/5 my-2"></div>

          <!-- 附加复选框 -->
          <div class="space-y-2 pt-1">
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input type="checkbox" v-model="smartFallback" class="rounded accent-blue-500 cursor-pointer" />
              <span>智能回退防漏片: 选仅导 RAW 时，若单张照片无 RAW 则导出 JPG</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input type="checkbox" v-model="splitFolders" class="rounded accent-blue-500 cursor-pointer" />
              <span>分设独立子目录: 分别存放至 /RAW、/JPG 与 /VIDEO 文件夹</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-[#E5E5E5]">
              <input type="checkbox" v-model="generateXmp" class="rounded accent-emerald-500 cursor-pointer" />
              <span class="text-emerald-400 font-semibold flex items-center space-x-1.5">
                <FileText class="w-3.5 h-3.5" />
                <span>生成 Lightroom Classic 兼容的 .xmp 侧车文件 (同步星级与旗标)</span>
              </span>
            </label>
          </div>
        </div>

        <!-- 5. 批量重命名 -->
        <div class="space-y-2">
          <label class="text-[11px] font-bold text-[#A1A1AA] uppercase tracking-wider flex items-center space-x-1.5">
            <Settings class="w-3.5 h-3.5" />
            <span>导出批量重命名</span>
          </label>
          <div class="space-y-1.5">
            <select
              v-model="renamePattern"
              class="w-full bg-[#0E0E12] border border-white/10 rounded-xl px-3 py-2 text-xs text-[#E5E5E5] font-mono focus:outline-hidden focus:border-sky-500/60 shadow-inner cursor-pointer"
            >
              <option v-for="r in renamePresets" :key="r.pattern" :value="r.pattern">
                {{ r.label }}
              </option>
            </select>
          </div>
        </div>

        <!-- 6. 实时导出进度条 -->
        <div v-if="isExporting || exportProgress" class="p-3.5 bg-black/40 rounded-xl border border-white/10 space-y-2.5">
          <div class="flex items-center justify-between text-[11px] font-mono">
            <span class="text-[#A1A1AA]">传输进度:</span>
            <span class="text-[#F4F4F5] font-bold">
              {{ exportProgress?.current || 0 }} / {{ exportProgress?.total || 0 }}
            </span>
          </div>
          <!-- 进度条 -->
          <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
            <div
              class="h-full bg-gradient-to-r from-emerald-500 to-teal-400 transition-all duration-150"
              :style="{
                width: `${exportProgress?.total ? ((exportProgress.current / exportProgress.total) * 100) : 0}%`,
              }"
            ></div>
          </div>
          <div class="text-[10px] font-mono text-[#A1A1AA] truncate">
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
      <div class="p-5 border-t border-white/10 bg-white/5 shrink-0">
        <button
          @click="handleStartExport"
          :disabled="isExporting || targetItems.length === 0"
          :class="[
            'w-full py-3 rounded-xl font-bold text-sm tracking-wide transition-all shadow-xl flex items-center justify-center space-x-2',
            isExporting || targetItems.length === 0
              ? 'bg-white/10 text-[#666666] cursor-not-allowed border border-white/5'
              : operation === 'move'
              ? 'bg-gradient-to-r from-rose-600 via-rose-500 to-amber-600 hover:from-rose-500 hover:to-amber-500 text-white shadow-rose-600/30 active:scale-98 cursor-pointer'
              : 'bg-gradient-to-r from-emerald-600 via-teal-600 to-emerald-500 hover:from-emerald-500 hover:to-teal-400 text-white shadow-emerald-600/30 active:scale-98 cursor-pointer',
          ]"
        >
          <component :is="operation === 'move' ? Trash2 : Download" class="w-4 h-4" />
          <span>
            {{ isExporting ? '多线程流式流转中...' : operation === 'move' ? `执行移动并删除原片 (共 ${targetItems.length} 张)` : `立即快捷导出 (共 ${targetItems.length} 张)` }}
          </span>
        </button>
      </div>
    </div>

    <!-- 模态框 1: 切换移动/删除模式确认 -->
    <div
      v-if="showMoveConfirmModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 select-none"
      @click.self="cancelEnableMoveMode"
    >
      <div
        class="w-full max-w-sm rounded-2xl bg-[#1A1A20] border border-amber-500/40 p-5 shadow-2xl flex flex-col space-y-4 shadow-amber-500/10"
      >
        <div class="flex items-start space-x-3">
          <div class="p-2.5 rounded-xl bg-amber-500/15 text-amber-400 shrink-0 border border-amber-500/30">
            <AlertTriangle class="w-6 h-6 stroke-[2.2]" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-white tracking-wide">切换至「移动 / 删除模式」？</h3>
            <p class="text-xs text-[#A1A1AA] mt-1 leading-relaxed">
              此模式在将照片导出到目标目录并确认大小一致后，将<span class="text-rose-400 font-bold">永久删除源存储介质（如 SD 卡或源文件夹）中的原始照片</span>以释放空间。
            </p>
          </div>
        </div>

        <div class="p-3 rounded-xl bg-black/40 border border-white/5 text-[11px] text-[#A1A1AA] space-y-1">
          <div class="text-amber-300/90 font-medium flex items-center gap-1">
            <AlertCircle class="w-3.5 h-3.5 text-amber-400" />
            <span>不可逆操作风险：</span>
          </div>
          <div>• 删除后的源卡文件无法从回收站恢复</div>
          <div>• 建议仅在需要腾出卡内空间且已有完备备份时使用</div>
        </div>

        <div class="flex items-center space-x-2 pt-1">
          <button
            @click="cancelEnableMoveMode"
            class="flex-1 py-2 px-3 rounded-xl bg-white/5 hover:bg-white/10 text-xs font-semibold text-[#D4D4D8] border border-white/10 transition-colors cursor-pointer"
          >
            取消 (保持复制)
          </button>
          <button
            @click="confirmEnableMoveMode"
            class="flex-1 py-2 px-3 rounded-xl bg-gradient-to-r from-amber-600 to-rose-600 hover:from-amber-500 hover:to-rose-500 text-xs font-bold text-white shadow-lg shadow-rose-600/20 transition-all cursor-pointer"
          >
            确认开启该模式
          </button>
        </div>
      </div>
    </div>

    <!-- 模态框 2: 最终执行导出并删除源文件二次确认 -->
    <div
      v-if="showExecuteConfirmModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4 select-none"
      @click.self="showExecuteConfirmModal = false"
    >
      <div
        class="w-full max-w-md rounded-2xl bg-[#18181D] border border-rose-500/50 p-6 shadow-2xl flex flex-col space-y-4 shadow-rose-500/15"
      >
        <div class="flex items-start space-x-3.5">
          <div class="p-3 rounded-xl bg-rose-500/15 text-rose-400 shrink-0 border border-rose-500/30">
            <Trash2 class="w-6 h-6 stroke-[2.2]" />
          </div>
          <div>
            <h3 class="text-base font-bold text-white tracking-wide">最终确认：执行移动并删除源文件？</h3>
            <p class="text-xs text-[#A1A1AA] mt-1 leading-relaxed">
              您当前处于<span class="text-rose-400 font-bold">【移动 / 删除模式】</span>，导出完成后源文件将被立即物理删除。
            </p>
          </div>
        </div>

        <!-- 详细参数确认卡片 -->
        <div class="p-3.5 rounded-xl bg-black/50 border border-white/10 space-y-2 text-xs">
          <div class="flex justify-between items-center text-[#D4D4D8]">
            <span class="text-[#8E8E98]">待处理照片数量:</span>
            <span class="font-mono font-bold text-white">{{ targetItems.length }} 张</span>
          </div>
          <div class="flex flex-col space-y-1 text-[#D4D4D8]">
            <span class="text-[#8E8E98]">导出目标目录:</span>
            <span class="font-mono text-[11px] text-amber-300 break-all bg-white/5 p-1.5 rounded border border-white/5">
              {{ targetDir }}
            </span>
          </div>
          <div class="flex justify-between items-center text-rose-400 font-semibold pt-1 border-t border-white/5">
            <span>源卡文件处理:</span>
            <span>落盘校验后永久删除源文件</span>
          </div>
        </div>

        <p class="text-[11px] text-rose-300/80 bg-rose-500/10 p-2.5 rounded-lg border border-rose-500/20">
          ⚠️ 警告：该操作不可撤销！请确认目标目录所在磁盘有充足空间，且源存储卡中的原始数据允许被清理。
        </p>

        <div class="flex items-center space-x-3 pt-2">
          <button
            @click="showExecuteConfirmModal = false"
            class="flex-1 py-2.5 px-4 rounded-xl bg-white/5 hover:bg-white/10 text-xs font-semibold text-[#E4E4E7] border border-white/10 transition-colors cursor-pointer"
          >
            放弃 (返回修改)
          </button>
          <button
            @click="confirmAndExecuteMove"
            class="flex-1 py-2.5 px-4 rounded-xl bg-gradient-to-r from-rose-600 to-red-700 hover:from-rose-500 hover:to-red-600 text-xs font-bold text-white shadow-lg shadow-rose-600/30 transition-all cursor-pointer flex items-center justify-center space-x-1.5"
          >
            <Trash2 class="w-3.5 h-3.5" />
            <span>确认执行并删除</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
