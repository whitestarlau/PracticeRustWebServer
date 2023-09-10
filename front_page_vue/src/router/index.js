import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import GoodsDetailView from '../views/GoodsDetailView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    // {
    //   path: '/',
    //   name: 'home',
    //   component: HomeView
    // },
    {
      path: '/about',
      name: 'about',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AboutView.vue')
    },
    {
      path: '/sign_up',
      name: 'sign_up',
      component: () => import('../views/SignUpView.vue')
    },
    {
      path: '/sign_in',
      name: 'signIn',
      component: () => import('../views/SignInView.vue')
    },
    {
      path: '/',
      name: 'goodsList',
      component: () => import('../views/GoodsListView.vue')
    },
    {
      path: '/goods_detail/:id',
      name: 'goodsDetail',
      component: GoodsDetailView
    },
  ]
})

export default router
