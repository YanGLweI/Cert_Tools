<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';

const router = useRouter();
const route = useRoute();

const menuItems = [
  { path: '/create-ca', name: '创建 CA 证书', icon: 'Lock' },
  { path: '/create-ssl', name: '创建 SSL 证书', icon: 'Connection' },
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
          <el-icon class="icon">
            <component :is="item.icon" />
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