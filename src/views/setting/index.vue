<template>
  <menu-layout>
    <template v-slot:header-left>
      设置
    </template>
    <template v-slot:header-right>
      <el-button type="primary" size="small" @click="updateConfig">保存</el-button>
    </template>
    <template v-slot:content>
      <div class="zn-setting">
        <el-form label-width="85px" class="zn-setting__form">
          <el-form-item class="tips">
            此处设置的属性均为全局属性
          </el-form-item>
          <el-form-item label="缓存路径">
            <el-input placeholder="请选择缓存路径" v-model="form.cachePath" readonly @click="openDialog"/>
          </el-form-item>
          <el-form-item label="java路径">
            <el-input placeholder="请选择java路径" v-model="form.javaHome" readonly/>
          </el-form-item>
          <el-form-item label="虚拟机参数">
            <el-input placeholder="暂时不可设置" disabled></el-input>
          </el-form-item>
        </el-form>
      </div>
    </template>
  </menu-layout>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue'
import MenuLayout from '../../layout/MenuLayout.vue'
import { queryGlobalConfig, updateGlobalConfig } from '../../api/config'
import { ConfigVo } from '../../types/config'
import { open } from '@tauri-apps/api/dialog'

const form = reactive<ConfigVo>({
  cachePath: '',
  javaHome: ''
})
onMounted(() => {
  queryGlobalConfig()
  .then(res => {
    form.cachePath = res.cachePath
    form.javaHome = res.javaHome
  })
})

const openDialog = async () => {
  const selected = await open({
    multiple: false,
    directory: true
  })
  if (selected) {
    // 点击了确认
    form.cachePath = selected as string
  }
}
const updateConfig = () => {
  updateGlobalConfig({config: form})
  .then(res => {
    // @ts-ignore
    ElMessage.success('更新成功,重启后生效')
  })
}

</script>

<style scoped lang="scss">
@include b(setting) {
  padding: 0px 40px;
  @include e(form) {
    .tips {
      color: red;
    }
  }
}
</style>