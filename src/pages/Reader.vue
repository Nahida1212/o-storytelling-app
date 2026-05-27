<template>
  <q-page padding class="reader-page">
    <!-- 顶部栏 -->
    <div class="reader-header row items-center q-mb-md">
      <q-btn flat dense rounded icon="arrow_back" color="pink-6" @click="goBack" />
      <div class="col q-ml-sm">
        <div class="text-subtitle1 text-weight-bold text-pink-7 ellipsis">{{ chapter?.title || '加载中...' }}</div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="text-center q-pa-lg">
      <q-spinner color="pink-6" size="40px" />
      <div class="q-mt-sm text-grey-6">加载中...</div>
    </div>

    <!-- 内容区 -->
    <div v-else-if="chapter" class="reader-content">
      <div class="chapter-title text-center q-mb-lg">
        <div class="text-h5 text-weight-bold text-pink-7">{{ chapter.title }}</div>
        <div class="text-caption text-grey-5 q-mt-sm">第 {{ chapterIndex + 1 }} 章</div>
      </div>

      <div class="chapter-body text-body1">
        {{ chapter.content }}
      </div>

      <!-- 章节导航 -->
      <div class="reader-nav row justify-between q-mt-xl q-mb-lg">
        <q-btn
          :disable="chapterIndex <= 0"
          flat
          rounded
          color="pink-6"
          icon="chevron_left"
          label="上一章"
          @click="prevChapter"
        />
        <q-btn
          :disable="!hasNext"
          flat
          rounded
          color="pink-6"
          icon="chevron_right"
          label="下一章"
          @click="nextChapter"
        />
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading" class="text-center q-pa-lg">
      <q-icon name="menu_book" color="grey-4" size="64px" />
      <div class="text-h6 text-grey-6 q-mt-md">未找到章节内容</div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";

interface ChapterContent {
  id: number;
  title: string;
  index: number;
  content: string | null;
  tts_generated: boolean;
}

const route = useRoute();
const router = useRouter();

const loading = ref(false);
const chapter = ref<ChapterContent | null>(null);
const totalChapters = ref(0);

const bookId = computed(() => Number(route.params.bookId));
const chapterIndex = computed(() => Number(route.params.chapterIndex));

const hasNext = computed(() => chapterIndex.value + 1 < totalChapters.value);

async function loadChapter() {
  loading.value = true;
  chapter.value = null;
  try {
    const result = await invoke<ChapterContent | null>("get_chapter_content", {
      novelId: bookId.value,
      chapterIndex: chapterIndex.value,
    });
    chapter.value = result;

    // 获取总章节数以判断是否有下一章
    const chapters = await invoke<ChapterContent[]>("get_book_chapters", {
      novelId: bookId.value,
    });
    totalChapters.value = chapters.length;
  } catch (err: any) {
    console.error("加载章节失败:", err);
  } finally {
    loading.value = false;
  }
}

function goBack() {
  router.push({ name: "bookDetail", params: { bookId: bookId.value } });
}

function prevChapter() {
  if (chapterIndex.value > 0) {
    router.push({
      name: "reader",
      params: { bookId: bookId.value, chapterIndex: chapterIndex.value - 1 },
    });
  }
}

function nextChapter() {
  router.push({
    name: "reader",
    params: { bookId: bookId.value, chapterIndex: chapterIndex.value + 1 },
  });
}

// 路由参数变化时重新加载
watch([bookId, chapterIndex], () => {
  loadChapter();
});

onMounted(() => {
  loadChapter();
});
</script>

<style scoped>
.reader-page {
  max-width: 800px;
  margin: 0 auto;
  min-height: 100vh;
}

.reader-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background: white;
  padding: 4px 0;
}

.chapter-title {
  padding: 20px 0;
  border-bottom: 1px solid rgba(233, 30, 99, 0.1);
}

.chapter-body {
  line-height: 2;
  text-align: justify;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: rgba(0, 0, 0, 0.85);
  padding: 0 4px;
}

.reader-nav {
  border-top: 1px solid rgba(233, 30, 99, 0.1);
  padding-top: 16px;
}
</style>
