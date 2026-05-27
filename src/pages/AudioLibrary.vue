<template>
  <q-page padding>
    <!-- 页面标题 -->
    <div class="row items-center q-mb-md">
      <div class="text-h5 text-weight-bold text-pink-7">语音播放</div>
      <q-space />
    </div>
    <div class="text-caption text-grey-6 q-mb-md" style="line-height: 1.5">
      浏览已生成语音的章节，选择并播放音频。
    </div>

    <!-- 加载 -->
    <div v-if="loading" class="text-center q-pa-lg">
      <q-spinner color="pink-6" size="40px" />
      <div class="q-mt-sm text-grey-6">加载中...</div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="novels.length === 0" class="text-center q-pa-lg">
      <q-icon name="headphones" color="grey-4" size="64px" />
      <div class="text-h6 text-grey-6 q-mt-md">暂无语音内容</div>
      <div class="text-caption text-grey-5 q-mt-sm">
        请先在"TTS 生成"页面为章节生成音频
      </div>
    </div>

    <!-- 按小说分组展示 -->
    <div v-else class="q-gutter-md">
      <q-card
        v-for="novel in novels"
        :key="novel.novel_id"
        bordered
        flat
      >
        <!-- 小说头部 -->
        <q-card-section class="bg-pink-1 q-py-sm">
          <div class="row items-center">
            <q-avatar rounded size="40px" class="q-mr-sm">
              <img
                v-if="novel.cover_image_path"
                :src="coverSrc(novel.cover_image_path)"
                @error="onCoverError($event)"
              />
              <q-icon v-else name="menu_book" color="pink-7" size="24px" />
            </q-avatar>
            <div>
              <div class="text-subtitle1 text-weight-bold text-pink-8">
                {{ novel.novel_title }}
              </div>
              <div v-if="novel.novel_author" class="text-caption text-grey-6">
                {{ novel.novel_author }}
              </div>
            </div>
            <q-space />
            <q-badge color="pink-4" rounded>
              {{ novel.chapter_count }} 章
            </q-badge>
          </div>
        </q-card-section>

        <q-separator />

        <!-- 章节列表 -->
        <div v-if="expandedNovel === novel.novel_id">
          <div
            v-for="ch in chapters[novel.novel_id]"
            :key="ch.chapter_id"
            class="row items-center no-wrap q-px-md q-py-sm"
            :class="{ 'bg-pink-1': hoverChapter === ch.chapter_id }"
            @mouseenter="hoverChapter = ch.chapter_id"
            @mouseleave="hoverChapter = null"
          >
            <q-icon name="article" color="pink-3" size="20px" class="q-mr-sm" />
            <div class="col">
              <div class="text-body2">
                第 {{ ch.chapter_index + 1 }} 章 · {{ ch.chapter_title }}
              </div>
              <div class="text-caption text-grey-6">
                {{ ch.scene_count }} 个场景
              </div>
            </div>
            <q-btn
              flat
              rounded
              color="pink-6"
              icon="play_arrow"
              label="播放"
              size="sm"
              @click="playChapter(ch)"
            />
          </div>
        </div>

        <!-- 展开/折叠按钮 -->
        <q-card-actions v-if="novel.chapter_count > 0" align="center" class="q-py-none">
          <q-btn
            flat
            dense
            size="sm"
            :icon="expandedNovel === novel.novel_id ? 'expand_less' : 'expand_more'"
            :label="expandedNovel === novel.novel_id ? '收起' : '展开章节'"
            color="pink-5"
            class="full-width"
            @click="toggleNovel(novel.novel_id)"
          />
        </q-card-actions>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

interface NovelWithAudioInfo {
  novel_id: number;
  novel_title: string;
  novel_author: string | null;
  cover_image_path: string | null;
  chapter_count: number;
}

interface ChapterWithAudioInfo {
  chapter_id: number;
  novel_id: number;
  chapter_index: number;
  chapter_title: string;
  scene_count: number;
}

const router = useRouter();

const loading = ref(true);
const novels = ref<NovelWithAudioInfo[]>([]);
const chapters = ref<Record<number, ChapterWithAudioInfo[]>>({});
const expandedNovel = ref<number | null>(null);
const hoverChapter = ref<number | null>(null);

onMounted(async () => {
  try {
    novels.value = await invoke<NovelWithAudioInfo[]>("get_novels_with_audio");
  } catch (err: any) {
    console.error("加载有声小说失败:", err);
  } finally {
    loading.value = false;
  }
});

function coverSrc(path: string): string {
  return convertFileSrc(path.replace(/\\/g, "/"));
}

function onCoverError(e: Event) {
  const img = e.target as HTMLImageElement;
  img.style.display = "none";
}

async function toggleNovel(novelId: number) {
  if (expandedNovel.value === novelId) {
    expandedNovel.value = null;
    return;
  }
  expandedNovel.value = novelId;

  if (!chapters.value[novelId]) {
    try {
      chapters.value[novelId] = await invoke<ChapterWithAudioInfo[]>(
        "get_chapters_with_audio",
        { novelId }
      );
    } catch (err: any) {
      console.error("加载章节列表失败:", err);
    }
  }
}

function playChapter(ch: ChapterWithAudioInfo) {
  console.log(`[AudioLibrary] 点击播放章节:`, { ...ch });
  console.log(`[AudioLibrary] 路由目标:`, { name: "audioPlayer", params: { chapterId: ch.chapter_id } });
  router.push({
    name: "audioPlayer",
    params: { chapterId: ch.chapter_id },
  });
}
</script>

<style scoped>
.q-card {
  border-radius: 12px;
  overflow: hidden;
}
</style>
