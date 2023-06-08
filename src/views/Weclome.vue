<template>
  <div class="container">
    <div class="zn-weclome">
      <h1 class="zn-weclome__title">欢迎使用zeperion</h1>
      <el-form :model="form" label-width="120px">
        <el-form-item label="缓存路径">
          <el-input v-model="form.cachePath" placeholder="请选择数据的缓存路径"/>
        </el-form-item>
        <el-form-item label="JAVA_HOME">
          <el-input v-model="form.javaHome" placeholder="请选择java路径"/>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit">下一步</el-button>
        </el-form-item>
      </el-form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api'
import { onMounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { BaseDirectory, writeFile, createDir } from '@tauri-apps/api/fs'

const router = useRouter()

const form = reactive({
  cachePath: '',
  javaHome: ''
})

const onSubmit = () => {
  writeFile({ path: 'app.conf', contents: 'file contents' }, { dir: BaseDirectory.Home })
  .then(res => {
    console.log(res)
  })
}

onMounted(() => {
  invoke('query_config')
  .then(res => {
    console.log(res)
  })

  ; 
})
</script>

<style scoped lang="scss">
.container {
  width: 100%;
  height: 100%;
  overflow: hidden;
  @include b(weclome) {
    width: 400px;
    height: 170px;
    margin: 0 auto;
    text-align: center;
    margin-top: calc(25vh);
    @include e(title) {
      font-weight: bold;
      padding: 15px;
      font-size: 25px;
    }
  }
}

</style>