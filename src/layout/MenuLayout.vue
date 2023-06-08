<template>
  <div class="zn-menu">
    <header class="zn-menu__header">
      <div>
        <slot name="header-left"></slot>
      </div>
      <div>
        <slot name="header-right"></slot>
      </div>
    </header>
    <main class="zn-menu__content">
      <div class="content-container">
        <slot name="content">
          <div class="content-default" ref="contentDefault">
            <div class="defalut-item">
              <img src="../assets/nothing.svg" alt="空空如也">
            </div>
            <span class="defalut-item">空空如也</span>
          </div>
        </slot>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { MenuStyleType } from '../types/menu'

const props = withDefaults(defineProps<{ contentStyle: MenuStyleType}>(),{
    contentStyle: MenuStyleType.Defalut
  }
)

const contentDefault = ref<HTMLDivElement>()
const contentStyle = computed(() => {
  if (contentDefault) {
    return MenuStyleType.Defalut
  }

  return props.contentStyle
})

</script>

<style scoped lang="scss">
@include b(menu) {
  width: 100%;
  height: 100%;
  background-color: v-bind('MenuStyleType.Menu');
  @include e(header) {
    width: calc(100vw - 170px);
    height: 32px;
    line-height: 32px;
    display: flex;
    padding: 5px 10px;
    justify-content: space-between;
    background-color: white;
    @include zn-shadow;
  }
  @include e(content) {
    height: calc(100vh - 102px); 
    overflow: auto;
    margin: 10px;
    background-color: v-bind('contentStyle');
    padding: 20px;
    @include zn-shadow;
    .content-container {
      width: 100%;
      height: 100%;
    }
    .content-default {
      text-align: center;
      display: flex;
      justify-content: center;
      flex-wrap: wrap;
      margin-top: 20vh;
      .defalut-item {
        width:100%;
        &:last-of-type {
          margin-top: 5px;
        }
      }
      img {
        width: 120px;
        vertical-align: middle;
      }
    }
  }
}
</style>