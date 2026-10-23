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
  Camera,
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
  // 初始化系统 UI 页面与字体缩放 (100% / 125% / 150% / 200%)
  store.initUiScale();
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
  <div class="w-full h-full flex flex-col bg-[#121212] text-[#E5E5E5] overflow-hidden select-none font-sans">
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

          <!-- 空状态：专业级暗房工作台直达 (Pro Darkroom Studio Viewfinder) -->
          <div v-else class="relative w-full h-full flex items-center justify-center p-6 overflow-hidden">
            <!-- 核心取景框容器 (Camera Viewfinder Frame) -->
            <div
              class="relative z-10 w-full max-w-xl p-8 md:p-10 rounded-2xl transition-all duration-200 border"
              :class="[
                isDragOver
                  ? 'border-sky-500 bg-sky-500/10 shadow-[0_0_30px_rgba(14,165,233,0.2)] scale-[1.01]'
                  : store.theme === 'light'
                  ? 'border-[#E4E4E7] bg-white shadow-xl'
                  : 'border-[#26262C] bg-[#151518] shadow-[0_20px_50px_rgba(0,0,0,0.5)]'
              ]"
            >
              <!-- 取景框四角准星刻度 (Viewfinder Reticle Marks) -->
              <div class="absolute top-2.5 left-2.5 w-3.5 h-3.5 border-t border-l border-white/20 pointer-events-none"></div>
              <div class="absolute top-2.5 right-2.5 w-3.5 h-3.5 border-t border-r border-white/20 pointer-events-none"></div>
              <div class="absolute bottom-2.5 left-2.5 w-3.5 h-3.5 border-b border-l border-white/20 pointer-events-none"></div>
              <div class="absolute bottom-2.5 right-2.5 w-3.5 h-3.5 border-b border-r border-white/20 pointer-events-none"></div>

              <!-- 顶部精工镜头刻度图形 -->
              <div class="flex flex-col items-center mb-6">
                <div
                  class="w-14 h-14 rounded-2xl flex items-center justify-center border mb-3.5 transition-transform duration-200"
                  :class="[
                    store.theme === 'light'
                      ? 'bg-zinc-100 border-zinc-200 text-zinc-700 shadow-sm'
                      : 'bg-[#1C1C22] border-[#2E2E36] text-[#A1A1AA]'
                  ]"
                >
                  <Camera class="w-6 h-6 stroke-[1.75]" />
                </div>

                <div class="flex items-center space-x-2">
                  <h2
                    class="text-lg font-semibold tracking-tight"
                    :class="store.theme === 'light' ? 'text-zinc-900' : 'text-zinc-100'"
                  >
                    PhotoSelect
                  </h2>
                  <span
                    class="px-1.5 py-0.5 rounded font-mono text-[10px] font-medium border"
                    :class="[
                      store.theme === 'light'
                        ? 'bg-zinc-100 border-zinc-200 text-zinc-600'
                        : 'bg-[#222228] border-white/10 text-zinc-400'
                    ]"
                  >
                    v1.1.0
                  </span>
                </div>

                <p
                  class="text-xs mt-1.5 leading-relaxed text-center font-normal"
                  :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
                >
                  秒级零导入直读原生 RAW+JPG • 毫秒级连拍聚合与峰值对焦
                </p>
              </div>

              <!-- 主操作按钮：精工物理质感按键 -->
              <div class="flex justify-center mb-5">
                <button
                  @click="handleSelectFolder"
                  class="group px-6 py-2.5 rounded-xl font-medium text-xs tracking-wide transition-all duration-150 flex items-center space-x-2.5 cursor-pointer border shadow-sm active:scale-95"
                  :class="[
                    store.theme === 'light'
                      ? 'bg-zinc-900 hover:bg-zinc-800 text-white border-zinc-900 shadow-zinc-200'
                      : 'bg-[#222228] hover:bg-[#2C2C34] text-zinc-100 border-[#32323A] hover:border-zinc-500/50'
                  ]"
                >
                  <FolderOpen class="w-4 h-4 text-amber-400 group-hover:scale-105 transition-transform" />
                  <span>{{ t.topbar.openFolder }}</span>
                  <kbd
                    class="px-1.5 py-0.5 rounded text-[10px] font-mono border"
                    :class="[
                      store.theme === 'light'
                        ? 'bg-zinc-800 border-zinc-700 text-zinc-300'
                        : 'bg-black/40 border-white/10 text-zinc-400'
                    ]"
                  >
                    Ctrl+O
                  </kbd>
                </button>
              </div>

              <!-- 拖拽提示 -->
              <div
                class="text-[11px] flex items-center justify-center space-x-1.5 mb-5 font-normal"
                :class="store.theme === 'light' ? 'text-zinc-400' : 'text-zinc-500'"
              >
                <UploadCloud class="w-3.5 h-3.5" />
                <span>或将 SD 卡、CFexpress 卷或照片文件夹拖放至此</span>
              </div>

              <!-- 扫描错误提示 -->
              <div v-if="store.scanError" class="mb-4 text-xs text-rose-400 bg-rose-500/10 p-2.5 rounded-lg border border-rose-500/20 max-w-md mx-auto break-all">
                {{ store.scanError }}
              </div>

              <!-- 快速检测到的系统卷与存储卡 -->
              <div v-if="store.mountPoints.length > 0" class="w-full pt-4 border-t border-white/5 text-left">
                <div
                  class="text-[11px] font-mono mb-2 flex items-center justify-between"
                  :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
                >
                  <div class="flex items-center space-x-1.5">
                    <HardDrive class="w-3 h-3 text-zinc-400" />
                    <span>存储驱动器 / SD 卡:</span>
                  </div>
                  <span class="text-[10px] text-zinc-500">点击直达</span>
                </div>
                <div class="flex flex-wrap gap-1.5">
                  <button
                    v-for="mount in store.mountPoints"
                    :key="mount"
                    @click="store.loadFolder(mount)"
                    class="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-mono transition-all cursor-pointer"
                    :class="[
                      store.theme === 'light'
                        ? 'bg-zinc-50 hover:bg-zinc-100 border-zinc-200 text-zinc-700'
                        : 'bg-[#1C1C22] hover:bg-[#25252D] border-[#2A2A32] text-zinc-300'
                    ]"
                  >
                    <HardDrive class="w-3 h-3 text-zinc-400" />
                    <span class="font-medium">{{ mount }}</span>
                    <ArrowUpRight class="w-2.5 h-2.5 text-zinc-500" />
                  </button>
                </div>
              </div>

              <!-- 最近工作区快捷入口 -->
              <div v-else-if="store.recentWorkspaces.length > 0" class="w-full pt-4 border-t border-white/5 text-left">
                <div
                  class="text-[11px] font-mono mb-2 flex items-center space-x-1.5"
                  :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
                >
                  <Layers class="w-3 h-3 text-zinc-400" />
                  <span>历史工作区:</span>
                </div>
                <div class="space-y-1">
                  <button
                    v-for="ws in store.recentWorkspaces.slice(0, 3)"
                    :key="ws.id"
                    @click="store.loadFolder(ws.folder_path)"
                    class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg border text-xs font-mono transition-all text-left cursor-pointer"
                    :class="[
                      store.theme === 'light'
                        ? 'bg-zinc-50 hover:bg-zinc-100 border-zinc-200 text-zinc-700'
                        : 'bg-[#1C1C22] hover:bg-[#25252D] border-[#2A2A32] text-zinc-300'
                    ]"
                  >
                    <span class="truncate max-w-[380px]">{{ ws.folder_path }}</span>
                    <ArrowUpRight class="w-3 h-3 text-zinc-500 flex-shrink-0" />
                  </button>
                </div>
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
