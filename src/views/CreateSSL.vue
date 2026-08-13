<script setup lang="ts">
import { ref, reactive, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile as readFsFile } from '@tauri-apps/plugin-fs';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ElMessage } from 'element-plus';
import type { UnlistenFn } from '@tauri-apps/api/event';
import CertInfoDisplay from '../components/CertInfoDisplay.vue';
import SANEditor from '../components/SANEditor.vue';
import type { SslParams, CertInfo, SanEntry } from '../types';
import { KeyAlgorithm } from '../types';

const generating = ref(false);
const result = ref<CertInfo | null>(null);

// CA state
const caCertPem = ref('');
const caKeyPem = ref('');
const caInfo = ref<CertInfo | null>(null);

// Drag state for visual feedback
const dragOverCert = ref(false);
const dragOverKey = ref(false);

const form = reactive<SslParams>({
  subject: {
    common_name: '',
    organization: '',
    organizational_unit: '',
    country: '',
  },
  san: {
    dns_names: [''],
    ip_addresses: [''],
  },
  key_algorithm: KeyAlgorithm.Rsa2048,
  validity_days: 730,
});

const formRules = {
  'subject.common_name': [{ required: true, message: '请输入通用名称', trigger: 'blur' }],
};

const formRef = ref();
let unlistenDragDrop: UnlistenFn | null = null;

onMounted(async () => {
  // Listen for Tauri native drag-drop events
  unlistenDragDrop = await getCurrentWindow().onDragDropEvent((event) => {
    const type = event.payload.type;

    if (type === 'over') {
      const pos = event.payload.position;
      const zoneWidth = (window.innerWidth - 240 - 48) / 2;
      const relativeX = pos.x - 240 - 24;
      dragOverCert.value = relativeX < zoneWidth;
      dragOverKey.value = relativeX >= zoneWidth;
    } else if (type === 'leave') {
      dragOverCert.value = false;
      dragOverKey.value = false;
    } else if (type === 'enter' || type === 'drop') {
      dragOverCert.value = false;
      dragOverKey.value = false;

      const filePath = event.payload.paths?.[0];
      if (!filePath) return;

      const lower = filePath.toLowerCase();
      const isCert = lower.endsWith('.crt') || lower.endsWith('.cer');
      const isKey = lower.endsWith('.key');
      const isPem = lower.endsWith('.pem');

      if (!isCert && !isKey && !isPem) {
        ElMessage.warning('请拖入证书文件 (.crt/.cer/.pem) 或私钥文件 (.key)');
        return;
      }

      handleDroppedFile(filePath, isCert || isPem);
    }
  });
});

onBeforeUnmount(() => {
  unlistenDragDrop?.();
});

async function handleDroppedFile(filePath: string, preferCert: boolean) {
  try {
    const content = await readFsFile(filePath);

    // If it's .pem, try to detect type from content
    const isCertContent = content.includes('BEGIN CERTIFICATE');
    const isKeyContent = content.includes('BEGIN PRIVATE KEY') || content.includes('BEGIN RSA PRIVATE KEY');

    if (preferCert && isCertContent) {
      await importCertContent(content);
    } else if (isKeyContent) {
      caKeyPem.value = content;
      ElMessage.success('CA 私钥导入成功');
    } else if (isCertContent) {
      await importCertContent(content);
    } else {
      ElMessage.warning('无法识别文件类型，请确认文件内容');
    }
  } catch (e) {
    ElMessage.error('文件读取失败: ' + String(e));
  }
}

async function importCertContent(content: string) {
  const info = await invoke<CertInfo>('parse_certificate', { certPem: content });
  caCertPem.value = content;
  caInfo.value = info;
  ElMessage.success('CA 证书导入成功');
}

async function importCaCert() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'Certificate Files', extensions: ['crt', 'pem', 'cer'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
    if (!selected) return;
    const content = await readFsFile(selected as string);
    await importCertContent(content);
  } catch (e) {
    ElMessage.error('CA 证书导入失败: ' + String(e));
  }
}

async function importCaKey() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'Key Files', extensions: ['key', 'pem'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
    if (!selected) return;
    const content = await readFsFile(selected as string);
    caKeyPem.value = content;
    ElMessage.success('CA 私钥导入成功');
  } catch (e) {
    ElMessage.error('CA 私钥导入失败: ' + String(e));
  }
}

async function generate() {
  if (!formRef.value) return;
  const valid = await formRef.value.validate().catch(() => false);
  if (!valid) return;

  if (!caCertPem.value || !caKeyPem.value) {
    ElMessage.warning('请先导入 CA 证书和私钥');
    return;
  }

  generating.value = true;
  result.value = null;
  try {
    const info = await invoke<CertInfo>('generate_ssl', {
      params: { ...form },
      caCertPem: caCertPem.value,
      caKeyPem: caKeyPem.value,
    });
    result.value = info;
    ElMessage.success('SSL 证书生成成功');
  } catch (e) {
    ElMessage.error('生成失败: ' + String(e));
  } finally {
    generating.value = false;
  }
}

function updateSan(san: SanEntry) {
  form.san = san;
}

function clearCert() {
  caCertPem.value = '';
  caInfo.value = null;
}

function clearAll() {
  caCertPem.value = '';
  caKeyPem.value = '';
  caInfo.value = null;
}
</script>

<template>
  <div>
    <!-- CA Import Section -->
    <div class="form-card">
      <div class="form-card-title">CA 证书导入</div>

      <!-- Step 1: Import both cert and key (nothing imported yet) -->
      <div v-if="!caInfo && !caKeyPem" class="upload-row">
        <div
          class="upload-zone"
          :class="{ 'drag-over': dragOverCert }"
          @click="importCaCert"
        >
          <el-icon><UploadFilled /></el-icon>
          <div class="text">点击或拖动 CA 证书到此处</div>
          <div class="hint">支持 .crt / .pem / .cer 格式</div>
        </div>
        <div
          class="upload-zone"
          :class="{ 'drag-over': dragOverKey }"
          @click="importCaKey"
        >
          <el-icon><Key /></el-icon>
          <div class="text">点击或拖动 CA 私钥到此处</div>
          <div class="hint">支持 .key / .pem 格式</div>
        </div>
      </div>

      <!-- Step 2: Cert imported, key still needed -->
      <div v-else-if="caInfo && !caKeyPem" class="import-status">
        <div class="imported-item success">
          <el-icon><CircleCheckFilled /></el-icon>
          <div class="imported-info">
            <div class="imported-label">CA 证书</div>
            <div class="imported-detail">{{ caInfo.subject }}</div>
          </div>
          <el-button size="small" text @click="clearCert">
            <el-icon><Close /></el-icon>
          </el-button>
        </div>

        <div
          class="upload-zone single"
          :class="{ 'drag-over': dragOverKey }"
          @click="importCaKey"
        >
          <el-icon><Key /></el-icon>
          <div class="text">点击或拖动 CA 私钥到此处</div>
          <div class="hint">需要与证书配对的私钥才能签发 SSL 证书</div>
        </div>
      </div>

      <!-- Step 3: Both imported -->
      <div v-else class="import-status">
        <div class="imported-item success">
          <el-icon><CircleCheckFilled /></el-icon>
          <div class="imported-info">
            <div class="imported-label">CA 证书</div>
            <div class="imported-detail">{{ caInfo?.subject }}</div>
          </div>
        </div>
        <div class="imported-item success">
          <el-icon><CircleCheckFilled /></el-icon>
          <div class="imported-info">
            <div class="imported-label">CA 私钥</div>
            <div class="imported-detail">已就绪</div>
          </div>
        </div>
        <el-button size="small" @click="clearAll">
          <el-icon><Refresh /></el-icon>
          重新导入
        </el-button>
      </div>
    </div>

    <!-- SSL Form -->
    <div class="form-card">
      <div class="form-card-title">SSL 证书信息</div>
      <el-form
        ref="formRef"
        :model="form"
        :rules="formRules"
        label-width="140px"
        label-position="left"
        size="default"
      >
        <el-form-item label="通用名称 (CN)" prop="subject.common_name">
          <el-input
            v-model="form.subject.common_name"
            placeholder="例如: example.com"
          />
        </el-form-item>
        <el-form-item label="组织 (O)" prop="subject.organization">
          <el-input
            v-model="form.subject.organization"
            placeholder="例如: IT Department"
          />
        </el-form-item>
        <el-form-item label="组织单位 (OU)" prop="subject.organizational_unit">
          <el-input
            v-model="form.subject.organizational_unit"
            placeholder="例如: Web Server"
          />
        </el-form-item>
        <el-form-item label="国家 (C)" prop="subject.country">
          <el-input
            v-model="form.subject.country"
            placeholder="例如: CN"
            maxlength="2"
            style="width: 120px"
          />
        </el-form-item>

        <el-divider content-position="left">Subject Alternative Names (SAN)</el-divider>

        <el-form-item label="SAN 条目">
          <SANEditor
            :san="form.san"
            @update:san="updateSan"
          />
        </el-form-item>

        <el-divider content-position="left">密钥与有效期</el-divider>

        <el-form-item label="密钥算法">
          <el-select v-model="form.key_algorithm" style="width: 240px">
            <el-option label="RSA 2048 (推荐)" :value="KeyAlgorithm.Rsa2048" />
            <el-option label="RSA 4096" :value="KeyAlgorithm.Rsa4096" />
            <el-option label="ECDSA P-256" :value="KeyAlgorithm.EcdsaP256" />
            <el-option label="ECDSA P-384" :value="KeyAlgorithm.EcdsaP384" />
          </el-select>
        </el-form-item>
        <el-form-item label="有效期（天）">
          <el-input-number
            v-model="form.validity_days"
            :min="1"
            :max="36500"
            :step="30"
            style="width: 200px"
          />
        </el-form-item>
        <el-form-item>
          <el-button
            type="success"
            class="btn-primary"
            :loading="generating"
            :disabled="!caCertPem || !caKeyPem"
            @click="generate"
          >
            <el-icon><Plus /></el-icon>
            生成 SSL 证书
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <CertInfoDisplay
      v-if="result"
      :info="result"
      :show-fullchain="true"
    />
  </div>
</template>

<style scoped>
.upload-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.upload-zone.single {
  margin-top: 12px;
}

.import-status {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.imported-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background-color: var(--color-primary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.imported-item.success {
  border-color: rgba(34, 197, 94, 0.3);
}

.imported-item .el-icon:first-child {
  font-size: 20px;
  color: var(--color-accent);
  flex-shrink: 0;
}

.imported-info {
  flex: 1;
  min-width: 0;
}

.imported-label {
  font-size: 12px;
  color: #94A3B8;
}

.imported-detail {
  font-size: 13px;
  color: var(--color-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>