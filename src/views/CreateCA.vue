<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import CertInfoDisplay from '../components/CertInfoDisplay.vue';
import type { CaParams, CertInfo } from '../types';
import { KeyAlgorithm } from '../types';

const generating = ref(false);
const result = ref<CertInfo | null>(null);

const form = reactive<CaParams>({
  subject: {
    common_name: '',
    organization: '',
    organizational_unit: '',
    country: '',
  },
  key_algorithm: KeyAlgorithm.Rsa4096,
  validity_days: 3650,
});

const formRules = {
  'subject.common_name': [{ required: true, message: '请输入通用名称', trigger: 'blur' }],
};

const formRef = ref();

async function generate() {
  if (!formRef.value) return;
  const valid = await formRef.value.validate().catch(() => false);
  if (!valid) return;

  generating.value = true;
  result.value = null;
  try {
    const info = await invoke<CertInfo>('generate_ca', { params: { ...form } });
    result.value = info;
    ElMessage.success('CA 证书生成成功');
  } catch (e) {
    ElMessage.error('生成失败: ' + String(e));
  } finally {
    generating.value = false;
  }
}
</script>

<template>
  <div>
    <div class="form-card">
      <div class="form-card-title">签发者信息</div>
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
            placeholder="例如: My Root CA"
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
            placeholder="例如: Security Team"
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
        <el-form-item label="密钥算法">
          <el-select v-model="form.key_algorithm" style="width: 240px">
            <el-option label="RSA 2048" :value="KeyAlgorithm.Rsa2048" />
            <el-option label="RSA 4096 (推荐)" :value="KeyAlgorithm.Rsa4096" />
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
        <el-form-item>
          <el-button
            type="success"
            class="btn-primary"
            :loading="generating"
            @click="generate"
          >
            <el-icon><Plus /></el-icon>
            生成 CA 证书
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <CertInfoDisplay
      v-if="result"
      :info="result"
    />
  </div>
</template>