<template>
  <el-dialog
    v-model="dialogVisible"
    title="创建服务器">
    <el-form
      ref="formRef"
      :model="createVo"
      :rules="rules"
      label-width="70px"
      status-icon>
      <el-form-item label="名称" prop="name">
        <el-input v-model="createVo.name" placeholder="请输入服务器名称" clearable/>
      </el-form-item>
      <el-form-item label="唯一id" prop="path">
        <el-tooltip
          effect="dark"
          content="该值为当前创建服务器的唯一值,自动生成无需填写"
          placement="bottom">
        {{ createVo.path }}
        </el-tooltip>
      </el-form-item>
      <el-form-item label="备注" prop="remark">
        <el-input type="textarea" v-model="createVo.remark" :rows="2" placeholder="请输入备注"/>
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="dialogVisible = false" size="small">取 消</el-button>
        <el-button type="primary" @click="cofirm" size="small" :loading="isLoading">确 认</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { ServerResourceVo } from '../../../types/server'
import { FormInstance, FormRules } from 'element-plus'
import { createServer } from '../../../api/server'
const dialogVisible = ref(false)
const isLoading = ref(false)

// 生成唯一id
const uuid = () => {
  const tempUrl = URL.createObjectURL(new Blob())
  const uuid = tempUrl.toString()
  URL.revokeObjectURL(tempUrl)
  return uuid.substr(uuid.lastIndexOf("/") + 1).replaceAll('-', '')
}

const rules = reactive<FormRules>({
  name: [
    { required: true, message: '请输入名称', trigger: 'blur' },
    { min: 3, max: 20, message: '服务器名称为3-20个字符', trigger: 'change' }
  ],
  path: [
    { required: true, message: '请输入唯一id', trigger: 'blur' }
  ]
})
const createVo = reactive<{
  itemId: number | undefined,
  name: string,
  path: string,
  remark: string
}>({
  itemId: undefined,
  name: '',
  path: '',
  remark: ''
})

// 重置数据
const resetData = () => {
  createVo.itemId = undefined
  createVo.name = ''
  createVo.path = uuid()
  createVo.remark = ''

  isLoading.value = false
}
const openDialog = (item: ServerResourceVo) => {
  dialogVisible.value = true
  resetData()
  createVo.itemId = item.id
}

// 校验表单
const formRef = ref<FormInstance>()
const validateForm = () => {
  return new Promise((resolve, reject) => {
    formRef.value?.validate(valid => {
      if (valid) {
        resolve("验证成功")
      } else {
        reject("验证失败")
      }
    })
  })
}

// 创建服务器
const cofirm = () => {
  isLoading.value = true
  validateForm().then(res => {
    return createServer({createVo: createVo})
  }).then(res => {
    // @ts-ignore
    ElMessage.success("创建成功,请前往列表启动")
    dialogVisible.value = false
  }).catch(err => {
    // @ts-ignore
    ElMessage.success(err)
  }).finally(() => {
    isLoading.value = false
  })
}
defineExpose({
  openDialog
})
</script>

<style scoped lang="scss">
</style>