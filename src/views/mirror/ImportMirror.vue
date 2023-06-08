<template>
  <menu-layout>
    <template v-slot:header-left>
      <el-button :icon="Back" text bg size="small" @click="router.back()"/>
      导入镜像
    </template>
    <template v-slot:header-right>
      <el-button type="primary" size="small" @click="saveMirrorEvent" :loading="isSaveLoading">
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
        <el-form-item label="镜像位置" prop="path">
          <el-input placeholder="请选择镜像位置" v-model="form.path" readonly @click="openDialog"/>
        </el-form-item>
      </el-form>
    </template>
  </menu-layout>
  <add-brand-dialog ref="addBrandDialog"/>
  <add-version-dialog ref="addVersionDialog"/>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { reactive, watch, ref, onMounted } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { Back } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/api/dialog'

import MenuLayout from '../../layout/MenuLayout.vue'
import { ServerBrand, ServerVersion, ServerImportMirrorVo } from '../../types/server'
import { queryBrandAll, queryVersionByBrandId, importMirror } from '../../api/server'
import AddBrandDialog from '../../components/AddBrandDialog.vue'
import AddVersionDialog from '../../components/AddVersionDialog.vue'

const router = useRouter()
const serverBrands = reactive<Array<ServerBrand>>([])
const form = reactive<ServerImportMirrorVo>({})

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
  path: [
    { required: true, message: '请选择镜像位置', trigger: 'blur' }
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
const saveMirrorEvent = () => {
  isSaveLoading.value = true
  validateForm()
    .then(res => {
      return importMirror({mirrorVo: {...form}})
    })
    .then(res => {
      // @ts-ignore
      ElMessage.success("保存成功")
      router.push("/mirror")
    }).finally(() => {
      isSaveLoading.value = false
    })
}

const openDialog = async () => {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{
      name: 'Jar',
      extensions: ['jar']
    }]
  })
  if (selected) {
    // 点击了确认
    form.path = selected as string
  }
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