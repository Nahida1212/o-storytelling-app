<script setup lang="ts">
import SidebarMenu from "../components/SidebarMenu.vue";
import HearderTitle from "../components/HearderTitle.vue";
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
      <q-footer class="text-white">
        <q-toolbar>
          <q-avatar>
            <img src="https://cdn.quasar.dev/logo-v2/svg/logo-mono-white.svg" />
          </q-avatar>
          <q-toolbar-title>播放器控制栏</q-toolbar-title>
          <q-btn flat round dense icon="play_arrow" />
          <q-btn flat round dense icon="pause" />
        </q-toolbar>
      </q-footer>
    </q-page-container>
  </q-layout>
</template>
<style scoped></style>

