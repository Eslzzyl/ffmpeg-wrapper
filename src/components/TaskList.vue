<template>
  <div class="left-panel">
    <div class="panel-header">任务列表</div>
    <div class="task-list" v-if="tasks.length > 0">
      <div 
        v-for="task in tasks" 
        :key="task.id" 
        class="task-card" 
        :class="{ active: selectedTaskId === task.id }"
        @click="$emit('selectTask', task.id)"
      >
        <div class="task-info">
          <div class="task-name">{{ task.name }}</div>
          <div class="task-meta">
            <span v-if="task.resolution === '音频文件'">🎵 音频文件</span>
            <span v-else>{{ task.resolution }}</span>
            | {{ task.size }}
            <span v-if="task.details" class="btn-details" @click.stop="$emit('openDetails', task)">[详情]</span>
          </div>
        </div>
        <div class="task-status">
          <span 
            v-if="task.status === 'waiting'" 
            @click.stop="$emit('removeTask', task.id)" 
            class="btn-remove"
          >
            ✕
          </span>
          <div v-else-if="task.status === 'processing'" class="mini-progress">
            <div class="progress-bar" :style="{ width: task.progress + '%' }"></div>
          </div>
          <span v-else-if="task.status === 'done'">✅</span>
          <span v-else-if="task.status === 'error'">❌</span>
        </div>
      </div>
    </div>
    <div class="drop-zone" @click="$emit('addFiles')" v-else>
      <div class="plus-icon">+</div>
      <div>拖入视频文件或点击添加</div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  tasks: {
    type: Array,
    required: true
  },
  selectedTaskId: {
    type: String,
    required: true
  }
});

defineEmits(['selectTask', 'openDetails', 'addFiles', 'removeTask']);
</script>

<style scoped>
.left-panel {
  border-right: 1px solid var(--border-color);
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
  overflow: hidden;
}

.mini-progress .progress-bar {
  height: 100%;
  background: var(--primary-color);
  width: 0%;
  transition: width 0.3s;
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
</style>