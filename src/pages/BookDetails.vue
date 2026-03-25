<script lang="ts" setup>
import { ref } from "vue";

interface BookDetail {
  id: number;
  title: string;
  author: string;
  cover: string;
  description: string;
  progress?: number;
}
let BookDetail = ref<BookDetail | null>;

let testBookDetail = ref<BookDetail>();
testBookDetail.value = {
  id: 1,
  title: "恋人不行",
  author: "人间入见",
  cover: "fssfsfsfs",
  description: "百合轻小说",
  progress: 0.3,
};
</script>
<template>
  <div class="container q-pa-md text-pink-5 bg-pink-2">
    <q-card class="img-card">
      <div class="q-rp-sm">
        <q-card-section horizontal>
          <q-img
            class="col-4 rounded-borders"
            src="https://cdn.quasar.dev/img/parallax2.jpg"
            style="min-width: 200px; border-radius: 4px; min-height: 400px"
          />

          <q-card-section class="col-4 column">
            <div class="text-h5 text-weight-bold">
              {{ testBookDetail?.title || "默认书名" }}
            </div>
            <div class="text-caption text-grey">
              这里可以放一些书籍的简介或标签...
            </div>

            <q-btn class="q-mt-auto" rounded>开始阅读</q-btn>
          </q-card-section>
        </q-card-section>
      </div>
    </q-card>
    <q-card class="my-card"></q-card>
    <q-card class="chapter-list">
      <div class="q-mt-md">这里是李彪</div>
    </q-card>
  </div>
</template>
<style scoped lang="scss"></style>
