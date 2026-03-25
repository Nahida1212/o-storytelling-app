<template>
  <q-drawer show-if-above :width="250" :breakpoint="700" class="">
    <q-scroll-area class="fit bg-pink-5">
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
          <q-item-section>{{ item.label }}</q-item-section>
        </q-item>
      </q-list>
    </q-scroll-area>
  </q-drawer>
</template>

<script setup>
// 菜单数据可以定义在组件内，或通过 props 传入，或从 store 获取
const menuItems = [
  { label: "书架", icon: "inbox", to: "/booklist" },
  { label: "生成队列", icon: "send", to: "/list" },
  { label: "角色vicoe", icon: "delete", to: "/character" },
  // 更多菜单项...
];
</script>
