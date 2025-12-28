<script setup>
import { ref, onMounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

const tasks = ref([]);
const ffmpegStatus = ref("Checking...");
const gpuAccel = ref(false);
const gpuInfo = ref({ has_nvenc: false, has_videotoolbox: false, has_amf: false, has_qsv: false });
const activeTab = ref('video');
const selectedTaskId = ref(null);
const isProcessing = ref(false);
const showDetails = ref(false);
const detailsTask = ref(null);

const selectedTask = computed(() => tasks.value.find(t => t.id === selectedTaskId.value));

// 当选中任务改变时，如果是音频文件且当前在视频选项卡，则自动切换到音频选项卡
watch(selectedTask, (newTask) => {
  if (newTask && newTask.resolution === '音频文件' && activeTab.value === 'video') {
    activeTab.value = 'audio';
  }
});

const globalProgress = computed(() => {
  if (tasks.value.length === 0) return 0;
  const total = tasks.value.reduce((acc, t) => acc + (t.progress || 0), 0);
  return Math.round(total / tasks.value.length);
});

const settings = ref({
  video: {
    codec: 'libx264',
    accel: 'cpu',
    mode: 'crf',
    crf: 23,
    bitrate: '2000k',
    preset: 'medium'
  },
  audio: {
    stream: 'copy',
    codec: 'aac',
    bitrate: '128k',
    sample_rate: '',
    channels: ''
  },
  advanced: {
    resolution: 'original',
    customWidth: 1920,
    customHeight: 1080,
    trimStart: '',
    trimEnd: ''
  },
  output: {
    dir: 'original',
    suffix: '_out'
  }
});

const previewOutputName = computed(() => {
  if (!selectedTask.value) {
    return '请选择一个任务';
  }

  const task = selectedTask.value;
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

  return `${name}${settings.value.output.suffix}${ext}`;
});

const formatSize = (bytes) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const formatDuration = (seconds) => {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return [h, m, s].map(v => v.toString().padStart(2, '0')).join(':');
};

const formatBitrate = (bitrate) => {
  const b = parseInt(bitrate);
  if (isNaN(b) || b === 0) return 'N/A';
  if (b < 1000) return b + ' bps';
  if (b < 1000000) return (b / 1000).toFixed(1) + ' kbps';
  return (b / 1000000).toFixed(1) + ' Mbps';
};

const openDetails = (task) => {
  detailsTask.value = task;
  showDetails.value = true;
};

const processFiles = async (paths) => {
  for (const path of paths) {
    const name = path.split(/[\\/]/).pop();
    const id = Math.random().toString(36).substring(7);

    const newTask = {
      id,
      name,
      path,
      status: 'loading',
      progress: 0,
      resolution: '...',
      size: '...',
      duration: '...',
      rawDuration: 0,
      details: null
    };
    tasks.value.push(newTask);
    if (!selectedTaskId.value) selectedTaskId.value = id;

    try {
      const info = await invoke("get_video_info", { path });
      // Find the task in the array to ensure reactivity
      const taskIndex = tasks.value.findIndex(t => t.id === id);
      if (taskIndex !== -1) {
        // Determine if it's an audio-only file
        const isAudioOnly = info.video_codec === 'none' || (info.width === 0 && info.height === 0);
        tasks.value[taskIndex] = {
          ...tasks.value[taskIndex],
          resolution: isAudioOnly ? '音频文件' : `${info.width}x${info.height}`,
          size: formatSize(info.size),
          duration: formatDuration(info.duration),
          rawDuration: info.duration,
          details: info,
          status: 'waiting'
        };
      }
    } catch (e) {
      console.error("Failed to get video info:", e);
      const taskIndex = tasks.value.findIndex(t => t.id === id);
      if (taskIndex !== -1) {
        tasks.value[taskIndex].status = 'error';
      }
    }
  }
};

onMounted(async () => {
  try {
    const info = await invoke("check_ffmpeg");
    if (info.is_available) {
      ffmpegStatus.value = `Ready (${info.version.split(' ')[2] || 'v?'})`;
    } else {
      ffmpegStatus.value = "Not Found";
    }

    const gpu = await invoke("check_gpu_accel");
    gpuInfo.value = gpu;
    gpuAccel.value = gpu.has_nvenc || gpu.has_videotoolbox || gpu.has_amf || gpu.has_qsv;
  } catch (e) {
    console.error("Failed to check ffmpeg:", e);
    ffmpegStatus.value = "Error";
  }

  // Listen for file drops
  await getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === 'drop') {
      processFiles(event.payload.paths);
    }
  });

  // Listen for progress
  await listen("task-progress", (event) => {
    const { task_id, progress, status } = event.payload;
    const task = tasks.value.find(t => t.id === task_id);
    if (task) {
      if (progress >= 0) task.progress = progress;
      task.status = status;
    }
  });
});

const addFiles = async () => {
  const selected = await open({
    multiple: true,
    filters: [{
      name: 'Media',
      extensions: ['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg']
    }]
  });
  if (selected) {
    processFiles(Array.isArray(selected) ? selected : [selected]);
  }
};

const selectOutputDir = async () => {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected) {
    settings.value.output.dir = selected;
  }
};

const startAll = async () => {
  if (isProcessing.value) return;
  isProcessing.value = true;

  for (const task of tasks.value) {
    if (task.status === 'waiting' || task.status === 'error') {
      try {
        // Convert settings to snake_case for Rust
        const rustSettings = {
          video: { ...settings.value.video, crf: parseInt(settings.value.video.crf) },
          audio: { ...settings.value.audio },
          advanced: {
            resolution: settings.value.advanced.resolution,
            custom_width: parseInt(settings.value.advanced.customWidth) || 1920,
            custom_height: parseInt(settings.value.advanced.customHeight) || 1080,
            trim_start: settings.value.advanced.trimStart,
            trim_end: settings.value.advanced.trimEnd
          },
          output: { ...settings.value.output }
        };

        await invoke("run_task", {
          taskId: task.id,
          inputPath: task.path,
          duration: task.rawDuration,
          settings: rustSettings
        });
      } catch (e) {
        console.error(`Task ${task.id} failed:`, e);
        task.status = 'error';
      }
    }
  }
  isProcessing.value = false;
};

const clearTasks = () => {
  if (isProcessing.value) return;
  tasks.value = [];
  selectedTaskId.value = null;
};

const removeTask = (id) => {
  if (isProcessing.value) return;
  tasks.value = tasks.value.filter(t => t.id !== id);
  if (selectedTaskId.value === id) {
    selectedTaskId.value = tasks.value[0]?.id || null;
  }
};
</script>

<template>
  <div class="app-container">
    <!-- 左侧：任务管理区 -->
    <div class="left-panel">
      <div class="panel-header">任务列表</div>
      <div class="task-list" v-if="tasks.length > 0">
        <div v-for="task in tasks" :key="task.id" class="task-card" :class="{ active: selectedTaskId === task.id }"
          @click="selectedTaskId = task.id">
          <div class="task-info">
            <div class="task-name">{{ task.name }}</div>
            <div class="task-meta">
              <span v-if="task.resolution === '音频文件'">🎵 音频文件</span>
              <span v-else>{{ task.resolution }}</span>
              | {{ task.size }}
              <span v-if="task.details" class="btn-details" @click.stop="openDetails(task)">[详情]</span>
            </div>
          </div>
          <div class="task-status">
            <span v-if="task.status === 'waiting'" @click.stop="removeTask(task.id)" class="btn-remove">✕</span>
            <div v-else-if="task.status === 'processing'" class="mini-progress">
              <div class="progress-bar" :style="{ width: task.progress + '%' }"></div>
            </div>
            <span v-else-if="task.status === 'done'">✅</span>
            <span v-else-if="task.status === 'error'">❌</span>
          </div>
        </div>
      </div>
      <div class="drop-zone" @click="addFiles" v-else>
        <div class="plus-icon">+</div>
        <div>拖入视频文件或点击添加</div>
      </div>
    </div>

    <!-- 右侧：参数配置区 -->
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
        <button :class="{ active: activeTab === 'video', disabled: selectedTask && selectedTask.resolution === '音频文件' }"
                @click="selectedTask && selectedTask.resolution === '音频文件' ? null : activeTab = 'video'">视频</button>
        <button :class="{ active: activeTab === 'audio' }" @click="activeTab = 'audio'">音频</button>
        <button :class="{ active: activeTab === 'advanced' }" @click="activeTab = 'advanced'">高级</button>
        <button :class="{ active: activeTab === 'output' }" @click="activeTab = 'output'">输出</button>
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
              <button @click="selectOutputDir">选择</button>
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

    <!-- 底部：执行与状态 -->
    <div class="bottom-bar">
      <div class="status-info">
        <span class="status-dot" :class="{ ready: ffmpegStatus.includes('Ready') }"></span>
        FFmpeg: {{ ffmpegStatus }} | GPU:
        <span v-if="gpuInfo.has_nvenc" class="gpu-tag">NVIDIA</span>
        <span v-if="gpuInfo.has_videotoolbox" class="gpu-tag">Apple</span>
        <span v-if="gpuInfo.has_amf" class="gpu-tag">AMD</span>
        <span v-if="gpuInfo.has_qsv" class="gpu-tag">Intel</span>
        <span v-if="!gpuInfo.has_nvenc && !gpuInfo.has_videotoolbox && !gpuInfo.has_amf && !gpuInfo.has_qsv" class="gpu-tag-none">None</span>
      </div>
      <div class="global-progress">
        <div class="progress-text">总进度: {{ globalProgress }}%</div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: globalProgress + '%' }"></div>
        </div>
      </div>
      <div class="actions">
        <button class="btn-clear" @click="clearTasks" :disabled="isProcessing">清空列表</button>
        <button class="btn-start" @click="startAll" :disabled="isProcessing || tasks.length === 0">
          {{ isProcessing ? '处理中...' : '立即开始' }}
        </button>
      </div>
    </div>

    <!-- 详情模态框 -->
    <div v-if="showDetails" class="modal-overlay" @click="showDetails = false">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>视频详细参数</h3>
          <button class="btn-close" @click="showDetails = false">✕</button>
        </div>
        <div class="modal-body" v-if="detailsTask && detailsTask.details">
          <div class="detail-item">
            <label>文件名:</label>
            <span>{{ detailsTask.name }}</span>
          </div>
          <div class="detail-item">
            <label>路径:</label>
            <span class="path-text">{{ detailsTask.path }}</span>
          </div>
          <div class="detail-grid">
            <div class="detail-item">
              <label>分辨率:</label>
              <span>{{ detailsTask.details.width }} x {{ detailsTask.details.height }}</span>
            </div>
            <div class="detail-item">
              <label>时长:</label>
              <span>{{ detailsTask.duration }}</span>
            </div>
            <div class="detail-item">
              <label>文件大小:</label>
              <span>{{ detailsTask.size }}</span>
            </div>
            <div class="detail-item">
              <label>封装格式:</label>
              <span>{{ detailsTask.details.format }}</span>
            </div>
            <div class="detail-item">
              <label>视频编码:</label>
              <span>{{ detailsTask.details.video_codec }}</span>
            </div>
            <div class="detail-item">
              <label>视频码率:</label>
              <span>{{ formatBitrate(detailsTask.details.video_bitrate) }}</span>
            </div>
            <div class="detail-item">
              <label>帧率:</label>
              <span>{{ detailsTask.details.frame_rate }} fps</span>
            </div>
            <div class="detail-item">
              <label>音频编码:</label>
              <span>{{ detailsTask.details.audio_codec }}</span>
            </div>
            <div class="detail-item">
              <label>音频码率:</label>
              <span>{{ formatBitrate(detailsTask.details.audio_bitrate) }}</span>
            </div>
            <div class="detail-item">
              <label>音频采样率:</label>
              <span>{{ detailsTask.details.audio_sample_rate }} Hz</span>
            </div>
            <div class="detail-item">
              <label>音频声道数:</label>
              <span>{{ detailsTask.details.audio_channels }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
:root {
  --bg-color: #f5f5f7;
  --panel-bg: #ffffff;
  --border-color: #e0e0e0;
  --primary-color: #007aff;
  --text-color: #333;
  --text-secondary: #666;
}

body {
  margin: 0;
  padding: 0;
  overflow: hidden;
}

.app-container {
  display: grid;
  grid-template-columns: 40% 60%;
  grid-template-rows: 1fr 60px;
  height: 100vh;
  background-color: var(--bg-color);
  color: var(--text-color);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}

.left-panel {
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  background-color: var(--panel-bg);
}

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

.drop-zone {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 2px dashed var(--border-color);
  margin: 20px;
  border-radius: 8px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.3s;
}

.drop-zone:hover {
  border-color: var(--primary-color);
  background-color: rgba(0, 122, 255, 0.05);
}

.plus-icon {
  font-size: 48px;
  margin-bottom: 10px;
}

.bottom-bar {
  grid-column: span 2;
  background-color: #fff;
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 20px;
}

.status-info {
  display: flex;
  align-items: center;
  font-size: 12px;
  min-width: 200px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #ccc;
  margin-right: 8px;
}

.status-dot.ready {
  background-color: #4cd964;
}

.global-progress {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.progress-text {
  font-size: 11px;
  color: var(--text-secondary);
}

.progress-track {
  height: 6px;
  background-color: #eee;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: var(--primary-color);
  transition: width 0.3s;
}

.actions {
  display: flex;
  gap: 10px;
}

button {
  padding: 8px 16px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: #fff;
  cursor: pointer;
  font-size: 14px;
}

.btn-start {
  background-color: var(--primary-color);
  color: white;
  border: none;
}

.btn-start:hover {
  background-color: #0062cc;
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

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.task-card {
  background: #fff;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 10px;
  margin-bottom: 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-name {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 4px;
}

.task-meta {
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-details {
  color: var(--primary-color);
  cursor: pointer;
  font-size: 11px;
}

.btn-details:hover {
  text-decoration: underline;
}

.mini-progress {
  width: 60px;
  height: 4px;
  background: #eee;
  border-radius: 2px;
}

.btn-remove {
  cursor: pointer;
  color: #999;
  padding: 5px;
  border-radius: 50%;
  transition: all 0.2s;
}

.btn-remove:hover {
  color: #ff3b30;
  background-color: rgba(255, 59, 48, 0.1);
}

.task-card.active {
  border-color: var(--primary-color);
  background-color: rgba(0, 122, 255, 0.02);
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

/* 全局基础样式 */
h1 {
  text-align: center;
}

input,
button {
  outline: none;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-color: #1a1a1a;
    --panel-bg: #2c2c2c;
    --border-color: #444;
    --text-color: #e0e0e0;
    --text-secondary: #aaa;
  }

  .panel-header {
    background-color: #333;
  }

  .tabs button.active {
    background-color: #444;
  }

  .task-card {
    background: #333;
  }

  .bottom-bar {
    background-color: #2c2c2c;
  }

  select,
  input[type="text"] {
    background-color: #333;
    color: #e0e0e0;
    border-color: #555;
  }

  button {
    background-color: #444;
    color: #e0e0e0;
    border-color: #555;
  }

  .btn-start {
    background-color: var(--primary-color);
    color: white;
  }
}

/* 模态框样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: var(--panel-bg);
  width: 500px;
  max-width: 90%;
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.modal-header {
  padding: 15px 20px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
}

.btn-close {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--text-secondary);
}

.gpu-tag {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 3px;
  border-radius: 4px;
  background-color: #4cd964;
  color: white;
  font-size: 11px;
  font-weight: bold;
}

.gpu-tag-none {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 3px;
  border-radius: 4px;
  background-color: #ff3b30;
  color: white;
  font-size: 11px;
  font-weight: bold;
}

.preview-output {
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: #f9f9f9;
  font-family: monospace;
  word-break: break-all;
}

.modal-body {
  padding: 20px;
}

.detail-item {
  margin-bottom: 12px;
}

.detail-item label {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 2px;
}

.detail-item span {
  font-size: 14px;
  word-break: break-all;
}

.path-text {
  font-family: monospace;
  font-size: 12px !important;
  background: rgba(0, 0, 0, 0.05);
  padding: 4px;
  border-radius: 4px;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
  margin-top: 10px;
  padding-top: 15px;
  border-top: 1px solid var(--border-color);
}
</style>
