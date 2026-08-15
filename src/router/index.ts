import { createRouter, createWebHistory } from 'vue-router';
import CreateCA from '../views/CreateCA.vue';
import CreateSSL from '../views/CreateSSL.vue';
import CreateDomainCert from '../views/CreateDomainCert.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/create-ca',
    },
    {
      path: '/create-ca',
      name: 'CreateCA',
      component: CreateCA,
      meta: { title: '创建 CA 证书' },
    },
    {
      path: '/create-ssl',
      name: 'CreateSSL',
      component: CreateSSL,
      meta: { title: '创建 SSL 证书' },
    },
    {
      path: '/domain-cert',
      name: 'DomainCert',
      component: CreateDomainCert,
      meta: { title: 'Windows 域控证书' },
    },
  ],
});

export default router;