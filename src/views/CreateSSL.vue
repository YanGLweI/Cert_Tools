<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { ElMessage } from 'element-plus';
import CertInfoDisplay from '../components/CertInfoDisplay.vue';
import SANEditor from '../components/SANEditor.vue';
import type { SslParams, CertInfo, SubjectInfo, SanEntry } from '../types';
import { KeyAlgorithm } from '../types';

const generating = ref(false);
const result = ref<CertInfo | null>(null);

// CA state
const caCertPem = ref('');
const caKeyPem = ref('');
const caInfo = ref<CertInfo | null>(null);

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
    const content = await readTextFile(selected as string);
    const info = await invoke<CertInfo>('parse_certificate', { certPem: content });
    caCertPem.value = content;
    caInfo.value = info;
    ElMessage.success('CA 证书导入成功');
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
    const content = await readTextFile(selected as string);
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

function clearCa() {
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

      <div v-if="!caInfo" class="upload-row">
        <div class="upload-zone" @click="importCaCert">
          <el-icon class="icon"><Upload /></el-icon>
          <div class="text">点击导入 CA 证书文件 (.crt / .pem)</div>
          <div class="hint">CA 证书用于签发 SSL 证书</div>
        </div>
        <div class="upload-zone" @click="importCaKey">
          <el-icon class="icon"><Key /></el-icon>
          <div class="text">点击导入 CA 私钥文件 (.key)</div>
          <div class="hint">与 CA 证书配对的私钥</div>
        </div>
      </div>

      <div v-else class="ca-ready">
        <div class="status-badge success">
          <span class="dot" />
          CA 已就绪
        </div>
        <div class="ca-info-summary">
          <div><strong>Subject:</strong> {{ caInfo.subject }}</div>
          <div v-if="caKeyPem"><strong>Private Key:</strong> 已导入</div>
        </div>
        <el-button size="small" @click="clearCa">
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

.ca-ready {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ca-info-summary {
  font-size: 13px;
  line-height: 1.8;
  color: var(--color-foreground);
}

.ca-info-summary strong {
  color: #94A3B8;
}
</style>