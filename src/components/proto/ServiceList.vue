<template>
  <div class="service-list">
    <div v-if="services.length > 0" class="services-content">
      <el-collapse v-model="activeServices">
        <el-collapse-item
          v-for="service in services"
          :key="service.name"
          :title="service.name"
          :name="service.name"
        >
          <div class="service-methods">
            <div
              v-for="method in service.methods"
              :key="method.name"
              class="method-item"
              :class="{ active: modelValue?.name === method.name }"
              @click="handleMethodSelect(service.name, method)"
            >
              <el-icon><Right /></el-icon>
              {{ method.name }}
              <el-tag size="small" :type="getMethodType(method.type)">
                {{ method.type }}
              </el-tag>
            </div>
          </div>
        </el-collapse-item>
      </el-collapse>
    </div>

    <div v-else class="no-proto">
      <el-empty description="No proto file loaded" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Right } from '@element-plus/icons-vue';
import type { Service, ServiceMethod } from '../../types/grpc';

const props = defineProps<{
  services: Service[];
  modelValue: (ServiceMethod & { serviceName?: string }) | null;
}>();
const emit = defineEmits<{ (e: 'update:modelValue', v: (ServiceMethod & { serviceName?: string }) | null): void }>();

const activeServices = ref<string[]>([]);

const getMethodType = (type: string) => {
  const typeMap: Record<string, string> = {
    unary: 'primary',
    'server-streaming': 'warning',
    'client-streaming': 'info',
    'bidirectional-streaming': 'success',
  };
  return typeMap[type] || 'primary';
};

function handleMethodSelect(serviceName: string, method: ServiceMethod) {
  emit('update:modelValue', { ...method, serviceName });
}
</script>

<style scoped>
.service-list {
  padding: 16px;
}

.services-content {
  margin-top: 8px;
}

.service-methods {
  margin-top: 8px;
}

.method-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.method-item:hover {
  background-color: #f5f7fa;
}

.method-item.active {
  background-color: #ecf5ff;
  color: #409eff;
}

.method-item .el-icon {
  font-size: 12px;
}

.no-proto {
  padding: 40px 16px;
  text-align: center;
}

::v-deep(.el-collapse-item__header) {
  font-weight: 600;
}

::v-deep(.el-collapse-item__content) {
  padding-bottom: 16px;
}
</style>
