<script setup lang="ts">
import {
  X,
  Moon,
  Sun,
  Globe,
  Monitor,
  Check,
  Palette,
  Eye,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n, type Locale } from "../../i18n";
import { onMounted, ref } from "vue";

const store = usePhotoStore();
const { t, locale, setLocale } = useI18n();
const aiEnabled = ref(false);
const aiBaseUrl = ref("https://api.openai.com/v1");
const aiModel = ref("gpt-4o-mini");
const aiApiKey = ref("");
const clearApiKey = ref(false);
const aiBusy = ref(false);
const aiMessage = ref("");

onMounted(async () => {
  try {
    await store.loadAiSettings();
    aiEnabled.value = store.aiSettings?.enabled ?? false;
    aiBaseUrl.value = store.aiSettings?.baseUrl ?? aiBaseUrl.value;
    aiModel.value = store.aiSettings?.model ?? aiModel.value;
  } catch (error) {
    aiMessage.value = String(error);
  }
});

const saveAi = async () => {
  aiBusy.value = true;
  aiMessage.value = "";
  try {
    await store.saveAiSettings({ enabled: aiEnabled.value, baseUrl: aiBaseUrl.value, model: aiModel.value, apiKey: aiApiKey.value, clearApiKey: clearApiKey.value });
    aiApiKey.value = "";
    clearApiKey.value = false;
    aiMessage.value = store.aiSettings?.hasApiKey ? t.value.settings.aiKeySaved : t.value.settings.aiKeyMissing;
  } catch (error) {
    aiMessage.value = String(error);
  } finally {
    aiBusy.value = false;
  }
};

const testAi = async () => {
  aiBusy.value = true;
  aiMessage.value = "";
  try {
    if (aiApiKey.value || aiBaseUrl.value !== store.aiSettings?.baseUrl || aiModel.value !== store.aiSettings?.model || aiEnabled.value !== store.aiSettings?.enabled) await saveAi();
    aiMessage.value = await store.testAiConnection();
  } catch (error) {
    aiMessage.value = String(error);
  } finally {
    aiBusy.value = false;
  }
};

const closeModal = () => {
  store.settingsModalOpen = false;
};

const handleSelectTheme = (theme: "dark" | "light") => {
  store.setTheme(theme);
};

const handleSelectLocale = (loc: Locale) => {
  setLocale(loc);
};

const handleSelectViewportBg = (bg: string) => {
  store.setViewportBg(bg);
};
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/65 backdrop-blur-sm select-none"
    @click.self="closeModal"
  >
    <!-- 设置对话框主体 -->
    <div
      class="w-[640px] max-w-[90vw] max-h-[88vh] flex flex-col rounded-lg bg-[#181818] border border-[#2A2A2A] shadow-2xl overflow-hidden transition-all duration-150"
    >
      <!-- 模态框顶部标头 -->
      <div class="h-12 px-6 flex items-center justify-between border-b border-[#2A2A2A] bg-[#161616]">
        <div class="flex items-center space-x-2.5">
          <div class="w-6 h-6 rounded-lg bg-sky-500/15 border border-sky-500/30 flex items-center justify-center text-sky-400">
            <Palette class="w-3.5 h-3.5" />
          </div>
          <h2 class="text-sm font-bold text-[#E5E5E5] tracking-wide">
            {{ t.settings.title }}
          </h2>
        </div>
        <button
          @click="closeModal"
          class="p-1 rounded-lg text-[#8C8C8C] hover:text-[#E5E5E5] hover:bg-[#262626] transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 设置核心内容区 -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <!-- 1. 外观色彩模式 (Dark vs Light) -->
        <div>
          <label class="block text-xs font-bold text-[#8C8C8C] uppercase tracking-wider mb-3">
            {{ t.settings.appearance }}
          </label>
          <div class="grid grid-cols-2 gap-3.5">
            <!-- 黑暗模式卡片 (纯正暗房标准预览) -->
            <button
              @click="handleSelectTheme('dark')"
              :class="[
                'p-4 rounded-xl border text-left transition-all cursor-pointer relative flex flex-col justify-between h-30 select-none',
                store.theme === 'dark'
                  ? 'border-blue-500 ring-2 ring-blue-500/70 shadow-xl'
                  : 'border-[#333338] hover:border-[#4E4E58]',
              ]"
              style="background-color: #18181B; color: #FFFFFF;"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2.5">
                  <div class="w-7 h-7 rounded-lg bg-[#27272A] border border-[#3F3F46] flex items-center justify-center text-amber-400">
                    <Moon class="w-4 h-4" />
                  </div>
                  <span class="text-xs font-bold" style="color: #F4F4F5;">{{ t.settings.themeDark }}</span>
                </div>
                <div v-if="store.theme === 'dark'" class="w-5 h-5 rounded-full bg-blue-500 flex items-center justify-center text-white shadow-sm">
                  <Check class="w-3.5 h-3.5 stroke-[2.5]" />
                </div>
              </div>
              <p class="text-[11px] leading-relaxed line-clamp-2 mt-2" style="color: #A1A1AA;">
                {{ t.settings.themeDarkDesc }}
              </p>
            </button>

            <!-- 白昼模式卡片 (纯正明亮工作室预览) -->
            <button
              @click="handleSelectTheme('light')"
              :class="[
                'p-4 rounded-xl border text-left transition-all cursor-pointer relative flex flex-col justify-between h-30 select-none',
                store.theme === 'light'
                  ? 'border-blue-500 ring-2 ring-blue-500/70 shadow-xl'
                  : 'border-[#E4E4E7] hover:border-[#C7C7CC]',
              ]"
              style="background-color: #FFFFFF; color: #18181B;"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2.5">
                  <div class="w-7 h-7 rounded-lg bg-amber-50 border border-amber-200 flex items-center justify-center text-amber-500">
                    <Sun class="w-4 h-4" />
                  </div>
                  <span class="text-xs font-bold" style="color: #18181B;">{{ t.settings.themeLight }}</span>
                </div>
                <div v-if="store.theme === 'light'" class="w-5 h-5 rounded-full bg-blue-500 flex items-center justify-center text-white shadow-sm">
                  <Check class="w-3.5 h-3.5 stroke-[2.5]" />
                </div>
              </div>
              <p class="text-[11px] leading-relaxed line-clamp-2 mt-2" style="color: #71717A;">
                {{ t.settings.themeLightDesc }}
              </p>
            </button>
          </div>
        </div>

        <!-- 2. 界面多语言选择 -->
        <div>
          <label class="block text-xs font-bold text-[#8C8C8C] uppercase tracking-wider mb-3 flex items-center space-x-1.5">
            <Globe class="w-3.5 h-3.5 text-sky-400" />
            <span>{{ t.settings.language }}</span>
          </label>
          <div class="grid grid-cols-3 gap-2.5">
            <!-- 简体中文 -->
            <button
              @click="handleSelectLocale('zh-CN')"
              :class="[
                'px-3 py-2.5 rounded-xl border text-center transition-all cursor-pointer font-medium text-xs flex items-center justify-center space-x-1.5',
                locale === 'zh-CN'
                  ? 'border-sky-500 bg-sky-500/15 text-sky-500 font-bold shadow-sm'
                  : 'border-[#2A2A2A] bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5] hover:border-[#3E3E3E]',
              ]"
            >
              <Check v-if="locale === 'zh-CN'" class="w-3.5 h-3.5 stroke-[2.5]" />
              <span>简体中文</span>
            </button>

            <!-- 繁體中文 -->
            <button
              @click="handleSelectLocale('zh-TW')"
              :class="[
                'px-3 py-2.5 rounded-xl border text-center transition-all cursor-pointer font-medium text-xs flex items-center justify-center space-x-1.5',
                locale === 'zh-TW'
                  ? 'border-sky-500 bg-sky-500/15 text-sky-500 font-bold shadow-sm'
                  : 'border-[#2A2A2A] bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5] hover:border-[#3E3E3E]',
              ]"
            >
              <Check v-if="locale === 'zh-TW'" class="w-3.5 h-3.5 stroke-[2.5]" />
              <span>繁體中文</span>
            </button>

            <!-- English (US) -->
            <button
              @click="handleSelectLocale('en-US')"
              :class="[
                'px-3 py-2.5 rounded-xl border text-center transition-all cursor-pointer font-medium text-xs flex items-center justify-center space-x-1.5',
                locale === 'en-US'
                  ? 'border-sky-500 bg-sky-500/15 text-sky-500 font-bold shadow-sm'
                  : 'border-[#2A2A2A] bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5] hover:border-[#3E3E3E]',
              ]"
            >
              <Check v-if="locale === 'en-US'" class="w-3.5 h-3.5 stroke-[2.5]" />
              <span>English</span>
            </button>
          </div>
        </div>

        <!-- 3. 单图视口背景基准色 -->
        <div>
          <label class="block text-xs font-bold text-[#8C8C8C] uppercase tracking-wider mb-3 flex items-center space-x-1.5">
            <Monitor class="w-3.5 h-3.5 text-purple-400" />
            <span>{{ t.settings.viewportBg }}</span>
          </label>
          <div class="grid grid-cols-3 gap-2.5">
            <!-- 标准摄影中性灰 (暗房18%中性灰 / 白昼浅灰基准) -->
            <button
              @click="handleSelectViewportBg(store.theme === 'light' ? '#E5E5EA' : '#1E1E1E')"
              :class="[
                'flex items-center space-x-2 px-3 py-2.5 rounded-xl border transition-all cursor-pointer text-xs',
                (store.viewportBg === '#1E1E1E' || store.viewportBg === '#E5E5EA')
                  ? 'border-purple-500 bg-purple-500/15 text-purple-500 font-bold shadow-sm'
                  : 'border-[#2A2A2A] bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <span
                class="w-4 h-4 rounded-full border shadow-sm flex-shrink-0"
                :style="{
                  backgroundColor: store.theme === 'light' ? '#E5E5EA' : '#1E1E1E',
                  borderColor: store.theme === 'light' ? '#AEAEB2' : '#555555'
                }"
              ></span>
              <span class="truncate font-medium">{{ t.settings.bgNeutral }}</span>
            </button>

            <!-- 纯黑底色 -->
            <button
              @click="handleSelectViewportBg('#000000')"
              :class="[
                'flex items-center space-x-2 px-3 py-2.5 rounded-xl border transition-all cursor-pointer text-xs',
                store.viewportBg === '#000000'
                  ? 'border-purple-500 bg-purple-500/15 text-purple-500 font-bold shadow-sm'
                  : 'border-[#2A2A2A] bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <span class="w-4 h-4 rounded-full bg-black border border-[#555555] shadow-sm flex-shrink-0"></span>
              <span class="truncate font-medium">{{ t.settings.bgBlack }}</span>
            </button>

            <!-- 纯白底色 -->
            <button
              @click="handleSelectViewportBg('#FFFFFF')"
              :class="[
                'flex items-center space-x-2 px-3 py-2.5 rounded-xl border transition-all cursor-pointer text-xs',
                store.viewportBg === '#FFFFFF'
                  ? 'border-purple-500 bg-purple-500/15 text-purple-500 font-bold shadow-sm'
                  : 'border-[#2A2A2A] bg-[#1C1C1C] text-[#8C8C8C] hover:text-[#E5E5E5]',
              ]"
            >
              <span class="w-4 h-4 rounded-full bg-white border border-[#D1D1D6] shadow-sm flex-shrink-0"></span>
              <span class="truncate font-medium">{{ t.settings.bgWhite }}</span>
            </button>
          </div>
        </div>

        <!-- 4. AI 人眼识别与特写质检设置 -->
        <div class="pt-4 border-t border-[#2A2A2A]">
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2.5">
              <div class="w-7 h-7 rounded-lg bg-sky-500/15 flex items-center justify-center text-sky-400">
                <Eye class="w-4 h-4" />
              </div>
              <div>
                <span class="text-xs font-bold text-[#E5E5E5] block">{{ t.eyeAf.settingsEyeAf }}</span>
                <p class="text-[11px] text-[#8C8C8C] leading-tight mt-0.5">{{ t.eyeAf.settingsEyeAfDesc }}</p>
              </div>
            </div>
            <!-- 开关按钮 -->
            <button
              @click="store.toggleEyeAf"
              :class="[
                'w-11 h-6 rounded-full transition-colors relative cursor-pointer flex items-center px-0.5',
                store.eyeAfEnabled ? 'bg-sky-500' : 'bg-[#333333]',
              ]"
            >
              <div
                :class="[
                  'w-5 h-5 rounded-full bg-white shadow-md transition-transform',
                  store.eyeAfEnabled ? 'translate-x-5' : 'translate-x-0',
                ]"
              />
            </button>
          </div>
        </div>

        <section class="pt-4 border-t border-[#2A2A2A] space-y-3">
          <h3 class="text-xs font-bold text-[#E5E5E5]">{{ t.settings.aiTitle }}</h3>
          <label class="flex items-center justify-between text-xs text-[#C8C8C8]">
            <span>{{ t.settings.aiEnabled }}</span>
            <input v-model="aiEnabled" type="checkbox" class="h-4 w-4 accent-sky-500" />
          </label>
          <label class="block text-[11px] text-[#A0A0A0]">
            {{ t.settings.aiBaseUrl }}
            <input v-model="aiBaseUrl" type="url" autocomplete="url" class="mt-1 w-full px-2.5 py-2 bg-[#111] border border-[#333] rounded text-xs text-[#EEE] outline-none focus:border-sky-500" placeholder="https://api.openai.com/v1" />
          </label>
          <label class="block text-[11px] text-[#A0A0A0]">
            {{ t.settings.aiModel }}
            <input v-model="aiModel" type="text" autocomplete="off" class="mt-1 w-full px-2.5 py-2 bg-[#111] border border-[#333] rounded text-xs text-[#EEE] outline-none focus:border-sky-500" placeholder="vision model id" />
          </label>
          <label class="block text-[11px] text-[#A0A0A0]">
            {{ t.settings.aiApiKey }}
            <input v-model="aiApiKey" type="password" autocomplete="new-password" class="mt-1 w-full px-2.5 py-2 bg-[#111] border border-[#333] rounded text-xs text-[#EEE] outline-none focus:border-sky-500" />
          </label>
          <div class="flex items-center justify-between gap-2 text-[11px] text-[#8C8C8C]">
            <span>{{ store.aiSettings?.hasApiKey ? t.settings.aiKeySaved : t.settings.aiKeyMissing }}</span>
            <label v-if="store.aiSettings?.hasApiKey" class="flex items-center gap-1.5"><input v-model="clearApiKey" type="checkbox" class="accent-rose-500" />{{ t.settings.aiClearKey }}</label>
          </div>
          <p class="text-[11px] text-amber-200/80">{{ t.settings.aiPrivacy }}</p>
          <p class="text-[11px] text-[#777]">{{ t.settings.aiVisionHint }}</p>
          <div class="flex gap-2">
            <button @click="saveAi" :disabled="aiBusy" class="px-3 py-1.5 rounded bg-sky-700 hover:bg-sky-600 disabled:opacity-50 text-xs text-white">{{ t.settings.aiSave }}</button>
            <button @click="testAi" :disabled="aiBusy" class="px-3 py-1.5 rounded border border-[#444] hover:bg-[#262626] disabled:opacity-50 text-xs text-[#DDD]">{{ t.settings.aiTest }}</button>
          </div>
          <p v-if="aiMessage" class="text-[11px] text-[#B9D9C5] break-words">{{ aiMessage }}</p>
        </section>

        <!-- 5. 关于与引擎规格 -->
        <div class="pt-4 border-t border-[#2A2A2A]">
          <div class="flex items-center justify-between text-xs text-[#8C8C8C] mb-2">
            <span class="font-bold text-[#E5E5E5]">{{ t.settings.about }}</span>
            <span class="font-mono px-2 py-0.5 rounded bg-[#202020] border border-[#333333] text-[10px]">
              {{ t.settings.version }} 1.0.0
            </span>
          </div>
          <p class="text-[11px] text-[#666666] leading-relaxed">
            {{ t.settings.engineDesc }}
          </p>
        </div>
      </div>

      <!-- 模态框底部 -->
      <div class="h-14 px-6 border-t border-[#2A2A2A] bg-[#161616] flex items-center justify-end">
        <button
          @click="closeModal"
          class="px-6 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-white font-bold text-xs transition-all shadow-md active:scale-95 cursor-pointer"
        >
          {{ t.settings.close }}
        </button>
      </div>
    </div>
  </div>
</template>
