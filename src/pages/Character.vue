<template>
  <q-page padding>
    <div class="row items-center q-mb-md">
      <div class="text-h5 text-weight-bold text-pink-7">角色语音配置</div>
      <q-space />
      <q-btn
        rounded
        color="pink-6"
        icon="add"
        label="添加角色"
        @click="openAddDialog"
      />
    </div>

    <!-- 加载 -->
    <div v-if="loading" class="text-center q-pa-lg">
      <q-spinner color="pink-6" size="40px" />
      <div class="q-mt-sm text-grey-6">加载中...</div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="voices.length === 0" class="text-center q-pa-lg">
      <q-icon name="record_voice_over" color="grey-4" size="64px" />
      <div class="text-h6 text-grey-6 q-mt-md">暂无角色配置</div>
      <div class="text-caption text-grey-5">点击右上角"添加角色"创建语音配置</div>
    </div>

    <!-- 角色列表 -->
    <div v-else class="q-gutter-sm">
      <q-card
        v-for="voice in voices"
        :key="voice.id"
        class="voice-card"
        bordered
        flat
      >
        <q-card-section class="q-py-sm">
          <div class="row items-center no-wrap">
            <q-avatar size="44px" color="pink-2" text-color="pink-7" class="q-mr-md">
              {{ voice.character_name.charAt(0) }}
            </q-avatar>
            <div class="column col">
              <div class="row items-center q-gutter-sm">
                <div class="text-subtitle1 text-weight-bold text-pink-7">
                  {{ voice.character_name }}
                </div>
                <q-chip
                  v-if="ttsStatus[voice.id!] === 'running'"
                  dense
                  size="12px"
                  color="positive"
                  text-color="white"
                  icon="check_circle"
                  class="q-ml-sm"
                >
                  running
                </q-chip>
                <q-chip
                  v-else-if="ttsStatus[voice.id!] === 'stopped'"
                  dense
                  size="12px"
                  color="grey-5"
                  text-color="white"
                  icon="power_off"
                  class="q-ml-sm"
                >
                  stopped
                </q-chip>
                <q-chip
                  v-else
                  dense
                  size="12px"
                  color="grey-3"
                  text-color="grey-6"
                  icon="hourglass_empty"
                  class="q-ml-sm"
                >
                  checking
                </q-chip>
              </div>
              <div class="row items-center q-gutter-x-md text-caption text-grey-6">
                <span class="text-no-wrap">
                  <q-icon name="audio_file" size="14px" class="q-mr-xs" />
                  {{ voice.ref_audio_path.split(/[/\\]/).pop() || voice.ref_audio_path }}
                </span>
                <span class="text-no-wrap">{{ voice.prompt_lang }} → {{ voice.text_lang }}</span>
              </div>
              <div class="row items-center q-gutter-x-md text-caption text-grey-6">
                <span v-if="voice.gpt_model" class="text-no-wrap">GPT: {{ voice.gpt_model }}</span>
                <span v-if="voice.sovits_model" class="text-no-wrap">SoVITS: {{ voice.sovits_model }}</span>
                <span v-if="voice.config_path" class="text-no-wrap text-grey-5">
                  <q-icon name="settings" size="14px" class="q-mr-xs" />
                  {{ voice.config_path.split(/[/\\]/).pop() }}
                </span>
              </div>
              <div class="text-caption text-grey-5">{{ voice.api_base_url }}</div>
            </div>
          </div>
        </q-card-section>
        <q-separator />
        <q-card-actions class="q-px-md q-py-sm justify-around">
          <q-btn
            flat
            rounded
            color="info"
            icon="wifi_find"
            :disable="ttsStatus[voice.id!] === 'checking'"
            @click="testConnection(voice)"
          >
            <q-tooltip>测试连通性</q-tooltip>
          </q-btn>
          <q-btn
            flat
            rounded
            color="positive"
            icon="play_arrow"
            :loading="engineStatus[voice.id!] === 'starting'"
            :disable="ttsStatus[voice.id!] === 'running'"
            @click="startEngine(voice)"
          >
            <q-tooltip>启动引擎</q-tooltip>
          </q-btn>
          <q-btn
            flat
            rounded
            color="negative"
            icon="stop"
            :disable="ttsStatus[voice.id!] !== 'running'"
            @click="stopEngine(voice)"
          >
            <q-tooltip>停止引擎</q-tooltip>
          </q-btn>
          <q-btn
            flat
            rounded
            color="pink-5"
            icon="mood"
            @click="openEmotionDialog(voice)"
          >
            <q-tooltip>情绪参考</q-tooltip>
          </q-btn>
          <q-btn
            flat
            rounded
            color="grey-6"
            icon="edit"
            @click="openEditDialog(voice)"
          >
            <q-tooltip>编辑</q-tooltip>
          </q-btn>
          <q-btn
            flat
            rounded
            color="negative"
            icon="delete"
            @click="confirmDelete(voice)"
          >
            <q-tooltip>删除</q-tooltip>
          </q-btn>
        </q-card-actions>
      </q-card>
    </div>

    <!-- 添加/编辑对话框 -->
    <q-dialog v-model="dialogVisible" persistent>
      <q-card style="min-width: 560px; max-width: 640px">
        <q-card-section class="q-pb-none">
          <div class="text-h6 text-pink-7">
            {{ editingVoice ? '编辑角色语音' : '添加角色语音' }}
          </div>
        </q-card-section>

        <q-card-section class="q-gutter-y-md">
          <q-input
            v-model="form.character_name"
            label="角色名 *"
            outlined
            dense
            color="pink-6"
            placeholder="请输入角色名称"
            :rules="[val => !!val || '角色名不能为空']"
          />

          <q-input
            v-model="form.ref_audio_path"
            label="参考音频路径 *"
            outlined
            dense
            color="pink-6"
            placeholder="选择或输入参考音频文件路径"
            :rules="[val => !!val || '参考音频不能为空']"
          >
            <template #append>
              <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickAudioFile">
                <q-tooltip>选择音频文件</q-tooltip>
              </q-btn>
            </template>
          </q-input>

          <q-input
            v-model="form.prompt_text"
            label="参考音频对应文本"
            outlined
            dense
            color="pink-6"
            placeholder="参考音频的原始文本（可选）"
          />

          <div class="row q-gutter-md">
            <q-select
              v-model="form.prompt_lang"
              :options="langOptions"
              label="参考音频语言"
              outlined
              dense
              color="pink-6"
              style="min-width: 140px"
            />
            <q-select
              v-model="form.text_lang"
              :options="langOptions"
              label="合成文本语言"
              outlined
              dense
              color="pink-6"
              style="min-width: 140px"
            />
          </div>

          <div class="row q-gutter-md">
            <q-input
              v-model="form.gpt_model"
              label="GPT 模型（可选）"
              outlined
              dense
              color="pink-6"
              placeholder="选择或输入 GPT 模型文件 (.ckpt)"
              style="min-width: 220px"
            >
              <template #append>
                <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickGptModel">
                  <q-tooltip>选择 GPT 模型文件</q-tooltip>
                </q-btn>
              </template>
            </q-input>
            <q-input
              v-model="form.sovits_model"
              label="SoVITS 模型（可选）"
              outlined
              dense
              color="pink-6"
              placeholder="选择或输入 SoVITS 模型文件 (.pth)"
              style="min-width: 220px"
            >
              <template #append>
                <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickSovitsModel">
                  <q-tooltip>选择 SoVITS 模型文件</q-tooltip>
                </q-btn>
              </template>
            </q-input>
          </div>

          <q-input
            v-model="form.api_base_url"
            label="API 地址"
            outlined
            dense
            color="pink-6"
            placeholder="http://127.0.0.1:9880"
          />

          <!-- config_path 折叠区域 -->
          <q-expansion-item
            dense
            dense-toggle
            expand-separator
            icon="settings"
            label="tts_infer.yaml 配置路径（可选）"
            caption="通过 YAML 文件统一配置模型和参考音频"
          >
            <q-card class="q-mt-sm">
              <q-card-section class="q-pa-sm">
                <q-input
                  v-model="form.config_path"
                  label="YAML 文件路径"
                  outlined
                  dense
                  color="pink-6"
                  placeholder="如: D:/GPT-SoVITS/configs/角色名.yaml"
                >
                  <template #append>
                    <q-btn flat dense round color="pink-6" icon="folder_open" @click="pickYamlFile">
                      <q-tooltip>选择 YAML 文件</q-tooltip>
                    </q-btn>
                  </template>
                </q-input>
                <div class="text-caption text-grey-5 q-mt-xs q-ml-xs">
                  设置此项后，API 将使用此配置文件中的参数
                </div>
              </q-card-section>
            </q-card>
          </q-expansion-item>
        </q-card-section>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn flat rounded label="取消" color="grey-6" v-close-popup />
          <q-btn
            rounded
            :label="editingVoice ? '保存' : '添加'"
            color="pink-6"
            :loading="saving"
            :disable="!form.character_name || !form.ref_audio_path"
            @click="saveVoice"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 情绪参考对话框 -->
    <q-dialog v-model="emotionDialogVisible" persistent>
      <q-card style="min-width: 520px; max-width: 600px">
        <q-card-section class="q-pb-none">
          <div class="text-h6 text-pink-7">
            情绪参考 — {{ editingEmotionVoice?.character_name }}
          </div>
          <div class="text-caption text-grey-5 q-mt-xs">
            为每种情绪指定参考音频，TTS 合成时据此调整语气
          </div>
        </q-card-section>

        <q-card-section class="q-gutter-y-sm">
          <div
            v-for="(emotion, index) in emotionList"
            :key="emotion"
            class="row items-center q-gutter-sm"
          >
            <q-chip dense :color="emotionColors[index]" text-color="white" style="min-width: 64px">
              {{ emotion }}
            </q-chip>
            <q-input
              :model-value="emotionAudio(emotion)"
              dense
              outlined
              color="pink-6"
              placeholder="选择参考音频文件"
              class="col"
              @update:model-value="val => setEmotionAudio(emotion, val)"
            >
              <template #append>
                <q-btn
                  flat
                  dense
                  round
                  color="pink-6"
                  icon="folder_open"
                  @click="pickEmotionAudio(emotion)"
                >
                  <q-tooltip>选择音频文件</q-tooltip>
                </q-btn>
              </template>
            </q-input>
            <q-btn
              flat
              dense
              round
              color="negative"
              icon="clear"
              size="sm"
              @click="clearEmotionAudio(emotion)"
            >
              <q-tooltip>清除</q-tooltip>
            </q-btn>
          </div>
        </q-card-section>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn flat rounded label="关闭" color="grey-6" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { useQuasar } from "quasar";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const $q = useQuasar();

interface CharacterVoice {
  id?: number;
  character_name: string;
  ref_audio_path: string;
  prompt_text: string | null;
  prompt_lang: string;
  text_lang: string;
  gpt_model: string | null;
  sovits_model: string | null;
  api_base_url: string;
  config_path: string | null;
}

const loading = ref(true);
const voices = ref<CharacterVoice[]>([]);
const ttsStatus = reactive<Record<number, string>>({});
const engineStatus = reactive<Record<number, string>>({});

// 情绪参考（前端存储，不写入后端）
const emotionList = ["neutral", "happy", "sad", "angry", "fearful", "surprised"];
const emotionColors = ["grey-6", "positive", "primary", "negative", "deep-purple-5", "amber-7"];
const emotionDialogVisible = ref(false);
const editingEmotionVoice = ref<CharacterVoice | null>(null);
const emotionRefs = ref({} as Record<number, Record<string, string>>);


const langOptions = ["zh", "en", "ja"];

const dialogVisible = ref(false);
const editingVoice = ref<CharacterVoice | null>(null);
const saving = ref(false);

const emptyForm = (): CharacterVoice => ({
  character_name: "",
  ref_audio_path: "",
  prompt_text: null,
  prompt_lang: "zh",
  text_lang: "zh",
  gpt_model: null,
  sovits_model: null,
  api_base_url: "http://127.0.0.1:9880",
  config_path: null,
});

const form = reactive<CharacterVoice>(emptyForm());

async function loadData() {
  loading.value = true;
  try {
    voices.value = await invoke<CharacterVoice[]>("get_all_character_voices");
    // 初始状态设为 stopped，不自动轮询
    for (const voice of voices.value) {
      if (voice.id != null) {
        ttsStatus[voice.id] = "stopped";
      }
    }
  } catch (err: any) {
    console.error("加载失败:", err);
  } finally {
    loading.value = false;
  }
}

async function checkTtsStatus(voice: CharacterVoice, maxRetries = 20) {
  const id = voice.id;
  if (id == null) return;
  const base = voice.api_base_url.replace(/\/+$/, "");
  for (let i = 0; i < maxRetries; i++) {
    // 外部已停止（如点了停止按钮），退出轮询
    if (ttsStatus[id] === 'stopped') return;

    ttsStatus[id] = "checking";
    try {
      const res = await fetch(`${base}/list_models`, { signal: AbortSignal.timeout(3000) });
      if (res.ok) {
        ttsStatus[id] = "running";
        $q.notify({ type: "positive", message: `引擎已就绪 — ${voice.character_name}`, timeout: 2000 });
        return;
      }
    } catch {
      // 服务尚未就绪，继续轮询
    }
    await new Promise(r => setTimeout(r, 3000));
  }
  ttsStatus[id] = "stopped";
  $q.notify({ type: "negative", message: `引擎启动超时 — ${voice.character_name}，请检查日志`, timeout: 5000 });
}

async function startEngine(voice: CharacterVoice) {
  const id = voice.id;
  if (id == null) return;
  if (engineStatus[id] === 'starting' || ttsStatus[id] === 'running') return;
  engineStatus[id] = 'starting';
  $q.notify({ type: "info", message: `正在启动引擎 — ${voice.character_name}`, timeout: 2000 });
  try {
    await invoke("start_engine", { characterId: id });
    engineStatus[id] = 'running';
    ttsStatus[id] = "checking";
    $q.notify({ type: "info", message: `进程已启动，等待 API 就绪... ${voice.character_name}`, timeout: 3000 });
    await checkTtsStatus(voice);
  } catch (err: any) {
    console.error("启动引擎失败:", err);
    engineStatus[id] = 'stopped';
    ttsStatus[id] = 'stopped';
    $q.notify({ type: "negative", message: `启动失败: ${err}`, timeout: 5000 });
  }
}

async function stopEngine(voice: CharacterVoice) {
  const id = voice.id;
  if (id == null) return;
  try {
    await invoke("stop_engine", { characterId: id });
    engineStatus[id] = 'stopped';
    ttsStatus[id] = "stopped";
  } catch (err: any) {
    console.error("停止引擎失败:", err);
  }
}

async function testConnection(voice: CharacterVoice) {
  const id = voice.id;
  if (id == null) return;
  ttsStatus[id] = "checking";
  const base = voice.api_base_url.replace(/\/+$/, "");
  try {
    const res = await fetch(`${base}/list_models`, { signal: AbortSignal.timeout(5000) });
    if (res.ok) {
      ttsStatus[id] = "running";
      $q.notify({ type: "positive", message: `引擎连接正常 — ${voice.character_name}`, timeout: 2000 });
    } else {
      ttsStatus[id] = "stopped";
      $q.notify({ type: "negative", message: `引擎响应异常 — ${voice.character_name}`, timeout: 3000 });
    }
  } catch {
    ttsStatus[id] = "stopped";
    $q.notify({ type: "negative", message: `引擎连接失败 — ${voice.character_name}，请确认引擎已启动`, timeout: 3000 });
  }
}

function openAddDialog() {
  editingVoice.value = null;
  Object.assign(form, emptyForm());
  dialogVisible.value = true;
}

function openEditDialog(voice: CharacterVoice) {
  editingVoice.value = voice;
  Object.assign(form, { ...voice });
  dialogVisible.value = true;
}

async function saveVoice() {
  saving.value = true;
  try {
    if (editingVoice.value) {
      await invoke("update_character_voice", {
        id: editingVoice.value.id,
        data: { ...form },
      });
    } else {
      await invoke("create_character_voice", {
        data: { ...form },
      });
    }
    dialogVisible.value = false;
    await loadData();
  } catch (err: any) {
    console.error("保存失败:", err);
  } finally {
    saving.value = false;
  }
}

async function confirmDelete(voice: CharacterVoice) {
  try {
    await invoke("delete_character_voice", { id: voice.id });
    await loadData();
  } catch (err: any) {
    console.error("删除失败:", err);
  }
}

async function pickAudioFile() {
  try {
    const selected = await open({
      filters: [{ name: "音频文件", extensions: ["wav", "mp3", "flac", "ogg"] }],
      multiple: false,
    });
    if (selected) form.ref_audio_path = selected;
  } catch (err: any) {
    console.error("文件选择失败:", err);
  }
}

async function pickYamlFile() {
  try {
    const selected = await open({
      filters: [{ name: "YAML 文件", extensions: ["yaml", "yml"] }],
      multiple: false,
    });
    if (selected) form.config_path = selected;
  } catch (err: any) {
    console.error("文件选择失败:", err);
  }
}

async function pickGptModel() {
  try {
    const selected = await open({
      title: "选择 GPT 模型文件",
      filters: [{ name: "GPT 模型", extensions: ["ckpt"] }],
      multiple: false,
    });
    if (selected) form.gpt_model = selected;
  } catch (err: any) {
    console.error("文件选择失败:", err);
  }
}

async function pickSovitsModel() {
  try {
    const selected = await open({
      title: "选择 SoVITS 模型文件",
      filters: [{ name: "SoVITS 模型", extensions: ["pth"] }],
      multiple: false,
    });
    if (selected) form.sovits_model = selected;
  } catch (err: any) {
    console.error("文件选择失败:", err);
  }
}

// ----- 情绪参考 -----

function openEmotionDialog(voice: CharacterVoice) {
  editingEmotionVoice.value = voice;
  const id = voice.id;
  if (id != null && !emotionRefs.value[id]) {
    emotionRefs.value[id] = {};
  }
  emotionDialogVisible.value = true;
}

function emotionAudio(emotion: string): string {
  const id = editingEmotionVoice.value?.id;
  if (id == null) return "";
  return emotionRefs.value[id]?.[emotion] ?? "";
}

function setEmotionAudio(emotion: string, val: any) {
  const id = editingEmotionVoice.value?.id;
  if (id == null) return;
  if (!emotionRefs.value[id]) emotionRefs.value[id] = {};
  emotionRefs.value[id][emotion] = val ?? "";
}

function clearEmotionAudio(emotion: string) {
  const id = editingEmotionVoice.value?.id;
  if (id == null) return;
  if (emotionRefs.value[id]) {
    delete emotionRefs.value[id][emotion];
  }
}

async function pickEmotionAudio(emotion: string) {
  try {
    const selected = await open({
      title: `选择情绪 "${emotion}" 参考音频`,
      filters: [{ name: "音频文件", extensions: ["wav", "mp3", "flac", "ogg"] }],
      multiple: false,
    });
    if (selected) setEmotionAudio(emotion, selected);
  } catch (err: any) {
    console.error("文件选择失败:", err);
  }
}

onMounted(loadData);
</script>

<style scoped lang="scss">
.voice-card {
  border-radius: 10px;
  transition: background-color 0.15s;
}
.voice-card:hover {
  background-color: rgba(233, 30, 99, 0.03);
}
</style>
