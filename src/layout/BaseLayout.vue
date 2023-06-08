<template>
  <div class="zn-base">
    <div class="zn-base__left">
      <div class="item">
        <router-link to="/home">主页</router-link>
      </div>
      <div class="item">
        <router-link to="/list">列表</router-link>
      </div>
      <div class="item">
        <router-link to="/download">下载</router-link>
      </div>
      <div class="item">
        <router-link to="/mirror">镜像</router-link>
      </div>
      <div class="item">
        <router-link to="/setting">设置</router-link>
      </div>
      <div class="item">
        <router-link to="/about">关于</router-link>
      </div>
    </div>
    <div class="zn-base__right">
      <router-view></router-view>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAppConfigStore } from '../store'
const appConfig = useAppConfigStore()

</script>

<style scoped lang="scss">
@include b(base) {
  display: flex;
  width: 100%;
  height: 100%;
  @include e(left) {
    width: 150px;
    height: 100%;
    display: flex;
    background-color: #b0bccf;
    // background-color: white;
    flex-wrap: wrap;
    align-content: center;
    @include zn-shadow;
    .item {
      width:100%;
      text-align: center;
      a {
        text-decoration: none;
        outline: none;
        display: inline-block;
        padding: 7px 50px;
        color: black;
      }
      .router-link-active {
        background-color: #cacad5;
        color: v-bind('appConfig.themeColor');
      }
    }
  }
  @include e(right) {
    width: calc(100vw - 150px);
    overflow: auto;
  }
}
</style>