import { reactive } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";

// ============================================================
// 歌词行数据结构
// ============================================================
export interface SceneLyric {
  id: number;
  sceneIndex: number;
  content: string;
  characterName: string | null;
  emotion: string;
  speed: number;
  pauseDuration: number;
  audioPath: string | null;

  // 运行时计算
  estimatedDuration: number; // 估算时长（秒）
  startTime: number; // 累计起始时间（秒）
}

// ============================================================
// 模块级单例状态（跨页面共享）
// ============================================================

interface PlayerState {
  currentChapterId: number | null;
  currentChapterTitle: string;
  currentNovelTitle: string;
  lyrics: SceneLyric[];
  currentLyricIndex: number;
  isPlaying: boolean;
  currentTime: number; // 当前播放进度（秒）
  duration: number; // 总时长（秒）
}

const state = reactive<PlayerState>({
  currentChapterId: null,
  currentChapterTitle: "",
  currentNovelTitle: "",
  lyrics: [],
  currentLyricIndex: 0,
  isPlaying: false,
  currentTime: 0,
  duration: 0,
});

let audioElement: HTMLAudioElement | null = null;
let rafId: number | null = null;

// ============================================================
// 工具函数
// ============================================================

/** 中文语速约 3.5 字/秒（speed=1.0 时），加上前置停顿 */
function estimateDuration(content: string, speed: number, pauseDuration: number): number {
  if (!content) return 0;
  const speakingTime = content.length / 3.5 / speed;
  return speakingTime + pauseDuration;
}

/** 格式化秒数为 mm:ss */
export function formatTime(seconds: number): string {
  if (seconds < 0 || !isFinite(seconds)) return "00:00";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

// ============================================================
// 内部函数
// ============================================================

function stopTimeTracking() {
  if (rafId !== null) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
}

function startTimeTracking() {
  stopTimeTracking();
  const scene = state.lyrics[state.currentLyricIndex];
  if (!scene) return;

  const baseTime = performance.now();
  const baseOffset = scene.startTime;

  function tick() {
    if (!state.isPlaying) return;
    const elapsed = (performance.now() - baseTime) / 1000;
    state.currentTime = baseOffset + elapsed;
    updateActiveLyric();
    rafId = requestAnimationFrame(tick);
  }
  rafId = requestAnimationFrame(tick);
}

function updateActiveLyric() {
  const t = state.currentTime;
  const list = state.lyrics;
  for (let i = list.length - 1; i >= 0; i--) {
    if (t >= list[i].startTime) {
      state.currentLyricIndex = i;
      return;
    }
  }
  state.currentLyricIndex = 0;
}

function playScene(index: number) {
  if (index >= state.lyrics.length) {
    // 全部播完
    stop();
    return;
  }

  const scene = state.lyrics[index];

  // 跳过无音频的场景
  if (!scene.audioPath) {
    state.currentLyricIndex = index;
    state.currentTime = scene.startTime;
    setTimeout(() => playScene(index + 1), 50);
    return;
  }

  stopAudio();

  state.currentLyricIndex = index;

  try {
    const src = convertFileSrc(scene.audioPath);
    audioElement = new Audio(src);
    audioElement.playbackRate = 1.0;

    audioElement.onended = () => {
      // 停止当前时间追踪
      stopTimeTracking();
      state.currentTime = scene.startTime + scene.estimatedDuration;
      updateActiveLyric();

      const pauseMs = scene.pauseDuration * 1000;

      // 在停顿期间继续推进 currentTime，让歌词进度条平滑过渡
      if (pauseMs > 0) {
        const pauseBase = performance.now();
        const pauseStartTime = scene.startTime + scene.estimatedDuration;
        function pauseTick() {
          const elapsed = (performance.now() - pauseBase) / 1000;
          state.currentTime = pauseStartTime + elapsed;
          updateActiveLyric();
          rafId = requestAnimationFrame(pauseTick);
        }
        rafId = requestAnimationFrame(pauseTick);
      }

      console.log(`[useAudioPlayer] 场景 ${index} 播完, pause_duration=${scene.pauseDuration}s 后播下一句`);
      setTimeout(() => {
        stopTimeTracking();
        playScene(index + 1);
      }, pauseMs);
    };

    audioElement.onerror = () => {
      console.warn(`[useAudioPlayer] 音频播放失败，跳过 scene_id=${scene.id}`, scene.audioPath);
      playScene(index + 1);
    };

    audioElement.play().then(() => {
      state.isPlaying = true;
      startTimeTracking();
    }).catch((err) => {
      console.warn(`[useAudioPlayer] play() 被拒绝:`, err);
      playScene(index + 1);
    });
  } catch (e) {
    console.error("[useAudioPlayer] 创建 Audio 失败:", e);
    playScene(index + 1);
  }
}

function stopAudio() {
  if (audioElement) {
    audioElement.pause();
    audioElement.onended = null;
    audioElement.onerror = null;
    audioElement = null;
  }
}

// ============================================================
// 公开 API
// ============================================================

export function useAudioPlayer() {
  /** 加载歌词并计算时间偏移 */
  function loadLyrics(scenes: SceneLyric[]) {
    let cumulative = 0;
    state.lyrics = scenes.map((s) => {
      const dur = estimateDuration(s.content, s.speed, s.pauseDuration);
      const entry: SceneLyric = { ...s, estimatedDuration: dur, startTime: cumulative };
      cumulative += dur;
      return entry;
    });
    state.duration = cumulative;
    state.currentLyricIndex = 0;
    state.currentTime = 0;
  }

  /** 播放指定章节的所有场景 */
  function playChapter(
    chapterId: number,
    chapterTitle: string,
    novelTitle: string,
    scenes: SceneLyric[]
  ) {
    stop();
    loadLyrics(scenes);
    if (state.lyrics.length === 0) return;

    state.currentChapterId = chapterId;
    state.currentChapterTitle = chapterTitle;
    state.currentNovelTitle = novelTitle;

    playScene(0);
  }

  /** 暂停 */
  function pause() {
    if (audioElement) {
      audioElement.pause();
    }
    state.isPlaying = false;
    stopTimeTracking();
  }

  /** 恢复播放 */
  function resume() {
    if (audioElement && audioElement.src) {
      audioElement.play().then(() => {
        state.isPlaying = true;
        startTimeTracking();
      }).catch((err) => {
        console.warn("[useAudioPlayer] resume() 被拒绝:", err);
      });
    }
  }

  /** 停止 */
  function stop() {
    stopAudio();
    state.isPlaying = false;
    stopTimeTracking();
  }

  /** 跳转到指定歌词行 */
  function seekTo(index: number) {
    if (index < 0 || index >= state.lyrics.length) return;
    playScene(index);
  }

  /** 下一句 */
  function nextScene() {
    seekTo(state.currentLyricIndex + 1);
  }

  /** 上一句 */
  function prevScene() {
    seekTo(Math.max(0, state.currentLyricIndex - 1));
  }

  /** 切换播放/暂停 */
  function togglePlay() {
    if (state.isPlaying) {
      pause();
    } else {
      resume();
    }
  }

  return {
    state,

    loadLyrics,
    playChapter,
    pause,
    resume,
    stop,
    seekTo,
    nextScene,
    prevScene,
    togglePlay,
  };
}
