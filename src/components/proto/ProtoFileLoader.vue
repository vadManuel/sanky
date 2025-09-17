<template>
  <div class="proto-file-loader">
    <div class="loader-header">
      <h3>Services</h3>
      <el-button type="primary" size="small" :loading="loading" @click="pickProto">
        <el-icon><FolderOpened /></el-icon>
        Load Proto File
      </el-button>
    </div>
  </div>
  
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpened } from '@element-plus/icons-vue';

const emit = defineEmits<{
  (e: 'loaded', payload: { path: string; content: string }): void;
}>();

const loading = ref(false);

async function pickProto() {
  loading.value = true;
  try {
    const path = await open({ filters: [{ name: 'Proto', extensions: ['proto'] }] });
    if (!path || Array.isArray(path)) return;
    const content = await invoke<string>('read_file_at_path', { path });
    emit('loaded', { path, content });
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.proto-file-loader {
  border-bottom: 1px solid #e4e7ed;
}

.loader-header {
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.loader-header h3 {
  margin: 0;
  color: #303133;
}
</style>
