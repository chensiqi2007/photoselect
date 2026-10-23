<script setup lang="ts">
import {
  X,
  Moon,
  Sun,
  Globe,
  Monitor,
  Check,
  Eye,
  ZoomIn,
  Sliders,
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
    await store.saveAiSettings({
      enabled: aiEnabled.value,
      baseUrl: aiBaseUrl.value,
      model: aiModel.value,
      apiKey: aiApiKey.value,
      clearApiKey: clearApiKey.value,
    });
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
    if (
      aiApiKey.value ||
      aiBaseUrl.value !== store.aiSettings?.baseUrl ||
      aiModel.value !== store.aiSettings?.model ||
      aiEnabled.value !== store.aiSettings?.enabled
    ) {
      await saveAi();
    }
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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md select-none"
    @click.self="closeModal"
  >
    <!-- 设置对话框主体 (精工暗房/明亮工作室自适应) -->
    <div
      class="w-[660px] max-w-[94vw] max-h-[90vh] flex flex-col rounded-2xl border shadow-2xl overflow-hidden transition-colors duration-150"
      :class="[
        store.theme === 'light'
          ? 'bg-white border-zinc-200 text-zinc-900 shadow-zinc-400/20'
          : 'bg-[#18181B] border-[#2A2A30] text-zinc-100 shadow-black/80'
      ]"
    >
      <!-- 模态框顶部标头 -->
      <div
        class="h-13 px-6 flex items-center justify-between border-b"
        :class="[
          store.theme === 'light'
            ? 'bg-zinc-50 border-zinc-200'
            : 'bg-[#141416] border-[#26262C]'
        ]"
      >
        <div class="flex items-center space-x-2.5">
          <div
            class="w-7 h-7 rounded-lg flex items-center justify-center border"
            :class="[
              store.theme === 'light'
                ? 'bg-zinc-200/80 border-zinc-300 text-zinc-800'
                : 'bg-zinc-800/80 border-zinc-700 text-zinc-200'
            ]"
          >
            <Sliders class="w-3.5 h-3.5" />
          </div>
          <h2 class="text-sm font-semibold tracking-wide">
            {{ t.settings.title }}
          </h2>
        </div>
        <button
          @click="closeModal"
          class="p-1.5 rounded-lg transition-colors cursor-pointer"
          :class="[
            store.theme === 'light'
              ? 'text-zinc-500 hover:text-zinc-900 hover:bg-zinc-200'
              : 'text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800'
          ]"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 设置核心内容区 -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <!-- 1. 外观色彩模式 (Dark vs Light) -->
        <div>
          <label
            class="block text-[11px] font-semibold uppercase tracking-wider mb-2.5"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            {{ t.settings.appearance }}
          </label>
          <div class="grid grid-cols-2 gap-3">
            <!-- 黑暗模式卡片 (纯正暗房标准) -->
            <button
              @click="handleSelectTheme('dark')"
              :class="[
                'p-3.5 rounded-xl border text-left transition-all cursor-pointer relative flex flex-col justify-between h-28 select-none',
                store.theme === 'dark'
                  ? 'border-blue-500 ring-2 ring-blue-500/40 shadow-md bg-[#202024] text-white'
                  : 'border-zinc-700/60 bg-[#141416] text-zinc-400 hover:border-zinc-600',
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2">
                  <div class="w-6 h-6 rounded-md bg-zinc-800 border border-zinc-700 flex items-center justify-center text-amber-300">
                    <Moon class="w-3.5 h-3.5" />
                  </div>
                  <span class="text-xs font-semibold text-zinc-100">{{ t.settings.themeDark }}</span>
                </div>
                <div v-if="store.theme === 'dark'" class="w-4 h-4 rounded-full bg-blue-500 flex items-center justify-center text-white">
                  <Check class="w-2.5 h-2.5 stroke-[3]" />
                </div>
              </div>
              <p class="text-[11px] leading-relaxed text-zinc-400 line-clamp-2 mt-1">
                {{ t.settings.themeDarkDesc }}
              </p>
            </button>

            <!-- 白昼模式卡片 (明亮工作室) -->
            <button
              @click="handleSelectTheme('light')"
              :class="[
                'p-3.5 rounded-xl border text-left transition-all cursor-pointer relative flex flex-col justify-between h-28 select-none',
                store.theme === 'light'
                  ? 'border-blue-500 ring-2 ring-blue-500/40 shadow-md bg-white text-zinc-900'
                  : 'border-zinc-300 bg-zinc-50 text-zinc-600 hover:border-zinc-400',
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2">
                  <div class="w-6 h-6 rounded-md bg-amber-50 border border-amber-200 flex items-center justify-center text-amber-500">
                    <Sun class="w-3.5 h-3.5" />
                  </div>
                  <span class="text-xs font-semibold text-zinc-900">{{ t.settings.themeLight }}</span>
                </div>
                <div v-if="store.theme === 'light'" class="w-4 h-4 rounded-full bg-blue-500 flex items-center justify-center text-white">
                  <Check class="w-2.5 h-2.5 stroke-[3]" />
                </div>
              </div>
              <p class="text-[11px] leading-relaxed text-zinc-500 line-clamp-2 mt-1">
                {{ t.settings.themeLightDesc }}
              </p>
            </button>
          </div>
        </div>

        <!-- 2. 界面多语言选择 -->
        <div>
          <label
            class="block text-[11px] font-semibold uppercase tracking-wider mb-2.5 flex items-center space-x-1.5"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            <Globe class="w-3.5 h-3.5" />
            <span>{{ t.settings.language }}</span>
          </label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="item in [
                { id: 'zh-CN', label: '简体中文' },
                { id: 'zh-TW', label: '繁體中文' },
                { id: 'en-US', label: 'English' }
              ]"
              :key="item.id"
              @click="handleSelectLocale(item.id as Locale)"
              :class="[
                'px-3 py-2 rounded-xl border text-center transition-all cursor-pointer font-medium text-xs flex items-center justify-center space-x-1.5',
                locale === item.id
                  ? store.theme === 'light'
                    ? 'border-zinc-800 bg-zinc-900 text-white shadow-sm font-semibold'
                    : 'border-zinc-400 bg-zinc-100 text-zinc-900 shadow-sm font-semibold'
                  : store.theme === 'light'
                  ? 'border-zinc-200 bg-zinc-50 text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900'
                  : 'border-[#2E2E36] bg-[#1E1E24] text-zinc-400 hover:bg-[#282830] hover:text-zinc-100'
              ]"
            >
              <Check v-if="locale === item.id" class="w-3.5 h-3.5 stroke-[2.5]" />
              <span>{{ item.label }}</span>
            </button>
          </div>
        </div>

        <!-- 3. 页面及字体缩放 (High-DPI 缩放 100% / 125% / 150% / 200%) -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <label
              class="text-[11px] font-semibold uppercase tracking-wider flex items-center space-x-1.5"
              :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
            >
              <ZoomIn class="w-3.5 h-3.5" />
              <span>{{ t.settings.uiScale }}</span>
            </label>
            <span class="text-[11px] font-mono text-zinc-400">
              Ctrl + = / - / 0
            </span>
          </div>
          <div class="grid grid-cols-4 gap-2">
            <button
              v-for="scale in [100, 125, 150, 200]"
              :key="scale"
              @click="store.setUiScale(scale)"
              :class="[
                'px-2.5 py-2 rounded-xl border text-center transition-all cursor-pointer text-xs flex flex-col items-center justify-center space-y-0.5 select-none',
                store.uiScale === scale
                  ? store.theme === 'light'
                    ? 'border-zinc-800 bg-zinc-900 text-white shadow-sm font-bold'
                    : 'border-zinc-300 bg-zinc-100 text-zinc-900 shadow-sm font-bold'
                  : store.theme === 'light'
                  ? 'border-zinc-200 bg-zinc-50 text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900'
                  : 'border-[#2E2E36] bg-[#1E1E24] text-zinc-400 hover:bg-[#282830] hover:text-zinc-100'
              ]"
            >
              <div class="flex items-center space-x-1">
                <Check v-if="store.uiScale === scale" class="w-3 h-3 stroke-[2.5]" />
                <span class="text-xs font-semibold">{{ scale }}%</span>
              </div>
              <span class="text-[10px] opacity-75 font-mono">
                {{ scale === 100 ? '1.0x' : scale === 125 ? '1.25x' : scale === 150 ? '1.5x' : '2.0x' }}
              </span>
            </button>
          </div>
          <p
            class="text-[11px] mt-1.5 leading-relaxed"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            {{ t.settings.uiScaleDesc }}
          </p>
        </div>

        <!-- 4. 单图视口背景基准色 -->
        <div>
          <label
            class="block text-[11px] font-semibold uppercase tracking-wider mb-2.5 flex items-center space-x-1.5"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            <Monitor class="w-3.5 h-3.5" />
            <span>{{ t.settings.viewportBg }}</span>
          </label>
          <div class="grid grid-cols-3 gap-2">
            <!-- 标准摄影中性灰 (暗房18%中性灰) -->
            <button
              @click="handleSelectViewportBg(store.theme === 'light' ? '#E5E5EA' : '#1E1E1E')"
              :class="[
                'flex items-center space-x-2 px-3 py-2 rounded-xl border transition-all cursor-pointer text-xs',
                (store.viewportBg === '#1E1E1E' || store.viewportBg === '#E5E5EA')
                  ? store.theme === 'light'
                    ? 'border-zinc-800 bg-zinc-900 text-white shadow-sm font-semibold'
                    : 'border-zinc-300 bg-zinc-100 text-zinc-900 shadow-sm font-semibold'
                  : store.theme === 'light'
                  ? 'border-zinc-200 bg-zinc-50 text-zinc-700 hover:bg-zinc-100'
                  : 'border-[#2E2E36] bg-[#1E1E24] text-zinc-300 hover:bg-[#282830]'
              ]"
            >
              <span
                class="w-3.5 h-3.5 rounded-full border shadow-sm flex-shrink-0"
                :style="{
                  backgroundColor: store.theme === 'light' ? '#E5E5EA' : '#1E1E1E',
                  borderColor: store.theme === 'light' ? '#AEAEB2' : '#555555'
                }"
              ></span>
              <span class="truncate">{{ t.settings.bgNeutral }}</span>
            </button>

            <!-- 纯黑底色 -->
            <button
              @click="handleSelectViewportBg('#000000')"
              :class="[
                'flex items-center space-x-2 px-3 py-2 rounded-xl border transition-all cursor-pointer text-xs',
                store.viewportBg === '#000000'
                  ? store.theme === 'light'
                    ? 'border-zinc-800 bg-zinc-900 text-white shadow-sm font-semibold'
                    : 'border-zinc-300 bg-zinc-100 text-zinc-900 shadow-sm font-semibold'
                  : store.theme === 'light'
                  ? 'border-zinc-200 bg-zinc-50 text-zinc-700 hover:bg-zinc-100'
                  : 'border-[#2E2E36] bg-[#1E1E24] text-zinc-300 hover:bg-[#282830]'
              ]"
            >
              <span class="w-3.5 h-3.5 rounded-full bg-black border border-zinc-600 shadow-sm flex-shrink-0"></span>
              <span class="truncate">{{ t.settings.bgBlack }}</span>
            </button>

            <!-- 纯白底色 -->
            <button
              @click="handleSelectViewportBg('#FFFFFF')"
              :class="[
                'flex items-center space-x-2 px-3 py-2 rounded-xl border transition-all cursor-pointer text-xs',
                store.viewportBg === '#FFFFFF'
                  ? store.theme === 'light'
                    ? 'border-zinc-800 bg-zinc-900 text-white shadow-sm font-semibold'
                    : 'border-zinc-300 bg-zinc-100 text-zinc-900 shadow-sm font-semibold'
                  : store.theme === 'light'
                  ? 'border-zinc-200 bg-zinc-50 text-zinc-700 hover:bg-zinc-100'
                  : 'border-[#2E2E36] bg-[#1E1E24] text-zinc-300 hover:bg-[#282830]'
              ]"
            >
              <span class="w-3.5 h-3.5 rounded-full bg-white border border-zinc-300 shadow-sm flex-shrink-0"></span>
              <span class="truncate">{{ t.settings.bgWhite }}</span>
            </button>
          </div>
        </div>

        <!-- 5. 智能对焦与识别检测 -->
        <div
          class="pt-4 border-t flex items-center justify-between"
          :class="store.theme === 'light' ? 'border-zinc-200' : 'border-zinc-800'"
        >
          <div class="flex items-center space-x-2.5">
            <div
              class="w-7 h-7 rounded-lg flex items-center justify-center border"
              :class="[
                store.theme === 'light'
                  ? 'bg-zinc-100 border-zinc-200 text-zinc-700'
                  : 'bg-zinc-800 border-zinc-700 text-zinc-300'
              ]"
            >
              <Eye class="w-3.5 h-3.5" />
            </div>
            <div>
              <span
                class="text-xs font-semibold block"
                :class="store.theme === 'light' ? 'text-zinc-900' : 'text-zinc-100'"
              >
                {{ t.eyeAf.settingsEyeAf }}
              </span>
              <p
                class="text-[11px] leading-tight mt-0.5"
                :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
              >
                {{ t.eyeAf.settingsEyeAfDesc }}
              </p>
            </div>
          </div>
          <!-- 切换开关 -->
          <button
            @click="store.toggleEyeAf"
            :class="[
              'w-10 h-5.5 rounded-full transition-colors relative cursor-pointer flex items-center px-0.5',
              store.eyeAfEnabled ? 'bg-sky-600' : store.theme === 'light' ? 'bg-zinc-300' : 'bg-zinc-700',
            ]"
          >
            <div
              :class="[
                'w-4.5 h-4.5 rounded-full bg-white shadow-sm transition-transform',
                store.eyeAfEnabled ? 'translate-x-4.5' : 'translate-x-0',
              ]"
            />
          </button>
        </div>

        <!-- 6. AI 选片服务配置 -->
        <section
          class="pt-4 border-t space-y-3"
          :class="store.theme === 'light' ? 'border-zinc-200' : 'border-zinc-800'"
        >
          <h3
            class="text-xs font-semibold"
            :class="store.theme === 'light' ? 'text-zinc-900' : 'text-zinc-100'"
          >
            {{ t.settings.aiTitle }}
          </h3>
          <label
            class="flex items-center justify-between text-xs"
            :class="store.theme === 'light' ? 'text-zinc-700' : 'text-zinc-300'"
          >
            <span>{{ t.settings.aiEnabled }}</span>
            <input v-model="aiEnabled" type="checkbox" class="h-4 w-4 accent-sky-600" />
          </label>
          <label
            class="block text-[11px]"
            :class="store.theme === 'light' ? 'text-zinc-600' : 'text-zinc-400'"
          >
            {{ t.settings.aiBaseUrl }}
            <input
              v-model="aiBaseUrl"
              type="url"
              autocomplete="url"
              class="mt-1 w-full px-2.5 py-1.5 rounded-lg border text-xs outline-none transition-colors"
              :class="[
                store.theme === 'light'
                  ? 'bg-zinc-50 border-zinc-300 text-zinc-900 focus:border-sky-600'
                  : 'bg-[#141418] border-[#2E2E36] text-zinc-100 focus:border-sky-500'
              ]"
              placeholder="https://api.openai.com/v1"
            />
          </label>
          <label
            class="block text-[11px]"
            :class="store.theme === 'light' ? 'text-zinc-600' : 'text-zinc-400'"
          >
            {{ t.settings.aiModel }}
            <input
              v-model="aiModel"
              type="text"
              autocomplete="off"
              class="mt-1 w-full px-2.5 py-1.5 rounded-lg border text-xs outline-none transition-colors"
              :class="[
                store.theme === 'light'
                  ? 'bg-zinc-50 border-zinc-300 text-zinc-900 focus:border-sky-600'
                  : 'bg-[#141418] border-[#2E2E36] text-zinc-100 focus:border-sky-500'
              ]"
              placeholder="gpt-4o-mini"
            />
          </label>
          <label
            class="block text-[11px]"
            :class="store.theme === 'light' ? 'text-zinc-600' : 'text-zinc-400'"
          >
            {{ t.settings.aiApiKey }}
            <input
              v-model="aiApiKey"
              type="password"
              autocomplete="new-password"
              class="mt-1 w-full px-2.5 py-1.5 rounded-lg border text-xs outline-none transition-colors"
              :class="[
                store.theme === 'light'
                  ? 'bg-zinc-50 border-zinc-300 text-zinc-900 focus:border-sky-600'
                  : 'bg-[#141418] border-[#2E2E36] text-zinc-100 focus:border-sky-500'
              ]"
            />
          </label>
          <div
            class="flex items-center justify-between gap-2 text-[11px]"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            <span>{{ store.aiSettings?.hasApiKey ? t.settings.aiKeySaved : t.settings.aiKeyMissing }}</span>
            <label v-if="store.aiSettings?.hasApiKey" class="flex items-center gap-1.5 cursor-pointer">
              <input v-model="clearApiKey" type="checkbox" class="accent-rose-500" />
              <span>{{ t.settings.aiClearKey }}</span>
            </label>
          </div>
          <div class="flex gap-2 pt-1">
            <button
              @click="saveAi"
              :disabled="aiBusy"
              class="px-3.5 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-xs font-medium text-white transition-colors cursor-pointer"
            >
              {{ t.settings.aiSave }}
            </button>
            <button
              @click="testAi"
              :disabled="aiBusy"
              class="px-3.5 py-1.5 rounded-lg border disabled:opacity-50 text-xs font-medium transition-colors cursor-pointer"
              :class="[
                store.theme === 'light'
                  ? 'border-zinc-300 hover:bg-zinc-100 text-zinc-700'
                  : 'border-[#32323A] hover:bg-[#25252D] text-zinc-300'
              ]"
            >
              {{ t.settings.aiTest }}
            </button>
          </div>
          <p v-if="aiMessage" class="text-[11px] text-emerald-400 break-words">{{ aiMessage }}</p>
        </section>

        <!-- 7. 关于与引擎技术规格 -->
        <div
          class="pt-4 border-t"
          :class="store.theme === 'light' ? 'border-zinc-200' : 'border-zinc-800'"
        >
          <div
            class="flex items-center justify-between text-xs mb-2"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            <span
              class="font-semibold"
              :class="store.theme === 'light' ? 'text-zinc-900' : 'text-zinc-100'"
            >
              {{ t.settings.about }}
            </span>
            <span
              class="font-mono px-2 py-0.5 rounded border text-[10px]"
              :class="[
                store.theme === 'light'
                  ? 'bg-zinc-100 border-zinc-200 text-zinc-600'
                  : 'bg-[#202026] border-[#2E2E36] text-zinc-400'
              ]"
            >
              PhotoSelect v1.1.0 • AGPL-3.0
            </span>
          </div>
          <p
            class="text-[11px] leading-relaxed"
            :class="store.theme === 'light' ? 'text-zinc-500' : 'text-zinc-400'"
          >
            {{ t.settings.engineDesc }}
          </p>
        </div>
      </div>

      <!-- 模态框底部 -->
      <div
        class="h-14 px-6 border-t flex items-center justify-end"
        :class="[
          store.theme === 'light'
            ? 'bg-zinc-50 border-zinc-200'
            : 'bg-[#141416] border-[#26262C]'
        ]"
      >
        <button
          @click="closeModal"
          class="px-6 py-2 rounded-xl font-medium text-xs tracking-wide transition-all shadow-sm active:scale-95 cursor-pointer"
          :class="[
            store.theme === 'light'
              ? 'bg-zinc-900 hover:bg-zinc-800 text-white'
              : 'bg-zinc-100 hover:bg-white text-zinc-900'
          ]"
        >
          {{ t.settings.close }}
        </button>
      </div>
    </div>
  </div>
</template>
