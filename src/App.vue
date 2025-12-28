<script setup>
import { ref, onMounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import TaskList from './components/TaskList.vue';
import ConfigPanel from './components/ConfigPanel.vue';
import BottomBar from './components/BottomBar.vue';
import DetailModal from './components/DetailModal.vue';

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
</script>

<template>
  <div class="app-container">
    <TaskList
      :tasks="tasks"
      :selectedTaskId="selectedTaskId"
      @selectTask="selectedTaskId = $event"
      @openDetails="openDetails($event)"
      @addFiles="addFiles"
      @removeTask="removeTask"
    />

    <ConfigPanel
      :settings="settings"
      :selectedTask="selectedTask"
      :activeTab="activeTab"
      :gpuInfo="gpuInfo"
      @update:activeTab="activeTab = $event"
      @selectOutputDir="selectOutputDir"
    />

    <BottomBar
      :ffmpegStatus="ffmpegStatus"
      :gpuInfo="gpuInfo"
      :globalProgress="globalProgress"
      :isProcessing="isProcessing"
      :tasks="tasks"
      @clearTasks="clearTasks"
      @startAll="startAll"
    />

    <DetailModal
      :show="showDetails"
      :task="detailsTask"
      @close="showDetails = false"
    />
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
</style>
