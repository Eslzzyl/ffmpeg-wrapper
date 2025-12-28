<template>
  <div v-if="show" class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h3>视频详细参数</h3>
        <button class="btn-close" @click="$emit('close')">✕</button>
      </div>
      <div class="modal-body" v-if="task && task.details">
        <div class="detail-item">
          <label>文件名:</label>
          <span>{{ task.name }}</span>
        </div>
        <div class="detail-item">
          <label>路径:</label>
          <span class="path-text">{{ task.path }}</span>
        </div>
        <div class="detail-grid">
          <div class="detail-item">
            <label>分辨率:</label>
            <span>{{ task.details.width }} x {{ task.details.height }}</span>
          </div>
          <div class="detail-item">
            <label>时长:</label>
            <span>{{ formatDuration(task.duration) }}</span>
          </div>
          <div class="detail-item">
            <label>文件大小:</label>
            <span>{{ formatSize(task.size) }}</span>
          </div>
          <div class="detail-item">
            <label>封装格式:</label>
            <span>{{ task.details.format }}</span>
          </div>
          <div class="detail-item">
            <label>视频编码:</label>
            <span>{{ task.details.video_codec }}</span>
          </div>
          <div class="detail-item">
            <label>视频码率:</label>
            <span>{{ formatBitrate(task.details.video_bitrate) }}</span>
          </div>
          <div class="detail-item">
            <label>帧率:</label>
            <span>{{ task.details.frame_rate }} fps</span>
          </div>
          <div class="detail-item">
            <label>音频编码:</label>
            <span>{{ task.details.audio_codec }}</span>
          </div>
          <div class="detail-item">
            <label>音频码率:</label>
            <span>{{ formatBitrate(task.details.audio_bitrate) }}</span>
          </div>
          <div class="detail-item">
            <label>音频采样率:</label>
            <span>{{ task.details.audio_sample_rate }} Hz</span>
          </div>
          <div class="detail-item">
            <label>音频声道数:</label>
            <span>{{ task.details.audio_channels }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  show: {
    type: Boolean,
    required: true
  },
  task: {
    type: Object,
    default: null
  }
});

defineEmits(['close']);

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

<style scoped>
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