import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    component: () => import('../layout/BaseLayout.vue'),
    redirect: '/home',
    children: [
      {
        path: '/weclome',
        component: () => import('../views/Weclome.vue')
      },
      {
        path: '/home',
        component: () => import('../views/home/index.vue')
      },
      {
        path: '/list',
        component: () => import('../views/list/index.vue')
      },
      {
        path: '/list/eula',
        component: () => import('../views/list/Eula.vue')
      },
      {
        path: '/list/terminal',
        component: () => import('../views/list/TerminalView.vue')
      },
      {
        path: '/list/edit',
        component: () => import('../views/list/EditConfig.vue')
      },
      {
        path: '/download',
        component: () => import('../views/download/index.vue')
      },
      {
        path: '/download/import',
        component: () => import('../views/download/ImportResource.vue')
      },
      {
        path: '/mirror',
        component: () => import('../views/mirror/index.vue'),
      },
      {
        path: '/mirror/ImportMirror',
        component: () => import('../views/mirror/ImportMirror.vue'),
      },
      {
        path: '/setting',
        component: () => import('../views/setting/index.vue')
      },
      {
        path: '/about',
        component: () => import('../views/about/index.vue')
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router