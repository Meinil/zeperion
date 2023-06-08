<template>
  <menu-layout :contentStyle="MenuStyleType.Menu">
    <template v-slot:header-left>
      下载
    </template>
    <template v-slot:header-right>
      <el-button type="primary" @click="search" size="small">搜索</el-button>
      <el-button type="primary" @click="importEvent" size="small">新增资源</el-button>
    </template>
    <template v-slot:content>
      <list-layout v-for="item in serverResources" :key="item.id">
        <template v-slot:list-left>
          <span>类型: {{ item.brand }}</span>&nbsp;&nbsp;
          <span>版本: {{ item.version }}</span>&nbsp;&nbsp;
          <el-progress v-if="item.progress_percent" class="progress" :percentage="item.progress_percent" />
        </template>
        <template v-slot:list-right>
          <el-button :icon="Edit" circle type="success" @click="openEditDialog(item)"/>
          <el-button type="primary" :icon="Download" circle  @click="downloadServer(item)" :loading="item.isDownloading" :disabled="item.download_url === null || item.download_url === undefined || item.download_url === ''"/>
          <el-button type="danger" :icon="Delete" circle @click="removeResourceEvent(item)" :disabled="item.isDownloading"/>
        </template>
      </list-layout>
    </template>
  </menu-layout>
  <search-dialog ref="searchDialog" @on-confirm="onConfirm" />
  <edit-dialog ref="editDialog" @on-confirm="onConfirm" />
</template>

<script setup lang="ts">
import { Delete, Download, Edit } from '@element-plus/icons-vue'
import { reactive, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'

import { queryServer, download, removeResource } from '../../api/server'

import { ServerResourceVo, DownloadVo, ServerQueryVo, SearchDialogForm } from '../../types/server'
import MenuLayout from '../../layout/MenuLayout.vue'
import SearchDialog from './components/SearchDialog.vue'
import EditDialog from './components/EditDialog.vue'
import { MenuStyleType } from '../../types/menu'
import ListLayout from '../../layout/ListLayout.vue'
const router = useRouter()

const form = reactive<ServerQueryVo>({
  is_download: '0'
})

// 查询服务器列表
const serverResources = reactive<Array<ServerResourceVo>>([])
const queryServerList = () => {
  queryServer({queryVo: form})
    .then(servers => {
      serverResources.length = 0
      for(const server of servers) {
        server.isDownloading = false
        serverResources.push(server)
      }

      console.log(serverResources)
    })
}
onMounted(() => {
  queryServerList()
})

// 删除资源
const removeResourceEvent = (item: ServerResourceVo) => {
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
    return removeResource({itemId: item.id})
  })
  .then(() => {
    queryServerList()
    // @ts-ignore
    ElMessage.success("删除成功")
  })
}

// 下载服务器
const downloadServer = async (item: ServerResourceVo) => {
  item.isDownloading = true
  let isDwonload = true
  // 监听下载事件
  const unlisten = await listen<DownloadVo>(`download-${item.id}`, event => {
    item.content_length = event.payload.content_length
    item.progress_percent = Number(event.payload.progress_percent)
    if (item.progress_percent >= 100 && unlisten && isDwonload) {
      isDwonload = false
      unlisten()
      setTimeout(() => {
        queryServerList()
        // @ts-ignore
        ElMessage.success("下载成功")
        item.isDownloading = false
      }, 1000)
    }
  }).catch(err => {
    if (unlisten) {
      item.isDownloading = false
      unlisten()
    }
  })

  download({item})
  .then(res => {
    console.log("res", res)
  }).finally(() => {
    item.isDownloading = false
  })
}

// 导入服务器资源
const importEvent = () => {
  router.push({
    path: '/download/import'
  })
}

// 弹窗确认
const onConfirm = (params: SearchDialogForm | undefined) => {
  form.brand_id = params?.brand
  form.version_id = params?.version
  queryServerList()
}

// 搜索弹框
const searchDialog = ref<typeof SearchDialog>()
const search = () => {
  searchDialog.value?.openDialog()
}

// 打开编辑资源弹窗
const editDialog = ref<typeof EditDialog>()
const openEditDialog = (item: ServerResourceVo) => {
  editDialog.value?.openDialog(item)
}
</script>
<style scoped lang="scss">
.progress {
  width: 150px;
}
</style>