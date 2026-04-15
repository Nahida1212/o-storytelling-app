<script setup lang="ts">
// import { route } from "quasar/wrappers"; // 未使用
import { ref } from "vue";
import { useRouter } from 'vue-router'
const router = useRouter();

function goFoward() {
  router.forward();
}
function goBack(){
  router.back()
}

let search = ref();
</script>
<template>
  <q-header class="bg-pink-8 text-white shadow-2">
    <q-toolbar class="border">
      <!-- 汉堡菜单按钮：用于切换左侧抽屉（如果抽屉可隐藏） -->

      <div class="q-px-md text-h6">o-storyteller</div>
      <q-btn round flat dense size="sm" @click="goBack">
        <q-icon name="arrow_back_ios" class="q-ml-sm"></q-icon>
      </q-btn>
      <q-btn class="q-ml-sm" size="sm" flat round dense @click="goFoward">
        <q-icon name="arrow_forward_ios"></q-icon>
      </q-btn>

      <q-space> </q-space>

      <q-btn flat round dense icon="settings" class="q-ml-sm" />
      <q-btn flat round dense icon="notifications" class="q-ml-sm">
        <q-badge color="red" floating>3</q-badge>
      </q-btn>

      <q-input rounded outlined v-model="search" label="搜索" bg-color="pink-3" dense class="q-ml-sm" />

      <q-avatar color="red" text-color="white" icon="directions" class="q-ml-sm" />
    </q-toolbar>
  </q-header>
</template>
