<template>
  <el-dialog
    v-model="dialogVisible"
    title="搜索"
    width="360px"
    align-center
    v-loading="isLoading">
    <el-form :inline="true" :model="form">
      <el-form-item label="服务器类型">
        <el-select class="filter-item" v-model="form.brand" placeholder="请选择服务器类型" clearable>
          <el-option
            v-for="brand in serverBrands"
            :key="brand.id"
            :label="brand.brand"
            :value="brand.id" 
            size="small"/>
        </el-select>
      </el-form-item>
      <el-form-item label="服务器版本">
        <el-select class="filter-item" v-model="form.version" placeholder="请选择服务器版本" clearable>
          <el-option
            v-for="item in versions"
            :key="item.id"
            :label="item.version"
            :value="item.id" 
            size="small"/>
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="dialogVisible = false" size="small">取 消</el-button>
        <el-button type="primary" @click="confirmDialog" size="small">确 认</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, ref, watch, onMounted } from 'vue'
import { ServerBrand, ServerVersion, SearchDialogForm } from '../../../types/server'
import { queryBrandAll, queryVersionByBrandId } from '../../../api/server'

const form = reactive<SearchDialogForm>({})

const emit = defineEmits<{
  (e: "on-confirm", form: SearchDialogForm): void
}>()

const isLoading = ref(false)

onMounted(() => {
  init()
})

const serverBrands = reactive<Array<ServerBrand>>([])

// 初始化
const init = () => {
  isLoading.value = true
  // 清空数组
  serverBrands.length = 0
  queryBrandAll()
  .then(brands => {
    serverBrands.push(...brands)
  }).finally(() => {
    isLoading.value = false
  })
}

// 打开弹窗
const dialogVisible = ref(false)
const openDialog = () => {
  dialogVisible.value = true
}

// 版本
const versions = reactive<Array<ServerVersion>>([])
watch(
  () => form.brand, 
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

// 确认弹框
const confirmDialog = () => {
  emit('on-confirm', form)
  dialogVisible.value = false
}
defineExpose({
  openDialog
})
</script>

<style scoped lang="scss">
</style>