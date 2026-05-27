<script setup lang="ts">
import { ref, reactive } from "vue";
import { useRouter } from 'vue-router';
import { open } from "@tauri-apps/plugin-dialog";
import { getConfigState, updataConfigState } from "../generated/commands";

const router = useRouter();

function goFoward() { router.forward(); }
function goBack() { router.back(); }

const search = ref("");

// ----- 设置 -----
const settingsVisible = ref(false);
const saving = ref(false);
const config = reactive({
  useCustomDir: false,
  novelPath: "",
  mp3Path: "",
  imagePath: "",
  gptSovitsPath: "",
});

async function openSettings() {
  try {
    const data = await getConfigState();
    Object.assign(config, data);
  } catch (err: any) {
    console.error("加载配置失败:", err);
  }
  settingsVisible.value = true;
}

async function saveSettings() {
  saving.value = true;
  try {
    await updataConfigState({ ...config });
    settingsVisible.value = false;
  } catch (err: any) {
    console.error("保存配置失败:", err);
  } finally {
    saving.value = false;
  }
}

async function pickDir(field: keyof typeof config) {
  try {
    const dir = await open({
      title: "选择目录",
      directory: true,
      multiple: false,
    });
    if (dir) (config as any)[field] = dir;
  } catch (err: any) {
    console.error("目录选择失败:", err);
  }
}
</script>

<template>
  <q-header class="bg-pink-8 text-white shadow-2">
    <q-toolbar class="border">
      <div class="q-px-md text-h6">o-storyteller</div>
      <q-btn round flat dense size="sm" @click="goBack">
        <q-icon name="arrow_back_ios" class="q-ml-sm"></q-icon>
      </q-btn>
      <q-btn class="q-ml-sm" size="sm" flat round dense @click="goFoward">
        <q-icon name="arrow_forward_ios"></q-icon>
      </q-btn>

      <q-space />

      <q-btn flat round dense icon="settings" class="q-ml-sm" @click="openSettings" />
      <q-btn flat round dense icon="notifications" class="q-ml-sm">
        <q-badge color="red" floating>3</q-badge>
      </q-btn>

      <q-input rounded outlined v-model="search" label="搜索" bg-color="pink-3" dense class="q-ml-sm" />

      <q-avatar color="red" text-color="white" icon="directions" class="q-ml-sm" />
    </q-toolbar>
  </q-header>

  <!-- 设置对话框 -->
  <q-dialog v-model="settingsVisible" persistent>
    <q-card style="min-width: 520px; max-width: 600px">
      <q-card-section class="q-pb-none">
        <div class="text-h6 text-pink-7">应用设置</div>
      </q-card-section>

      <q-card-section class="q-gutter-y-md">
        <q-input
          v-model="config.gptSovitsPath"
          label="GPT-SoVITS 引擎目录"
          outlined
          dense
          color="pink-6"
          placeholder="选择 GPT-SoVITS 引擎所在目录"
        >
          <template #append>
            <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickDir('gptSovitsPath')">
              <q-tooltip>选择目录</q-tooltip>
            </q-btn>
          </template>
        </q-input>

        <q-separator />

        <q-input
          v-model="config.novelPath"
          label="小说文件目录"
          outlined
          dense
          color="pink-6"
          placeholder="默认：应用数据目录"
        >
          <template #append>
            <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickDir('novelPath')">
              <q-tooltip>选择目录</q-tooltip>
            </q-btn>
          </template>
        </q-input>

        <q-input
          v-model="config.mp3Path"
          label="音频输出目录"
          outlined
          dense
          color="pink-6"
          placeholder="默认：应用数据目录"
        >
          <template #append>
            <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickDir('mp3Path')">
              <q-tooltip>选择目录</q-tooltip>
            </q-btn>
          </template>
        </q-input>

        <q-input
          v-model="config.imagePath"
          label="图片输出目录"
          outlined
          dense
          color="pink-6"
          placeholder="默认：应用数据目录"
        >
          <template #append>
            <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickDir('imagePath')">
              <q-tooltip>选择目录</q-tooltip>
            </q-btn>
          </template>
        </q-input>
      </q-card-section>

      <q-card-actions align="right" class="q-pa-md">
        <q-btn flat rounded label="取消" color="grey-6" v-close-popup />
        <q-btn rounded label="保存" color="pink-6" :loading="saving" @click="saveSettings" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
