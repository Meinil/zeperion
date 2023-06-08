<template>
  <menu-layout>
    <template v-slot:header-left>
      <el-button :icon="Back" text bg size="small" @click="router.back()"/>
      新增资源
    </template>
    <template v-slot:header-right>
      <el-button type="primary" size="small" @click="saveResourceEvent" :loading="isSaveLoading">
        保存
      </el-button>
    </template>
    <template v-slot:content>
      <el-form 
        :model="form"
        ref="formRef"
        label-width="100px" 
        class="zn-import"
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
          <el-button type="primary" text size="small" @click="addBrandEvent">新增类型</el-button>
        </el-form-item>
        <el-form-item label="服务器版本" prop="versionId">
          <el-select class="filter-item" v-model="form.versionId" placeholder="请选择服务器版本" clearable>
            <el-option
              v-for="item in versions"
              :key="item.id"
              :label="item.version"
              :value="item.id" 
              size="small"/>
          </el-select>
          <el-button type="primary" text size="small" @click="addVersionEvent">新增版本</el-button>
        </el-form-item>
        <el-form-item label="下载链接" prop="downloadUrl">
          <el-input placeholder="请输入下载链接" v-model="form.downloadUrl"/>
        </el-form-item>
      </el-form>
    </template>
  </menu-layout>
  <add-brand-dialog ref="addBrandDialog"/>
  <add-version-dialog ref="addVersionDialog"/>
</template>

<script setup lang="ts">
import { reactive, watch, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { Back } from '@element-plus/icons-vue'

import { ServerBrand, ServerVersion, ServerImportResourceVo } from '../../types/server'
import { queryBrandAll, queryVersionByBrandId, importResource } from '../../api/server'

import MenuLayout from '../../layout/MenuLayout.vue'
import AddBrandDialog from '../../components/AddBrandDialog.vue'
import AddVersionDialog from '../../components/AddVersionDialog.vue'

const router = useRouter()
const serverBrands = reactive<Array<ServerBrand>>([])
const form = reactive<ServerImportResourceVo>({})

// 初始化
const init = () => {
  // 清空数组
  serverBrands.length = 0
  queryBrandAll()
  .then(brands => {
    serverBrands.push(...brands)
  })
}

// 版本
const versions = reactive<Array<ServerVersion>>([])
watch(
  () => form.brandId, 
  val => {
    console.log("val", val)
    if (val) {
      queryVersionByBrandId({brandId: val}).then(res => {
        versions.length = 0
        versions.push(...res)
      })
    } else {
      versions.length = 0
    }
  }
)

onMounted(() => {
  init()
})

const rules = reactive<FormRules>({
  brandId: [
    { required: true, message: '请选择服务器类型', trigger: 'blur' }
  ],
  versionId: [
    { required: true, message: '请选择服务器版本', trigger: 'blur' }
  ],
  downloadUrl: [
    { required: true, message: '请输入下载地址', trigger: 'blur' }
  ]
})

// 表单校验
const formRef = ref<FormInstance>()
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

// 保存
const isSaveLoading = ref(false)
const saveResourceEvent = () => {
  isSaveLoading.value = true
  validateForm()
    .then(res => {
      return importResource({
        importResourceVo: {...form}
      })
    })
    .then(res => {
      // @ts-ignore
      ElMessage.success("保存成功")
      router.push("/download")
    }).finally(() => {
      isSaveLoading.value = false
    })
}

// 新增类型
const addBrandDialog = ref<typeof AddBrandDialog>()
const addBrandEvent = () => {
  addBrandDialog.value?.openDialog()
}

// 新增版本
const addVersionDialog = ref<typeof AddVersionDialog>()
const addVersionEvent = () => {
  addVersionDialog.value?.openDialog()
}

</script>

<style scoped lang="scss">
@include b(import) {
  margin: 0 auto;
  width: 400px;
}
</style>