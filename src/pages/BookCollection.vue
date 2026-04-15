<!-- pages/LibraryPage.vue -->
<template>
  <q-page padding @click="clearSelection">
    <!-- 页面标题和操作栏 -->
    <div class="row items-center justify-between q-mb-md action-bar">
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
        <q-btn
          :color="selectionMode ? 'negative' : 'pink-6'"
          :label="selectionButtonLabel"
          @click.stop="toggleSelectionMode"
        />
      </div>
    </div>

    <!-- 图书card -->
    <div class="book-container row items-md-center q-mb-md q-gutter-md">

      <!-- 动态生成书籍卡片 -->
      <q-card
        v-for="book in books"
        :key="book.id"
        :class="['book-card', 'text-pink-7', { selected: isBookSelected(book.id) }]"
        @click="handleBookClick(book.id, $event)"
      >
        <img :src="getBookCover(book)" @error="onImageError">
        <q-linear-progress :value="0.5" color="pink-7" stripe animated />
        <q-card-section class="q-pa-sm" dense>
          <div class="text-subtitle2 text-weight-bolder text-pink-7 ellipsis" style="max-width: 100%;">
            {{ book.title }}
            <q-tooltip>
              {{ book.title }}
            </q-tooltip>
          </div>
          <div class="text-subtitle2 ellipsis" style="max-width: 100%;">
            {{ book.author || '未知作者' }}
          </div>
        </q-card-section>
      </q-card>

      <!-- 添加书籍卡片 -->
      <q-card class="book-card text-pink-8" @click.stop="clickLoad">
        <div class="column flex-center full-height">
          <q-icon name="add" size="48px" color="pink-8" />
          <div class="text-caption q-mt-sm">添加书籍</div>
        </div>
      </q-card>

    </div>

  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useQuasar } from 'quasar'
import { useRouter } from 'vue-router'
import testImage from "@/assets/images/9c72696a2815820251.gif"
import { open } from '@tauri-apps/plugin-dialog'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { getConfigState, updataConfigState } from '@/generated/commands'
import { AppConfig } from '@/generated/types'

const $q = useQuasar()


// 书籍数据（从数据库获取）
interface Book {
  id: number;
  title: string;
  author: string | null;
  file_path: string;
  cover_image_path: string | null;
}

const books = ref<Book[]>([])

// 多选状态
const selectionMode = ref(false)
const selectedBookIds = ref<number[]>([])
const selectedCount = computed(() => selectedBookIds.value.length)
const selectionButtonLabel = computed(() => {
  if (!selectionMode.value) return '多选'
  return selectedCount.value > 0 ? `删除 (${selectedCount.value})` : '删除'
})

// 状态
const router = useRouter()
const configInformation = ref<AppConfig | null>(null)
const pathInformation = ref()



// 进入book详情页面
function clickBook(bookId: number) {
  console.log("点击书籍:", bookId);
  router.push({
    name: 'bookDetail',
    params: { bookId }
  })
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
    console.log('导入成功:', res)
    // 导入成功后刷新书籍列表
    await fetchBooks()
    $q.notify({
      message: '导入成功',
      position: 'top',
      color: 'green'
    })
  } catch (e) {
    console.log(e);
    $q.notify({
      message: e as string,
      position: "top",
      color: "red"
    })
  }

}

// 切换选择模式/执行删除
async function toggleSelectionMode() {
  // 防止重复点击
  const win = window as any
  if (win.__toggleSelectionProcessing) {
    console.log('toggleSelectionMode 正在处理中，跳过')
    return
  }
  win.__toggleSelectionProcessing = true

  try {
    if (!selectionMode.value) {
      // 不在选择模式，点击"多选"按钮进入选择模式
      console.log('进入选择模式')
      selectionMode.value = true
      selectedBookIds.value = []
    } else {
      // 已经在选择模式
      if (selectedBookIds.value.length > 0) {
        // 有选中书籍，点击"删除"按钮，先弹出确认对话框
        console.log('点击删除按钮，选中书籍数量:', selectedBookIds.value.length)
        await deleteSelectedBooks() // 这个方法内部有确认对话框
      } else {
        // 没有选中书籍，点击"删除"按钮退出选择模式
        console.log('无选中书籍，退出选择模式')
        selectionMode.value = false
      }
    }
  } finally {
    setTimeout(() => {
      win.__toggleSelectionProcessing = false
    }, 100)
  }
}

// 删除选中的书籍
async function deleteSelectedBooks() {
  console.log('deleteSelectedBooks 开始，选中ID:', selectedBookIds.value)
  if (selectedBookIds.value.length === 0) {
    console.log('没有选中书籍')
    $q.notify({
      message: '请先选择要删除的书籍',
      position: 'top',
      color: 'warning'
    })
    return
  }

  // 使用 Quasar 对话框进行确认
  console.log('弹出确认对话框，检查 $q.dialog:', typeof $q.dialog, '$q对象:', $q)

  // 添加防重复标志
  const win = window as any
  if (win.__deletingBooks) {
    console.log('删除操作正在进行中，跳过重复调用')
    return
  }
  win.__deletingBooks = true

  try {
    // 检查 Dialog 插件是否可用
    if (typeof $q.dialog !== 'function') {
      console.error('Quasar Dialog 插件不可用，使用备用确认方式')
      const confirmed = window.confirm(`确定要删除选中的 ${selectedBookIds.value.length} 本书籍吗？`)
      if (!confirmed) {
        console.log('用户取消删除（备用方式）')
        return
      }
      console.log('用户确认删除（备用方式）')
    } else {
      console.log('准备显示 Quasar 对话框...')

      // 使用 Promise 包装对话框，等待用户操作
      const userConfirmed = await new Promise<boolean>((resolve) => {
        const dialog = $q.dialog({
          title: '确认删除',
          message: `确定要删除选中的 ${selectedBookIds.value.length} 本书籍吗？`,
          persistent: true,
          ok: {
            label: '确认删除',
            color: 'negative',
            flat: false
          },
          cancel: {
            label: '取消',
            color: 'grey',
            flat: true
          }
        })

        console.log('对话框对象:', dialog)

        // 监听用户确认
        dialog.onOk(() => {
          console.log('用户点击了确认删除')
          resolve(true)
        })

        // 监听用户取消
        dialog.onCancel(() => {
          console.log('用户点击了取消')
          resolve(false)
        })

        // 监听对话框关闭
        dialog.onDismiss(() => {
          console.log('对话框被关闭')
          resolve(false)
        })
      })

      console.log('用户确认结果:', userConfirmed)

      if (!userConfirmed) {
        console.log('用户取消删除')
        return
      }

      console.log('用户确认删除')
    }
  } catch (error) {
    // 对话框被取消或其他错误
    console.log('对话框取消或错误:', error)
    return
  } finally {
    win.__deletingBooks = false
  }

  console.log('开始调用Rust删除命令，bookIds:', selectedBookIds.value)
  // 调用删除命令
  await invoke('delete_books', { bookIds: selectedBookIds.value })
  console.log('Rust删除命令执行成功')
  // 删除成功后刷新书籍列表
  await fetchBooks()
  // 退出选择模式
  selectionMode.value = false
  selectedBookIds.value = []
  console.log('删除完成，已退出选择模式')
  $q.notify({
    message: '删除成功',
    position: 'top',
    color: 'green'
  })
}

// 处理书籍卡片点击
function handleBookClick(bookId: number, event: Event) {
  if (selectionMode.value) {
    toggleSelectBook(bookId, event)
  } else {
    clickBook(bookId)
  }
}

// 切换书籍选中状态
function toggleSelectBook(bookId: number, event: Event) {
  if (!selectionMode.value) return

  event.stopPropagation()
  const index = selectedBookIds.value.indexOf(bookId)
  if (index > -1) {
    // 已选中，移除
    selectedBookIds.value.splice(index, 1)
  } else {
    // 未选中，添加
    selectedBookIds.value.push(bookId)
  }
}

// 检查书籍是否被选中
function isBookSelected(bookId: number): boolean {
  return selectedBookIds.value.includes(bookId)
}

// 点击卡片外部清空选中并退出选择模式
function clearSelection(event: Event) {
  if (!selectionMode.value) return
  // 检查点击目标是否在卡片内或操作栏内
  const target = event.target as HTMLElement
  const isCard = target.closest('.book-card')
  const isActionBar = target.closest('.action-bar')
  // 如果点击的不是卡片也不是操作栏，则退出选择模式
  if (!isCard && !isActionBar) {
    selectedBookIds.value = []
    selectionMode.value = false
  }
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

// 从数据库获取所有书籍
async function fetchBooks() {
  try {
    const fetchedBooks = await invoke<Book[]>('get_all_books')
    books.value = fetchedBooks
    console.log('获取到书籍:', fetchedBooks)
  } catch (error) {
    console.error('获取书籍失败:', error)
    $q.notify({
      message: '获取书籍失败: ' + error,
      position: 'top',
      color: 'red'
    })
  }
}

// 获取书籍封面图片
function getBookCover(book: Book): string {
  // 确保 cover_image_path 存在且为非空字符串
  if (book.cover_image_path && book.cover_image_path.trim().length > 0) {
    // 标准化路径 - Windows 反斜杠转换为正斜杠
    const normalizedPath = book.cover_image_path.replace(/\\/g, '/');
    // 使用 Tauri 的 convertFileSrc 转换文件路径为安全的 URL
    const converted = convertFileSrc(normalizedPath);
    console.log('封面图片转换:', book.cover_image_path, '->', normalizedPath, '->', converted);
    return converted;
  }
  console.log('使用默认封面图片:', book.title);
  return testImage;
}

// 处理图片加载错误
function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  console.log('图片加载失败，使用默认图片:', img.src);
  img.src = testImage;
  // 防止事件继续传播
  event.preventDefault();
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
    // 加载书籍数据
    await fetchBooks()
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
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.book-card img {
  width: 100%;
  height: 220px;
  object-fit: cover;
  flex-shrink: 0;
}

.book-card .q-linear-progress {
  flex-shrink: 0;
}

.book-card .q-card-section {
  flex-grow: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  min-height: 60px;
}

.book-card .text-subtitle2 {
  line-height: 1.2;
  margin-bottom: 2px;
}

.book-card .ellipsis {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.book-card.selected {
  border: 3px solid #f48fb1; /* pink-4 */
  box-shadow: 0 0 15px rgba(244, 143, 177, 0.5);
  transform: translateY(-2px);
  transition: all 0.2s ease;
}
</style>