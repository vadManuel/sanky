<template>
  <div v-if="response || streamingResponses.length > 0" class="response-panel">
    <div v-if="response" class="response-section">
      <div class="panel-header">
        <h3>Response</h3>
        <el-tag :type="response.success ? 'success' : 'danger'">
          {{ response.success ? 'Success' : 'Error' }}
        </el-tag>
      </div>
      <div class="response-content">
        <pre>{{ JSON.stringify(response.response || response.error, null, 2) }}</pre>
      </div>
    </div>

    <div v-if="streamingResponses.length > 0" class="streaming-section">
      <div class="panel-header">
        <h3>Streaming Response</h3>
        <div class="streaming-status">
          <el-tag v-if="isStreaming" type="warning" effect="plain">
            <el-icon class="rotating"><Loading /></el-icon>
            Streaming...
          </el-tag>
          <el-tag v-else type="success"> Complete </el-tag>
        </div>
      </div>
      <div class="streaming-content">
        <div
          v-for="(streamResponse, index) in streamingResponses"
          :key="index"
          class="stream-item"
          :class="{
            'stream-error': !streamResponse.success,
            'stream-end': streamResponse.done,
            'stream-sent': streamResponse.direction === 'sent',
            'stream-received': streamResponse.direction === 'received',
          }"
        >
          <div class="stream-header">
            <span class="stream-index">#{{ index + 1 }}</span>
            <el-tag :type="getStreamTagType(streamResponse)" size="small">
              {{ getStreamTagText(streamResponse) }}
            </el-tag>
            <span class="stream-timestamp">{{ formatTimestamp(Date.now()) }}</span>
          </div>
          <div class="stream-body">
            <pre v-if="streamResponse.data">{{ JSON.stringify(streamResponse.data, null, 2) }}</pre>
            <div v-else-if="streamResponse.error" class="error-message">{{ streamResponse.error }}</div>
            <div v-else-if="streamResponse.done" class="end-message">Stream completed</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Loading } from '@element-plus/icons-vue';
import type { GrpcResponse, StreamingResponse } from '../../types/grpc';

interface Props {
  response: GrpcResponse | null;
  streamingResponses?: StreamingResponse[];
  isStreaming?: boolean;
}

defineProps<Props>();

const getStreamTagType = (streamResponse: StreamingResponse) => {
  if (streamResponse.done) return 'info';
  if (!streamResponse.success) return 'danger';
  if (streamResponse.direction === 'sent') return 'warning';
  return 'success';
};

const getStreamTagText = (streamResponse: StreamingResponse) => {
  if (streamResponse.done) return 'End';
  if (!streamResponse.success) return 'Error';
  if (streamResponse.direction === 'sent') return 'Sent';
  return 'Received';
};

const formatTimestamp = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString();
};
</script>

<style scoped>
.response-panel { background: white; border-radius: 8px; padding: 24px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
.response-section { margin-bottom: 24px; }
.streaming-section { margin-top: 24px; }
.panel-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 16px; padding-bottom: 16px; border-bottom: 1px solid #e4e7ed; }
.panel-header h3 { margin: 0; color: #303133; }
.streaming-status { display: flex; align-items: center; gap: 8px; }
.rotating { animation: rotate 1s linear infinite; }
@keyframes rotate { from { transform: rotate(0deg);} to { transform: rotate(360deg);} }
.response-content { background: #f8f9fa; border: 1px solid #e9ecef; border-radius: 4px; padding: 16px; overflow-x: auto; }
.response-content pre { margin: 0; font-family: 'Monaco','Menlo','Ubuntu Mono', monospace; font-size: 13px; line-height: 1.4; color: #333; }
.streaming-content { max-height: 400px; overflow-y: auto; border: 1px solid #e4e7ed; border-radius: 4px; }
.stream-item { border-bottom: 1px solid #f0f0f0; padding: 12px 16px; }
.stream-item.stream-sent { background-color: #fff7e6; border-left: 4px solid #faad14; }
.stream-item.stream-received { background-color: #f6ffed; border-left: 4px solid #52c41a; }
.stream-item.stream-error { background-color: #fef0f0; border-left: 4px solid #f56c6c; }
.stream-item.stream-end { background-color: #f0f9ff; border-left: 4px solid #409eff; }
.stream-item:last-child { border-bottom: none; }
.stream-header { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.stream-index { font-weight: 600; color: #606266; font-size: 12px; }
.stream-timestamp { font-size: 11px; color: #909399; margin-left: auto; }
.stream-body { margin-left: 8px; }
.stream-body pre { margin: 0; font-family: 'Monaco','Menlo','Ubuntu Mono', monospace; font-size: 12px; line-height: 1.4; color: #333; background: #f8f9fa; padding: 8px; border-radius: 3px; border: 1px solid #e9ecef; }
.error-message { color: #f56c6c; font-size: 13px; font-weight: 500; }
.end-message { color: #409eff; font-size: 13px; font-weight: 500; font-style: italic; }
</style>
