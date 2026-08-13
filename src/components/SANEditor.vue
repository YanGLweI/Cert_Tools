<script setup lang="ts">
import { ref } from 'vue';
import type { SanEntry } from '../types';

const emit = defineEmits<{
  (e: 'update:san', san: SanEntry): void;
}>();

const props = withDefaults(defineProps<{
  san?: SanEntry;
}>(), {
  san: () => ({ dns_names: [''], ip_addresses: [''] }),
});

const dnsNames = ref<string[]>([...props.san.dns_names]);
const ipAddresses = ref<string[]>([...props.san.ip_addresses]);

function emitChange() {
  emit('update:san', {
    dns_names: dnsNames.value.filter(Boolean),
    ip_addresses: ipAddresses.value.filter(Boolean),
  });
}

function addDns() {
  dnsNames.value.push('');
}

function removeDns(index: number) {
  dnsNames.value.splice(index, 1);
  emitChange();
}

function addIp() {
  ipAddresses.value.push('');
}

function removeIp(index: number) {
  ipAddresses.value.splice(index, 1);
  emitChange();
}

function validateIp(ip: string): boolean {
  if (!ip) return true;
  const parts = ip.split('.');
  if (parts.length !== 4) return false;
  return parts.every((p) => {
    const n = parseInt(p, 10);
    return !isNaN(n) && n >= 0 && n <= 255;
  });
}

function onDnsInput() {
  emitChange();
}

function onIpInput() {
  emitChange();
}
</script>

<template>
  <div class="san-editor">
    <div class="san-section">
      <div class="san-section-label">DNS 名称</div>
      <div v-for="(_dns, index) in dnsNames" :key="'dns-' + index" class="san-row">
        <el-input
          v-model="dnsNames[index]"
          placeholder="例如: example.com"
          size="small"
          @input="onDnsInput"
        >
          <template #prefix>
            <el-icon><Monitor /></el-icon>
          </template>
        </el-input>
        <el-button
          size="small"
          type="danger"
          :disabled="dnsNames.length <= 1"
          @click="removeDns(index)"
        >
          <el-icon><Delete /></el-icon>
        </el-button>
      </div>
      <el-button size="small" class="add-btn" @click="addDns">
        <el-icon><Plus /></el-icon>
        添加 DNS
      </el-button>
    </div>

    <div class="san-section">
      <div class="san-section-label">IP 地址</div>
      <div v-for="(ip, index) in ipAddresses" :key="'ip-' + index" class="san-row">
        <el-input
          v-model="ipAddresses[index]"
          placeholder="例如: 192.168.1.1"
          size="small"
          :class="{ 'is-error': ip && !validateIp(ip) }"
          @input="onIpInput"
        >
          <template #prefix>
            <el-icon><Iphone /></el-icon>
          </template>
        </el-input>
        <el-button
          size="small"
          type="danger"
          :disabled="ipAddresses.length <= 1"
          @click="removeIp(index)"
        >
          <el-icon><Delete /></el-icon>
        </el-button>
      </div>
      <el-button size="small" class="add-btn" @click="addIp">
        <el-icon><Plus /></el-icon>
        添加 IP
      </el-button>
    </div>
  </div>
</template>

<style scoped>
.san-editor {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.san-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.san-row .el-input {
  flex: 1;
}

.san-section {
  margin-bottom: 16px;
}

.san-section:last-child {
  margin-bottom: 0;
}

.san-section-label {
  font-size: 12px;
  color: #94A3B8;
  margin-bottom: 8px;
}

.add-btn {
  margin-top: 4px;
}

.is-error :deep(.el-input__wrapper) {
  border-color: var(--color-destructive) !important;
}
</style>