<template>
  <el-card class="zn-list">
    <div class="zn-list__left">
      <slot name="list-left"></slot>
    </div>
    <div class="zn-list__right">
      <slot name="list-right"></slot>
    </div>
  </el-card>
</template>

<script setup lang="ts">

</script>

<style scoped lang="scss">
@include b(list) {
  display: flex;
  margin-bottom: 5px;
  &:last-of-type {
    margin-bottom: 0px;
  }
  @include e(left) {
    display: flex;
    line-height: 32px;
    .progress {
      width: 150px;
    }
  }

  @include e(right) {
    display: inline-block;
  }
}
:deep(.el-card__body) {
  width: 100%;
  display: flex;
  justify-content: space-between;
}
</style>