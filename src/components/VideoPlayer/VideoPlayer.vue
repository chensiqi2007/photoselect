<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import {
  Play,
  Pause,
  RotateCcw,
  RotateCw,
  Volume2,
  Volume1,
  VolumeX,
  Maximize,
  Minimize,
  Repeat,
  Check,
  X,
  Star,
  Film,
  Loader2,
  AlertTriangle,
  ExternalLink,
} from "lucide-vue-next";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { usePhotoStore } from "../../stores/photoStore";
import type { FlagStatus } from "../../types/photo";

const store = usePhotoStore();
const currentPhoto = computed(() => store.currentPhoto);
const videoSrc = computed(() => store.getPreviewUrl(currentPhoto.value));

const videoRef = ref<HTMLVideoElement | null>(null);
const containerRef = ref<HTMLElement | null>(null);

// 播放控制状态
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const bufferedEnd = ref(0);
const volume = ref(1.0);
const isMuted = ref(false);
const playbackRate = ref(1.0);
const isLooping = ref(false);
const isFullscreen = ref(false);
const isBuffering = ref(false);
const playbackError = ref<string | null>(null);
const showCenterFeedback = ref<"play" | "pause" | null>(null);
const centerFeedbackTimer = ref<number | null>(null);

// HUD 控制栏自动隐现 (鼠标静止 2.5s 后渐隐)
const showControls = ref(true);
let hideControlsTimer: number | null = null;

const resetControlsTimer = () => {
  showControls.value = true;
  if (hideControlsTimer) clearTimeout(hideControlsTimer);
  hideControlsTimer = window.setTimeout(() => {
    if (isPlaying.value) {
      showControls.value = false;
    }
  }, 2500);
};

// 格式化时间 mm:ss 或 hh:mm:ss
const formatTime = (secs: number): string => {
  if (isNaN(secs) || secs < 0) return "00:00";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  const mStr = m.toString().padStart(2, "0");
  const sStr = s.toString().padStart(2, "0");
  if (h > 0) {
    return `${h.toString().padStart(2, "0")}:${mStr}:${sStr}`;
  }
  return `${mStr}:${sStr}`;
};

// 播放与暂停切换
const togglePlay = () => {
  const el = videoRef.value;
  if (!el) return;
  if (el.paused || el.ended) {
    el.play().then(() => {
      isPlaying.value = true;
      triggerFeedback("play");
    }).catch((err) => {
      console.warn("自动播放失败:", err);
    });
  } else {
    el.pause();
    isPlaying.value = false;
    triggerFeedback("pause");
  }
};

const triggerFeedback = (type: "play" | "pause") => {
  showCenterFeedback.value = type;
  if (centerFeedbackTimer.value) clearTimeout(centerFeedbackTimer.value);
  centerFeedbackTimer.value = window.setTimeout(() => {
    showCenterFeedback.value = null;
  }, 500);
};

// 快进与倒退 (步长 5s)
const seekRelative = (offsetSecs: number) => {
  const el = videoRef.value;
  if (!el) return;
  el.currentTime = Math.max(0, Math.min(el.duration || 0, el.currentTime + offsetSecs));
  currentTime.value = el.currentTime;
};

// 进度条拖拽与跳转
const isSeeking = ref(false);
const handleSeekInput = (e: Event) => {
  const val = parseFloat((e.target as HTMLInputElement).value);
  currentTime.value = val;
  if (videoRef.value) {
    videoRef.value.currentTime = val;
  }
};

// 倍速切换循环: 0.5x -> 1.0x -> 1.25x -> 1.5x -> 2.0x -> 0.5x
const cyclePlaybackRate = () => {
  const rates = [0.5, 1.0, 1.25, 1.5, 2.0];
  const curIdx = rates.indexOf(playbackRate.value);
  const nextRate = rates[(curIdx + 1) % rates.length];
  playbackRate.value = nextRate;
  if (videoRef.value) {
    videoRef.value.playbackRate = nextRate;
  }
};

// 静音切换
const toggleMute = () => {
  isMuted.value = !isMuted.value;
  if (videoRef.value) {
    videoRef.value.muted = isMuted.value;
  }
};

// 音量滑动调节
const handleVolumeChange = (e: Event) => {
  const val = parseFloat((e.target as HTMLInputElement).value);
  volume.value = val;
  isMuted.value = val === 0;
  if (videoRef.value) {
    videoRef.value.volume = val;
    videoRef.value.muted = val === 0;
  }
  try {
    localStorage.setItem("photoselect_video_volume", val.toString());
  } catch {}
};

// 循环模式切换
const toggleLoop = () => {
  isLooping.value = !isLooping.value;
  if (videoRef.value) {
    videoRef.value.loop = isLooping.value;
  }
};

// 全屏切换
const toggleFullscreen = async () => {
  if (!containerRef.value) return;
  if (!document.fullscreenElement) {
    try {
      await containerRef.value.requestFullscreen();
      isFullscreen.value = true;
    } catch (err) {
      console.warn("全屏模式不可用:", err);
    }
  } else {
    try {
      await document.exitFullscreen();
      isFullscreen.value = false;
    } catch (err) {
      console.warn("退出全屏失败:", err);
    }
  }
};

// 监听切换视频文件时重置播放器
watch(
  () => videoSrc.value,
  () => {
    currentTime.value = 0;
    duration.value = 0;
    bufferedEnd.value = 0;
    isPlaying.value = false;
    isBuffering.value = true;
    playbackError.value = null;
    showControls.value = true;
    if (videoRef.value) {
      videoRef.value.pause();
      videoRef.value.currentTime = 0;
      videoRef.value.load();
      // 切换视频时自动播放第一段
      videoRef.value.play().then(() => {
        isPlaying.value = true;
        isBuffering.value = false;
      }).catch(() => {
        isPlaying.value = false;
        isBuffering.value = false;
      });
    }
  }
);

// 原生 Video 事件钩子
const onLoadedMetadata = () => {
  const el = videoRef.value;
  if (!el) return;
  duration.value = Number.isFinite(el.duration) ? el.duration : 0;
  el.playbackRate = playbackRate.value;
  el.loop = isLooping.value;
  el.volume = volume.value;
  el.muted = isMuted.value;
  isBuffering.value = false;
  playbackError.value = null;
};

const onCanPlay = () => {
  isBuffering.value = false;
  playbackError.value = null;
};

const onVideoError = () => {
  isBuffering.value = false;
  isPlaying.value = false;
  const err = videoRef.value?.error;
  if (err?.code === 4) {
    playbackError.value = "当前系统解码器不支持该视频编码格式（如 10-bit 4:2:2 / ProRes / MTS）";
  } else if (err?.code === 2 || err?.code === 3) {
    playbackError.value = "视频流读取或解码失败，请检查文件是否完整";
  } else {
    playbackError.value = "无法在内置播放器中加载此视频";
  }
};

const handleOpenInSystemPlayer = async () => {
  const filePath = currentPhoto.value?.video_file;
  if (!filePath) return;
  try {
    await openExternal(filePath);
  } catch (err) {
    console.warn("调用系统播放器失败:", err);
  }
};

const onTimeUpdate = () => {
  const el = videoRef.value;
  if (!el || isSeeking.value) return;
  currentTime.value = el.currentTime;
  if (el.buffered.length > 0) {
    bufferedEnd.value = el.buffered.end(el.buffered.length - 1);
  }
};

const onPlay = () => {
  isPlaying.value = true;
  isBuffering.value = false;
  resetControlsTimer();
};

const onPause = () => {
  isPlaying.value = false;
  showControls.value = true;
};

const onWaiting = () => {
  isBuffering.value = true;
};

const onPlaying = () => {
  isBuffering.value = false;
};

const onEnded = () => {
  isPlaying.value = false;
  showControls.value = true;
};

// 键盘事件处理 (Space 播放/暂停, J/L 快进后退, M 静音)
const handlePlayerKeyDown = (e: KeyboardEvent) => {
  // 如果焦点在输入框中则不响应
  const activeEl = document.activeElement;
  if (
    activeEl &&
    (activeEl.tagName === "INPUT" ||
      activeEl.tagName === "TEXTAREA" ||
      (activeEl as HTMLElement).isContentEditable)
  ) {
    return;
  }

  // 空格键: 播放/暂停
  if (e.key === " " || e.code === "Space") {
    e.preventDefault();
    togglePlay();
    return;
  }

  // J / L 或 Shift+方向键: 快速搜寻 ±5 秒
  if (e.key === "j" || e.key === "J" || (e.shiftKey && e.key === "ArrowLeft")) {
    e.preventDefault();
    seekRelative(-5);
    return;
  }
  if (e.key === "l" || e.key === "L" || (e.shiftKey && e.key === "ArrowRight")) {
    e.preventDefault();
    seekRelative(5);
    return;
  }

  // K 键: 播放/暂停
  if (e.key === "k" || e.key === "K") {
    e.preventDefault();
    togglePlay();
    return;
  }

  // M 键: 静音
  if (e.key === "m" || e.key === "M") {
    e.preventDefault();
    toggleMute();
    return;
  }
};

const handleFullscreenChange = () => {
  isFullscreen.value = !!document.fullscreenElement;
};

onMounted(() => {
  // 恢复上次保存的音量设置
  try {
    const savedVol = localStorage.getItem("photoselect_video_volume");
    if (savedVol !== null) {
      volume.value = Math.max(0, Math.min(1, parseFloat(savedVol)));
    }
  } catch {}

  window.addEventListener("keydown", handlePlayerKeyDown);
  document.addEventListener("fullscreenchange", handleFullscreenChange);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handlePlayerKeyDown);
  document.removeEventListener("fullscreenchange", handleFullscreenChange);
  if (hideControlsTimer) clearTimeout(hideControlsTimer);
  if (centerFeedbackTimer.value) clearTimeout(centerFeedbackTimer.value);
});
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full h-full flex items-center justify-center bg-black overflow-hidden select-none group"
    @mousemove="resetControlsTimer"
    @mouseleave="isPlaying && (showControls = false)"
  >
    <!-- 1. 核心 HTML5 视频视口 (硬件加速解码与 Range 流式分块传输) -->
    <video
      ref="videoRef"
      :src="videoSrc"
      playsinline
      autoplay
      class="w-full h-full object-contain cursor-pointer"
      @click="togglePlay"
      @loadedmetadata="onLoadedMetadata"
      @canplay="onCanPlay"
      @loadeddata="onCanPlay"
      @error="onVideoError"
      @timeupdate="onTimeUpdate"
      @play="onPlay"
      @pause="onPause"
      @waiting="onWaiting"
      @playing="onPlaying"
      @ended="onEnded"
    ></video>

    <!-- 2. 中央播放/暂停快速反馈浮标 (动画淡出) -->
    <transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 scale-75"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-125"
    >
      <div
        v-if="showCenterFeedback"
        class="absolute inset-0 flex items-center justify-center pointer-events-none z-30"
      >
        <div class="w-18 h-18 rounded-full bg-black/75 backdrop-blur-md border border-white/20 flex items-center justify-center text-white shadow-2xl">
          <Play v-if="showCenterFeedback === 'play'" class="w-9 h-9 ml-1 fill-white" />
          <Pause v-else class="w-9 h-9 fill-white" />
        </div>
      </div>
    </transition>

    <!-- 3. 加载与缓冲状态微标 / 解码异常降级提示 -->
    <div
      v-if="playbackError"
      class="absolute inset-0 flex items-center justify-center z-25 bg-black/75 backdrop-blur-sm p-6"
    >
      <div class="max-w-md w-full rounded-2xl bg-[#18181C] border border-rose-500/35 p-5 text-center space-y-3 shadow-2xl">
        <AlertTriangle class="w-10 h-10 text-amber-400 mx-auto" />
        <div class="text-sm font-semibold text-white">视频内置解码受限</div>
        <p class="text-xs text-zinc-400 leading-relaxed">{{ playbackError }}</p>
        <button
          @click="handleOpenInSystemPlayer"
          class="inline-flex items-center space-x-1.5 px-4 py-2 rounded-xl bg-sky-600 hover:bg-sky-500 text-white text-xs font-medium cursor-pointer transition-all active:scale-95"
        >
          <ExternalLink class="w-3.5 h-3.5" />
          <span>使用系统默认播放器打开</span>
        </button>
      </div>
    </div>

    <div
      v-else-if="isBuffering"
      class="absolute inset-0 flex items-center justify-center pointer-events-none z-20 bg-black/25 backdrop-blur-[1px]"
    >
      <div class="flex items-center space-x-2.5 px-4 py-2 rounded-xl bg-black/80 border border-white/10 text-white shadow-2xl">
        <Loader2 class="w-4 h-4 text-sky-400 animate-spin" />
        <span class="text-xs font-mono font-medium">缓冲中...</span>
      </div>
    </div>

    <!-- 4. 顶部精工暗房 HUD 状态悬浮胶囊条 (文件名 + 格式 + 快速裁决) -->
    <transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div
        v-if="showControls && currentPhoto"
        class="absolute top-4 left-4 right-4 z-30 flex items-center justify-between pointer-events-auto"
      >
        <!-- 左侧文件名与视频格式标签 -->
        <div class="flex items-center space-x-2">
          <div class="flex items-center space-x-2 px-3 py-1.5 rounded-xl bg-black/80 backdrop-blur-xl border border-white/10 text-white shadow-lg">
            <Film class="w-4 h-4 text-purple-400" />
            <span class="font-mono text-xs font-semibold tracking-wide">{{ currentPhoto.base_name }}</span>
            <span class="px-1.5 py-0.2 rounded bg-purple-500/20 text-purple-300 font-mono text-[10px] font-bold border border-purple-500/30">
              VIDEO
            </span>
          </div>

          <!-- 拍摄时间与文件大小 -->
          <div
            v-if="currentPhoto.capture_time"
            class="hidden md:flex items-center px-2.5 py-1.5 rounded-xl bg-black/70 backdrop-blur-xl border border-white/10 text-zinc-300 font-mono text-[11px]"
          >
            <span>{{ currentPhoto.capture_time }}</span>
          </div>
        </div>

        <!-- 右侧快速旗标评星工具条 (一边放片一边裁决) -->
        <div class="flex items-center space-x-1.5 px-3 py-1.5 rounded-xl bg-black/80 backdrop-blur-xl border border-white/10 text-white shadow-lg">
          <!-- 留用 Pick [P] -->
          <button
            @click="store.setFlag(1 as FlagStatus)"
            :class="[
              'flex items-center space-x-1 px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer shadow-sm active:scale-95',
              currentPhoto.flag === 1
                ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-emerald-600/30'
                : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10',
            ]"
            title="标记留用 [P]"
          >
            <Check class="w-3.5 h-3.5 stroke-[2.5]" />
            <span>留用</span>
          </button>

          <!-- 淘汰 Reject [X] -->
          <button
            @click="store.setFlag(-1 as FlagStatus)"
            :class="[
              'flex items-center space-x-1 px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer shadow-sm active:scale-95',
              currentPhoto.flag === -1
                ? 'bg-gradient-to-r from-rose-600 to-red-600 text-white shadow-rose-600/30'
                : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10',
            ]"
            title="标记淘汰 [X]"
          >
            <X class="w-3.5 h-3.5 stroke-[2.5]" />
            <span>淘汰</span>
          </button>

          <div class="h-4 w-[1px] bg-white/15 mx-1"></div>

          <!-- 星级快捷点选 -->
          <div class="flex items-center space-x-0.5">
            <button
              v-for="star in 5"
              :key="star"
              @click="store.setRating(currentPhoto.rating === star ? 0 : star)"
              class="p-1 hover:scale-115 transition-transform cursor-pointer"
              :title="`评定为 ${star} 星 [${star}]`"
            >
              <Star
                class="w-3.5 h-3.5 transition-colors"
                :class="star <= currentPhoto.rating ? 'text-amber-400 fill-amber-400' : 'text-zinc-600 hover:text-zinc-400'"
              />
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 5. 底部播放控制条 HUD (精工暗房设计，流线质感) -->
    <transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0 translate-y-3"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-3"
    >
      <div
        v-if="showControls"
        class="absolute bottom-4 left-4 right-4 z-30 pointer-events-auto max-w-4xl mx-auto rounded-2xl bg-[#1C1C1E]/82 backdrop-blur-2xl backdrop-saturate-150 border border-white/[0.12] px-4 pt-2.5 pb-2.5 shadow-[0_16px_40px_rgba(0,0,0,0.55)] space-y-2"
        @click.stop
      >
        <!-- 进度条与缓冲指示器 (可点选、可拖拽) -->
        <div class="relative w-full h-4 flex items-center group/progress cursor-pointer">
          <!-- 背景轨道与缓冲段 -->
          <div class="relative w-full h-1.5 group-hover/progress:h-2 rounded-full bg-white/15 overflow-hidden transition-all duration-150 pointer-events-none">
            <div
              class="absolute inset-y-0 left-0 bg-white/25 transition-all duration-150"
              :style="{ width: `${duration ? Math.min(100, (bufferedEnd / duration) * 100) : 0}%` }"
            ></div>
            <!-- 主播放进度条 (Apple Blue) -->
            <div
              class="absolute inset-y-0 left-0 rounded-full bg-[#0A84FF] shadow-[0_0_10px_rgba(10,132,255,0.6)] transition-all duration-75"
              :style="{ width: `${duration ? Math.min(100, (currentTime / duration) * 100) : 0}%` }"
            ></div>
          </div>

          <!-- 进度条圆形拖拽滑块 -->
          <div
            class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-white shadow-[0_1px_6px_rgba(0,0,0,0.6)] pointer-events-none opacity-85 scale-90 group-hover/progress:opacity-100 group-hover/progress:scale-110 transition-all duration-150"
            :style="{ left: `${duration ? Math.min(100, (currentTime / duration) * 100) : 0}%` }"
          ></div>

          <!-- 交互 Slider 输入元素 -->
          <input
            type="range"
            min="0"
            :max="duration || 100"
            step="0.05"
            :value="currentTime"
            @input="handleSeekInput"
            @mousedown="isSeeking = true"
            @mouseup="isSeeking = false"
            class="absolute inset-0 z-10 w-full h-full opacity-0 cursor-pointer"
          />
        </div>

        <!-- 底栏按钮矩阵 (Play/Pause, Seek ±5s, Time, Speed, Volume, Loop, Fullscreen) -->
        <div class="flex items-center justify-between text-white">
          <!-- 左侧区: 播放/暂停 + 快进快退 + 时间显示 -->
          <div class="flex items-center space-x-3">
            <!-- 播放/暂停 -->
            <button
              @click="togglePlay"
              class="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center text-white transition-all cursor-pointer active:scale-90"
              :title="isPlaying ? '暂停 [Space / K]' : '播放 [Space / K]'"
            >
              <Pause v-if="isPlaying" class="w-4 h-4 fill-white" />
              <Play v-else class="w-4 h-4 ml-0.5 fill-white" />
            </button>

            <!-- 快退 5 秒 [J] -->
            <button
              @click="seekRelative(-5)"
              class="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer active:scale-95"
              title="后退 5 秒 [J]"
            >
              <RotateCcw class="w-4 h-4" />
            </button>

            <!-- 快进 5 秒 [L] -->
            <button
              @click="seekRelative(5)"
              class="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer active:scale-95"
              title="快进 5 秒 [L]"
            >
              <RotateCw class="w-4 h-4" />
            </button>

            <!-- 毫秒/秒级时间码展示 -->
            <div class="flex items-center space-x-1 text-xs font-mono font-medium tracking-wider text-zinc-300">
              <span class="text-white font-bold">{{ formatTime(currentTime) }}</span>
              <span class="text-zinc-500">/</span>
              <span>{{ formatTime(duration) }}</span>
            </div>
          </div>

          <!-- 右侧区: 倍速 + 音量 + 循环 + 全屏 -->
          <div class="flex items-center space-x-3">
            <!-- 播放倍速 -->
            <button
              @click="cyclePlaybackRate"
              class="px-2 py-0.5 rounded-md text-[11px] font-mono font-bold bg-white/5 hover:bg-white/15 text-sky-400 border border-sky-400/30 transition-all cursor-pointer active:scale-95"
              title="切换播放倍速 (0.5x - 2.0x)"
            >
              {{ playbackRate }}x
            </button>

            <!-- 音量控制器 (滑块与静音按钮) -->
            <div class="flex items-center space-x-1.5 group/volume">
              <button
                @click="toggleMute"
                class="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer"
                :title="isMuted ? '取消静音 [M]' : '静音 [M]'"
              >
                <VolumeX v-if="isMuted || volume === 0" class="w-4 h-4 text-rose-400" />
                <Volume1 v-else-if="volume < 0.5" class="w-4 h-4" />
                <Volume2 v-else class="w-4 h-4" />
              </button>

              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                :value="isMuted ? 0 : volume"
                @input="handleVolumeChange"
                class="w-16 h-1 bg-white/20 rounded-full appearance-none cursor-pointer accent-sky-400 group-hover/volume:w-20 transition-all"
                title="音量调节"
              />
            </div>

            <!-- 循环播放开关 -->
            <button
              @click="toggleLoop"
              :class="[
                'p-1.5 rounded-lg transition-all cursor-pointer',
                isLooping
                  ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40 shadow-sm'
                  : 'text-zinc-400 hover:text-white hover:bg-white/5',
              ]"
              :title="isLooping ? '循环播放: 开' : '循环播放: 关'"
            >
              <Repeat class="w-4 h-4" />
            </button>

            <div class="h-4 w-[1px] bg-white/10"></div>

            <!-- 全屏切换 -->
            <button
              @click="toggleFullscreen"
              class="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer active:scale-95"
              :title="isFullscreen ? '退出全屏' : '全屏播放'"
            >
              <Minimize v-if="isFullscreen" class="w-4 h-4" />
              <Maximize v-else class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>