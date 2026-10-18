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
  <aside class="fixed right-3 top-12 bottom-36 z-40 w-[min(440px,calc(100vw-24px))] flex flex-col border border-[#383838] bg-[#171717] shadow-2xl rounded-lg overflow-hidden">
    <header class="h-11 px-3 flex items-center justify-between border-b border-[#303030] shrink-0">
      <h2 class="flex items-center gap-2 text-sm font-semibold text-[#EEE]"><Sparkles class="w-4 h-4 text-sky-400" />{{ t.aiSelection.title }}</h2>
      <button @click="store.aiSelectionPanelOpen = false" :title="t.aiSelection.close" class="p-1.5 text-[#AAA] hover:text-white"><X class="w-4 h-4" /></button>
    </header>
    <div class="p-3 space-y-2 border-b border-[#303030] shrink-0">
      <div class="flex gap-2">
        <button @click="run" :disabled="store.aiSelectionBusy || store.filteredGroups.length === 0" class="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded bg-sky-700 hover:bg-sky-600 disabled:opacity-50 text-xs font-semibold text-white">
          <Sparkles class="w-3.5 h-3.5" />{{ store.aiSelectionBusy ? t.aiSelection.running : t.aiSelection.run }}
        </button>
        <button @click="store.settingsModalOpen = true" :title="t.aiSelection.configure" class="px-2.5 rounded border border-[#444] text-[#CCC] hover:bg-[#262626]"><Settings2 class="w-4 h-4" /></button>
      </div>
      <div v-if="store.aiSelectionProgress.total" class="space-y-1">
        <div class="flex justify-between text-[11px] text-[#AAA]"><span>{{ t.aiSelection.progress.replace('{0}', store.aiSelectionProgress.completed.toString()).replace('{1}', store.aiSelectionProgress.total.toString()) }}</span><span>{{ store.aiSelectionProgress.batch }}/{{ store.aiSelectionProgress.batches }}</span></div>
        <progress class="w-full h-1.5 accent-sky-500" :value="store.aiSelectionProgress.completed" :max="store.aiSelectionProgress.total" />
      </div>
      <p v-if="store.aiSelectionError" class="text-[11px] text-rose-300 break-words">{{ store.aiSelectionError }}</p>
    </div>
    <div class="flex-1 overflow-y-auto divide-y divide-[#2A2A2A]">
      <div v-if="!store.aiSelectionResults.length" class="p-6 text-center text-xs text-[#888]">{{ t.aiSelection.noResults }}</div>
      <article v-for="(result, index) in store.aiSelectionResults" :key="result.id" class="p-3 flex gap-2.5">
        <input v-model="result.selected" type="checkbox" class="mt-1 accent-sky-500" :aria-label="t.aiSelection.choose" />
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <button class="truncate text-xs font-semibold text-[#E6E6E6] hover:text-sky-300" @click="() => { const i = store.filteredGroups.findIndex(p => p.id === result.id); if (i >= 0) { store.selectIndex(i); store.viewMode = 'loupe'; } }">{{ store.groups.find(p => p.id === result.id)?.base_name || result.id }}</button>
            <span class="ml-auto shrink-0 text-[10px] text-[#AAA]">{{ t.aiSelection.score }} {{ result.score }}</span>
          </div>
          <div class="mt-1 text-[10px] text-sky-300">{{ recommendationLabel(result.recommendation) }}</div>
          <ul class="mt-1 text-[10px] text-[#999] list-disc pl-4 space-y-0.5"><li v-for="reason in result.reasons" :key="reason">{{ reason }}</li></ul>
          <div class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[10px] text-[#BDBDBD]">
            <label class="flex items-center gap-1"><input v-model="result.applyRating" type="checkbox" :disabled="!result.selected" class="accent-amber-500" />{{ t.aiSelection.applyRating }}: {{ result.suggestedRating }}★</label>
            <label class="flex items-center gap-1"><input v-model="result.applyFlag" type="checkbox" :disabled="!result.selected || result.suggestedFlag === -1" class="accent-emerald-500" />{{ t.aiSelection.applyFlag }}: {{ result.suggestedFlag === 1 ? t.aiSelection.pick : result.suggestedFlag === -1 ? t.aiSelection.reject : t.aiSelection.review }}</label>
          </div>
        </div>
        <span class="text-[10px] text-[#666]">{{ index + 1 }}</span>
      </article>
    </div>
    <footer class="p-3 border-t border-[#303030] flex items-center justify-between gap-2 shrink-0">
      <span class="text-[10px] text-[#888]">{{ selectedCount }}</span>
      <button @click="applySelected" :disabled="applying || !selectedCount" class="px-3 py-2 rounded bg-emerald-700 hover:bg-emerald-600 disabled:opacity-40 text-xs font-semibold text-white"><Check class="inline w-3.5 h-3.5 mr-1" />{{ t.aiSelection.applySelected }}</button>
    </footer>
    <p v-if="message" class="px-3 pb-2 text-[11px] text-emerald-300">{{ message }}</p>
  </aside>
</template>
