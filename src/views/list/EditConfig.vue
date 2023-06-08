<template>
  <menu-layout>
    <template v-slot:header-left>
      <el-button :icon="Back" text bg size="small" @click="router.back()"/>
      编辑配置
    </template>
    <template v-slot:header-right>
      <el-button type="primary" size="small" @click="saveEvent">保存</el-button>
    </template>
    <template v-slot:content>
      <el-form
        ref="formRef"
        :model="form" 
        :rules="rules"
        label-width="120px"
        v-loading="isLoading">
        <el-form-item label="服务器名称" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="服务器核心" prop="brand">
          {{ form.brand }}
        </el-form-item>
        <el-form-item label="服务器版本" prop="version">
          {{ form.version }}
        </el-form-item>
        <el-form-item label="服务器进程id">
          {{ form.p_id }}
        </el-form-item>
        <el-form-item label="日志进程id">
          {{ form.t_id }}
        </el-form-item>
        <el-form-item label="JVM参数">
          <el-input v-model="form.vm_options" type="textarea" placeholder="非高玩勿动"/>
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea"/>
        </el-form-item>
        <el-form-item label="properties" prop="properties">
          <el-input v-model="form.properties" type="textarea" :rows="60"/>
        </el-form-item>
      </el-form>
    </template>
  </menu-layout>
</template>

<script setup lang="ts">
import { reactive, onMounted, ref} from 'vue'
import { useRouter, useRoute, LocationQueryValue } from 'vue-router'
import { Back } from '@element-plus/icons-vue'
import { ElMessage, FormInstance, FormRules } from 'element-plus'

import { ServerInstanceVo } from '../../types/server'

import MenuLayout from '../../layout/MenuLayout.vue'
import { queryInstanceById, updateServerInstanceById } from '../../api/server'

const router = useRouter()
const route = useRoute()
const instanceId = Number(route.query.instanceId as LocationQueryValue)
const form = reactive<ServerInstanceVo>({
  id: 0,
  name: '',
  path: '',
  item_id: 0,
  brand: '',
  version: '',
  remark: ''
})

onMounted(() => {
  query()
})

// 表单校验
const formRef = ref<FormInstance>()
const rules = reactive<FormRules>({
  name: [
    { required: true, message: '请填写服务器名称', trigger: 'blur' },
    { min: 3, max: 20, message: '服务器名称为3-20个字符', trigger: 'change' }
  ],
  brand: [
    { required: true, message: '请选择服务器类型', trigger: 'change' }
  ],
  version: [
    { required: true, message: '请选择服务器版本', trigger: 'blur' }
  ],
  properties: [
    { required: true, message: '请填写服务器名称', trigger: 'blur' }
  ]
})
const validateForm = () => {
  return new Promise((resolve, reject) => {
    formRef.value?.validate(valid => {
      if (valid) {
        resolve(valid)
      } else {
        reject(valid)
      }
    })
  })
}

// 确认弹框
const isLoading = ref(false)
const saveEvent = () => {
  isLoading.value = true
  validateForm()
  .then(res => {
    return updateServerInstanceById({
      instance: {...form}
    })
  }).then(res => {
    // @ts-ignore
    ElMessage.success("更新成功")
    router.back()
  }).finally(() =>  {
    isLoading.value = false
  })

}

// 查询当前服务器实例信息
const query = () => {
  isLoading.value = true
  queryInstanceById({instanceId: instanceId}).then(res => {
    form.id = res.id
    form.name = res.name
    form.path = res.path
    form.t_id = res.t_id
    form.p_id = res.p_id
    form.item_id = res.item_id
    form.brand = res.brand
    form.version = res.version
    form.remark = res.remark
    form.properties = res.properties
    form.vm_options = res.vm_options
  }).finally(() => {
    isLoading.value = false
  })
}
</script>

<style scoped lang="scss">
</style>