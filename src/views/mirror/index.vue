<template>
  <menu-layout :contentStyle="MenuStyleType.Menu">
    <template v-slot:header-left>
      镜像
    </template>
    <template v-slot:header-right>
      <el-button type="primary" @click="router.push('/mirror/ImportMirror')" size="small">导入镜像</el-button>
    </template>
    <template v-slot:content>
      <list-layout v-for="item in serverResources" :key="item.id">
        <template v-slot:list-left>
          <span>类型: {{ item.brand }}</span>&nbsp;&nbsp;
          <span>版本: {{ item.version }}</span>&nbsp;&nbsp;
        </template>
        <template v-slot:list-right>
          <el-button type="primary" :icon="VideoPlay" circle @click="openServerCreateDialog(item)"/>
          <el-button type="danger" :icon="Delete" circle @click="removeMirrorEvent(item)"/>
        </template>
      </list-layout>
    </template>
  </menu-layout>
  <server-create-dialog ref="serverCreateDialog"/>
</template>

<script setup lang="ts">
import { Delete, VideoPlay } from '@element-plus/icons-vue'
import { ref, reactive, onMounted } from 'vue'
import { ServerResourceVo, ServerQueryVo } from '../../types/server'
import MenuLayout from '../../layout/MenuLayout.vue'
import { MenuStyleType } from '../../types/menu'
import ListLayout from '../../layout/ListLayout.vue'
import { queryServer, removeMirror } from '../../api/server'
import ServerCreateDialog from './components/ServerCreateDialog.vue'
import { useRouter } from 'vue-router'
const router = useRouter()

const form = reactive<ServerQueryVo>({
  is_download: '1'
})

const serverResources = reactive<Array<ServerResourceVo>>([])
onMounted(() => {
  query()
})

// 查询所有镜像
const query = () => {
  queryServer({queryVo: form})
  .then(servers => {
    serverResources.length = 0
    for (const server of servers) {
      server.isDeleteLoaind = false
      serverResources.push(server)
    }
  }).catch(err => {
    console.log(err)
  })
}

// 创建服务器实例
const serverCreateDialog = ref<typeof ServerCreateDialog>()
const openServerCreateDialog = (item: ServerResourceVo) => {
  serverCreateDialog.value?.openDialog(item)
}

// 删除镜像
const removeMirrorEvent = (item: ServerResourceVo) => {
  // @ts-ignore
  ElMessageBox.confirm(
    `您确定要删除${item.brand}-${item.version}`,
    '删除',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  )
  .then(() => {
    return removeMirror({itemId: item.id})
  }).then(() => {
    query()
    // @ts-ignore
    ElMessage.success("删除成功")
  })
}
</script>