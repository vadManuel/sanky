<template>
  <div class="grpc-client">
    <el-container class="client-container">
      <el-aside width="300px" class="sidebar">
        <ProtoFileLoader @loaded="handleProtoLoaded" />
        <ServiceList :services="parsedServices" v-model="selectedMethod" />
      </el-aside>
      <el-main class="main-content">
        <RequestForm
          v-if="selectedMethod"
          :selected-method="selectedMethod"
          :loading="grpcCall.loading.value"
          :sample-data="sampleData"
          :is-streaming="grpcCall.isStreaming.value"
          :proto-content="protoParser.protoContent.value"
          @submit="handleRequestSubmit"
          @clear="handleRequestClear"
          @send-signal="handleSendSignal"
          @send-message="handleSendMessage"
        />
        <div v-else class="no-method-selected">
          <el-empty description="Select a method to make a gRPC call">
            <template #image>
              <el-icon size="100"><Connection /></el-icon>
            </template>
          </el-empty>
        </div>
        <ResponsePanel
          :response="grpcCall.lastResponse.value"
          :streaming-responses="grpcCall.streamingResponses.value"
          :is-streaming="grpcCall.isStreaming.value"
        />
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { Connection } from '@element-plus/icons-vue';
import ProtoFileLoader from '../components/proto/ProtoFileLoader.vue';
import ServiceList from '../components/proto/ServiceList.vue';
import RequestForm from '../components/request/RequestForm.vue';
import ResponsePanel from '../components/response/ResponsePanel.vue';
import { useProtoParser } from '../composables/useProtoParser';
import { useGrpcCall } from '../composables/useGrpcCall';
import type { ServiceMethod, RequestForm as RequestFormType, StreamingSignal } from '../types/grpc';

const protoParser = useProtoParser();
const grpcCall = useGrpcCall();

const selectedMethod = ref<(ServiceMethod & { serviceName?: string }) | null>(null);
const sampleData = ref('');
const currentFormData = ref<RequestFormType>({ address: 'localhost:50051', requestData: '', streamingData: '' });

const { parsedServices } = protoParser;

const handleProtoLoaded = (payload: { path: string; content: string }) => {
  protoParser.protoContent.value = payload.content;
};

const handleRequestSubmit = (formData: RequestFormType) => {
  if (selectedMethod.value) {
    currentFormData.value = { ...formData };
    grpcCall.makeGrpcCall(selectedMethod.value, formData, protoParser.protoContent.value);
  }
};

const handleRequestClear = () => { grpcCall.clearResponse(); };

const handleSendSignal = (signal: StreamingSignal) => {
  if (selectedMethod.value) {
    grpcCall.sendStreamingSignal(selectedMethod.value, currentFormData.value, protoParser.protoContent.value, signal);
  }
};

const handleSendMessage = (messageData: any) => {
  if (selectedMethod.value) {
    grpcCall.sendStreamingMessage(selectedMethod.value, currentFormData.value, messageData);
  }
};

// Keep sample data in sync when method or proto content changes
watch([
  () => selectedMethod.value,
  () => protoParser.protoContent.value,
], () => {
  const method = selectedMethod.value;
  if (!method) return;
  const generatedSampleData = protoParser.generateSampleDataForMethod(method, protoParser.protoContent.value);
  sampleData.value = JSON.stringify(generatedSampleData, null, 2);
});
</script>

<style scoped>
.grpc-client { height: 100%; }
.client-container { height: 100%; }
.sidebar { background: white; border-right: 1px solid #e4e7ed; overflow-y: auto; }
.main-content { background: #f5f5f5; padding: 20px; overflow-y: auto; }
.no-method-selected { display: flex; align-items: center; justify-content: center; height: 400px; }
</style>
