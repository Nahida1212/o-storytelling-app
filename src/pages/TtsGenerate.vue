<template>
  <q-page padding>
    <div class="row items-center q-mb-md">
      <div class="text-h5 text-weight-bold text-pink-7">TTS 生成</div>
      <q-space />
    </div>
    <div class="text-caption text-grey-6 q-mb-md" style="line-height: 1.5">
      展示所有章节，已生成 TTS 剧本的章节可查看详细场景和角色信息。
      点击"生成语音"为章节中每个角色配置语音模型并合成音频。
    </div>

    <!-- 加载 -->
    <div v-if="loading" class="text-center q-pa-lg">
      <q-spinner color="pink-6" size="40px" />
      <div class="q-mt-sm text-grey-6">加载中...</div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="novelGroups.length === 0" class="text-center q-pa-lg">
      <q-icon name="auto_stories" color="grey-4" size="64px" />
      <div class="text-h6 text-grey-6 q-mt-md">暂无章节</div>
      <div class="text-caption text-grey-5 q-mt-sm">导入书籍后，章节将在此处显示</div>
    </div>

    <!-- 按小说分组展示 -->
    <div v-else class="q-gutter-md">
      <q-card
        v-for="group in novelGroups"
        :key="group.novelId"
        bordered
        flat
      >
        <q-card-section class="bg-pink-1 q-py-sm">
          <div class="row items-center">
            <q-icon name="menu_book" color="pink-7" size="24px" class="q-mr-sm" />
            <div class="text-subtitle1 text-weight-bold text-pink-8">
              {{ group.novelTitle }}
            </div>
            <span v-if="group.novelAuthor" class="text-caption text-grey-6 q-ml-sm">
              — {{ group.novelAuthor }}
            </span>
            <q-space />
            <q-badge color="pink-4" rounded>
              {{ group.chapters.length }} 章
            </q-badge>
          </div>
        </q-card-section>

        <q-separator />

        <div
          v-for="ch in group.chapters"
          :key="ch.chapter_id"
          :class="['cursor-pointer', { 'bg-pink-1': expandedChapter === ch.chapter_id }]"
        >
          <div class="row items-center no-wrap q-px-md q-py-sm" @click="toggleChapter(ch)">
            <q-icon
              :name="expandedChapter === ch.chapter_id ? 'expand_less' : 'expand_more'"
              color="pink-4"
              size="20px"
              class="q-mr-sm"
            />
            <q-icon name="article" color="pink-3" size="20px" class="q-mr-sm" />
            <div class="col">
              <div class="text-body2">
                第 {{ ch.chapter_index + 1 }} 章 · {{ ch.chapter_title }}
              </div>
              <div class="row items-center q-gutter-x-xs">
                <q-badge v-if="ch.tts_generated" color="positive" outline size="12px">
                  {{ ch.scene_count }} 个场景
                </q-badge>
                <q-badge v-else outline size="12px" color="grey-4" text-color="grey-6">
                  未生成 TTS
                </q-badge>
              </div>
            </div>
            <q-btn
              flat
              rounded
              color="pink-6"
              icon="record_voice_over"
              label="生成语音"
              size="sm"
              :disable="!ch.tts_generated || generatingChapterId === ch.chapter_id"
              @click.stop="openVoiceDialog(ch)"
            >
              <q-tooltip v-if="!ch.tts_generated">请先在书籍详情页生成 TTS 剧本</q-tooltip>
              <q-tooltip v-else>配置角色语音并生成音频</q-tooltip>
            </q-btn>
          </div>

          <q-slide-transition>
            <div v-if="expandedChapter === ch.chapter_id" class="q-px-md q-pb-sm">
              <q-separator class="q-mb-sm" />
              <div v-if="chapterDetail[ch.chapter_id]" class="text-caption text-grey-7">
                <div class="row q-gutter-x-lg q-mb-sm">
                  <div><span class="text-weight-bold">总场景：</span>{{ chapterDetail[ch.chapter_id].scene_count }}</div>
                  <div><span class="text-weight-bold">对白场景：</span>{{ chapterDetail[ch.chapter_id].dialogue_count }}</div>
                  <div><span class="text-weight-bold">文本长度：</span>{{ chapterDetail[ch.chapter_id].total_text_length }} 字</div>
                </div>
                <div>
                  <span class="text-weight-bold">角色：</span>
                  <template v-if="chapterDetail[ch.chapter_id].characters.length > 0">
                    <q-chip
                      v-for="name in chapterDetail[ch.chapter_id].characters"
                      :key="name"
                      dense
                      size="12px"
                      color="pink-2"
                      text-color="pink-8"
                      class="q-mr-xs"
                    >
                      {{ name }}
                    </q-chip>
                  </template>
                  <span v-else class="text-grey-5">无</span>
                </div>
              </div>
              <div v-else class="text-center text-grey-5 text-caption">
                <q-spinner size="16px" color="pink-4" /> 加载中...
              </div>
            </div>
          </q-slide-transition>
          <q-separator v-if="expandedChapter === ch.chapter_id" />
        </div>
      </q-card>
    </div>

    <!-- 角色语音配置对话框 -->
    <q-dialog v-model="dialogVisible" persistent>
      <q-card style="min-width: 520px; max-width: 600px">
        <q-card-section class="q-pb-none">
          <div class="text-h6 text-pink-7">
            配置角色语音
          </div>
          <div class="text-caption text-grey-6 q-mt-xs" v-if="dialogChapter">
            第 {{ dialogChapter.chapter_index + 1 }} 章 · {{ dialogChapter.chapter_title }}
          </div>
        </q-card-section>

        <q-card-section v-if="chapterCharacters.length === 0" class="text-center text-grey-5 q-py-lg">
          该章节暂无角色数据
        </q-card-section>

        <q-card-section v-else class="q-gutter-y-sm">
          <div
            v-for="charName in chapterCharacters"
            :key="charName"
            class="row items-center q-gutter-sm"
          >
            <q-chip dense color="pink-2" text-color="pink-8" style="min-width: 80px">
              {{ charName }}
            </q-chip>
            <q-select
              :model-value="characterMappings[charName] || null"
              :options="voiceOptions"
              option-label="label"
              option-value="value"
              outlined
              dense
              color="pink-6"
              placeholder="选择语音配置"
              class="col"
              clearable
              @update:model-value="val => setMapping(charName, val)"
            />
          </div>
        </q-card-section>

        <!-- 生成进度 -->
        <q-card-section v-if="generating" class="q-pt-none">
          <q-linear-progress
            :value="genProgress.total > 0 ? genProgress.processed / genProgress.total : 0"
            color="pink-6"
            class="q-mb-sm"
          />
          <div class="text-caption text-grey-6 text-center">
            {{ genProgress.message }}
          </div>
        </q-card-section>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn flat rounded label="取消" color="grey-6" v-close-popup :disable="generating" />
          <q-btn
            rounded
            color="grey-7"
            label="保存映射"
            :loading="saving"
            :disable="generating"
            @click="saveMappings"
          />
          <q-btn
            rounded
            color="pink-6"
            label="开始生成"
            icon="play_arrow"
            :loading="generating"
            :disable="chapterCharacters.length === 0"
            @click="startAudioGeneration"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useQuasar } from "quasar";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const $q = useQuasar();

interface ChapterItem {
  chapter_id: number;
  novel_id: number;
  novel_title: string;
  novel_author: string | null;
  chapter_index: number;
  chapter_title: string;
  tts_generated: boolean;
  scene_count: number;
}

interface ChapterTtsSummary {
  chapter_id: number;
  characters: string[];
  scene_count: number;
  dialogue_count: number;
  total_text_length: number;
}

interface ChapterGroup {
  novelId: number;
  novelTitle: string;
  novelAuthor: string | null;
  chapters: ChapterItem[];
}

interface CharacterVoice {
  id: number;
  character_name: string;
  api_base_url: string;
  ref_audio_path: string;
  prompt_text: string | null;
  prompt_lang: string;
  text_lang: string;
  gpt_model: string | null;
  sovits_model: string | null;
  config_path: string | null;
}

interface TtsAudioProgress {
  phase: string;
  total: number;
  processed: number;
  message: string;
}

const loading = ref(true);
const novelGroups = ref<ChapterGroup[]>([]);
const expandedChapter = ref<number | null>(null);
const chapterDetail = ref<Record<number, ChapterTtsSummary>>({});

// Dialog
const dialogVisible = ref(false);
const dialogChapter = ref<ChapterItem | null>(null);
const chapterCharacters = ref<string[]>([]);
const characterVoices = ref<CharacterVoice[]>([]);
const characterMappings = ref<Record<string, number | null>>({});
const saving = ref(false);

// Generation
const generating = ref(false);
const generatingChapterId = ref<number | null>(null);
const genProgress = ref<TtsAudioProgress>({ phase: "", total: 0, processed: 0, message: "" });

let unlistenProgress: (() => void) | null = null;

// 计算 select 选项
const voiceOptions = computed(() => {
  return characterVoices.value.map(v => ({
    label: `${v.character_name} (${v.api_base_url})`,
    value: v.id,
  }));
});


function groupByNovel(chapters: ChapterItem[]): ChapterGroup[] {
  const map = new Map<number, ChapterGroup>();
  for (const ch of chapters) {
    let group = map.get(ch.novel_id);
    if (!group) {
      group = {
        novelId: ch.novel_id,
        novelTitle: ch.novel_title,
        novelAuthor: ch.novel_author,
        chapters: [],
      };
      map.set(ch.novel_id, group);
    }
    group.chapters.push(ch);
  }
  return Array.from(map.values());
}

async function toggleChapter(ch: ChapterItem) {
  if (expandedChapter.value === ch.chapter_id) {
    expandedChapter.value = null;
    return;
  }
  expandedChapter.value = ch.chapter_id;
  if (!chapterDetail.value[ch.chapter_id]) {
    try {
      const detail = await invoke<ChapterTtsSummary>("get_chapter_tts_summary", { chapterId: ch.chapter_id });
      chapterDetail.value[ch.chapter_id] = detail;
    } catch (err: any) {
      console.error("加载章节详情失败:", err);
    }
  }
}

async function loadData() {
  loading.value = true;
  try {
    const chapters = await invoke<any[]>("get_tts_generated_chapters");
    novelGroups.value = groupByNovel(chapters.map(ch => ({
      ...ch,
      tts_generated: true,
    })) as ChapterItem[]);
  } catch (err: any) {
    console.error("加载失败:", err);
  } finally {
    loading.value = false;
  }
}

// ----- 角色语音配置对话框 -----

async function openVoiceDialog(ch: ChapterItem) {
  dialogChapter.value = ch;
  dialogVisible.value = true;
  genProgress.value = { phase: "", total: 0, processed: 0, message: "" };

  try {
    // 加载章节角色列表
    const detail = await invoke<ChapterTtsSummary>("get_chapter_tts_summary", { chapterId: ch.chapter_id });
    chapterCharacters.value = detail.characters;

    // 加载所有语音配置
    characterVoices.value = await invoke<CharacterVoice[]>("get_all_character_voices");

    // 加载已有映射
    const mappings = await invoke<any[]>("get_chapter_character_mappings", { chapterId: ch.chapter_id });
    const map: Record<string, number | null> = {};
    for (const m of mappings) {
      map[m.character_name] = m.voice_id;
    }
    characterMappings.value = map;
  } catch (err: any) {
    console.error("加载对话框数据失败:", err);
    $q.notify({ type: "negative", message: "加载数据失败: " + err, timeout: 3000 });
  }
}

function setMapping(charName: string, val: any) {
  characterMappings.value[charName] = val?.value ?? val ?? null;
}

async function saveMappings() {
  if (!dialogChapter.value) return;
  saving.value = true;
  try {
    const mappings: string[][] = [];
    for (const [charName, voiceId] of Object.entries(characterMappings.value)) {
      if (voiceId != null) {
        mappings.push([charName, String(voiceId)]);
      }
    }
    await invoke("save_chapter_character_mappings", {
      chapterId: dialogChapter.value.chapter_id,
      mappings,
    });
    $q.notify({ type: "positive", message: "映射已保存", timeout: 2000 });
  } catch (err: any) {
    console.error("保存映射失败:", err);
    $q.notify({ type: "negative", message: "保存失败: " + err, timeout: 3000 });
  } finally {
    saving.value = false;
  }
}

async function startAudioGeneration() {
  if (!dialogChapter.value) return;

  // 先保存映射
  await saveMappings();

  const ch = dialogChapter.value;
  generating.value = true;
  generatingChapterId.value = ch.chapter_id;

  // 监听进度事件
  unlistenProgress = await listen<TtsAudioProgress>("tts-audio-progress", (event) => {
    genProgress.value = event.payload;
    if (event.payload.phase === "complete") {
      $q.notify({ type: "positive", message: "音频生成完成", timeout: 3000 });
      cleanupGeneration();
    } else if (event.payload.phase === "error") {
      $q.notify({ type: "negative", message: event.payload.message, timeout: 5000 });
      cleanupGeneration();
    }
  });

  try {
    await invoke("generate_chapter_audio", { chapterId: ch.chapter_id });
  } catch (err: any) {
    console.error("启动音频生成失败:", err);
    $q.notify({ type: "negative", message: "启动失败: " + err, timeout: 3000 });
    cleanupGeneration();
  }
}

function cleanupGeneration() {
  generating.value = false;
  generatingChapterId.value = null;
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
}

onMounted(loadData);

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
});
</script>

<style scoped lang="scss">
.voice-card {
  border-radius: 10px;
  transition: background-color 0.15s;
}
.voice-card:hover {
  background-color: rgba(233, 30, 99, 0.03);
}
</style>
