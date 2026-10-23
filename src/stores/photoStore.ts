import { defineStore } from "pinia";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type {
  ColorLabel,
  FilterState,
  FlagStatus,
  PhotoGroup,
  ScanResult,
  ViewMode,
  WorkspaceRecord,
  BurstConfig,
  BurstCluster,
  SimilarityConfig,
  SubjectCluster,
} from "../types/photo";
import type { DetectedFace, DetectedEye } from "../types/eye";
import { detectEyesInImage } from "../utils/eyeDetector";

export interface AiSelectionSettings {
  enabled: boolean;
  baseUrl: string;
  model: string;
  hasApiKey: boolean;
}

export interface AiPhotoResult {
  id: string;
  score: number;
  recommendation: "pick" | "review" | "reject";
  reasons: string[];
  suggestedRating: number;
  suggestedFlag: FlagStatus;
  selected: boolean;
  applyRating: boolean;
  applyFlag: boolean;
}

export interface AiBatchProgress {
  completed: number;
  total: number;
  batch: number;
  batches: number;
  results: Omit<AiPhotoResult, "selected" | "applyRating" | "applyFlag">[];
  error: string | null;
  finished: boolean;
}

let detectEyesTimer: ReturnType<typeof setTimeout> | null = null;
let detectRequestId = 0;

export const usePhotoStore = defineStore("photo", {
  state: () => ({
    currentFolderPath: "" as string,
    groups: [] as PhotoGroup[],
    selectedIndex: 0 as number,
    isScanning: false as boolean,
    mountPoints: [] as string[],
    recentWorkspaces: [] as WorkspaceRecord[],
    scanError: null as string | null,

    // 视图模式与显示配置
    viewMode: "loupe" as ViewMode, // loupe | grid | compare
    autoAdvance: true as boolean, // 打星/打标后自动推进到下一张
    rawPreviewMode: (typeof localStorage !== "undefined" && localStorage.getItem("photoselect_raw_preview_mode") === "true") as boolean,
    focusPeaking: false as boolean, // WebGL 峰值对焦开关 [F]
    viewportBg: (typeof localStorage !== "undefined" && localStorage.getItem("photoselect_viewport_bg")) || "#1E1E1E", // #1E1E1E (18%标准中性灰), #000000, #FFFFFF

    // 对比模式与网格配置
    compareIndex: null as number | null, // 第二张对比图的索引 (通过 Ctrl+点击 选择)
    syncCompareZoom: true as boolean, // 对比模式下双图是否联动缩放和平移
    gridItemSize: (typeof localStorage !== "undefined" && parseInt(localStorage.getItem("photoselect_grid_size") || "220", 10)) || 220, // 网格卡片基准尺寸(px)

    // 交叉过滤器状态
    filter: {
      min_rating: 0,
      flags: [] as FlagStatus[],
      colors: [] as ColorLabel[],
      raw_plus_jpg_only: false,
      smart_collection: "all",
    } as FilterState,

    // 界面侧边栏与抽屉
    sidebarCollapsed: false as boolean,
    exportDrawerOpen: false as boolean,
    settingsModalOpen: false as boolean,
    aiSelectionPanelOpen: false as boolean,
    aiSettings: null as AiSelectionSettings | null,
    aiSelectionBusy: false as boolean,
    aiSelectionProgress: { completed: 0, total: 0, batch: 0, batches: 0 },
    aiSelectionResults: [] as AiPhotoResult[],
    aiSelectionError: "" as string,

    // 系统外观模式 (dark / light)
    theme: (typeof localStorage !== "undefined" && (localStorage.getItem("photoselect_theme") as "dark" | "light")) || "dark",

    // 页面与字体缩放级别 (100% / 125% / 150% / 200%)
    uiScale: (typeof localStorage !== "undefined" && parseInt(localStorage.getItem("photoselect_ui_scale") || "100", 10)) || 100,

    // 智能人眼对焦与人脸识别检测 (Narrative Select 级 Eye AF 引擎)
    eyeAfEnabled: (typeof localStorage !== "undefined" && localStorage.getItem("photoselect_eye_af") !== "false"), // 是否开启人眼识别
    eyePanelOpen: (typeof localStorage !== "undefined" && localStorage.getItem("photoselect_eye_panel") !== "false"), // 是否展开人眼特写检视面板
    currentFaces: [] as DetectedFace[], // 当前照片识别到的人脸与双眼实体
    isDetectingEyes: false as boolean,
    eyeDetectionFailed: false as boolean,
    targetLockOnEye: null as { x: number; y: number } | null, // 智能吸附定位目标

    // 毫秒级高速连拍聚合实体状态
    burstClusters: [] as BurstCluster[],
    isBurstClustering: false as boolean,

    // 严格去重与同主体视觉聚类状态
    subjectClusters: [] as SubjectCluster[],
    isAnalyzingSimilarity: false as boolean,

    // 分组折叠选片视图 (Group Stacking / Folding View)
    isGroupFoldedMode: (typeof localStorage !== "undefined" && localStorage.getItem("photoselect_group_folded") === "true") as boolean,
    expandedGroupIds: [] as string[],
  }),

  getters: {
    /// 根据当前所有过滤条件得出的动态展示序列 (秒级响应)
    filteredGroups(state): PhotoGroup[] {
      const baseFiltered = state.groups.filter((g) => {
        // 1. 最低星级过滤
        if (g.rating < state.filter.min_rating) return false;

        // 2. 旗标过滤 (Pick / Reject / Unflagged)
        if (state.filter.flags.length > 0 && !state.filter.flags.includes(g.flag)) {
          return false;
        }

        // 3. 色标分类过滤 (红/黄/绿/蓝/紫)
        if (state.filter.colors.length > 0 && !state.filter.colors.includes(g.color_label)) {
          return false;
        }

        // 4. 仅看 RAW+JPG 智能同名绑定照片
        if (state.filter.raw_plus_jpg_only && (!g.raw_image || !g.std_image)) {
          return false;
        }

        // 5. 智能分类视图快速筛选
        switch (state.filter.smart_collection) {
          case "picked":
            return g.flag === 1;
          case "rejected":
            return g.flag === -1;
          case "unflagged":
            return g.flag === 0;
          case "five_star":
            return g.rating === 5;
          default:
            return true;
        }
      });

      // 如果未开启分组折叠模式，直接平铺展示全量照片
      if (!state.isGroupFoldedMode) {
        return baseFiltered;
      }

      // 分组折叠逻辑: 未展开的组仅展示其 Best Shot 最佳代表帧 (若无可退化为组内第一张)
      const groupRepPhotoIdMap = new Map<string, string>();
      for (const photo of baseFiltered) {
        const gid = photo.burst_group_id || photo.subject_group_id;
        if (!gid) continue;
        if (!groupRepPhotoIdMap.has(gid)) {
          groupRepPhotoIdMap.set(gid, photo.id);
        }
        if (photo.is_burst_best || photo.is_subject_best) {
          groupRepPhotoIdMap.set(gid, photo.id);
        }
      }

      return baseFiltered.filter((photo) => {
        const gid = photo.burst_group_id || photo.subject_group_id;
        if (!gid) return true; // 独立散片正常显示
        // 如果该组已被展开，显示组内全部成员照片
        if (state.expandedGroupIds.includes(gid)) {
          return true;
        }
        // 如果处于折叠叠放状态，仅展示被选为代表帧的照片
        return photo.id === groupRepPhotoIdMap.get(gid);
      });
    },

    /// 聚合组元数据映射表，方便网格卡片即时获取叠放张数与展开状态
    groupInfoMap(state): Map<string, { count: number; bestId: string | null; isExpanded: boolean; type: "burst" | "subject" }> {
      const map = new Map<string, { count: number; bestId: string | null; isExpanded: boolean; type: "burst" | "subject" }>();
      for (const bc of state.burstClusters) {
        map.set(bc.group_id, {
          count: bc.count,
          bestId: bc.best_photo_id,
          isExpanded: state.expandedGroupIds.includes(bc.group_id),
          type: "burst",
        });
      }
      for (const sc of state.subjectClusters) {
        if (!map.has(sc.group_id)) {
          map.set(sc.group_id, {
            count: sc.count,
            bestId: sc.best_photo_id,
            isExpanded: state.expandedGroupIds.includes(sc.group_id),
            type: "subject",
          });
        }
      }
      return map;
    },

    /// 当前选中的高亮照片实体 (在对比模式中作为基准图 A)
    currentPhoto(state): PhotoGroup | null {
      const list = this.filteredGroups;
      if (list.length === 0) return null;
      if (state.selectedIndex < 0) return list[0];
      if (state.selectedIndex >= list.length) return list[list.length - 1];
      return list[state.selectedIndex];
    },

    /// 对比图 A (基准/参考图，对应 selectedIndex)
    comparePhotoA(): PhotoGroup | null {
      return this.currentPhoto;
    },

    /// 对比图 B (候选/对比图，可通过 Ctrl+点击 任意照片设定)
    comparePhotoB(state): PhotoGroup | null {
      const list = this.filteredGroups;
      if (list.length <= 1) return null;
      if (
        state.compareIndex !== null &&
        state.compareIndex >= 0 &&
        state.compareIndex < list.length &&
        state.compareIndex !== state.selectedIndex
      ) {
        return list[state.compareIndex];
      }
      // 默认优雅降级：若未按 Ctrl 指定第二张，默认对比相邻的一张照片
      const defaultIdx = state.selectedIndex > 0 ? state.selectedIndex - 1 : 1;
      return list[defaultIdx] || null;
    },

    /// 智能分类数量徽章统计 (供左侧栏实时显示)
    smartCounts(state): { all: number; picked: number; rejected: number; unflagged: number; five_star: number } {
      let picked = 0;
      let rejected = 0;
      let unflagged = 0;
      let five_star = 0;

      for (const g of state.groups) {
        if (g.flag === 1) picked++;
        else if (g.flag === -1) rejected++;
        else unflagged++;

        if (g.rating === 5) five_star++;
      }

      return {
        all: state.groups.length,
        picked,
        rejected,
        unflagged,
        five_star,
      };
    },

    /// 导出统计摘要
    exportCounts(state): { total: number; picked: number; filtered: number } {
      return {
        total: state.groups.length,
        picked: state.groups.filter((g) => g.flag === 1).length,
        filtered: this.filteredGroups.length,
      };
    },
  },

  actions: {
    async loadAiSettings() {
      this.aiSettings = await invoke<AiSelectionSettings>("get_ai_selection_settings");
    },

    async saveAiSettings(input: { enabled: boolean; baseUrl: string; model: string; apiKey: string; clearApiKey: boolean }) {
      this.aiSettings = await invoke<AiSelectionSettings>("save_ai_selection_settings", {
        enabled: input.enabled, baseUrl: input.baseUrl, model: input.model,
        apiKey: input.apiKey || null, clearApiKey: input.clearApiKey,
      });
    },

    async testAiConnection() {
      return await invoke<string>("test_ai_connection");
    },

    async startAiSelection(photos: PhotoGroup[]) {
      if (photos.length === 0) throw new Error("至少需要一张照片");
      if (photos.length > 500) throw new Error("每次最多分析 500 张，请分批选择");
      this.aiSelectionResults = [];
      this.aiSelectionProgress = { completed: 0, total: photos.length, batch: 0, batches: Math.ceil(photos.length / 8) };
      this.aiSelectionError = "";
      this.aiSelectionBusy = true;
      try {
        await invoke("start_ai_selection", { photos });
      } catch (error) {
        this.aiSelectionError = String(error);
        throw error;
      } finally {
        this.aiSelectionBusy = false;
      }
    },

    consumeAiProgress(progress: AiBatchProgress) {
      this.aiSelectionProgress = { completed: progress.completed, total: progress.total, batch: progress.batch, batches: progress.batches };
      if (progress.error) this.aiSelectionError = progress.error;
      if (progress.results.length) this.aiSelectionResults.push(...progress.results.map((result) => ({
        ...result, selected: false, applyRating: false, applyFlag: false,
      })));
      if (progress.finished) this.aiSelectionBusy = false;
    },

    async applyAiResults() {
      const selected = this.aiSelectionResults.filter((result) => result.selected && (result.applyRating || result.applyFlag));
      if (!selected.length) return;
      const records: [string, string, string, number, FlagStatus, string][] = [];
      const updates: { photo: PhotoGroup; rating: number; flag: FlagStatus }[] = [];
      for (const result of selected) {
        const photo = this.groups.find((group) => group.id === result.id);
        if (!photo) continue;
        const rating = result.applyRating ? result.suggestedRating : photo.rating;
        const flag = result.applyFlag ? result.suggestedFlag : photo.flag;
        const signature = photo.folder_path.replace(/\\/g, "/") + ":" + photo.base_name.toUpperCase();
        records.push([signature, photo.base_name, photo.folder_path, rating, flag, photo.color_label]);
        updates.push({ photo, rating, flag });
      }
      if (!records.length) return;
      await invoke("batch_update_photo_status", { records });
      for (const { photo, rating, flag } of updates) { photo.rating = rating; photo.flag = flag; }
      for (const result of selected) result.selected = false;
    },

    /// 触发原生多线程目录扫描 (调用 Rust 后端 scan_directory 命令)
    async loadFolder(folderPath: string, recursive = false) {
      if (!folderPath) return;
      this.isScanning = true;
      this.scanError = null;
      try {
        const result = await invoke<ScanResult>("scan_directory", {
          path: folderPath,
          recursive,
        });

        this.currentFolderPath = result.folder_path;
        this.groups = result.groups;
        this.burstClusters = [];
        this.subjectClusters = [];
        this.expandedGroupIds = [];
        this.selectedIndex = 0;
        await this.loadRecentWorkspaces();
      } catch (err) {
        console.error("加载目录失败:", err);
        this.scanError = String(err);
        throw err;
      } finally {
        this.isScanning = false;
      }
    },

    /// 切换选中照片索引 (支持 isCtrlOrMeta 区分普通选中与按 Ctrl 选第二张对比)
    selectIndex(index: number, isCtrlOrMeta = false) {
      const max = this.filteredGroups.length - 1;
      if (max < 0) {
        this.selectedIndex = 0;
        this.compareIndex = null;
        return;
      }
      const target = Math.max(0, Math.min(index, max));

      if (isCtrlOrMeta) {
        // 按住 Ctrl/Cmd 点击：选定该张作为第二张对比照片，并自动进入对比视图
        if (target !== this.selectedIndex) {
          this.compareIndex = target;
          this.viewMode = "compare";
        }
      } else {
        // 普通点击：切为主选当前照片
        this.selectedIndex = target;
        // 如果当前照片刚好和已选对比图重合，清除对比下标
        if (this.compareIndex === target) {
          this.compareIndex = null;
        }
      }
    },

    /// 设置第二张对比照片索引
    setCompareIndex(index: number | null) {
      this.compareIndex = index;
    },

    /// 互换对比模式下的两张照片位置 (S 快捷键)
    swapComparePhotos() {
      const list = this.filteredGroups;
      if (list.length <= 1) return;
      const curBIdx =
        this.compareIndex !== null && this.compareIndex !== this.selectedIndex
          ? this.compareIndex
          : this.selectedIndex > 0
          ? this.selectedIndex - 1
          : 1;

      const temp = this.selectedIndex;
      this.selectedIndex = curBIdx;
      this.compareIndex = temp;
    },

    /// 对比模式下候选图前进 (保持基准图 A 不变)
    nextCompareCandidate() {
      const list = this.filteredGroups;
      if (list.length <= 1) return;
      const currentB =
        this.compareIndex !== null
          ? this.compareIndex
          : this.selectedIndex > 0
          ? this.selectedIndex - 1
          : 1;
      let next = currentB + 1;
      if (next === this.selectedIndex) next++;
      if (next < list.length) {
        this.compareIndex = next;
      }
    },

    /// 对比模式下候选图后退 (保持基准图 A 不变)
    prevCompareCandidate() {
      const list = this.filteredGroups;
      if (list.length <= 1) return;
      const currentB =
        this.compareIndex !== null
          ? this.compareIndex
          : this.selectedIndex > 0
          ? this.selectedIndex - 1
          : 1;
      let prev = currentB - 1;
      if (prev === this.selectedIndex) prev--;
      if (prev >= 0) {
        this.compareIndex = prev;
      }
    },

    /// 更新网格缩略图尺寸 (并持久化至本地)
    setGridItemSize(size: number) {
      const clamped = Math.max(110, Math.min(480, size));
      this.gridItemSize = clamped;
      try {
        localStorage.setItem("photoselect_grid_size", String(clamped));
      } catch {
        // ignore
      }
    },

    /// 下一张照片
    nextPhoto() {
      const max = this.filteredGroups.length - 1;
      if (this.selectedIndex < max) {
        this.selectedIndex++;
      }
    },

    /// 上一张照片
    prevPhoto() {
      if (this.selectedIndex > 0) {
        this.selectedIndex--;
      }
    },

    /// 针对特定照片实体设置星级评分 (0-5) 并持久化至本地 SQLite
    async setRatingForPhoto(photo: PhotoGroup | null, rating: number) {
      if (!photo) return;
      photo.rating = rating;

      const sig = `${photo.folder_path.replace(/\\/g, "/")}:${photo.base_name.toUpperCase()}`;
      try {
        await invoke("update_photo_rating", {
          signature: sig,
          baseName: photo.base_name,
          folderPath: photo.folder_path,
          rating,
        });
      } catch (err) {
        console.error("更新星级持久化失败:", err);
      }
    },

    /// 设置当前照片星级评分 (0-5)
    async setRating(rating: number) {
      await this.setRatingForPhoto(this.currentPhoto, rating);
      if (this.autoAdvance && this.viewMode === "loupe") {
        this.nextPhoto();
      }
    },

    /// 针对特定照片实体设置旗标状态 (Pick: 1, Reject: -1, Unflag: 0) 并持久化
    async setFlagForPhoto(photo: PhotoGroup | null, flag: FlagStatus) {
      if (!photo) return;
      const targetFlag: FlagStatus = photo.flag === flag ? 0 : flag;
      photo.flag = targetFlag;

      const sig = `${photo.folder_path.replace(/\\/g, "/")}:${photo.base_name.toUpperCase()}`;
      try {
        await invoke("update_photo_flag", {
          signature: sig,
          baseName: photo.base_name,
          folderPath: photo.folder_path,
          flag: targetFlag,
        });
      } catch (err) {
        console.error("更新旗标持久化失败:", err);
      }
    },

    /// 设置当前照片旗标状态
    async setFlag(flag: FlagStatus) {
      await this.setFlagForPhoto(this.currentPhoto, flag);
      if (this.autoAdvance && this.viewMode === "loupe") {
        this.nextPhoto();
      }
    },

    /// 针对特定照片实体设置色标并持久化
    async setColorLabelForPhoto(photo: PhotoGroup | null, color: ColorLabel) {
      if (!photo) return;
      const targetColor: ColorLabel = photo.color_label === color ? "none" : color;
      photo.color_label = targetColor;

      const sig = `${photo.folder_path.replace(/\\/g, "/")}:${photo.base_name.toUpperCase()}`;
      try {
        await invoke("update_photo_color_label", {
          signature: sig,
          baseName: photo.base_name,
          folderPath: photo.folder_path,
          colorLabel: targetColor,
        });
      } catch (err) {
        console.error("更新色标持久化失败:", err);
      }
    },

    /// 设置当前照片色彩标签
    async setColorLabel(color: ColorLabel) {
      await this.setColorLabelForPhoto(this.currentPhoto, color);
    },

    /// 获取系统物理磁盘与 SD 卡读卡器挂载点
    async loadMountPoints() {
      try {
        this.mountPoints = await invoke<string[]>("check_mount_points");
      } catch (err) {
        console.error("获取挂载设备失败:", err);
      }
    },

    /// 获取历史工作区记录
    async loadRecentWorkspaces() {
      try {
        this.recentWorkspaces = await invoke<WorkspaceRecord[]>("get_recent_workspaces");
      } catch (err) {
        console.error("获取历史工作区失败:", err);
      }
    },

    /// 生成 photoselect:// 零拷贝原生二进制图片流 URL
    getPreviewUrl(photo: PhotoGroup | null): string {
      if (!photo) return "";
      const path = (this.rawPreviewMode && photo.raw_image) || photo.std_image || photo.raw_image;
      if (!path) return "";
      try {
        if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__?.convertFileSrc) {
          return convertFileSrc(path, "photoselect");
        }
      } catch (e) {
        console.warn("convertFileSrc failed, falling back:", e);
      }
      // 优雅降级回退 (Windows 下 WebView2 映射到 http://photoselect.localhost/)
      const isWindows = typeof navigator !== "undefined" && navigator.userAgent.toLowerCase().includes("win");
      const encoded = encodeURIComponent(path);
      return isWindows
        ? `http://photoselect.localhost/${encoded}`
        : `photoselect://localhost/${encoded}`;
    },

    setRawPreviewMode(enabled: boolean) {
      this.rawPreviewMode = enabled;
      try {
        localStorage.setItem("photoselect_raw_preview_mode", enabled ? "true" : "false");
      } catch {
        // ignore
      }
    },

    /// 生成轻量级高速缩略图 URL (胶片条与网格卡片专用，避免解压 30MB 原图导致卡顿)
    getThumbnailUrl(photo: PhotoGroup | null): string {
      if (!photo) return "";
      const path = photo.std_image || photo.raw_image;
      if (!path) return "";
      try {
        if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__?.convertFileSrc) {
          const base = convertFileSrc(path, "photoselect");
          return `${base}?type=thumb`;
        }
      } catch (e) {
        // fallback
      }
      const isWindows = typeof navigator !== "undefined" && navigator.userAgent.toLowerCase().includes("win");
      const encoded = encodeURIComponent(path);
      return isWindows
        ? `http://photoselect.localhost/${encoded}?type=thumb`
        : `photoselect://localhost/${encoded}?type=thumb`;
    },

    /// 切换系统主题 (dark / light) 并持久化
    setTheme(newTheme: "dark" | "light") {
      this.theme = newTheme;
      try {
        localStorage.setItem("photoselect_theme", newTheme);
      } catch (e) {
        // ignore
      }
      if (typeof document !== "undefined") {
        if (newTheme === "light") {
          document.documentElement.classList.add("theme-light");
        } else {
          document.documentElement.classList.remove("theme-light");
        }
      }

      // 切换模式时自动同步视口基准中性底色 (避免在白昼模式下呈现黑洞大视口)
      if (newTheme === "light" && (this.viewportBg === "#1E1E1E" || !this.viewportBg)) {
        this.setViewportBg("#E5E5EA");
      } else if (newTheme === "dark" && this.viewportBg === "#E5E5EA") {
        this.setViewportBg("#1E1E1E");
      }
    },

    /// 初始化系统主题
    initTheme() {
      if (typeof document !== "undefined") {
        if (this.theme === "light") {
          document.documentElement.classList.add("theme-light");
          if (this.viewportBg === "#1E1E1E" || !this.viewportBg) {
            this.viewportBg = "#E5E5EA";
          }
        } else {
          document.documentElement.classList.remove("theme-light");
          if (this.viewportBg === "#E5E5EA") {
            this.viewportBg = "#1E1E1E";
          }
        }
      }
    },

    /// 设置单图视口基准底色并持久化
    setViewportBg(bg: string) {
      this.viewportBg = bg;
      try {
        localStorage.setItem("photoselect_viewport_bg", bg);
      } catch (e) {
        // ignore
      }
    },

    /// 页面与字体缩放级别设定 (100 | 125 | 150 | 200) 并持久化
    setUiScale(scale: number) {
      const allowedScales = [100, 125, 150, 200];
      const target = allowedScales.includes(scale) ? scale : 100;
      this.uiScale = target;
      try {
        localStorage.setItem("photoselect_ui_scale", String(target));
      } catch (e) {
        // ignore
      }
      this.applyUiScale(target);
    },

    /// 将缩放比例应用到视图 (支持 Tauri WebView2 原生 setZoom 与 CSS zoom 双模自适应)
    async applyUiScale(scale: number) {
      const factor = scale / 100;
      let nativeApplied = false;
      try {
        const webview = getCurrentWebview();
        if (webview && typeof webview.setZoom === "function") {
          await webview.setZoom(factor);
          nativeApplied = true;
        }
      } catch (e) {
        // 在非 Tauri Webview 环境静默回退
      }

      if (typeof document !== "undefined") {
        if (!nativeApplied) {
          (document.documentElement.style as any).zoom = `${factor}`;
        } else {
          // 原生 webview 已缩放，重置 CSS zoom 避免二次放大
          (document.documentElement.style as any).zoom = "1";
        }
        document.documentElement.style.setProperty("--app-ui-scale", `${factor}`);
        document.documentElement.setAttribute("data-ui-scale", `${scale}`);

        // 关键：通知所有 Canvas / WebGL 视口与虚拟列表重算尺寸
        window.dispatchEvent(new Event("resize"));
        setTimeout(() => {
          window.dispatchEvent(new Event("resize"));
        }, 50);
      }
    },

    /// 初始化 UI 缩放比例
    initUiScale() {
      this.applyUiScale(this.uiScale);
    },

    /// 快捷键步进放大 (100% -> 125% -> 150% -> 200%)
    zoomInUiScale() {
      const scales = [100, 125, 150, 200];
      const currentIdx = scales.indexOf(this.uiScale);
      if (currentIdx < scales.length - 1) {
        this.setUiScale(scales[currentIdx + 1]);
      }
    },

    /// 快捷键步进缩小 (200% -> 150% -> 125% -> 100%)
    zoomOutUiScale() {
      const scales = [100, 125, 150, 200];
      const currentIdx = scales.indexOf(this.uiScale);
      if (currentIdx > 0) {
        this.setUiScale(scales[currentIdx - 1]);
      }
    },

    /// 快捷键重置为 100%
    resetUiScale() {
      this.setUiScale(100);
    },

    /// 切换人眼对焦识别开关并持久化
    toggleEyeAf() {
      this.eyeAfEnabled = !this.eyeAfEnabled;
      try {
        localStorage.setItem("photoselect_eye_af", this.eyeAfEnabled ? "true" : "false");
      } catch (e) {
        // ignore
      }
      this.detectEyesForPhoto(this.eyeAfEnabled ? this.currentPhoto : null);
    },

    /// 切换人眼特写检查面板展开/收起
    toggleEyePanel() {
      this.eyePanelOpen = !this.eyePanelOpen;
      try {
        localStorage.setItem("photoselect_eye_panel", this.eyePanelOpen ? "true" : "false");
      } catch (e) {
        // ignore
      }
    },

    /// 针对指定照片异步执行人眼识别与对焦计算 (附带防抖，快速切图不卡顿)
    detectEyesForPhoto(photo: PhotoGroup | null) {
      const requestId = ++detectRequestId;
      if (detectEyesTimer) {
        clearTimeout(detectEyesTimer);
        detectEyesTimer = null;
      }
      if (!photo || !this.eyeAfEnabled) {
        this.currentFaces = [];
        this.isDetectingEyes = false;
        this.eyeDetectionFailed = false;
        return;
      }
      // Never keep the previous photo's boxes visible while a new image is pending.
      this.currentFaces = [];
      this.isDetectingEyes = false;
      this.eyeDetectionFailed = false;
      this.targetLockOnEye = null;

      detectEyesTimer = setTimeout(async () => {
        if (requestId !== detectRequestId || !this.eyeAfEnabled) return;
        const previewUrl = this.getPreviewUrl(photo);
        if (!previewUrl) {
          if (requestId === detectRequestId) this.eyeDetectionFailed = true;
          return;
        }

        this.isDetectingEyes = true;
        try {
          const img = new Image();
          img.crossOrigin = "anonymous";
          await new Promise<void>((resolve, reject) => {
            img.onload = () => resolve();
            img.onerror = (e) => reject(e);
            img.src = previewUrl;
          });
          if (requestId !== detectRequestId || !this.eyeAfEnabled) return;

          const sig = `${photo.folder_path}/${photo.base_name}`;
          const result = await detectEyesInImage(
            img,
            sig,
            () => requestId === detectRequestId && this.eyeAfEnabled && this.currentPhoto?.id === photo.id,
          );

          // 竞态锁：确保当前仍停留在同一张照片
          if (requestId === detectRequestId && this.currentPhoto?.id === photo.id) {
            this.currentFaces = result.faces;
          }
        } catch (err) {
          if (requestId === detectRequestId && (err as DOMException)?.name !== "AbortError") {
            console.warn("人眼自动识别失败:", err);
            this.eyeDetectionFailed = true;
          }
        } finally {
          if (requestId === detectRequestId) this.isDetectingEyes = false;
        }
      }, 150);
    },

    /// 触发人眼吸附 Smart Lock-On 居中放大
    lockOnEye(eye: DetectedEye | null) {
      if (!eye) return;
      this.targetLockOnEye = { x: eye.x, y: eye.y };
    },

    /// 触发高速连拍动作聚合识别 (毫秒级与文件名序列混合判定，标定 Best Shot)
    async clusterBursts(config?: BurstConfig) {
      if (!this.groups.length) return [];
      this.isBurstClustering = true;
      try {
        const [updatedPhotos, clusters] = await invoke<[PhotoGroup[], BurstCluster[]]>(
          "cluster_burst_photos",
          {
            photos: this.groups,
            config: config || { threshold_ms: 500, min_burst_count: 2, auto_flag_best: false },
          }
        );
        this.groups = updatedPhotos;
        this.burstClusters = clusters;
        return clusters;
      } catch (err) {
        console.error("连拍聚合识别失败:", err);
        throw err;
      } finally {
        this.isBurstClustering = false;
      }
    },

    /// 为指定单张照片在其原图同目录下写入/更新行业标准 XMP 侧车文件 (含星级、旗标、色彩标签及生态关键字)
    async writePhotoXmp(photo?: PhotoGroup | null, extraKeywords?: string[]): Promise<string | null> {
      const target = photo || this.currentPhoto;
      if (!target) return null;
      try {
        const xmpPath = await invoke<string>("write_photo_xmp", {
          photo: target,
          extraKeywords: extraKeywords || null,
        });
        target.xmp_sidecar = xmpPath;
        return xmpPath;
      } catch (err) {
        console.error("写入 XMP 侧车文件失败:", err);
        throw err;
      }
    },

    /// 为指定照片更新标签/关键字 (持久化至 SQLite)
    async updatePhotoTags(photo: PhotoGroup | null, tags: string[]) {
      if (!photo) return;
      photo.tags = [...tags];
      const sig = `${photo.folder_path.replace(/\\/g, "/")}:${photo.base_name.toUpperCase()}`;
      try {
        await invoke("update_photo_tags", {
          signature: sig,
          baseName: photo.base_name,
          folderPath: photo.folder_path,
          tags,
        });
      } catch (err) {
        console.error("更新照片标签持久化失败:", err);
      }
    },

    /// 为指定照片添加标签 (若不存在)
    async addPhotoTag(photo: PhotoGroup | null, tag: string) {
      if (!photo || !tag.trim()) return;
      const cleanTag = tag.trim();
      const currentTags = photo.tags || [];
      if (!currentTags.includes(cleanTag)) {
        await this.updatePhotoTags(photo, [...currentTags, cleanTag]);
      }
    },

    /// 为指定照片移除标签
    async removePhotoTag(photo: PhotoGroup | null, tag: string) {
      if (!photo) return;
      const currentTags = photo.tags || [];
      const filtered = currentTags.filter((t) => t !== tag);
      await this.updatePhotoTags(photo, filtered);
    },

    /// 切换「分组折叠选片」视图模式
    toggleGroupFoldedMode(enabled?: boolean) {
      const target = typeof enabled === "boolean" ? enabled : !this.isGroupFoldedMode;
      this.isGroupFoldedMode = target;
      try {
        localStorage.setItem("photoselect_group_folded", target ? "true" : "false");
      } catch {}
    },

    /// 展开或折叠单个连拍/同主体动作组
    toggleGroupExpanded(groupId: string) {
      const idx = this.expandedGroupIds.indexOf(groupId);
      if (idx >= 0) {
        this.expandedGroupIds.splice(idx, 1);
      } else {
        this.expandedGroupIds.push(groupId);
      }
    },

    /// 展开全部组
    expandAllGroups() {
      const allGroupIds = new Set<string>();
      for (const g of this.groups) {
        const gid = g.burst_group_id || g.subject_group_id;
        if (gid) allGroupIds.add(gid);
      }
      this.expandedGroupIds = Array.from(allGroupIds);
    },

    /// 折叠全部组
    collapseAllGroups() {
      this.expandedGroupIds = [];
    },

    /// 触发严格去重与同主体视觉聚类分析 (基于 dHash 算法与滑动时序窗口)
    async detectSimilarityAndDuplicates(config?: SimilarityConfig) {
      if (!this.groups.length) return [];
      this.isAnalyzingSimilarity = true;
      try {
        const [updatedPhotos, clusters] = await invoke<[PhotoGroup[], SubjectCluster[]]>(
          "detect_similarity_and_duplicates",
          {
            photos: this.groups,
            config: config || {
              dedup_threshold: 3,
              subject_threshold: 12,
              time_window_secs: 60,
              min_group_count: 2,
              auto_reject_duplicates: false,
              auto_flag_best: false,
            },
          }
        );
        this.groups = updatedPhotos;
        this.subjectClusters = clusters;
        return clusters;
      } catch (err) {
        console.error("相似度分析与去重失败:", err);
        throw err;
      } finally {
        this.isAnalyzingSimilarity = false;
      }
    },

    /// 核心暗房高能动作：保留组内最佳帧 (Pick: 1)，并自动将组内其余所有候选过渡帧标记为淘汰 (Reject: -1)
    async pickBestAndRejectRest(groupId: string) {
      const memberPhotos = this.groups.filter(
        (g) => g.burst_group_id === groupId || g.subject_group_id === groupId
      );
      if (memberPhotos.length === 0) return;

      const bestPhoto =
        memberPhotos.find((p) => p.is_burst_best || p.is_subject_best) || memberPhotos[0];

      const records: [string, string, string, number, FlagStatus, ColorLabel][] = [];
      for (const p of memberPhotos) {
        const flag: FlagStatus = p.id === bestPhoto.id ? 1 : -1;
        p.flag = flag;
        const sig = `${p.folder_path.replace(/\\/g, "/")}:${p.base_name.toUpperCase()}`;
        records.push([sig, p.base_name, p.folder_path, p.rating, flag, p.color_label]);
      }

      try {
        await invoke("batch_update_photo_status", { records });
      } catch (err) {
        console.error("批量更新组内打标失败:", err);
      }
    },
  },
});
