<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  FolderOpen,
  HardDrive,
  ArrowUpRight,
  UploadCloud,
  Layers,
} from "lucide-vue-next";
import { usePhotoStore } from "./stores/photoStore";
import { useHotkeys } from "./composables/useHotkeys";
import { useI18n } from "./i18n";
import TopBar from "./components/TopBar.vue";
import FilterSidebar from "./components/FilterSidebar.vue";
import Filmstrip from "./components/Filmstrip/Filmstrip.vue";
import GridView from "./components/Grid/GridView.vue";
import InspectorPanel from "./components/Inspector/InspectorPanel.vue";
import ViewportWebGL from "./components/ViewportWebGL/ViewportWebGL.vue";
import CompareView from "./components/Compare/CompareView.vue";
import ExportDrawer from "./components/ExportDrawer/ExportDrawer.vue";
import SettingsModal from "./components/Settings/SettingsModal.vue";
import AISelectionPanel from "./components/AISelection/AISelectionPanel.vue";

const store = usePhotoStore();
const { t } = useI18n();
useHotkeys(); // 激活全局全键盘暗房交互状态机

const isDragOver = ref(false);
let unlistenDragDrop: (() => void) | null = null;
let unlistenAiProgress: UnlistenFn | null = null;

const handleSelectFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择包含照片或相机 RAW 的目录",
    });
    if (selected && typeof selected === "string") {
      await store.loadFolder(selected);
    }
  } catch (err) {
    console.error("选择目录失败:", err);
  }
};

onMounted(async () => {
  // 初始化系统外观模式 (黑暗/白昼)
  store.initTheme();
  try {
    await store.loadAiSettings();
    unlistenAiProgress = await listen("ai-selection-progress", (event) => {
      store.consumeAiProgress(event.payload as any);
    });
  } catch (error) {
    console.warn("AI 设置初始化失败:", error);
  }

  // 监听 Tauri 2.0 系统原生文件/文件夹拖拽放入事件
  try {
    const webview = getCurrentWebviewWindow();
    unlistenDragDrop = await webview.onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        isDragOver.value = true;
      } else if (event.payload.type === "drop") {
        isDragOver.value = false;
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          store.loadFolder(paths[0]);
        }
      } else {
        isDragOver.value = false;
      }
    });
  } catch (err) {
    // 非 Webview 环境忽略
  }
});

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
  }
  if (unlistenAiProgress) unlistenAiProgress();
});
</script>

<template>
  <div class="w-screen h-screen flex flex-col bg-[#121212] text-[#E5E5E5] overflow-hidden select-none font-sans">
    <!-- 1. 顶部极简暗房工具栏 (36px) -->
    <TopBar />

    <!-- 中间主工作区分栏 (左侧目录树 + 中央主视口 + 右侧参数栏) -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 2. 左侧资源与智能分类栏 (240px, 可折叠) -->
      <FilterSidebar v-if="!store.sidebarCollapsed" />

      <!-- 3. 中央主视口 (自适应 flex-1, 基于 WebGL 2.0 着色器加速) -->
      <main
        class="flex-1 relative flex flex-col items-center justify-center overflow-hidden transition-colors"
        :style="{ backgroundColor: store.viewportBg }"
      >
        <!-- 单图 Loupe 模式 (WebGL 2.0 硬件渲染加速 + 峰值对焦) -->
        <template v-if="store.viewMode === 'loupe'">
          <ViewportWebGL v-if="store.currentPhoto" />

          <!-- 空状态：极速挂载与目录直达 -->
          <div
            v-else
            class="text-center p-8 max-w-lg flex flex-col items-center justify-center border-2 border-dashed rounded-2xl transition-all"
            :class="[
              isDragOver
                ? 'border-sky-400 bg-sky-500/10 scale-102'
                : store.theme === 'light'
                ? 'border-[#D1D1D6] bg-white/90 shadow-xl'
                : 'border-[#2A2A2A] bg-[#161616]/70 shadow-xl backdrop-blur-sm'
            ]"
          >
            <div class="w-16 h-16 rounded-2xl bg-[#1C1C1C] border border-[#333333] flex items-center justify-center mb-4 text-sky-400 shadow-xl">
              <UploadCloud class="w-8 h-8 stroke-[1.8]" />
            </div>

            <h2 class="text-lg font-bold text-[#E5E5E5] mb-2 tracking-wide">
              挂载相机卡或选择照片目录
            </h2>
            <p class="text-xs text-[#8C8C8C] mb-6 leading-relaxed max-w-md">
              直接拖拽 SD 卡、CFexpress 卷或包含 JPG 与 RAW 的物理文件夹至窗口；或点击下方按钮选择本地相册
            </p>

            <button
              @click="handleSelectFolder"
              class="px-6 py-2.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-bold text-xs transition-all shadow-xl hover:shadow-2xl active:scale-95 flex items-center space-x-2 cursor-pointer mb-6"
            >
              <FolderOpen class="w-4 h-4 text-white" />
              <span>{{ t.topbar.openFolder }}</span>
            </button>

            <!-- 扫描错误提示 -->
            <div v-if="store.scanError" class="mb-4 text-xs text-rose-400 bg-rose-500/10 p-2.5 rounded border border-rose-500/20 max-w-md break-all">
              {{ store.scanError }}
            </div>

            <!-- 快速直达检测到的系统驱动器与移动存储卡 -->
            <div v-if="store.mountPoints.length > 0" class="w-full pt-4 border-t border-[#262626] text-left">
              <div class="text-[11px] font-mono text-[#666666] mb-2 flex items-center space-x-1">
                <HardDrive class="w-3 h-3 text-[#8C8C8C]" />
                <span>已检测到的系统驱动器 / 移动存储卡:</span>
              </div>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="mount in store.mountPoints"
                  :key="mount"
                  @click="store.loadFolder(mount)"
                  class="flex items-center space-x-1.5 px-3 py-1.5 rounded bg-[#202020] hover:bg-[#2A2A2A] border border-[#333333] text-xs font-mono text-[#E5E5E5] transition-colors cursor-pointer"
                >
                  <HardDrive class="w-3.5 h-3.5 text-blue-400" />
                  <span>{{ mount }}</span>
                  <ArrowUpRight class="w-3 h-3 text-[#8C8C8C]" />
                </button>
              </div>
            </div>

            <!-- 最近工作区快捷入口 -->
            <div v-else-if="store.recentWorkspaces.length > 0" class="w-full pt-4 border-t border-[#262626] text-left">
              <div class="text-[11px] font-mono text-[#666666] mb-2 flex items-center space-x-1">
                <Layers class="w-3 h-3 text-[#8C8C8C]" />
                <span>最近打开的历史工作区:</span>
              </div>
              <div class="space-y-1">
                <button
                  v-for="ws in store.recentWorkspaces.slice(0, 3)"
                  :key="ws.id"
                  @click="store.loadFolder(ws.folder_path)"
                  class="w-full flex items-center justify-between px-3 py-1.5 rounded bg-[#202020] hover:bg-[#2A2A2A] border border-[#333333] text-xs font-mono text-[#8C8C8C] hover:text-[#E5E5E5] transition-colors text-left cursor-pointer"
                >
                  <span class="truncate max-w-[340px]">{{ ws.folder_path }}</span>
                  <ArrowUpRight class="w-3 h-3 flex-shrink-0" />
                </button>
              </div>
            </div>
          </div>
        </template>

        <!-- 网格 Grid 模式 -->
        <GridView v-else-if="store.viewMode === 'grid'" />

        <!-- 双图对比 Compare 模式 -->
        <CompareView v-else-if="store.viewMode === 'compare'" />
      </main>

      <!-- 5. 右侧 EXIF 与相机信息栏 (280px) -->
      <InspectorPanel />
    </div>

    <!-- 4. 底部横向虚拟滚动胶片栏 (130px) -->
    <Filmstrip />

    <!-- 6. 类 LrC 快捷导出侧边抽屉 -->
    <ExportDrawer />

    <!-- 7. 系统偏好设置模态框 (外观/主题/多语言) -->
    <SettingsModal v-if="store.settingsModalOpen" />
    <AISelectionPanel v-if="store.aiSelectionPanelOpen" />
  </div>
</template>
