<template>
  <div class="request-form-panel">
    <div class="panel-header">
      <h2>{{ selectedMethod?.name || 'Select a Method' }}</h2>
      <el-tag v-if="selectedMethod" :type="getMethodType(selectedMethod.type)">
        {{ selectedMethod.type }}
      </el-tag>
    </div>

    <el-form
      :model="formData"
      label-width="120px"
      class="request-form"
      @submit.prevent="handleSubmit"
    >
      <el-form-item label="Server Address">
        <el-input v-model="formData.address" placeholder="localhost:50051" />
      </el-form-item>

      <el-form-item v-if="needsRequestData" :label="getRequestDataLabel()">
        <div class="json-editor-container">
          <el-input
            v-model="formData.requestData"
            type="textarea"
            :rows="8"
            :placeholder="getRequestDataPlaceholder()"
            @blur="handleRequestDataBlur"
          />
          <div class="json-format-buttons">
            <el-button size="small" @click="formatRequestData">
              <el-icon><Document /></el-icon>
              Format
            </el-button>
            <el-button size="small" @click="minifyRequestData">
              <el-icon><Minus /></el-icon>
              Minify
            </el-button>
          </div>
        </div>
      </el-form-item>

      <el-form-item v-if="needsStreamingData" :label="getStreamingDataLabel()">
        <div class="json-editor-container">
          <el-input
            v-model="formData.streamingData"
            type="textarea"
            :rows="8"
            :placeholder="getStreamingDataPlaceholder()"
            @blur="handleStreamingDataBlur"
          />
          <div class="json-format-buttons">
            <el-button size="small" @click="formatStreamingData">
              <el-icon><Document /></el-icon>
              Format
            </el-button>
            <el-button size="small" @click="minifyStreamingData">
              <el-icon><Minus /></el-icon>
              Minify
            </el-button>
          </div>
        </div>
        <div class="form-help-text">
          Enter a single JSON object to send as a message
        </div>
      </el-form-item>

      <el-form-item v-if="hasValidationErrors" label="Validation Errors">
        <div class="validation-errors">
          <pre>{{ payloadValidationErrors }}</pre>
        </div>
      </el-form-item>

      <el-form-item>
        <el-button
          type="primary"
          :loading="loading"
          :disabled="!isFormValid"
          @click="handleSubmit"
        >
          <el-icon><VideoPlay /></el-icon>
          {{ getSubmitButtonText() }}
        </el-button>
        <el-button @click="handleClear"> Clear </el-button>
      </el-form-item>

      <el-form-item v-if="isStreaming && needsStreamingControls" label="Stream Controls">
        <div class="streaming-controls">
          <el-select v-model="selectedSignal" placeholder="Select signal" style="width: 200px">
            <el-option
              v-for="signal in getAvailableSignals()"
              :key="signal.value"
              :label="signal.label"
              :value="signal.value"
            />
          </el-select>
          <el-button type="warning" :disabled="!selectedSignal" @click="handleSendSignal">
            <el-icon><Right /></el-icon>
            Send Signal
          </el-button>
        </div>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch, computed, ref } from 'vue';
import { VideoPlay, Right, Document, Minus } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { formatJson, minifyJson } from '../../utils/jsonFormatter';
import {
  validatePayload,
  formatValidationErrors,
  type ValidationResult,
} from '../../utils/payloadValidator';
import type {
  ServiceMethod,
  RequestForm as RequestFormType,
  RpcType,
  StreamingSignal,
} from '../../types/grpc';

interface Props {
  selectedMethod?: (ServiceMethod & { serviceName?: string }) | null;
  loading?: boolean;
  sampleData?: string;
  isStreaming?: boolean;
  protoContent?: string;
}

interface Emits {
  (e: 'submit', formData: RequestFormType): void;
  (e: 'clear'): void;
  (e: 'sendSignal', signal: StreamingSignal): void;
  (e: 'sendMessage', messageData: any): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const formData = reactive<RequestFormType>({
  address: 'localhost:50051',
  requestData: '',
  streamingData: '',
});

const selectedSignal = ref<StreamingSignal | ''>('');
const validationResult = ref<ValidationResult | null>(null);

const payloadValidationErrors = computed(() => {
  if (!validationResult.value || validationResult.value.isValid) return '';
  return formatValidationErrors(validationResult.value.errors);
});

const hasValidationErrors = computed(() => {
  return validationResult.value && !validationResult.value.isValid;
});

const formatRequestData = () => {
  const result = formatJson(formData.requestData);
  if (result.success) {
    formData.requestData = result.formatted;
    ElMessage.success('JSON formatted successfully');
  } else {
    ElMessage.error(`Format error: ${result.error}`);
  }
};

const minifyRequestData = () => {
  const result = minifyJson(formData.requestData);
  if (result.success) {
    formData.requestData = result.formatted;
    ElMessage.success('JSON minified successfully');
  } else {
    ElMessage.error(`Minify error: ${result.error}`);
  }
};

const formatStreamingData = () => {
  const result = formatJson(formData.streamingData || '');
  if (result.success) {
    formData.streamingData = result.formatted;
    ElMessage.success('JSON formatted successfully');
  } else {
    ElMessage.error(`Format error: ${result.error}`);
  }
};

const minifyStreamingData = () => {
  const result = minifyJson(formData.streamingData || '');
  if (result.success) {
    formData.streamingData = result.formatted;
    ElMessage.success('JSON minified successfully');
  } else {
    ElMessage.error(`Minify error: ${result.error}`);
  }
};

const handleRequestDataBlur = () => {
  if (formData.requestData.trim()) {
    const result = formatJson(formData.requestData);
    if (result.success && result.formatted !== formData.requestData) {
      formData.requestData = result.formatted;
    } else if (!result.success) {
      ElMessage.error(`Auto-format error: ${result.error}`);
    }
  }
};

const handleStreamingDataBlur = () => {
  if (formData.streamingData && formData.streamingData.trim()) {
    const result = formatJson(formData.streamingData);
    if (result.success && result.formatted !== formData.streamingData) {
      formData.streamingData = result.formatted;
    } else if (!result.success) {
      ElMessage.error(`Auto-format error: ${result.error}`);
    }
  }
};

const validateCurrentPayload = () => {
  if (!props.selectedMethod || !props.protoContent) {
    validationResult.value = null;
    return;
  }
  const rpcType = props.selectedMethod.type as RpcType;
  let payloadToValidate: any = null;
  let messageType = '';
  if (rpcType === 'unary' || rpcType === 'server-streaming') {
    if (formData.requestData.trim()) {
      try {
        payloadToValidate = JSON.parse(formData.requestData);
        messageType = props.selectedMethod.inputType;
      } catch {
        validationResult.value = null;
        return;
      }
    }
  } else if (rpcType === 'client-streaming' || rpcType === 'bidirectional-streaming') {
    if (formData.streamingData && formData.streamingData.trim()) {
      try {
        payloadToValidate = JSON.parse(formData.streamingData);
        messageType = props.selectedMethod.inputType;
      } catch {
        validationResult.value = null;
        return;
      }
    }
  }
  if (payloadToValidate !== null && messageType) {
    validationResult.value = validatePayload(payloadToValidate, messageType, props.protoContent);
  } else {
    validationResult.value = null;
  }
};

const getMethodType = (type: string) => {
  const typeMap: Record<string, string> = {
    unary: 'primary',
    'server-streaming': 'warning',
    'client-streaming': 'info',
    'bidirectional-streaming': 'success',
  };
  return typeMap[type] || 'primary';
};

const needsRequestData = computed(() => {
  if (!props.selectedMethod) return false;
  const rpcType = props.selectedMethod.type as RpcType;
  return rpcType === 'unary' || rpcType === 'server-streaming';
});

const needsStreamingData = computed(() => {
  if (!props.selectedMethod) return false;
  const rpcType = props.selectedMethod.type as RpcType;
  return rpcType === 'client-streaming' || rpcType === 'bidirectional-streaming';
});

const needsStreamingControls = computed(() => {
  if (!props.selectedMethod) return false;
  const rpcType = props.selectedMethod.type as RpcType;
  return (
    rpcType === 'server-streaming' ||
    rpcType === 'client-streaming' ||
    rpcType === 'bidirectional-streaming'
  );
});

const isFormValid = computed(() => {
  if (!formData.address || !props.selectedMethod) return false;
  const rpcType = props.selectedMethod.type as RpcType;
  let hasRequiredData = false;
  if (rpcType === 'unary' || rpcType === 'server-streaming') {
    hasRequiredData = !!formData.requestData;
  } else if (rpcType === 'client-streaming' || rpcType === 'bidirectional-streaming') {
    hasRequiredData = !!formData.streamingData;
  }
  if (!hasRequiredData) return false;
  if (hasValidationErrors.value) return false;
  return true;
});

const getRequestDataLabel = () => {
  if (!props.selectedMethod) return 'Request Data';
  const rpcType = props.selectedMethod.type as RpcType;
  return rpcType === 'server-streaming' ? 'Initial Request' : 'Request Data';
};

const getRequestDataPlaceholder = () => {
  if (!props.selectedMethod) return 'Enter JSON request data';
  const rpcType = props.selectedMethod.type as RpcType;
  return rpcType === 'server-streaming'
    ? 'Enter initial request data for server streaming'
    : 'Enter JSON request data';
};

const getStreamingDataLabel = () => {
  if (!props.selectedMethod) return 'Streaming Data';
  const rpcType = props.selectedMethod.type as RpcType;
  return rpcType === 'bidirectional-streaming' ? 'Messages to Send' : 'Data to Stream';
};

const getStreamingDataPlaceholder = () => {
  if (!props.selectedMethod) return 'Enter message data';
  const rpcType = props.selectedMethod.type as RpcType;
  return rpcType === 'bidirectional-streaming'
    ? 'Enter a message to send'
    : 'Enter a message to stream to server';
};

const getSubmitButtonText = () => {
  if (!props.selectedMethod) return 'Send Request';
  const rpcType = props.selectedMethod.type as RpcType;
  switch (rpcType) {
    case 'server-streaming':
      return 'Start Server Stream';
    case 'client-streaming':
    case 'bidirectional-streaming':
      return 'Send Message';
    default:
      return 'Send Request';
  }
};

const getAvailableSignals = () => {
  if (!props.selectedMethod) return [];
  const rpcType = props.selectedMethod.type as RpcType;
  const signals = [] as Array<{ label: string; value: StreamingSignal }>;
  switch (rpcType) {
    case 'server-streaming':
    case 'client-streaming':
      signals.push(
        { label: 'Cancel Stream', value: 'cancel' },
        { label: 'End Stream', value: 'end' }
      );
      break;
    case 'bidirectional-streaming':
      signals.push(
        { label: 'Cancel Stream', value: 'cancel' },
        { label: 'End Stream', value: 'end' },
        { label: 'Pause Stream', value: 'pause' },
        { label: 'Resume Stream', value: 'resume' }
      );
      break;
  }
  return signals;
};

const handleSubmit = () => {
  if (!props.selectedMethod) {
    emit('submit', { ...formData });
    return;
  }
  const rpcType = props.selectedMethod.type as RpcType;
  if ((rpcType === 'client-streaming' || rpcType === 'bidirectional-streaming') && props.isStreaming) {
    if (formData.streamingData) {
      try {
        const messageData = JSON.parse(formData.streamingData);
        emit('sendMessage', messageData);
        formData.streamingData = '';
      } catch {
        ElMessage.error('Invalid JSON in message data');
      }
    } else {
      ElMessage.warning('Please enter a message to send');
    }
  } else {
    emit('submit', { ...formData });
  }
};

const handleClear = () => {
  formData.requestData = '';
  formData.streamingData = '';
  selectedSignal.value = '';
  emit('clear');
};

const handleSendSignal = () => {
  if (selectedSignal.value) {
    emit('sendSignal', selectedSignal.value as StreamingSignal);
    selectedSignal.value = '';
  }
};

watch(
  () => props.sampleData,
  newSampleData => {
    if (newSampleData && props.selectedMethod) {
      const rpcType = props.selectedMethod.type as RpcType;
      if (rpcType === 'client-streaming' || rpcType === 'bidirectional-streaming') {
        formData.streamingData = newSampleData;
      } else {
        formData.requestData = newSampleData;
      }
    }
  },
  { immediate: true }
);

watch([
  () => formData.requestData,
  () => formData.streamingData,
  () => props.selectedMethod,
  () => props.protoContent,
], () => {
  validateCurrentPayload();
}, { immediate: true });
</script>

<style scoped>
.request-form-panel {
  background: white;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e4e7ed;
}

.panel-header h2 {
  margin: 0;
  color: #303133;
}

.request-form {
  max-width: 600px;
}

.form-help-text {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
  line-height: 1.4;
}

.streaming-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.json-editor-container {
  position: relative;
}

.json-format-buttons {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  justify-content: flex-end;
}

.json-format-buttons .el-button {
  font-size: 12px;
  padding: 4px 8px;
  height: auto;
}

.validation-errors {
  background: #fef0f0;
  border: 1px solid #fbc4c4;
  border-radius: 4px;
  padding: 12px;
  margin-top: 8px;
}

.validation-errors pre {
  margin: 0;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  line-height: 1.4;
  color: #f56c6c;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
