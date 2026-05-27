<script setup lang="ts">
import SidebarMenu from "../components/SidebarMenu.vue";
import HearderTitle from "../components/HearderTitle.vue";
import { useAudioPlayer } from "../composables/useAudioPlayer";
import { useRouter } from "vue-router";
import { watch } from "vue";

const player = useAudioPlayer();
const router = useRouter();

console.log("[MiniPlayer] 组件已挂载, 初始状态:", {
  currentChapterId: player.state.currentChapterId,
  isPlaying: player.state.isPlaying,
});

watch(
  () => player.state.currentChapterId,
  (val) => {
    console.log(`[MiniPlayer] currentChapterId 变化: ${val}`);
  }
);

watch(
  () => player.state.isPlaying,
  (val) => {
    console.log(`[MiniPlayer] isPlaying 变化: ${val}`);
  }
);

function onTogglePlay() {
  console.log(`[MiniPlayer] 点击播放/暂停, 当前状态: isPlaying=${player.state.isPlaying}`);
  player.togglePlay();
}

function onNextScene() {
  console.log(`[MiniPlayer] 点击下一句, 当前索引: ${player.state.currentLyricIndex}/${player.state.lyrics.length - 1}`);
  player.nextScene();
}

function goToPlayer() {
  if (player.state.currentChapterId) {
    console.log(`[MiniPlayer] 点击标题跳转到播放器 chapterId=${player.state.currentChapterId}`);
    router.push({
      name: "audioPlayer",
      params: { chapterId: player.state.currentChapterId },
    });
  }
}
</script>
<template>
  <q-layout view="hHh lpR fFf" class="layout">
    <!-- 导航栏 -->
    <HearderTitle></HearderTitle>
    <!-- 侧边栏：QDrawer 作为左侧抽屉 -->
    <SidebarMenu></SidebarMenu>

    <!-- 主页面区域：包含内容和底部播放器 -->
    <q-page-container class="">
      <!-- 内容页面 -->
      <router-view />

      <!-- 底部播放器：QFooter 固定在底部 -->
      <q-footer v-if="player.state.currentChapterId" class="text-white">
        <q-toolbar class="bg-pink-7">
          <q-avatar>
            <q-icon name="music_note" />
          </q-avatar>
          <q-toolbar-title class="cursor-pointer" @click="goToPlayer">
            <div class="text-weight-bold ellipsis">{{ player.state.currentChapterTitle }}</div>
            <div class="text-caption ellipsis">{{ player.state.currentNovelTitle }}</div>
          </q-toolbar-title>
          <q-btn
            flat
            round
            dense
            :icon="player.state.isPlaying ? 'pause' : 'play_arrow'"
            @click="onTogglePlay"
          />
          <q-btn flat round dense icon="skip_next" @click="onNextScene" />
        </q-toolbar>
      </q-footer>
    </q-page-container>
  </q-layout>
</template>
<style>
html, body {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
html::-webkit-scrollbar,
body::-webkit-scrollbar {
  display: none;
}
</style>

