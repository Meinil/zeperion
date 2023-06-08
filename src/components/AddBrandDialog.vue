<template>
  <el-dialog
    v-model="dialogVisible"
    title="添加服务器类型"
    width="360px"
    align-center>
    <el-form
      ref="formRef"
      :inline="true" 
      :model="form"
      :rules="rules">
      <el-form-item label="服务器类型" prop="brand">
        <el-input v-model="form.brand" placeholder="请填写类型名称名称" />
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="dialogVisible = false" size="small">取 消</el-button>
        <el-button type="primary" @click="confirmDialog" :loading="isLoading" size="small">确 认</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { addServerBrand } from '../api/server'
import type { FormInstance, FormRules } from 'element-plus'

const form = reactive({
  brand: ''
})


// 打开弹窗
const dialogVisible = ref(false)
const openDialog = () => {
  form.brand = ''
  dialogVisible.value = true
}

// 表单校验
const formRef = ref<FormInstance>()
const rules = reactive<FormRules>({
  brand: [
    { required: true, message: '请填写类型名称名称', trigger: 'blur' },
    { min: 3, max: 10, message: '类型名称为3-10个字符', trigger: 'change' }
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
const confirmDialog = () => {
  isLoading.value = true
  validateForm().then(res => {
    return addServerBrand({
      brand: form.brand
    })
  }).then(res => {
    dialogVisible.value = false
  }).finally(() =>  {
    isLoading.value = false
  })
}
defineExpose({
  openDialog
})
</script>

<style scoped lang="scss">
</style>