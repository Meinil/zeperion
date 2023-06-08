<template>
  <el-dialog
    v-model="dialogVisible"
    title="添加服务器版本"
    width="360px"
    align-center>
    <el-form
      ref="formRef"
      :inline="true" 
      :model="form"
      :rules="rules">
      <el-form-item label="服务器类型" prop="brandId">
        <el-select class="filter-item" v-model="form.brandId" placeholder="请选择服务器类型" clearable>
          <el-option
            v-for="brand in serverBrands"
            :key="brand.id"
            :label="brand.brand"
            :value="brand.id" 
            size="small"/>
        </el-select>
      </el-form-item>
      <el-form-item label="服务器版本" prop="version">
        <el-input v-model="form.version" placeholder="请填写服务器版本" />
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
import { ElMessage, FormInstance, FormRules } from 'element-plus'

import { ServerBrand } from '../types/server'
import { addServerVersion, queryBrandAll } from '../api/server'

const form = reactive<{
  brandId?: number,
  version?: string
}>({})

const serverBrands = reactive<Array<ServerBrand>>([])

// 打开弹窗
const dialogVisible = ref(false)
const openDialog = () => {
  form.version = undefined
  form.brandId = undefined
  dialogVisible.value = true

  // 清空数组
  serverBrands.length = 0
  queryBrandAll()
  .then(brands => {
    serverBrands.push(...brands)
  })
}

// 表单校验
const formRef = ref<FormInstance>()
const rules = reactive<FormRules>({
  brandId: [
    { required: true, message: '请选择服务器类型', trigger: 'blur' },
  ],
  version: [
    { required: true, message: '请填写服务器版本', trigger: 'blur' },
    { min: 3, max: 10, message: '版本名称为3-10个字符', trigger: 'change' }
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
    return addServerVersion({
      ...form
    })
  }).then(res => {
    // @ts-ignore
    ElMessage.success("添加成功")
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