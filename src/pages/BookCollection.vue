<!-- pages/LibraryPage.vue -->
<template>
  <q-page padding>
    <!-- 页面标题和操作栏 -->
    <div class="row items-center justify-between q-mb-md">
      <div class="text-h5 text-pink-7 text-weight-bold">图书馆
        <div class="text-subtitle1">upload your book</div>

      </div>
      <q-btn color="pink-7" label="导入书籍" @click="clickLoad" icon="upload" />
    </div>

    <!-- 图书card -->
    <div class="book-container row items-md-center q-mb-md q-gutter-md">


      <q-card class="book-card text-pink-7" @click="clickBook" style="">
        <img :src="testImage">
        <q-linear-progress :value="0.5" color="pink-7" stripe animated />
        <q-card-section class="q-pa-sm" dense>
          <div class="text-subtitle2 text-weight-bolder text-pink-7">Our Changing Planet</div>
          <div class="text-subtitle2 ">by John Doe</div>
        </q-card-section>
        <q-card-section class="q-pa-sm" dense>
          Lorem ipsum dolor sit amet,
        </q-card-section>
      </q-card>


      <q-card class="book-card text-pink-8" @click="clickLoad">
        <div class="column flex-center full-height">
          <q-icon name="add" size="48px" color="pink-8"/>
          <div class="text-caption q-mt-sm">添加书籍</div>
        </div>
      </q-card>


      <q-card class="my-card">
        <img src="f">
        <q-card-section>
          <div class="text-h6">Our Changing Planet</div>
          <div class="text-subtitle2">by John Doe</div>
        </q-card-section>
        <q-card-section>
          Lorem ipsum dolor sit amet, consectetur adipiscing elit
        </q-card-section>
      </q-card>

    </div>

  </q-page>
</template>

<script setup>
import { ref } from 'vue'
import { useQuasar } from 'quasar'
import BookDetails from './BookDetails.vue'
import { useRouter } from 'vue-router'
import testImage from "@/assets/images/9c72696a2815820251.gif"
import testImage1 from "@/assets/images/smile.30bd11c.png"

const $q = useQuasar()

// 表格列定义
const columns = [
  { name: 'id', label: 'ID', field: 'id', align: 'left' },
  { name: 'title', label: '书名', field: 'title', align: 'left', sortable: true },
  { name: 'author', label: '作者', field: 'author', align: 'left', sortable: true },
  { name: 'isbn', label: 'ISBN', field: 'isbn', align: 'left' },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' }
]

// 书籍数据（模拟）
const books = ref([
  { id: 1, title: '三体', author: '刘慈欣', isbn: '978-7-5366-9293-0' },
  { id: 2, title: '活着', author: '余华', isbn: '978-7-5063-6549-1' }
])

// 状态
const loading = ref(false)
const importDialogVisible = ref(false)
const importing = ref(false)
const file = ref(null)
const previewData = ref([])
const router = useRouter()


// 导入书籍
async function clickLoad() {
  console.log("log");
  
}


// 进入book详情页面

function clickBook() {
  console.log("book");

  router.push("bookDetail")
}

// 预览文件内容（仅显示前3条）


// 执行导入
const importBooks = async () => {
}

// 删除书籍
const deleteBook = (id) => {
}
</script>

<style scoped>
/* 可选：自定义样式 */
.book-card {
  width: 220px;
  height: 330px;

}
</style>