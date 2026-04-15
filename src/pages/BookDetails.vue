<script lang="ts" setup>
import { ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

interface BookDetail {
  id: number;
  title: string;
  author: string;
  cover: string;
  cover_image_path: string | null;
  description: string;
  progress?: number;
}

interface Chapter {
  id: number;
  title: string;
  index: number;
}

const route = useRoute();
const router = useRouter();

const bookDetail = ref<BookDetail | null>(null);
const chapters = ref<Chapter[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

// 从路由参数获取书籍ID
const bookId = ref<number | null>(null);
if (route.params.bookId) {
  const id = parseInt(route.params.bookId as string, 10);
  console.log(id);
  
  if (!isNaN(id)) {
    bookId.value = id;
  }
}

// 加载书籍详情和章节
async function loadBookData() {
  if (!bookId.value) {
    error.value = "无效的书籍ID";
    loading.value = false;
    return;
  }

  try {
    loading.value = true;
    error.value = null;

    // 获取书籍详情
    const novelInfo: any = await invoke("get_book_details", { novelId: bookId.value });
    if (novelInfo) {
      const coverImagePath = novelInfo.cover_image_path;
      // 确保封面图片路径有效且非空
      let coverUrl = '';
      if (coverImagePath && coverImagePath.trim().length > 0) {
        // 标准化路径 - Windows 反斜杠转换为正斜杠
        const normalizedPath = coverImagePath.replace(/\\/g, '/');
        coverUrl = convertFileSrc(normalizedPath);
        console.log('书籍详情封面图片:', coverImagePath, '标准化:', normalizedPath, '转换后:', coverUrl);
      }
      bookDetail.value = {
        id: novelInfo.id,
        title: novelInfo.title,
        author: novelInfo.author || "未知作者",
        cover: coverUrl, // 转换后的 URL 或空字符串
        cover_image_path: coverImagePath,
        description: `文件路径: ${novelInfo.file_path}`, // 暂时用文件路径作为描述
        progress: 0, // 暂时为0，后续可以从数据库获取阅读进度
      };
    } else {
      error.value = "书籍不存在";
      return;
    }

    // 获取章节列表
    const chapterInfos: any[] = await invoke("get_book_chapters", { novelId: bookId.value });
    chapters.value = chapterInfos.map((ch: any) => ({
      id: ch.id,
      title: ch.title,
      index: ch.index,
    }));

  } catch (err: any) {
    error.value = `加载失败: ${err.message || err}`;
    console.error("加载书籍数据失败:", err);
  } finally {
    loading.value = false;
  }
}

// 点击章节跳转阅读
function readChapter(chapter: Chapter) {
  if (!bookDetail.value) return;
  router.push({
    path: "/readbook",
    query: { bookId: bookDetail.value.id, chapterId: chapter.id },
  });
}

// 开始阅读（从第一章节或上次阅读位置）
function startReading() {
  const firstChapter = chapters.value[0];
  if (firstChapter) {
    readChapter(firstChapter);
  }
}

// 处理图片加载错误
function onCoverImageError(event: Event) {
  console.log('封面图片加载失败:', event);
  // 对于 q-img，我们可能需要设置默认图片
  // 但由于 q-img 已经设置了默认图片，我们只需要记录错误
}

// 页面加载时获取数据
onMounted(() => {
  loadBookData();
});
</script>
<template>
  <q-page class="q-pa-md">
    <!-- 加载状态 -->
    <div v-if="loading" class="text-center q-pa-lg">
      <q-spinner color="pink-6" size="50px" />
      <div class="text-subtitle1 q-mt-md">加载中...</div>
    </div>

    <!-- 错误提示 -->
    <div v-else-if="error" class="text-center q-pa-lg">
      <q-icon name="error" color="negative" size="50px" />
      <div class="text-subtitle1 q-mt-md text-negative">{{ error }}</div>
      <q-btn
        class="q-mt-md"
        color="pink-6"
        rounded
        label="重试"
        @click="loadBookData"
      />
    </div>

    <!-- 书籍详情内容 -->
    <div v-else-if="bookDetail">
      <!-- 书籍信息卡片 -->
      <q-card class="book-info-card q-mb-md">
        <q-card-section horizontal>
          <q-img
            class="book-cover"
            :src="bookDetail.cover || 'https://cdn.quasar.dev/img/parallax2.jpg'"
            style="min-width: 200px; max-width: 240px; min-height: 320px"
            @error="onCoverImageError"
          />

          <q-card-section class="column q-pa-md">
            <div class="text-h5 text-weight-bold text-pink-7">
              {{ bookDetail.title || "默认书名" }}
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
          <div class="text-h6 text-weight-bold text-pink-7 q-px-sm">
            章节列表
            <q-badge color="pink-4" :label="chapters.length + '章'" class="q-ml-sm" />
          </div>
        </q-card-section>

        <q-separator color="pink-3" />

        <!-- 空状态 -->
        <div v-if="chapters.length === 0" class="text-center q-pa-lg">
          <q-icon name="menu_book" color="grey-5" size="50px" />
          <div class="text-subtitle1 q-mt-md text-grey-6">暂无章节</div>
        </div>

        <!-- 章节列表 -->
        <q-list separator class="chapter-list" v-else>
          <q-item
            v-for="chapter in chapters"
            :key="chapter.id"
            clickable
            v-ripple
            class="chapter-item"
            @click="readChapter(chapter)"
          >
            <q-item-section avatar>
              <q-badge color="pink-5" :label="chapter.index" rounded />
            </q-item-section>

            <q-item-section>
              <q-item-label class="text-body1 text-pink-7">
                {{ chapter.title }}
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <q-icon name="chevron_right" color="pink-4" />
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>

    <!-- 无数据状态 -->
    <div v-else class="text-center q-pa-lg">
      <q-icon name="book" color="grey-5" size="50px" />
      <div class="text-subtitle1 q-mt-md text-grey-6">书籍数据不存在</div>
      <q-btn
        class="q-mt-md"
        color="pink-6"
        rounded
        label="返回书架"
        @click="router.push('/booklist')"
      />
    </div>
  </q-page>
</template>
<style scoped lang="scss">
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
</style>
