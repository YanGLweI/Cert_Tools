<script setup lang="ts">
import { ref } from 'vue';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { ElMessage } from 'element-plus';
import type { CertInfo } from '../types';

const props = defineProps<{
  info: CertInfo;
  showFullchain?: boolean;
}>();

const saving = ref(false);

async function saveFile(defaultName: string, content: string) {
  saving.value = true;
  try {
    const filePath = await save({
      defaultPath: defaultName,
      filters: [
        { name: 'Certificate Files', extensions: ['pem', 'crt', 'key', 'cer'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
    if (filePath) {
      await writeTextFile(filePath, content);
      ElMessage.success('文件保存成功');
    }
  } catch (e) {
    ElMessage.error('保存失败: ' + String(e));
  } finally {
    saving.value = false;
  }
}

function saveCert() {
  const cn = props.info.subject.match(/CN\s*=\s*([^,\n]+)/)?.[1] || 'certificate';
  saveFile(`${cn}.crt`, props.info.cert_pem);
}

function saveKey() {
  const cn = props.info.subject.match(/CN\s*=\s*([^,\n]+)/)?.[1] || 'certificate';
  saveFile(`${cn}.key`, props.info.key_pem);
}

function saveFullchain() {
  const cn = props.info.subject.match(/CN\s*=\s*([^,\n]+)/)?.[1] || 'certificate';
  saveFile(`${cn}-fullchain.crt`, props.info.cert_pem);
}

function formatDate(dateStr: string): string {
  // Format is "YYYY-MM-DD HH:MM:SS UTC" from Rust backend, already readable
  if (!dateStr || dateStr === 'Invalid Date') return '-';
  return dateStr;
}
</script>

<template>
  <div class="cert-info-panel">
    <el-descriptions
      :column="1"
      border
      size="small"
      title="证书信息"
    >
      <el-descriptions-item label="主题 (Subject)">
        {{ info.subject }}
      </el-descriptions-item>
      <el-descriptions-item v-if="info.issuer" label="签发者 (Issuer)">
        {{ info.issuer }}
      </el-descriptions-item>
      <el-descriptions-item label="序列号">
        <code class="serial">{{ info.serial_number }}</code>
      </el-descriptions-item>
      <el-descriptions-item label="有效起始">
        {{ formatDate(info.valid_from) }}
      </el-descriptions-item>
      <el-descriptions-item label="有效截止">
        {{ formatDate(info.valid_to) }}
      </el-descriptions-item>
      <el-descriptions-item label="SHA-256 指纹">
        <code class="fingerprint">{{ info.sha256_fingerprint }}</code>
      </el-descriptions-item>
      <el-descriptions-item label="SHA-1 指纹">
        <code class="fingerprint">{{ info.sha1_fingerprint }}</code>
      </el-descriptions-item>
      <el-descriptions-item label="密钥算法">
        {{ info.key_algorithm }}
      </el-descriptions-item>
      <el-descriptions-item v-if="info.san && info.san.length > 0" label="SAN">
        <div v-for="(dns, i) in info.san" :key="i">
          <code>{{ dns }}</code>
        </div>
      </el-descriptions-item>
    </el-descriptions>

    <div class="actions">
      <el-button
        type="success"
        class="btn-primary"
        :loading="saving"
        @click="saveCert"
      >
        <el-icon><Download /></el-icon>
        保存证书
      </el-button>
      <el-button
        type="warning"
        :loading="saving"
        @click="saveKey"
      >
        <el-icon><Key /></el-icon>
        保存私钥
      </el-button>
      <el-button
        v-if="showFullchain"
        :loading="saving"
        @click="saveFullchain"
      >
        <el-icon><Files /></el-icon>
        保存完整链
      </el-button>
    </div>
  </div>
</template>

<style scoped>
.cert-info-panel {
  margin-top: 20px;
}

.actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
  flex-wrap: wrap;
}

code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  word-break: break-all;
}

.serial {
  color: var(--el-color-primary);
}

.fingerprint {
  color: #94A3B8;
}
</style>