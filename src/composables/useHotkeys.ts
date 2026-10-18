import { onMounted, onUnmounted } from "vue";
import { usePhotoStore } from "../stores/photoStore";
import type { ColorLabel, FlagStatus } from "../types/photo";

/**
 * 全局单手快捷键状态机 (Keyboard-First UX)
 * 专为专业摄影暗房选片设计，支持全键盘盲操与连续翻页
 */
export function useHotkeys() {
  const store = usePhotoStore();

  const handleKeyDown = (e: KeyboardEvent) => {
    // 若焦点位于文本输入框内，暂停全局选片快捷键
    const activeEl = document.activeElement;
    if (
      activeEl &&
      (activeEl.tagName === "INPUT" ||
        activeEl.tagName === "TEXTAREA" ||
        (activeEl as HTMLElement).isContentEditable)
    ) {
      return;
    }

    const key = e.key.toLowerCase();

    // 1. 快捷导出抽屉呼出: Ctrl/Cmd + Shift + E
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && key === "e") {
      e.preventDefault();
      store.exportDrawerOpen = !store.exportDrawerOpen;
      return;
    }

    // 2. 翻页导航: 方向键 ← / → 或 A / D (在对比模式下切换候选图 B)
    if (e.key === "ArrowRight" || key === "d") {
      e.preventDefault();
      if (store.viewMode === "compare") {
        store.nextCompareCandidate();
      } else {
        store.nextPhoto();
      }
      return;
    }
    if (e.key === "ArrowLeft" || key === "a") {
      e.preventDefault();
      if (store.viewMode === "compare") {
        store.prevCompareCandidate();
      } else {
        store.prevPhoto();
      }
      return;
    }

    // 2.1 对比模式互换位置快捷键: S
    if (key === "s" && store.viewMode === "compare") {
      e.preventDefault();
      store.swapComparePhotos();
      return;
    }

    // 3. 旗标标记: P (留用) / X (淘汰) / U (取消旗标)
    if (key === "p") {
      e.preventDefault();
      store.setFlag(1 as FlagStatus);
      return;
    }
    if (key === "x") {
      e.preventDefault();
      store.setFlag(-1 as FlagStatus);
      return;
    }
    if (key === "u") {
      e.preventDefault();
      store.setFlag(0 as FlagStatus);
      return;
    }

    // 4. 星级评定: 数字键 0 - 5
    if (["0", "1", "2", "3", "4", "5"].includes(e.key)) {
      e.preventDefault();
      store.setRating(parseInt(e.key, 10));
      return;
    }

    // 5. 色彩标签: 数字键 6 (红) / 7 (黄) / 8 (绿) / 9 (蓝)
    const colorMap: Record<string, ColorLabel> = {
      "6": "red",
      "7": "yellow",
      "8": "green",
      "9": "blue",
    };
    if (colorMap[e.key]) {
      e.preventDefault();
      store.setColorLabel(colorMap[e.key]);
      return;
    }

    // 6. 视图模式切换: E (单图 Loupe) / G (网格 Grid) / C (对比 Compare)
    if (key === "e") {
      e.preventDefault();
      store.viewMode = "loupe";
      return;
    }
    if (key === "g") {
      e.preventDefault();
      store.viewMode = "grid";
      return;
    }
    if (key === "c") {
      e.preventDefault();
      store.viewMode = "compare";
      return;
    }

    // 7. WebGL 峰值对焦高亮开关: F
    if (key === "f") {
      e.preventDefault();
      store.focusPeaking = !store.focusPeaking;
      return;
    }

    // 8. 视口背景中性灰切换: B (可在 标准中性灰 / 纯黑 #000000 / 纯白 #FFFFFF 循环切换)
    if (key === "b") {
      e.preventDefault();
      if (store.viewportBg === "#1E1E1E" || store.viewportBg === "#E5E5EA") {
        store.setViewportBg("#000000");
      } else if (store.viewportBg === "#000000") {
        store.setViewportBg("#FFFFFF");
      } else {
        store.setViewportBg(store.theme === "light" ? "#E5E5EA" : "#1E1E1E");
      }
      return;
    }

    // 9. 折叠/展开侧边栏: Tab 键
    if (e.key === "Tab") {
      e.preventDefault();
      store.sidebarCollapsed = !store.sidebarCollapsed;
      return;
    }

    // 10. AI 人眼对焦识别开关: O
    if (key === "o") {
      e.preventDefault();
      store.toggleEyeAf();
      return;
    }

    // 11. 智能人眼吸附放大对焦: Z
    if (key === "z") {
      e.preventDefault();
      if (store.currentFaces.length > 0) {
        const face = store.currentFaces[0];
        const targetEye = face.primaryEye || face.leftEye || face.rightEye;
        if (targetEye) {
          store.lockOnEye(targetEye);
        }
      }
      return;
    }
  };

  onMounted(() => {
    window.addEventListener("keydown", handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeyDown);
  });
}
