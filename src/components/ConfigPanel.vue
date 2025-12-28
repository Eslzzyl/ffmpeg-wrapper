<template>
  <div class="right-panel">
    <div class="panel-header">
      编码设置
      <span v-if="selectedTask" class="selected-info">
        - {{ selectedTask.name }}
        <template v-if="selectedTask.resolution === '音频文件'">
          (音频文件, {{ selectedTask.duration }})
        </template>
        <template v-else>
          ({{ selectedTask.resolution }}, {{ selectedTask.duration }})
        </template>
      </span>
    </div>

    <div class="tabs">
      <button 
        :class="{ 
          active: activeTab === 'video', 
          disabled: selectedTask && selectedTask.resolution === '音频文件' 
        }"
        @click="selectedTask && selectedTask.resolution === '音频文件' ? null : setActiveTab('video')"
      >
        视频
      </button>
      <button :class="{ active: activeTab === 'audio' }" @click="setActiveTab('audio')">音频</button>
      <button :class="{ active: activeTab === 'advanced' }" @click="setActiveTab('advanced')">高级</button>
      <button :class="{ active: activeTab === 'output' }" @click="setActiveTab('output')">输出</button>
    </div>

    <div class="config-content">
      <div v-if="activeTab === 'video'" class="config-section">
        <div class="form-item">
          <label>编码器</label>
          <select v-model="settings.video.codec">
            <option value="libx264">H.264 (AVC)</option>
            <option value="libx265">H.265 (HEVC)</option>
            <option value="libsvtav1">AV1</option>
          </select>
        </div>
        <div class="form-item">
          <label>硬件加速</label>
          <select v-model="settings.video.accel">
            <option value="cpu">CPU (Software)</option>
            <option v-if="gpuInfo.has_nvenc" value="nvenc">NVIDIA NVENC</option>
            <option v-if="gpuInfo.has_videotoolbox" value="videotoolbox">Apple VideoToolbox</option>
            <option v-if="gpuInfo.has_amf" value="amf">AMD AMF</option>
            <option v-if="gpuInfo.has_qsv" value="qsv">Intel QSV</option>
          </select>
        </div>
        <div class="form-item">
          <label>质量控制</label>
          <div class="radio-group">
            <label><input type="radio" value="crf" v-model="settings.video.mode"> CRF</label>
            <label><input type="radio" value="bitrate" v-model="settings.video.mode"> 码率</label>
          </div>
        </div>
        <div class="form-item" v-if="settings.video.mode === 'crf'">
          <label>CRF 值 (越小质量越高)</label>
          <div class="slider-container">
            <input type="range" min="0" max="51" v-model="settings.video.crf">
            <span>{{ settings.video.crf }}</span>
          </div>
        </div>
        <div class="form-item" v-else>
          <label>目标码率 (例如: 2000k)</label>
          <input type="text" v-model="settings.video.bitrate">
        </div>
        <div class="form-item">
          <label>预设速度 (Preset)</label>
          <select v-model="settings.video.preset">
            <option value="ultrafast">Ultrafast</option>
            <option value="superfast">Superfast</option>
            <option value="veryfast">Veryfast</option>
            <option value="faster">Faster</option>
            <option value="fast">Fast</option>
            <option value="medium">Medium</option>
            <option value="slow">Slow</option>
            <option value="slower">Slower</option>
            <option value="veryslow">Veryslow</option>
          </select>
        </div>
      </div>

      <div v-if="activeTab === 'audio'" class="config-section">
        <div class="form-item">
          <label>音频流处理</label>
          <select v-model="settings.audio.stream">
            <option value="copy">复制 (Copy)</option>
            <option value="none">移除 (None)</option>
            <option value="encode">重新编码</option>
          </select>
        </div>
        <template v-if="settings.audio.stream === 'encode'">
          <div class="form-item">
            <label>音频编码器</label>
            <select v-model="settings.audio.codec">
              <option value="aac">AAC</option>
              <option value="libmp3lame">MP3</option>
              <option value="opus">Opus</option>
              <option value="flac">FLAC</option>
              <option value="libvorbis">Vorbis</option>
              <option value="pcm_s16le">WAV (PCM)</option>
            </select>
          </div>
          <div class="form-item">
            <label>音频码率</label>
            <select v-model="settings.audio.bitrate">
              <option value="64k">64k</option>
              <option value="96k">96k</option>
              <option value="128k">128k</option>
              <option value="160k">160k</option>
              <option value="192k">192k</option>
              <option value="256k">256k</option>
              <option value="320k">320k</option>
            </select>
          </div>
          <div class="form-item">
            <label>采样率</label>
            <select v-model="settings.audio.sample_rate">
              <option value="">默认</option>
              <option value="22050">22050 Hz</option>
              <option value="44100">44100 Hz</option>
              <option value="48000">48000 Hz</option>
              <option value="96000">96000 Hz</option>
            </select>
          </div>
          <div class="form-item">
            <label>声道数</label>
            <select v-model="settings.audio.channels">
              <option value="">默认</option>
              <option value="1">单声道 (Mono)</option>
              <option value="2">立体声 (Stereo)</option>
              <option value="6">5.1声道</option>
            </select>
          </div>
        </template>
      </div>

      <div v-if="activeTab === 'advanced'" class="config-section">
        <div class="form-item">
          <label>分辨率缩放</label>
          <select v-model="settings.advanced.resolution">
            <option value="original">原始分辨率</option>
            <option value="1920:1080">1920x1080 (1080p)</option>
            <option value="1280:720">1280x720 (720p)</option>
            <option value="854:480">854x480 (480p)</option>
            <option value="custom">自定义分辨率</option>
          </select>
        </div>

        <div class="form-item" v-if="settings.advanced.resolution === 'custom'">
          <label>自定义分辨率</label>
          <div class="input-with-btn">
            <input type="number" v-model="settings.advanced.customWidth" placeholder="宽度" min="1" />
            <span>×</span>
            <input type="number" v-model="settings.advanced.customHeight" placeholder="高度" min="1" />
          </div>
        </div>
        <div class="form-item">
          <label>裁剪 (Trim)</label>
          <div class="input-with-btn">
            <input type="text" v-model="settings.advanced.trimStart" placeholder="开始时间 (00:00:00)">
            <span>至</span>
            <input type="text" v-model="settings.advanced.trimEnd" placeholder="结束时间">
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'output'" class="config-section">
        <div class="form-item">
          <label>输出目录</label>
          <select v-model="settings.output.dir">
            <option value="original">原文件夹</option>
            <option value="">自定义目录</option>
          </select>
          <div v-if="settings.output.dir !== 'original'" class="input-with-btn" style="margin-top: 8px;">
            <input type="text" v-model="settings.output.dir" placeholder="选择输出目录">
            <button @click="$emit('selectOutputDir')">选择</button>
          </div>
        </div>
        <div class="form-item">
          <label>文件名后缀</label>
          <input type="text" v-model="settings.output.suffix">
        </div>
        <div class="form-item">
          <label>预览输出文件名</label>
          <div class="preview-output">{{ previewOutputName }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  settings: {
    type: Object,
    required: true
  },
  selectedTask: {
    type: Object,
    default: null
  },
  activeTab: {
    type: String,
    required: true
  },
  gpuInfo: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['update:activeTab', 'selectOutputDir']);

const previewOutputName = computed(() => {
  if (!props.selectedTask) {
    return '请选择一个任务';
  }

  const task = props.selectedTask;
  // 提取文件名和扩展名
  const lastSlashIndex = task.path.lastIndexOf('/');
  const lastBackslashIndex = task.path.lastIndexOf('\\');
  const lastSeparatorIndex = Math.max(lastSlashIndex, lastBackslashIndex);
  const fileName = task.path.substring(lastSeparatorIndex + 1);

  const lastDotIndex = fileName.lastIndexOf('.');
  let name, ext;
  if (lastDotIndex === -1) {
    name = fileName;
    ext = '';
  } else {
    name = fileName.substring(0, lastDotIndex);
    ext = fileName.substring(lastDotIndex);
  }

  return `${name}${props.settings.output.suffix}${ext}`;
});

const setActiveTab = (tab) => {
  emit('update:activeTab', tab);
};
</script>

<style scoped>
.right-panel {
  display: flex;
  flex-direction: column;
  background-color: var(--panel-bg);
}

.panel-header {
  padding: 15px;
  font-weight: bold;
  border-bottom: 1px solid var(--border-color);
  background-color: #fafafa;
}

.tabs {
  display: flex;
  padding: 10px;
  gap: 5px;
  border-bottom: 1px solid var(--border-color);
}

.tabs button {
  flex: 1;
  border: none;
  background: none;
  padding: 8px;
  border-radius: 4px;
}

.tabs button.active {
  background-color: #eee;
  font-weight: bold;
}

.tabs button.disabled {
  color: #999;
  cursor: not-allowed;
  opacity: 0.6;
}

.config-content {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.config-section {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-item label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

select,
input[type="text"] {
  padding: 8px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.radio-group {
  display: flex;
  gap: 15px;
}

.input-with-btn {
  display: flex;
  gap: 5px;
}

.input-with-btn input {
  flex: 1;
}

.selected-info {
  font-weight: normal;
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 10px;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 10px;
}

.slider-container input {
  flex: 1;
}

.preview-output {
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: #f9f9f9;
  font-family: monospace;
  word-break: break-all;
}
</style>