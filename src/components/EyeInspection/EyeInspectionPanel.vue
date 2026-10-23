<script setup lang="ts">
import { computed } from "vue";
import {
  Eye,
  EyeOff,
  Crosshair,
  AlertTriangle,
  CheckCircle2,
  ChevronRight,
  ChevronLeft,
  Sparkles,
  Zap,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type { DetectedEye } from "../../types/eye";

const store = usePhotoStore();
const { t } = useI18n();

const faces = computed(() => store.currentFaces);

const handleLockOn = (eye: DetectedEye | null) => {
  if (eye) {
    store.lockOnEye(eye);
  }
};
</script>

<template>
  <!-- 人眼特写检视浮动面板 (Narrative Select 风格) -->
  <div
    class="absolute top-14 right-4 z-30 flex flex-col items-end pointer-events-none select-none transition-all duration-200"
  >
    <!-- 折叠状态下的紧凑唤醒气泡 -->
    <button
      v-if="!store.eyePanelOpen"
      @click="store.toggleEyePanel"
      class="pointer-events-auto flex items-center space-x-2 px-3.5 py-2 rounded-xl bg-[#141418]/90 hover:bg-[#18181E] backdrop-blur-xl border border-white/10 hover:border-sky-500/30 shadow-2xl text-xs text-[#E5E5E5] transition-all cursor-pointer hover:scale-105 active:scale-95 group"
      :title="t.eyeAf.togglePanel"
    >
      <Eye class="w-4 h-4 text-sky-400 group-hover:scale-110 transition-transform" />
      <span class="font-medium text-[11px] tracking-wide">{{ t.eyeAf.title }}</span>
      <span
        v-if="faces.length > 0"
        class="ml-0.5 px-1.5 py-0.5 rounded-full bg-sky-500/20 border border-sky-500/30 text-sky-300 font-mono text-[10px] font-bold"
      >
        {{ faces.length }}
      </span>
      <ChevronLeft class="w-3.5 h-3.5 text-[#71717A] group-hover:text-white transition-colors" />
    </button>

    <!-- 展开状态的主面板 -->
    <div
      v-else
      class="pointer-events-auto w-80 max-h-[72vh] flex flex-col rounded-2xl bg-[#141418]/92 backdrop-blur-2xl border border-white/10 shadow-[0_20px_50px_rgba(0,0,0,0.7)] overflow-hidden transition-all duration-150"
    >
      <!-- 面板顶部标头 -->
      <div class="h-10 px-3.5 flex items-center justify-between border-b border-white/10 bg-white/5">
        <div class="flex items-center space-x-2">
          <Eye class="w-4 h-4 text-sky-400" />
          <span class="text-xs font-bold text-[#F4F4F5] tracking-wide">
            {{ t.eyeAf.title }}
          </span>
          <span
            v-if="faces.length > 0"
            class="px-1.5 py-0.5 rounded-md bg-sky-500/20 border border-sky-500/30 text-sky-300 font-mono text-[10px] font-bold"
          >
            {{ faces.length }}
          </span>
        </div>

        <div class="flex items-center space-x-1.5">
          <!-- 识别状态脉冲指示灯 -->
          <span
            v-if="store.isDetectingEyes"
            class="w-2 h-2 rounded-full bg-amber-400 animate-ping mr-1"
            title="正在分析焦点..."
          />

          <!-- 收起按钮 -->
          <button
            @click="store.toggleEyePanel"
            class="p-1 rounded-lg hover:bg-white/10 text-[#71717A] hover:text-[#F4F4F5] transition-colors cursor-pointer"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- 面板主体内容区 -->
      <div class="flex-1 overflow-y-auto p-3 space-y-3">
        <!-- 正在分析状态 -->
        <div
          v-if="store.isDetectingEyes"
          class="flex items-center space-x-2 p-3 rounded-xl bg-sky-500/10 border border-sky-500/20 text-sky-300 text-xs animate-pulse"
        >
          <Sparkles class="w-4 h-4 animate-spin text-sky-400" />
          <span>{{ t.eyeAf.detecting }}</span>
        </div>
        <div
          v-else-if="store.eyeDetectionFailed"
          class="p-4 text-center text-rose-300 text-xs rounded-xl bg-rose-500/10 border border-rose-500/20"
        >
          <AlertTriangle class="w-6 h-6 mx-auto mb-1.5 text-rose-400" />
          <span>{{ t.eyeAf.detectionFailed }}</span>
        </div>

        <!-- 空状态: 未检出人脸 -->
        <div
          v-else-if="faces.length === 0"
          class="p-6 text-center text-[#71717A] text-xs"
        >
          <EyeOff class="w-6 h-6 mx-auto mb-2 opacity-50 text-[#71717A]" />
          <span>{{ t.eyeAf.noFaces }}</span>
        </div>

        <!-- 人物列表卡片 -->
        <div
          v-for="face in faces"
          :key="face.id"
          class="p-3 rounded-xl bg-white/5 border border-white/8 space-y-2.5 shadow-md hover:border-white/15 transition-all"
        >
          <!-- 人物头部栏: 缩略图 + 综合对焦得分 + 眨眼预警 -->
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2">
              <img
                v-if="face.faceCropDataUrl"
                :src="face.faceCropDataUrl"
                alt=""
                class="w-7 h-7 rounded-full object-cover border border-white/20 shadow-sm"
              />
              <span class="font-mono text-xs font-bold text-[#E5E5E5]">
                {{ t.eyeAf.person }} #{{ face.id }}
              </span>
            </div>

            <div class="flex items-center space-x-1.5">
              <!-- 眨眼警告徽章 -->
              <span
                v-if="face.isBlinking"
                class="flex items-center space-x-0.5 px-1.5 py-0.5 rounded-md text-[10px] bg-rose-500/20 text-rose-300 border border-rose-500/30 font-medium"
              >
                <AlertTriangle class="w-2.5 h-2.5" />
                <span>{{ t.eyeAf.eyeBlink }}</span>
              </span>
              <span
                v-else
                class="flex items-center space-x-0.5 px-1.5 py-0.5 rounded-md text-[10px] bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 font-medium"
              >
                <CheckCircle2 class="w-2.5 h-2.5" />
                <span>{{ t.eyeAf.eyeOpen }}</span>
              </span>

              <!-- 综合焦点清晰度 -->
              <span
                :class="[
                  'px-2 py-0.5 rounded-md text-[10px] font-mono font-bold border',
                  face.overallScore >= 75
                    ? 'bg-emerald-500/20 border-emerald-500/30 text-emerald-300'
                    : face.overallScore >= 50
                    ? 'bg-amber-500/20 border-amber-500/30 text-amber-300'
                    : 'bg-rose-500/20 border-rose-500/30 text-rose-300',
                ]"
              >
                {{ face.overallScore }}%
              </span>
            </div>
          </div>

          <!-- 双眼 100% 裁切对比排版 -->
          <div class="grid grid-cols-2 gap-2">
            <!-- 左眼特写卡片 -->
            <div
              v-if="face.leftEye"
              @click="handleLockOn(face.leftEye)"
              class="group relative rounded-xl bg-black/50 border border-white/10 hover:border-sky-500/70 overflow-hidden cursor-pointer transition-all hover:scale-[1.02] shadow-sm"
              :title="t.eyeAf.lockOnTip"
            >
              <div class="aspect-square relative overflow-hidden bg-[#0A0A0D]">
                <img
                  :src="face.leftEye.cropDataUrl"
                  alt=""
                  class="w-full h-full object-cover"
                />
                <!-- 焦点状态边框角标 -->
                <div class="absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded bg-black/75 backdrop-blur-sm font-mono text-[9px] text-[#D4D4D8] border border-white/10">
                  {{ t.eyeAf.leftEye }}
                </div>
                <!-- 悬浮时显现吸附放大提示 -->
                <div class="absolute inset-0 bg-sky-500/20 backdrop-blur-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                  <Crosshair class="w-6 h-6 text-sky-300 drop-shadow" />
                </div>
              </div>

              <!-- 底部评分条 -->
              <div class="p-1 px-2 flex items-center justify-between text-[10px] font-mono bg-black/60 border-t border-white/5">
                <span
                  :class="[
                    'font-bold',
                    face.leftEye.sharpness >= 75 ? 'text-emerald-400' : face.leftEye.sharpness >= 50 ? 'text-amber-400' : 'text-rose-400',
                  ]"
                >
                  {{ face.leftEye.sharpness }}%
                </span>
                <span class="text-[9px] text-[#A1A1AA]">
                  {{ face.leftEye.isOpen ? t.eyeAf.eyeOpen : t.eyeAf.eyeBlink }}
                </span>
              </div>
            </div>

            <!-- 右眼特写卡片 -->
            <div
              v-if="face.rightEye"
              @click="handleLockOn(face.rightEye)"
              class="group relative rounded-xl bg-black/50 border border-white/10 hover:border-sky-500/70 overflow-hidden cursor-pointer transition-all hover:scale-[1.02] shadow-sm"
              :title="t.eyeAf.lockOnTip"
            >
              <div class="aspect-square relative overflow-hidden bg-[#0A0A0D]">
                <img
                  :src="face.rightEye.cropDataUrl"
                  alt=""
                  class="w-full h-full object-cover"
                />
                <div class="absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded bg-black/75 backdrop-blur-sm font-mono text-[9px] text-[#D4D4D8] border border-white/10">
                  {{ t.eyeAf.rightEye }}
                </div>
                <div class="absolute inset-0 bg-sky-500/20 backdrop-blur-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                  <Crosshair class="w-6 h-6 text-sky-300 drop-shadow" />
                </div>
              </div>

              <!-- 底部评分条 -->
              <div class="p-1 px-2 flex items-center justify-between text-[10px] font-mono bg-black/60 border-t border-white/5">
                <span
                  :class="[
                    'font-bold',
                    face.rightEye.sharpness >= 75 ? 'text-emerald-400' : face.rightEye.sharpness >= 50 ? 'text-amber-400' : 'text-rose-400',
                  ]"
                >
                  {{ face.rightEye.sharpness }}%
                </span>
                <span class="text-[9px] text-[#A1A1AA]">
                  {{ face.rightEye.isOpen ? t.eyeAf.eyeOpen : t.eyeAf.eyeBlink }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 面板底部提示微条 -->
      <div class="h-7 px-3.5 bg-black/40 border-t border-white/5 flex items-center justify-between text-[10px] text-[#71717A] font-mono">
        <div class="flex items-center space-x-1.5">
          <Zap class="w-3 h-3 text-amber-400" />
          <span>点击眼部 100% 智能吸附</span>
        </div>
        <kbd class="px-1.5 py-0.5 rounded bg-white/10 text-[#D4D4D8] font-mono text-[9px] border border-white/10">Z</kbd>
      </div>
    </div>
  </div>
</template>
