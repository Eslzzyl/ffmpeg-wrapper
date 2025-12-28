<template>
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
      <button class="btn-clear" @click="$emit('clearTasks')" :disabled="isProcessing">清空列表</button>
      <button 
        class="btn-start" 
        @click="$emit('startAll')" 
        :disabled="isProcessing || tasks.length === 0"
      >
        {{ isProcessing ? '处理中...' : '立即开始' }}
      </button>
    </div>
  </div>
</template>

<script setup>
defineProps({
  ffmpegStatus: {
    type: String,
    required: true
  },
  gpuInfo: {
    type: Object,
    required: true
  },
  globalProgress: {
    type: Number,
    required: true
  },
  isProcessing: {
    type: Boolean,
    required: true
  },
  tasks: {
    type: Array,
    required: true
  }
});

defineEmits(['clearTasks', 'startAll']);
</script>

<style scoped>
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
</style>