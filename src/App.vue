<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { Lock, Connection, Monitor } from '@element-plus/icons-vue';

const router = useRouter();
const route = useRoute();

const menuItems = [
  { path: '/create-ca', name: '创建 CA 证书', iconComponent: Lock },
  { path: '/create-ssl', name: '创建 SSL 证书', iconComponent: Connection },
  { path: '/domain-cert', name: 'Windows 域控证书', iconComponent: Monitor },
];

function navigate(path: string) {
  router.push(path);
}
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h1>CertTools</h1>
        <div class="subtitle">自签证书生成工具</div>
      </div>
      <el-menu
        :default-active="route.path"
        class="sidebar-menu"
        @select="navigate"
      >
        <el-menu-item
          v-for="item in menuItems"
          :key="item.path"
          :index="item.path"
        >
          <el-icon :size="18">
            <component :is="item.iconComponent" />
          </el-icon>
          <span>{{ item.name }}</span>
        </el-menu-item>
      </el-menu>
    </aside>
    <main class="main-area">
      <div class="page-header">
        <h2>{{ route.meta.title }}</h2>
      </div>
      <div class="page-content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style scoped>
.sidebar-menu {
  padding: 8px 0;
}
</style>