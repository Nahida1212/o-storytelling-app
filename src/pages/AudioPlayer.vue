<template>
  <q-page padding class="column player-page">
    <!-- 歌词滚动区域（全屏，无控制栏） -->
    <div class="col lyric-wrapper" ref="lyricWrapperRef">
      <div class="lyric-container">
        <div
          v-for="(line, idx) in player.state.lyrics"
          :key="line.id"
          class="lyric-line"
          :class="lyricClass(idx)"
          @click="player.seekTo(idx)"
        >
          <div v-if="line.characterName" class="character-label">
            {{ line.characterName }}
          </div>
          <div class="lyric-text">{{ line.content }}</div>
        </div>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { useAudioPlayer, SceneLyric } from "../composables/useAudioPlayer";

interface TtsSceneWithAudio {
  id: number;
  novel_id: number;
  chapter_id: number;
  scene_index: number;
  scene_type: string;
  content: string;
  character_name: string | null;
  emotion: string;
  speed: number;
  pause_duration: number;
  audio_path: string | null;
}

const route = useRoute();
const player = useAudioPlayer();

const lyricWrapperRef = ref<HTMLElement | null>(null);

const chapterTitle = ref("");
const novelTitle = ref("");

// 歌词行 CSS class
function lyricClass(idx: number): Record<string, boolean> {
  const cur = player.state.currentLyricIndex;
  return {
    past: idx < cur,
    current: idx === cur,
    future: idx > cur,
  };
}

/** 滚动容器使当前行居中 */
function scrollToCurrent() {
  const wrapper = lyricWrapperRef.value;
  if (!wrapper) return;
  const el = wrapper.querySelector(".lyric-line.current") as HTMLElement | null;
  if (!el) return;
  const targetTop = el.offsetTop + el.offsetHeight / 2 - wrapper.clientHeight / 2;
  wrapper.scrollTo({ top: Math.max(0, targetTop), behavior: "smooth" });
}

// 监听当前歌词行变化 → 自动滚动居中
watch(
  () => player.state.currentLyricIndex,
  () => {
    requestAnimationFrame(() => scrollToCurrent());
  }
);

onMounted(async () => {
  console.log(`[AudioPlayer] onMounted 触发`);
  console.log(`[AudioPlayer] route.params:`, route.params);
  console.log(`[AudioPlayer] route.fullPath:`, route.fullPath);

  const chapterId = Number(route.params.chapterId);
  console.log(`[AudioPlayer] chapterId:`, chapterId);
  if (!chapterId) {
    console.warn(`[AudioPlayer] chapterId 无效，跳过加载`);
    return;
  }

  try {
    console.log(`[AudioPlayer] 开始调用 get_chapter_scenes_with_audio, chapterId=${chapterId}`);
    const scenes = await invoke<TtsSceneWithAudio[]>("get_chapter_scenes_with_audio", {
      chapterId,
    });
    console.log(`[AudioPlayer] 获取到场景数量:`, scenes.length);
    if (scenes.length > 0) {
      console.log(`[AudioPlayer] 第一个场景:`, { ...scenes[0] });
    }

    if (scenes.length === 0) {
      console.warn(`[AudioPlayer] 该章节没有有音频的场景`);
      return;
    }

    // 从第一个场景获取小说/章节名（所有场景属于同一章节）
    chapterTitle.value = `第 ${scenes[0].scene_index} 章`;
    novelTitle.value = `小说 ${scenes[0].novel_id}`;

    // 尝试获取更友好的章节名
    try {
      const chapters = await invoke<any[]>("get_chapters_with_audio", {
        novelId: scenes[0].novel_id,
      });
      const ch = chapters.find((c: any) => c.chapter_id === chapterId);
      if (ch) {
        chapterTitle.value = ch.chapter_title || `第 ${ch.chapter_index + 1} 章`;
      }
    } catch {
      // 静默失败，用默认名
    }

    // 转换为 SceneLyric
    const lyricScenes: SceneLyric[] = scenes.map((s) => ({
      id: s.id,
      sceneIndex: s.scene_index,
      content: s.content,
      characterName: s.character_name,
      emotion: s.emotion,
      speed: s.speed,
      pauseDuration: s.pause_duration,
      audioPath: s.audio_path,
      estimatedDuration: 0,
      startTime: 0,
    }));

    player.playChapter(chapterId, chapterTitle.value, novelTitle.value, lyricScenes);
  } catch (err: any) {
    console.error("加载场景音频失败:", err);
  }
});

onUnmounted(() => {
  // 离开页面时停止播放
  player.stop();
});
</script>

<style scoped>
.player-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 歌词滚动容器 */
.lyric-wrapper {
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  position: relative;
}

.lyric-wrapper::-webkit-scrollbar {
  display: none;
}

.lyric-container {
  padding: 15% 0 15% 0;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
}

/* 歌词行 */
.lyric-line {
  text-align: center;
  padding: 6px 24px;
  cursor: pointer;
  transition: all 0.35s ease;
  user-select: none;
  min-height: 48px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.lyric-line:hover {
  background: rgba(233, 30, 99, 0.04);
  border-radius: 8px;
}

/* 角色标签 */
.character-label {
  display: inline-block;
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 10px;
  margin-bottom: 2px;
  background: rgba(233, 30, 99, 0.1);
  color: #e91e63;
  max-width: 80%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lyric-text {
  line-height: 1.5;
}

/* 已过行 */
.lyric-line.past .lyric-text {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.3);
  font-weight: 400;
}

/* 当前行 */
.lyric-line.current .lyric-text {
  font-size: 20px;
  font-weight: 700;
  color: #e91e63;
  transform: scale(1.05);
}

.lyric-line.current .character-label {
  background: #e91e63;
  color: #fff;
  font-weight: 600;
}

/* 未到行 */
.lyric-line.future .lyric-text {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.5);
  font-weight: 400;
}

/* 底部控制栏（已移除，保留样式作为占位） */
</style>
