<template>
  <menu-layout>
    <template v-slot:header-left>
      关于
    </template>
    <template v-slot:content>
      <el-form label-width="100px" class="zn-about">
        <el-form-item label="Author">
          Meinil
        </el-form-item>
        <el-form-item label="Github">
          <el-link href="https://github.com/Meinil" target="_blank">Zeperion</el-link>
        </el-form-item>
        <el-form-item label="版本">
          0.0.1
        </el-form-item>
        <el-form-item label="Tauri交流群">
          780521085
        </el-form-item>
      </el-form>
    </template>
  </menu-layout>
</template>

<script setup lang="ts">
import MenuLayout from '../../layout/MenuLayout.vue'

</script>

<style scoped lang="scss">
@include b(about) {
  width: 200px;
  margin: 0 auto;
}
</style>