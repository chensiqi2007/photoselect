<script setup lang="ts">
import { computed, ref } from "vue";
import { Check, X, Sparkles, Settings2 } from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";

const store = usePhotoStore();
const { t } = useI18n();
const applying = ref(false);
const message = ref("");
const selectedCount = computed(() => store.aiSelectionResults.filter((result) => result.selected && (result.applyRating || result.applyFlag)).length);

const run = async () => {
  const photos = store.filteredGroups;
  if (photos.length > 500) {
    store.aiSelectionError = t.value.aiSelection.countLimit;
    return;
  }
  if (!store.aiSettings) await store.loadAiSettings();
  if (!store.aiSettings?.enabled || !store.aiSettings.hasApiKey) {
    store.settingsModalOpen = true;
    return;
  }
  try { await store.startAiSelection(photos); } catch { /* error is shown from store */ }
};

const applySelected = async () => {
  applying.value = true;
  message.value = "";
  try {
    await store.applyAiResults();
    message.value = t.value.aiSelection.confirmed;
  } catch (error) {
    message.value = String(error);
  } finally {
    applying.value = false;
  }
};

const recommendationLabel = (recommendation: string) => {
  if (recommendation === "pick") return t.value.aiSelection.pick;
  if (recommendation === "reject") return t.value.aiSelection.reject;
  return t.value.aiSelection.review;
};
</script>

<template>
  <aside class="fixed right-4 top-14 bottom-36 z-40 w-[min(460px,calc(100vw-32px))] flex flex-col border border-white/10 bg-[#141418]/95 backdrop-blur-2xl shadow-[0_25px_60px_rgba(0,0,0,0.8)] rounded-2xl overflow-hidden transition-all">
    <header class="h-11 px-4 flex items-center justify-between border-b border-white/10 bg-white/5 shrink-0">
      <h2 class="flex items-center gap-2 text-xs font-bold text-[#F4F4F5] tracking-wide">
        <Sparkles class="w-4 h-4 text-sky-400" />
        <span>{{ t.aiSelection.title }}</span>
      </h2>
      <button
        @click="store.aiSelectionPanelOpen = false"
        :title="t.aiSelection.close"
        class="p-1 rounded-lg text-[#71717A] hover:text-[#F4F4F5] hover:bg-white/10 transition-colors cursor-pointer"
      >
        <X class="w-4 h-4" />
      </button>
    </header>

    <div class="p-3.5 space-y-2.5 border-b border-white/10 bg-white/[0.02] shrink-0">
      <div class="flex gap-2">
        <button
          @click="run"
          :disabled="store.aiSelectionBusy || store.filteredGroups.length === 0"
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-blue-600 via-sky-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-40 text-xs font-semibold text-white shadow-lg shadow-sky-600/20 active:scale-98 transition-all cursor-pointer"
        >
          <Sparkles class="w-3.5 h-3.5" />
          <span>{{ store.aiSelectionBusy ? t.aiSelection.running : t.aiSelection.run }}</span>
        </button>
        <button
          @click="store.settingsModalOpen = true"
          :title="t.aiSelection.configure"
          class="px-3 rounded-xl border border-white/10 bg-white/5 text-[#D4D4D8] hover:bg-white/10 hover:text-white transition-all cursor-pointer flex items-center justify-center"
        >
          <Settings2 class="w-4 h-4" />
        </button>
      </div>

      <div v-if="store.aiSelectionProgress.total" class="space-y-1.5 p-2 rounded-lg bg-black/40 border border-white/5">
        <div class="flex justify-between text-[11px] font-mono text-[#A1A1AA]">
          <span>{{ t.aiSelection.progress.replace('{0}', store.aiSelectionProgress.completed.toString()).replace('{1}', store.aiSelectionProgress.total.toString()) }}</span>
          <span class="text-sky-400 font-bold">{{ store.aiSelectionProgress.batch }}/{{ store.aiSelectionProgress.batches }}</span>
        </div>
        <div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-sky-500 to-blue-500 transition-all duration-200"
            :style="{ width: `${(store.aiSelectionProgress.completed / store.aiSelectionProgress.total) * 100}%` }"
          ></div>
        </div>
      </div>

      <p v-if="store.aiSelectionError" class="text-[11px] text-rose-300 p-2 rounded-lg bg-rose-500/10 border border-rose-500/20 break-words">
        {{ store.aiSelectionError }}
      </p>
    </div>

    <div class="flex-1 overflow-y-auto p-2.5 space-y-2">
      <div v-if="!store.aiSelectionResults.length" class="p-8 text-center text-xs text-[#71717A] italic">
        {{ t.aiSelection.noResults }}
      </div>
      <article
        v-for="(result, index) in store.aiSelectionResults"
        :key="result.id"
        class="p-3 rounded-xl bg-white/5 hover:bg-white/[0.08] border border-white/8 hover:border-white/15 transition-all flex gap-3 group"
      >
        <input
          v-model="result.selected"
          type="checkbox"
          class="mt-1 accent-sky-500 rounded cursor-pointer"
          :aria-label="t.aiSelection.choose"
        />
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <button
              class="truncate text-xs font-semibold text-[#F4F4F5] hover:text-sky-300 font-mono text-left transition-colors cursor-pointer"
              @click="() => { const i = store.filteredGroups.findIndex(p => p.id === result.id); if (i >= 0) { store.selectIndex(i); store.viewMode = 'loupe'; } }"
            >
              {{ store.groups.find(p => p.id === result.id)?.base_name || result.id }}
            </button>
            <span class="ml-auto shrink-0 text-[10px] font-mono px-1.5 py-0.5 rounded bg-sky-500/15 border border-sky-500/30 text-sky-300 font-bold">
              {{ t.aiSelection.score }} {{ result.score }}
            </span>
          </div>

          <div class="mt-1 text-[11px] font-medium text-sky-400">
            {{ recommendationLabel(result.recommendation) }}
          </div>

          <ul class="mt-1.5 text-[10px] text-[#A1A1AA] list-disc pl-3.5 space-y-0.5">
            <li v-for="reason in result.reasons" :key="reason">{{ reason }}</li>
          </ul>

          <div class="mt-2.5 flex flex-wrap gap-x-3 gap-y-1.5 text-[10px] text-[#D4D4D8] pt-2 border-t border-white/5">
            <label class="flex items-center gap-1.5 cursor-pointer">
              <input v-model="result.applyRating" type="checkbox" :disabled="!result.selected" class="accent-amber-500 rounded" />
              <span>{{ t.aiSelection.applyRating }}: <span class="text-amber-400 font-bold">{{ result.suggestedRating }}★</span></span>
            </label>
            <label class="flex items-center gap-1.5 cursor-pointer">
              <input v-model="result.applyFlag" type="checkbox" :disabled="!result.selected || result.suggestedFlag === -1" class="accent-emerald-500 rounded" />
              <span>{{ t.aiSelection.applyFlag }}: <span class="font-bold" :class="result.suggestedFlag === 1 ? 'text-emerald-400' : 'text-[#A1A1AA]'">{{ result.suggestedFlag === 1 ? t.aiSelection.pick : result.suggestedFlag === -1 ? t.aiSelection.reject : t.aiSelection.review }}</span></span>
            </label>
          </div>
        </div>
        <span class="text-[10px] font-mono text-[#52525B] shrink-0">{{ index + 1 }}</span>
      </article>
    </div>

    <footer class="p-3.5 border-t border-white/10 bg-white/5 flex items-center justify-between gap-3 shrink-0">
      <span class="text-[11px] font-mono text-[#A1A1AA]">
        已选: <span class="text-[#F4F4F5] font-bold">{{ selectedCount }}</span> 张
      </span>
      <button
        @click="applySelected"
        :disabled="applying || !selectedCount"
        class="px-4 py-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 disabled:opacity-40 text-xs font-semibold text-white shadow-lg shadow-emerald-600/20 active:scale-98 transition-all cursor-pointer flex items-center gap-1.5"
      >
        <Check class="w-3.5 h-3.5 stroke-[2.5]" />
        <span>{{ t.aiSelection.applySelected }}</span>
      </button>
    </footer>
    <p v-if="message" class="px-3.5 pb-2.5 text-[11px] text-emerald-300 font-mono">{{ message }}</p>
  </aside>
</template>
