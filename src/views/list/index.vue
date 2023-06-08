<template>
  <menu-layout>
    <template v-slot:header-left>
      列表
    </template>
    <template v-slot:content>
      <list-layout v-for="item in serverInstances" :key="item.id">
        <template v-slot:list-left>
          <span>名称: {{ item.name }}</span>
        </template>
        <template v-slot:list-right>
          <template v-if="item.p_id">
            <el-button type="primary" circle v-if="item.isStartLoading" :loading="item.isStartLoading"/>
            <el-button type="info" circle @click="openTerminalEvent(item)" v-if="!item.isStartLoading">
              <template v-slot:icon>
                <svg t="1685625544690" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="4628" data-spm-anchor-id="a313x.7781069.0.i1" width="32" height="32" xmlns:xlink="http://www.w3.org/1999/xlink"><path d="M864 960H160c-70.6 0-128-57.4-128-128V192c0-70.6 57.4-128 128-128h704c70.6 0 128 57.4 128 128v640c0 70.6-57.4 128-128 128zM160 128c-35.2 0-64 28.8-64 64v640c0 35.2 28.8 64 64 64h704c35.2 0 64-28.8 64-64V192c0-35.2-28.8-64-64-64H160z" p-id="4629" fill="#FFFFFF" data-spm-anchor-id="a313x.7781069.0.i0" class="selected"></path><path d="M192 512c-9.4 0-18.6-4.2-25-12-11-13.8-8.8-34 5-45l128.8-103L172 249c-13.8-11-16-31.2-5-45s31.2-16 45-5l160 128c7.6 6 12 15.2 12 25s-4.4 19-12 25l-160 128c-6 4.8-13 7-20 7z m320 0h-128c-17.6 0-32-14.4-32-32s14.4-32 32-32h128c17.6 0 32 14.4 32 32s-14.4 32-32 32z" p-id="4630" fill="#FFFFFF"></path></svg>
              </template>
            </el-button>
            <!-- 在服务器启动之前,不能停止服务器 -->
            <el-button type="danger" :icon="VideoPause" circle :disabled="item.isStartLoading" :loading="item.isStopLoading" @click="stopServerEvent(item)"/>
          </template>
          <template v-else>
            <el-button type="primary" :icon="VideoPlay" circle @click="startServerEvent(item)" :loading="item.isStartLoading"/>
            <el-button type="danger" :icon="Delete" circle v-if="!item.isStartLoading" @click="removeInstanceEvent(item)"/>
          </template>
          <el-popover placement="bottom" trigger="hover">
            <template #reference>
              <el-button text size="small" type='primary'>更多</el-button>
            </template>
            <el-button :icon="Edit" circle type="success" @click="goEditEvent(item)"/>
            <el-button type="warning" circle @click="openFileManagerEvent(item)" :loading="item.isOpenLoading">
              <template v-slot:icon>
                <svg t="1685913787553" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="3218" width="16" height="16"><path d="M960.975595 272.065396l0-61.152778c0-33.789571-27.496238-61.275575-61.285808-61.275575l-529.86795 0c-0.818645-0.429789-3.499707-2.148943-8.258081-8.135284-4.91187-6.170536-9.915837-14.152325-15.20633-22.604834-15.687284-25.009603-31.906687-50.878784-58.809407-50.878784L63.085804 68.01814c-33.779338 0-61.275575 27.486004-61.275575 61.275575l0 693.801603c0 0.337691 0.010233 0.685615 0.030699 1.023306 0.010233 0.245593 0.030699 0.50142 0.051165 0.76748 0.020466 0.225127 0.040932 0.450255 0.071631 0.675382 0.010233 0.112564 0.030699 0.225127 0.051165 0.347924 0.010233 0.173962 0.040932 0.337691 0.071631 0.511653 0.051165 0.276293 0.102331 0.552585 0.153496 0.818645 0.040932 0.225127 0.092098 0.460488 0.153496 0.685615 0.173962 0.685615 0.36839 1.350764 0.603751 2.015913 0.102331 0.26606 0.204661 0.532119 0.306992 0.798179 0.081864 0.204661 0.173962 0.409322 0.255827 0.613984 0.051165 0.092098 0.092098 0.194428 0.13303 0.296759 0.102331 0.214894 0.214894 0.440022 0.327458 0.654916 0.173962 0.347924 0.36839 0.695848 0.573051 1.043772 0.081864 0.153496 0.173962 0.306992 0.26606 0.450255 0.849344 1.381463 1.862417 2.660596 3.018753 3.806699 0.163729 0.163729 0.327458 0.327458 0.50142 0.480954 0.286526 0.26606 0.583285 0.521886 0.880043 0.76748 0.255827 0.204661 0.511653 0.409322 0.777713 0.613984 0.26606 0.194428 0.532119 0.388856 0.798179 0.562818 0.26606 0.184195 0.532119 0.358157 0.808412 0.521886 0.317225 0.194428 0.644683 0.378623 0.982374 0.552585 0.184195 0.112564 0.378623 0.204661 0.573051 0.306992 0.204661 0.092098 0.409322 0.194428 0.603751 0.286526 0.010233 0 0.020466 0 0.020466 0 0.26606 0.122797 0.542352 0.245593 0.818645 0.347924 0.552585 0.225127 1.115404 0.429789 1.698688 0.603751 0.573051 0.173962 1.166569 0.327458 1.77032 0.450255 0.757247 0.153496 1.50426 0.26606 2.251274 0.337691 0.624217 0.061398 1.248434 0.092098 1.87265 0.092098l0.061398 0c0.347924 0 0.685615-0.010233 1.033539-0.030699 0.245593 0 0.491187-0.020466 0.73678-0.051165 0.245593-0.020466 0.50142-0.040932 0.747014-0.081864 0.081864 0 0.163729-0.010233 0.23536-0.030699 0.214894-0.020466 0.429789-0.051165 0.644683-0.092098 0.26606-0.040932 0.521886-0.092098 0.777713-0.143263 0.194428-0.040932 0.388856-0.081864 0.583285-0.143263 0.716314-0.163729 1.412163-0.36839 2.097778-0.613984 0.26606-0.102331 0.532119-0.204661 0.798179-0.306992 0.26606-0.112564 0.521886-0.214894 0.777713-0.337691 0.010233 0 0.020466 0 0.020466 0 0.26606-0.122797 0.511653-0.245593 0.76748-0.378623 0.358157-0.173962 0.706081-0.358157 1.043772-0.573051 0.153496-0.081864 0.306992-0.173962 0.450255-0.26606 0.798179-0.480954 1.555425-1.023306 2.281973-1.627057 0.255827-0.204661 0.50142-0.419556 0.747014-0.644683 0.23536-0.214894 0.470721-0.440022 0.706081-0.675382 0.23536-0.23536 0.460488-0.470721 0.675382-0.706081 0.225127-0.245593 0.440022-0.491187 0.644683-0.747014 0.204661-0.255827 0.409322-0.511653 0.613984-0.777713 0.010233-0.010233 0.010233-0.010233 0.010233-0.010233 0.378623-0.521886 0.747014-1.064238 1.084705-1.616824 0.173962-0.286526 0.337691-0.573051 0.491187-0.859577 0.13303-0.245593 0.26606-0.491187 0.388856-0.73678 0.040932-0.092098 0.092098-0.184195 0.13303-0.276293 0.092098-0.204661 0.184195-0.409322 0.276293-0.624217 0.102331-0.214894 0.194428-0.450255 0.276293-0.675382 0.081864-0.194428 0.153496-0.388856 0.225127-0.583285 0.020466-0.071631 0.051165-0.143263 0.071631-0.225127 0.081864-0.225127 0.153496-0.460488 0.225127-0.695848 0.173962-0.573051 0.327458-1.166569 0.450255-1.77032l0.020466-0.102331 102.443184-491.678162 0-2.108011c0-11.215436 9.117658-20.343327 20.343327-20.343327l795.794531 0c10.683317 0 19.48375 8.28878 20.281929 18.787902l-101.542674 548.31816-0.081864 0.450255c-0.061398 0.317225-0.112564 0.63445-0.153496 0.941442-0.040932 0.296759-0.071631 0.583285-0.102331 0.86981-0.020466 0.286526-0.051165 0.573051-0.061398 0.859577-0.020466 0.327458-0.030699 0.665149-0.030699 0.992607l0 0.081864c-0.010233 11.205203-9.127891 20.322861-20.343327 20.322861l-596.034928 0c-11.307533 0-20.466124 9.15859-20.466124 20.466124 0 11.2973 9.15859 20.466124 20.466124 20.466124l596.034928 0c33.073256 0 60.109006-26.339902 61.234643-59.136865l0-0.010233 101.726869-549.341466 0.347924-1.841951 0-1.882883C1022.189771 299.571866 994.724233 272.096095 960.975595 272.065396zM920.043347 272.065396l-754.923682 0c-33.011858 0-60.016909 26.237571-61.234643 58.962903l-61.142545 293.494451 0-495.229035c0-11.215436 9.127891-20.343327 20.343327-20.343327l224.042659 0c0.706081 0.306992 3.50994 1.821485 8.677637 8.217149 5.157463 6.385431 10.601452 15.0733 15.871479 23.474644 15.380292 24.549116 31.292704 49.927109 57.489342 49.927109l530.522865 0c11.225669 0 20.35356 9.127891 20.35356 20.343327L920.043347 272.065396 920.043347 272.065396z" p-id="3219"></path><path d="M16.341177 842.650699c-0.573051-0.173962-1.146103-0.36839-1.698688-0.603751C15.195074 842.272076 15.757892 842.476737 16.341177 842.650699z" p-id="3220"></path></svg>
              </template>
            </el-button>
          </el-popover>
        </template>
      </list-layout>
    </template>
  </menu-layout>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue'
import { Delete, VideoPlay, VideoPause, Edit } from '@element-plus/icons-vue'
import MenuLayout from '../../layout/MenuLayout.vue'
import { Message, ServerInstance } from '../../types/server'
import { queryInstanceAll, runServer, stopServer, removeInstance } from '../../api/server'
import { openFileManager } from '../../api/other'
import ListLayout from '../../layout/ListLayout.vue'
import { listen } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'

const router = useRouter()

const serverInstances = reactive<Array<ServerInstance>>([])
onMounted(() => {
  query()
})

const startServerEvent = async (item: ServerInstance) => {

  // 启动中
  item.isStartLoading = true

  const unlisten = await listen<Message<string>>(`instance-${item.id}`, event => {
    const { status, content } = event.payload
    console.log(status)
    if (status === "ServerEula") {
      unlisten()
      // @ts-ignore
      ElMessage.error("请先同意用户协议")
      item.isStartLoading = false
      item.p_id = undefined
      router.push({
        path: '/list/eula',
        query: {
          instanceId: item.id
        }
      })
    } else if (status === "ServerFail") {
      unlisten()
      // @ts-ignore
      ElMessage.error(`启动失败: ${content}`)
      item.isStartLoading = false
      item.p_id = undefined
    } else if (status === "ServerSuccess") {
      item.isStartLoading = false
      unlisten()
      // @ts-ignore
      ElMessage.success("启动成功")
      // 启动成功重新查询服务实例状态
      query()
    }
  })

  // 启动服务器
  runServer({instanceId: item.id})
  .then(res => {
    item.p_id = res
  }).catch(err => {
    // 启动失败
    item.isStartLoading = false
    unlisten()
  })
}

// 查询所有服务器实例
const query = () => {
  serverInstances.length = 0
  queryInstanceAll()
  .then(res => {
    serverInstances.push(...res)
    for(const instance of serverInstances) {
      instance.isStartLoading = false
      instance.isStopLoading = false
      instance.isOpenLoading = false
    }
  })
}

// 停止服务器
const stopServerEvent = (item: ServerInstance) => {
  item.isStartLoading = true
  item.isStopLoading = true
  stopServer({pId: item.p_id, instanceId: item.id})
  .then(res => {
    setTimeout(() => {
      // 停止成功重新查询服务实例状态
      // @ts-ignore
      ElMessage.success("停止成功")
      query()
    }, 1000)
  }).catch(err => {
    console.log(err)
  }).finally(() => {
    item.isStartLoading = false
    item.isStopLoading = false
  })
}

// 打开文件管理器
const openFileManagerEvent = (item: ServerInstance) => {
  item.isOpenLoading = true
  openFileManager({instanceId: item.id})
  .then(res => {

  }).finally(() => {
    item.isOpenLoading = false
  })
}

// 打开terminal日志
const openTerminalEvent = (item: ServerInstance) => {
  router.push({
    path: "/list/terminal",
    query: {
      instanceId: item.id
    }
  })
}

// 删除该服务器示例
const removeInstanceEvent = (item: ServerInstance) => {
  // @ts-ignore
  ElMessageBox.confirm(
    `您确定要删除【${item.name}】`,
    '删除',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }
  )
  .then(() => {
    return removeInstance({instanceId: item.id})
  })
  .then(() => {
    query()
    // @ts-ignore
    ElMessage.success("删除成功")
  })
}

const goEditEvent = (item: ServerInstance) => {
  router.push({
    path: '/list/edit',
    query: {
      instanceId: item.id
    }
  })
}
</script>

<style scoped lang="scss">
</style>