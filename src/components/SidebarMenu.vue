<template>
  <q-drawer
    show-if-above
    :width="250"
    :breakpoint="700"
    :mini="miniState"
    bordered
    overlaid
    mini-to-overlay
    :mini-width="60"
  >
    <div class="fit bg-pink-5 column">
      <q-btn
        flat
        dense
        round
        :icon="miniState ? 'chevron_right' : 'chevron_left'"
        class="q-ma-sm text-white self-end"
        @click="miniState = !miniState"
      />
      <q-list padding class="bg-pink-5 text-white">
        <q-item
          v-for="item in menuItems"
          :key="item.label"
          clickable
          v-ripple
          :to="item.to"
          exact
          active-class="bg-pink-3 text-primary"
        >
          <q-item-section avatar>
            <q-icon :name="item.icon" />
          </q-item-section>
          <q-item-section>
            <q-item-label>{{ item.label }}</q-item-label>
          </q-item-section>
        </q-item>
      </q-list>
    </div>
  </q-drawer>
</template>

<script setup>
import { ref } from "vue";

const miniState = ref(true);

const menuItems = [
  { label: "书架", icon: "menu_book", to: "/booklist" },
  { label: "tts生成", icon: "record_voice_over", to: "/tts_generate" },
  { label: "角色vicoe", icon: "face", to: "/character" },
  { label: "语音播放", icon: "headphones", to: "/audio_library" },
];
</script>
