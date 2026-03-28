<!-- pages/LibraryPage.vue -->
<template>
  <q-page padding>
    <!-- 页面标题和操作栏 -->
    <div class="row items-center justify-between q-mb-md">
      <div class="text-h5 text-pink-7 text-weight-bold">书架
        <div class="text-subtitle1">upload your book</div>

      </div>
      <div class=" q-gutter-md">
        <q-btn color="pink-1" label="导入书籍" @click="clickLoad" icon="upload" class=" text-pink-6" />
        <q-btn color="pink-6" label="存放路径" @click="selectSavePath">
          <q-tooltip>
            这里是导入的书籍的本地存放地址,所有你导入的书籍都将存放在这里
            当前存放路径 {{ pathInformation }}
          </q-tooltip>
        </q-btn>
        <q-btn color="pink-6" label="多选" @click="" />
      </div>
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
          <q-icon name="add" size="48px" color="pink-8" />
          <div class="text-caption q-mt-sm">添加书籍</div>
        </div>
      </q-card>




    </div>

  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import BookDetails from './BookDetails.vue'
import { useRouter } from 'vue-router'
import testImage from "@/assets/images/9c72696a2815820251.gif"
import testImage1 from "@/assets/images/smile.30bd11c.png"
import { message, open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { invoke } from '@tauri-apps/api/core'
import { fileUpload, getConfigState, updataConfigState } from '@/generated/commands'
import { AppConfig } from '@/generated/types'
import { show } from '@tauri-apps/api/app'
import { Notify } from 'quasar'
import { Position } from '@tauri-apps/api/dpi'

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
const router = useRouter()
const configInformation = ref<AppConfig | null>(null)
const pathInformation = ref()



// 进入book详情页面
function clickBook() {
  console.log("book");
  router.push("bookDetail")
}

// 导入书架
async function clickLoad() {
  const filePath = await selectFile()
  console.log("读取文件", filePath);
  if (filePath == null) {
    console.log("用户取消导入");
    return
  }
  try {

    const res = await invoke("file_upload", { file: filePath })
  } catch (e) {
    console.log(e);
    $q.notify({
      message: e,
      position: "top",
      color: "red"
    })
  }


}


// 删除书籍
const deleteBook = () => {

}

// 选择导入文件
async function selectFile() {
  // 打开文件选择对话框
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'epub,gif',
      extensions: ['epub', 'gif']
    }]
  })
  return selected
}

// 选择导入文件夹
async function selectSavePath() {
  const selected = await open({
    multiple: false,
    directory: true,
  })


  const config: AppConfig = await getConfigState()
  console.log("config", config);

    if (selected == null) {
      console.log("not select path")
      return
    } else {
      config.useCustomDir = true
      config.novelPath = selected
      pathInformation.value = selected
      
      updataConfigState(config)
    }


  return selected
}


onMounted(async () => {
  try {
    configInformation.value = await getConfigState()
    pathInformation.value = configInformation.value.novelPath
  } catch (error) {
    console.error('加载配置失败', error)
  }
})
</script>

<style scoped>
/* 可选：自定义样式 */
.book-card {
  width: 220px;
  height: 330px;

}
</style>