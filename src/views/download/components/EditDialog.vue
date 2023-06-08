<template>
  <el-dialog
    v-model="dialogVisible"
    title="编辑"
    width="360px"
    align-center>
    <el-form
      ref="formRef"
      :model="form" 
      :rules="rules"
      label-width="120px"
      v-loading="isLoading">
      <el-form-item label="服务器类型" prop="brand_id">
        <el-select v-model="form.brand_id" placeholder="请选择服务器类型" clearable>
          <el-option
            v-for="brand in serverBrands"
            :key="brand.id"
            :label="brand.brand"
            :value="brand.id" 
            size="small"/>
        </el-select>
      </el-form-item>
      <el-form-item label="服务器版本" prop="version_id">
        <el-select v-model="form.version_id" placeholder="请选择服务器类型" clearable>
          <el-option
            v-for="version in versions"
            :key="version.id"
            :label="version.version"
            :value="version.id" 
            size="small"/>
        </el-select>
      </el-form-item>
      <el-form-item label="下载地址" prop="download_url">
        <el-input v-model="form.download_url" placeholder="请填写下载地址"/>
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="dialogVisible = false" size="small">取 消</el-button>
        <el-button type="primary" @click="confirmDialog" size="small" :loading="isLoading">确 认</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref,reactive, watch } from 'vue'
import { FormInstance, FormRules } from 'element-plus'

import { ServerBrand, ServerResourceVo, ServerVersion } from '../../../types/server'
import { onMounted } from 'vue';
import { queryBrandAll, queryVersionByBrandId, updateServerItem } from '../../../api/server'

const emit = defineEmits<{
  (e: "on-confirm"): void
}>()

const dialogVisible = ref(false)
const isLoading = ref(false)
let isFirst = true

const form = reactive<ServerResourceVo>({
  id: 0
})

// 打开弹窗
const openDialog = (item: ServerResourceVo) => {
  form.id = item.id
  form.brand_id = item.brand_id
  form.brand = item.brand
  form.version_id = item.version_id
  form.version = item.version
  form.download_url = item.download_url

  dialogVisible.value = true
}

// 确认弹窗
const confirmDialog = () => {
  isLoading.value = true
  validateForm()
  .then(res => {
    return updateServerItem({
      item: {
        id: form.id,
        brand_id: form.brand_id,
        version_id: form.version_id,
        download_url: form.download_url
      }
    })
  }).then(_ => {
    // @ts-ignore
    ElMessage.success("更新成功")
    emit('on-confirm')
    dialogVisible.value = false
  }).finally(() => {
    isLoading.value = false
  })
}

// 表单校验
const formRef = ref<FormInstance>()
const rules = reactive<FormRules>({
  brand_id: [
    { required: true, message: '请选择服务器类型', trigger: 'change' }
  ],
  version_id: [
    { required: true, message: '请选择服务器版本', trigger: 'change' }
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

// 服务器类型
const serverBrands = reactive<Array<ServerBrand>>([])
onMounted(() => {
  // 清空数组
  isLoading.value = true
  serverBrands.length = 0
  queryBrandAll()
  .then(brands => {
    serverBrands.push(...brands)
  }).finally(() => {
    isLoading.value = false
  })
})


// 版本
const versions = reactive<Array<ServerVersion>>([])
watch(
  () => form.brand_id, 
  val => {
    if (isFirst && form.brand_id) {
      isFirst = false
    } else {
      form.version = undefined
      form.version_id = undefined
    }
    if (val) {
      queryVersionByBrandId({brandId: val}).then(res => {
        versions.length = 0
        versions.push(...res)
      })
    } else {
      versions.length = 0
    }
  },
  {
    immediate: true
  }
)

defineExpose({
  openDialog
})
</script>

<style scoped lang="scss">
</style>