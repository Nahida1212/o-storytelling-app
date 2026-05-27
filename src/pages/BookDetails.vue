<template>
  <q-page padding>
    <!-- 加载状态 -->
    <div v-if="loading" class="text-center q-pa-lg">
      <q-spinner color="pink-6" size="50px" />
      <div class="text-subtitle1 q-mt-md">解析 EPUB 中...</div>
    </div>

    <!-- 错误提示 -->
    <div v-else-if="error" class="text-center q-pa-lg">
      <q-icon name="error" color="negative" size="50px" />
      <div class="text-subtitle1 q-mt-md text-negative">{{ error }}</div>
      <q-btn class="q-mt-md" color="pink-6" rounded label="重新选择" @click="openEpub" />
    </div>

    <!-- 未加载时显示打开按钮 -->
    <div v-else-if="!bookDetail" class="text-center q-pa-lg">
      <q-icon name="menu_book" color="grey-4" size="64px" />
      <div class="text-h6 text-grey-6 q-mt-md">选择一本 EPUB 书籍查看详情</div>
      <q-btn class="q-mt-md" color="pink-6" icon="upload_file" label="打开 EPUB" @click="openEpub" />
    </div>

    <!-- 书籍详情内容 -->
    <div v-else>


      <!-- 书籍信息卡片 -->
      <q-card class="book-info-card q-mb-md">
        <q-card-section horizontal>
          <q-img
            class="book-cover"
            :src="bookDetail.cover"
            style="min-width: 200px; max-width: 240px; min-height: 320px"
            @error="onCoverImageError"
          />

          <q-card-section class="column q-pa-md">
            <div class="text-h5 text-weight-bold text-pink-7">
              {{ bookDetail.title || "未知书名" }}
            </div>
            <div class="text-subtitle1 text-grey-7 q-mt-xs">
              作者：{{ bookDetail.author || "未知" }}
            </div>
            <div class="text-body2 text-grey-6 q-mt-sm">
              {{ bookDetail.description }}
            </div>

            <!-- 阅读进度 -->
            <div class="q-mt-md">
              <q-linear-progress
                :value="bookDetail.progress || 0"
                color="pink-6"
                size="8px"
                stripe
                animated
              />
              <div class="text-caption text-grey-6 q-mt-xs">
                阅读进度：{{ Math.round((bookDetail.progress || 0) * 100) }}%
              </div>
            </div>

            <q-btn
              class="q-mt-auto q-mt-md"
              color="pink-6"
              rounded
              label="开始阅读"
              icon="play_arrow"
              @click="startReading"
              :disable="chapters.length === 0"
            />
          </q-card-section>
        </q-card-section>
      </q-card>

      <!-- 章节列表 -->
      <q-card class="chapter-list-card">
        <q-card-section class="q-pa-sm">
          <div class="row items-center justify-between q-px-sm">
            <div class="text-h6 text-weight-bold text-pink-7">
              章节列表
              <q-badge color="pink-4" :label="chapters.length + '章'" class="q-ml-sm" />
            </div>
            <div class="row items-center q-gutter-sm">
              <q-btn
                v-if="!selectMode"
                dense
                flat
                rounded
                color="pink-6"
                icon="checklist"
                label="选择"
                @click="toggleSelectMode"
              />
              <template v-else>
                <q-btn
                  dense
                  flat
                  rounded
                  color="pink-6"
                  icon="close"
                  label="取消"
                  @click="toggleSelectMode"
                />
                <q-btn
                  rounded
                  color="pink-6"
                  label="提交"
                  padding="sm md"
                  :disable="selectedChapters.size === 0"
                  @click="submitSelection"
                />
              </template>
            </div>
          </div>
        </q-card-section>

        <q-separator color="pink-3" />

        <!-- 空状态 -->
        <div v-if="chapters.length === 0" class="text-center q-pa-lg">
          <q-icon name="menu_book" color="grey-5" size="50px" />
          <div class="text-subtitle1 q-mt-md text-grey-6">暂无章节</div>
        </div>

        <!-- 章节列表 -->

        <!-- TTS 生成进度 -->
        <div v-if="ttsProcessing" class="q-px-md q-py-sm bg-pink-1">
          <div class="row items-center q-mb-xs">
            <q-spinner color="pink-6" size="20px" class="q-mr-sm" />
            <span class="text-caption text-pink-8">{{ ttsProgress.message }}</span>
          </div>
          <q-linear-progress
            :value="ttsProgress.total > 0 ? ttsProgress.processed / ttsProgress.total : 0"
            color="pink-6"
            size="6px"
            stripe
            animated
          />
          <div v-if="ttsProgress.total > 0" class="text-caption text-grey-6 text-right">
            {{ ttsProgress.processed }} / {{ ttsProgress.total }}
          </div>
        </div>

        <q-list separator class="chapter-list" v-else>
          <q-item
            v-for="(chapter, idx) in chapters"
            :key="idx"
            :clickable="!selectMode"
            v-ripple
            class="chapter-item"
            :class="{
              'chapter-selected': selectedChapters.has(chapter.index),
              'chapter-tts-done': chapter.tts_generated && !selectMode,
            }"
            @click="selectMode ? toggleChapterSelection(chapter.index) : readChapter(chapter)"
          >
            <q-item-section avatar>
              <q-checkbox
                v-if="selectMode"
                :model-value="selectedChapters.has(chapter.index)"
                color="pink-6"
                @click.prevent="toggleChapterSelection(chapter.index)"
              />
              <q-badge v-else color="pink-5" :label="chapter.index + 1" rounded />
            </q-item-section>

            <q-item-section>
              <q-item-label class="text-body1 text-pink-7">
                {{ chapter.title }}
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <q-icon v-if="!selectMode" name="chevron_right" color="pink-4" />
              <q-icon
                v-else
                :name="selectedChapters.has(chapter.index) ? 'check_circle' : 'radio_button_unchecked'"
                :color="selectedChapters.has(chapter.index) ? 'pink-6' : 'grey-4'"
              />
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { parseEpub } from "@/tools/epub";
import type { ParsedChapter } from "@/tools/epub";

interface BookDetail {
  id: number;
  title: string;
  author: string;
  cover: string;
  description: string;
  progress: number;
}

interface TtsGenerationProgress {
  phase: string;
  total_chunks: number;
  processed_chunks: number;
  message: string;
}

const route = useRoute();
const router = useRouter();
const bookDetail = ref<BookDetail | null>(null);
const chapters = ref<ParsedChapter[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

// 多选状态
const selectMode = ref(false);
const selectedChapters = ref<Set<number>>(new Set());

// TTS 处理状态
const ttsProcessing = ref(false);
const ttsProgress = ref({ processed: 0, total: 0, message: "" });
let unlistenTts: (() => void) | null = null;

// 从 BookCollection 导航过来时，自动加载书籍数据
onMounted(async () => {
  const bookId = route.params.bookId as string;
  if (bookId) {
    await loadBook(parseInt(bookId));
  }
});

function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  if (!selectMode.value) {
    selectedChapters.value = new Set();
  }
}

function toggleChapterSelection(chapterIndex: number) {
  const newSet = new Set(selectedChapters.value);
  if (newSet.has(chapterIndex)) {
    newSet.delete(chapterIndex);
  } else {
    newSet.add(chapterIndex);
  }
  selectedChapters.value = newSet;
}

function submitSelection() {
  const indices = Array.from(selectedChapters.value);
  if (indices.length === 0) return;

  ttsProcessing.value = true;
  ttsProgress.value = { processed: 0, total: 0, message: "准备生成..." };

  // 监听 TTS 进度事件
  listen<TtsGenerationProgress>("tts-progress", (event) => {
    ttsProgress.value = {
      processed: event.payload.processed_chunks,
      total: event.payload.total_chunks,
      message: event.payload.message,
    };
    if (event.payload.phase === "complete" || event.payload.phase === "error") {
      ttsProcessing.value = false;
      if (unlistenTts) {
        unlistenTts();
        unlistenTts = null;
      }
    }
  }).then((unlisten) => {
    unlistenTts = unlisten;
  });

  invoke("start_tts_generation", {
    novelId: bookDetail.value!.id,
    chapterIds: indices,
  }).catch((err: any) => {
    ttsProcessing.value = false;
    ttsProgress.value.message = `启动失败: ${err}`;
    if (unlistenTts) {
      unlistenTts();
      unlistenTts = null;
    }
  });
}

onUnmounted(() => {
  if (unlistenTts) {
    unlistenTts();
    unlistenTts = null;
  }
});

function readChapter(chapter: ParsedChapter) {
  const bookId = route.params.bookId as string;
  if (!bookId) return;
  router.push({
    name: "reader",
    params: { bookId, chapterIndex: chapter.index },
  });
}

function startReading() {
  const first = chapters.value[0];
  if (first) readChapter(first);
}

function onCoverImageError() {
  // 封面加载失败时的处理
}

/** 从后端加载书籍详情和章节列表 */
async function loadBook(bookId: number) {
  loading.value = true;
  error.value = null;
  try {
    const [detail, backendChapters] = await Promise.all([
      invoke<any>("get_book_details", { novelId: bookId }),
      invoke<any[]>("get_book_chapters", { novelId: bookId }),
    ]);

    if (detail) {
      let coverUrl = "https://cdn.quasar.dev/img/parallax2.jpg";
      if (detail.cover_image_path) {
        coverUrl = convertFileSrc(detail.cover_image_path.replace(/\\/g, "/"));
      }

      bookDetail.value = {
        id: detail.id,
        title: detail.title,
        author: detail.author || "未知作者",
        cover: coverUrl,
        description: `${detail.file_path.split(/[/\\]/).pop() || ""} · ${backendChapters.length} 章`,
        progress: 0,
      };
    }

    chapters.value = backendChapters.map((ch: any) => ({
      index: ch.index,
      title: ch.title,
      content: ch.content || "",
      imageRefs: [],
      tts_generated: ch.tts_generated ?? false,
    }));

    console.log("[loadBook] bookDetail:", JSON.stringify(bookDetail.value));
    console.log("[loadBook] chapters:", JSON.stringify(chapters.value));
  } catch (err: any) {
    console.error("加载书籍失败:", err);
    error.value = `加载失败: ${err.message || err}`;
  } finally {
    loading.value = false;
  }
}

async function openEpub() {
  loading.value = true;
  error.value = null;
  bookDetail.value = null;
  chapters.value = [];

  try {
    const selected = await open({
      filters: [{ name: "EPUB 文件", extensions: ["epub"] }],
      multiple: false,
    });

    if (!selected) {
      loading.value = false;
      return;
    }

    const data = await readFile(selected);
    const fileName = selected.split(/[/\\]/).pop() || "unknown.epub";
    const result = await parseEpub(data.buffer as ArrayBuffer, fileName);

    bookDetail.value = {
      id: -1,
      title: result.fileName,
      author: result.metadata.author || "未知作者",
      cover: "null",
      description: `EPUB 文件 · ${result.chapters.length} 章`,
      progress: 0,
    };

    chapters.value = result.chapters;

    console.log("[openEpub] bookDetail:", JSON.stringify(bookDetail.value));
    console.log("[openEpub] chapters:", JSON.stringify(chapters.value));
    console.log(`解析完成: ${result.chapters.length} 章`);
  } catch (err: any) {
    console.error("解析 EPUB 失败:", err);
    error.value = `加载失败: ${err.message || err}`;
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped lang="scss">
.q-page {
  scrollbar-width: none;
  -ms-overflow-style: none;
  &::-webkit-scrollbar {
    display: none;
  }
}

.book-info-card {
  border-radius: 12px;
}

.book-cover {
  border-radius: 8px;
}

.chapter-list-card {
  border-radius: 12px;
}

.chapter-list {
  max-height: 400px;
  overflow-y: auto;
}

.chapter-item:hover {
  background-color: rgba(233, 30, 99, 0.05);
}

.chapter-selected {
  background-color: rgba(233, 30, 99, 0.08);
}

.chapter-selected:hover {
  background-color: rgba(233, 30, 99, 0.12);
}

.chapter-tts-done {
  background-color: rgba(76, 175, 80, 0.08);
}

.chapter-tts-done:hover {
  background-color: rgba(76, 175, 80, 0.14);
}
</style>
