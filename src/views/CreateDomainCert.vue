<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { writeFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { save } from '@tauri-apps/plugin-dialog';
import { ElMessage } from 'element-plus';
import CertInfoDisplay from '../components/CertInfoDisplay.vue';
import SANEditor from '../components/SANEditor.vue';
import type { DomainCertResult, SanEntry } from '../types';
import { KeyAlgorithm } from '../types';

const generating = ref(false);
const result = ref<DomainCertResult | null>(null);

const form = reactive({
  subject: {
    common_name: '',
    organization: '',
    organizational_unit: '',
    country: '',
  },
  san: {
    dns_names: [''],
    ip_addresses: [''] as string[],
  },
  key_algorithm: KeyAlgorithm.Rsa2048,
  validity_days: 3650,
  pfx_password: '',
});

const formRules = {
  'subject.common_name': [{ required: true, message: '请输入通用名称', trigger: 'blur' }],
  pfx_password: [
    { required: true, message: '请输入 PFX 密码', trigger: 'blur' },
    { min: 6, max: 128, message: '密码长度需在 6-128 之间', trigger: 'blur' },
  ],
};

const formRef = ref();

async function generate() {
  if (!formRef.value) return;
  const valid = await formRef.value.validate().catch(() => false);
  if (!valid) return;

  generating.value = true;
  result.value = null;
  try {
    const res = await invoke<DomainCertResult>('generate_domain_certificate', {
      params: {
        subject: { ...form.subject },
        san: { ...form.san },
        key_algorithm: form.key_algorithm,
        validity_days: form.validity_days,
      },
      pfxPassword: form.pfx_password,
    });
    result.value = res;
    ElMessage.success('域控证书生成成功');
  } catch (e) {
    ElMessage.error('生成失败：' + String(e));
  } finally {
    generating.value = false;
  }
}

async function downloadPfx() {
  if (!result.value) return;
  
  try {
    const filePath = await save({
      defaultPath: 'server.pfx',
      filters: [
        { name: 'PFX Files', extensions: ['pfx'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
    
    if (!filePath) return;
    
    const pfxData = new Uint8Array(result.value.pfx_buffer);
    await writeFile(filePath, pfxData);
    ElMessage.success('PFX 文件下载成功');
  } catch (e) {
    ElMessage.error('下载失败：' + String(e));
  }
}

async function downloadCaCert() {
  if (!result.value) return;
  
  try {
    const filePath = await save({
      defaultPath: 'ca.crt',
      filters: [
        { name: 'Certificate Files', extensions: ['crt', 'pem'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
    
    if (!filePath) return;
    
    await writeTextFile(filePath, result.value.ca_cert_pem);
    ElMessage.success('CA 证书下载成功');
  } catch (e) {
    ElMessage.error('下载失败：' + String(e));
  }
}

function clearAll() {
  result.value = null;
}
</script>

<template>
  <div>
    <div class="form-card">
      <div class="form-card-title">Windows 域控证书信息</div>
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
            placeholder="例如：sm-dc01.sm.local"
          />
        </el-form-item>
        <el-form-item label="组织 (O)" prop="subject.organization">
          <el-input
            v-model="form.subject.organization"
            placeholder="例如：MyOrg"
          />
        </el-form-item>
        <el-form-item label="组织单位 (OU)" prop="subject.organizational_unit">
          <el-input
            v-model="form.subject.organizational_unit"
            placeholder="例如：IT Department"
          />
        </el-form-item>
        <el-form-item label="国家 (C)" prop="subject.country">
          <el-input
            v-model="form.subject.country"
            placeholder="例如：CN"
            maxlength="2"
            style="width: 120px"
          />
        </el-form-item>

        <el-divider content-position="left">Subject Alternative Names (SAN)</el-divider>

        <el-form-item label="SAN 条目">
          <SANEditor
            :san="form.san"
            @update:san="(val: SanEntry) => form.san = val"
          />
          <div class="hint-text">建议配置完整的域控制器 FQDN 和通配符域名</div>
        </el-form-item>

        <el-divider content-position-left>密钥与有效期</el-divider>

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
            :step="365"
            style="width: 200px"
          />
        </el-form-item>

        <el-divider content-position="left">PFX 保护密码</el-divider>

        <el-form-item label="PFX 密码" prop="pfx_password">
          <el-input
            v-model="form.pfx_password"
            type="password"
            show-password
            placeholder="请输入至少 6 位密码"
            maxlength="128"
            style="width: 300px"
          />
          <div class="hint-text">密码将用于保护 PFX 文件，请妥善保管</div>
        </el-form-item>

        <el-form-item>
          <el-button
            type="success"
            class="btn-primary"
            :loading="generating"
            @click="generate"
          >
            <el-icon><Plus /></el-icon>
            生成域控证书
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <CertInfoDisplay
      v-if="result"
      :info="result.cert_info"
      :show-fullchain="false"
    >
      <template #actions>
        <div class="download-section">
          <div class="download-instruction">
            <h4>使用指南：</h4>
            <ol>
              <li><strong>下载 PFX 文件</strong> - 将此文件导入 Windows 域控的本地计算机账户</li>
              <li><strong>下载 CA 证书</strong> - 将此证书部署到客户端以建立 LDAPS 信任链</li>
            </ol>
          </div>
          
          <div class="download-buttons">
            <el-button 
              type="primary" 
              @click="downloadPfx"
              plain
            >
              <el-icon><Download /></el-icon>
              下载 PFX 文件 (server.pfx)
            </el-button>
            <el-button 
              type="warning" 
              @click="downloadCaCert"
              plain
            >
              <el-icon><Document /></el-icon>
              下载 CA 证书 (ca.crt)
            </el-button>
            <el-button @click="clearAll">
              <el-icon><Refresh /></el-icon>
              重新生成
            </el-button>
          </div>
        </div>
      </template>
    </CertInfoDisplay>
  </div>
</template>

<style scoped>
.hint-text {
  font-size: 12px;
  color: var(--color-secondary);
  margin-top: 4px;
}

.download-section {
  padding: 16px;
  background-color: var(--color-primary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-top: 16px;
}

.download-instruction h4 {
  margin: 0 0 12px 0;
  color: var(--color-foreground);
  font-size: 14px;
}

.download-instruction ol {
  margin: 0;
  padding: 0 0 0 20px;
  color: var(--color-foreground);
  font-size: 13px;
  line-height: 1.6;
}

.download-buttons {
  display: flex;
  gap: 12px;
  margin-top: 16px;
  flex-wrap: wrap;
}

.download-buttons .el-button {
  flex: 0 0 auto;
}
</style>
