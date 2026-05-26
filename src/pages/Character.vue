<template>
  <q-page padding>
    <q-card class="q-mb-md">
      <q-card-section>
        <div class="text-h6 text-weight-bold text-pink-7">GPT-SoVITS TTS 测试</div>
        <div class="text-caption text-grey-6">API: POST http://127.0.0.1:9880/tts</div>
      </q-card-section>

      <q-card-section class="q-gutter-md">
        <!-- 快捷填充 -->
        <div class="row items-center q-gutter-sm">
          <span class="text-caption text-grey-7">快捷填充:</span>
          <q-btn
            v-for="(preset, i) in presets"
            :key="i"
            dense
            flat
            rounded
            size="sm"
            color="pink-6"
            :label="`示例 ${i + 1}`"
            @click="applyPreset(i)"
          />
        </div>

        <!-- 参考音频路径 -->
        <q-input
          v-model="refAudioPath"
          label="参考音频路径 (ref_audio_path)"
          stack-label
          outlined
          color="pink-6"
          placeholder="D:/GPT-SoVITS/orginal_audio/miyagi_slicer_opt/xxx.wav"
        />

        <div class="row q-gutter-md">
          <!-- 文本语言 -->
          <q-select
            v-model="textLang"
            :options="['zh', 'en', 'ja']"
            label="文本语言 (text_lang)"
            outlined
            color="pink-6"
            style="min-width: 120px"
          />

          <!-- 提示语言 -->
          <q-select
            v-model="promptLang"
            :options="['zh', 'en', 'ja']"
            label="提示语言 (prompt_lang)"
            outlined
            color="pink-6"
            style="min-width: 120px"
          />
        </div>

        <!-- 提示文本 -->
        <q-input
          v-model="promptText"
          label="参考音频对应文本 (prompt_text)"
          stack-label
          outlined
          color="pink-6"
          placeholder="参考音频的原始文本"
        />

        <!-- 合成文本 -->
        <q-input
          v-model="ttsText"
          label="要合成的文本 (text)"
          stack-label
          outlined
          color="pink-6"
          type="textarea"
          rows="4"
          placeholder="输入要转换成语音的文本..."
        />

        <!-- 操作按钮 -->
        <div class="row items-center q-gutter-md">
          <q-btn
            color="pink-6"
            icon="record_voice_over"
            label="生成语音"
            :loading="loading"
            :disable="!ttsText"
            rounded
            @click="generateTts"
          />
          <q-chip v-if="statusMsg" :color="statusColor" text-color="white" icon="info">
            {{ statusMsg }}
          </q-chip>
        </div>
      </q-card-section>
    </q-card>

    <!-- 音频播放 -->
    <q-card v-if="audioUrl" class="q-mb-md">
      <q-card-section>
        <div class="text-subtitle1 text-weight-bold text-pink-7 q-mb-sm">播放结果</div>
        <audio
          ref="audioPlayer"
          controls
          style="width: 100%"
          :src="audioUrl"
        >
          您的浏览器不支持 audio 标签
        </audio>
      </q-card-section>
    </q-card>

    <!-- 原始响应调试 -->
    <q-card v-if="rawResponse !== null">
      <q-card-section>
        <div class="text-subtitle2 text-weight-bold text-pink-7 q-mb-sm">原始响应</div>
        <pre class="bg-grey-2 q-pa-sm" style="overflow-x: auto; font-size: 12px">{{ rawResponse }}</pre>
      </q-card-section>
    </q-card>

    <!-- 模型管理 -->
    <q-card v-if="models !== null">
      <q-card-section>
        <div class="text-subtitle1 text-weight-bold text-pink-7 q-mb-sm">模型管理</div>

        <div class="row q-gutter-md">
          <!-- GPT 模型 -->
          <div class="col" style="min-width: 200px">
            <div class="text-caption text-grey-7 q-mb-xs">
              当前 GPT: <strong>{{ models.current.gpt }}</strong>
            </div>
            <q-list dense bordered separator>
              <q-item
                v-for="gpt in models.gpt_weights"
                :key="gpt"
                clickable
                v-ripple
                :active="gpt === models.current.gpt"
                active-class="bg-pink-2"
                @click="switchGpt(gpt)"
              >
                <q-item-section>
                  <q-item-label class="text-body2">{{ gpt }}</q-item-label>
                </q-item-section>
                <q-item-section side>
                  <q-icon v-if="gpt === models.current.gpt" name="check" color="pink-6" />
                </q-item-section>
              </q-item>
            </q-list>
          </div>

          <!-- SoVITS 模型 -->
          <div class="col" style="min-width: 200px">
            <div class="text-caption text-grey-7 q-mb-xs">
              当前 SoVITS: <strong>{{ models.current.sovits }}</strong>
            </div>
            <q-list dense bordered separator>
              <q-item
                v-for="sv in models.sovits_weights"
                :key="sv"
                clickable
                v-ripple
                :active="sv === models.current.sovits"
                active-class="bg-pink-2"
                @click="switchSovits(sv)"
              >
                <q-item-section>
                  <q-item-label class="text-body2">{{ sv }}</q-item-label>
                </q-item-section>
                <q-item-section side>
                  <q-icon v-if="sv === models.current.sovits" name="check" color="pink-6" />
                </q-item-section>
              </q-item>
            </q-list>
          </div>
        </div>

        <q-chip v-if="modelStatus" :color="modelStatusColor" text-color="white" size="sm" icon="info">
          {{ modelStatus }}
        </q-chip>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";

const API_BASE = "http://127.0.0.1:9880";

const presets = [
  {
    label: "示例 1",
    refAudioPath: "D:/GPT-SoVITS/orginal_audio/miyagi_slicer_opt/1_31807964932-1-30232_(Vocals)_0.0-5.0.wav_0000034240_0000160640.wav",
    textLang: "ja",
    promptLang: "ja",
    promptText: "仙台さん飲み物持ってくるから座って待ってて",
    ttsText: "紙に書かれた間取りは部屋が二つ。それとは別にキッチンやダイニング、バスルームもある。",
  },
  {
    label: "示例 2",
    refAudioPath: "D:/GPT-SoVITS/orginal_audio/miyagi_slicer_opt/1_31807964932-1-30232_(Vocals)_124.0-143.5.wav_0000041600_0000209600.wav",
    textLang: "ja",
    promptLang: "ja",
    promptText: "仙台さんがカバンから出してテーブルに置いた桜色の封筒は、",
    ttsText: "どう考えても一人で住むような部屋じゃない。",
  },
];

const refAudioPath = ref("");
const textLang = ref("zh");
const promptLang = ref("zh");
const promptText = ref("");
const ttsText = ref("世界这么大，我想去看看。");

const loading = ref(false);
const statusMsg = ref("");
const statusColor = ref("pink-6");
const audioUrl = ref<string | null>(null);
const rawResponse = ref<string | null>(null);

// 模型管理
const models = ref<{
  gpt_weights: string[];
  sovits_weights: string[];
  current: { gpt: string | null; sovits: string | null };
} | null>(null);
const modelStatus = ref("");
const modelStatusColor = ref("info");

async function fetchModels() {
  try {
    const res = await fetch(`${API_BASE}/list_models`);
    models.value = await res.json();
    modelStatus.value = "";
  } catch (err: any) {
    modelStatus.value = `获取失败: ${err.message}`;
    modelStatusColor.value = "negative";
  }
}

async function switchGpt(name: string) {
  if (!models.value || name === models.value.current.gpt) return;
  modelStatus.value = `切换 GPT 模型中...`;
  modelStatusColor.value = "info";
  try {
    const dir = "GPT_weights_v2Pro";
    const res = await fetch(`${API_BASE}/set_gpt_weights?weights_path=${dir}/${name}`);
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || data.Exception);
    models.value.current.gpt = name;
    modelStatus.value = `GPT 已切换至: ${name}`;
    modelStatusColor.value = "positive";
  } catch (err: any) {
    modelStatus.value = `切换失败: ${err.message}`;
    modelStatusColor.value = "negative";
  }
}

async function switchSovits(name: string) {
  if (!models.value || name === models.value.current.sovits) return;
  modelStatus.value = `切换 SoVITS 模型中...`;
  modelStatusColor.value = "info";
  try {
    const dir = "SoVITS_weights_v2Pro";
    const res = await fetch(`${API_BASE}/set_sovits_weights?weights_path=${dir}/${name}`);
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || data.Exception);
    models.value.current.sovits = name;
    modelStatus.value = `SoVITS 已切换至: ${name}`;
    modelStatusColor.value = "positive";
  } catch (err: any) {
    modelStatus.value = `切换失败: ${err.message}`;
    modelStatusColor.value = "negative";
  }
}

onMounted(() => {
  fetchModels();
});

function applyPreset(index: number) {
  const p = presets[index];
  refAudioPath.value = p.refAudioPath;
  textLang.value = p.textLang;
  promptLang.value = p.promptLang;
  promptText.value = p.promptText;
  ttsText.value = p.ttsText;
  statusMsg.value = `已填充: ${p.refAudioPath.split("/").pop()}`;
  statusColor.value = "info";
  audioUrl.value = null;
  rawResponse.value = null;
}

async function generateTts() {
  loading.value = true;
  statusMsg.value = "请求中...";
  statusColor.value = "info";
  audioUrl.value = null;
  rawResponse.value = null;

  const body = {
    text: ttsText.value,
    text_lang: textLang.value,
    ref_audio_path: refAudioPath.value,
    prompt_lang: promptLang.value,
    prompt_text: promptText.value,
    media_type: "wav",
    streaming_mode: false,
  };

  try {
    const res = await fetch(`${API_BASE}/tts`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: res.statusText }));
      throw new Error(err.Exception || err.message || `HTTP ${res.status}`);
    }

    // 成功 — 将 wav 二进制转为 blob URL
    const blob = await res.blob();
    audioUrl.value = URL.createObjectURL(blob);
    statusMsg.value = `成功 (${(blob.size / 1024).toFixed(1)} KB)`;
    statusColor.value = "positive";
  } catch (err: any) {
    statusMsg.value = `失败: ${err.message}`;
    statusColor.value = "negative";
    rawResponse.value = err.message;
  } finally {
    loading.value = false;
  }
}
</script>
