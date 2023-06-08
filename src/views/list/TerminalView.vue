<template>
  <menu-layout>
    <template v-slot:header-left>
      <el-button :icon="Back" text bg size="small" @click="router.back()"/>
      终端
    </template>
    <template v-slot:content>
      <div class="zn-log" id="logs">
        <div v-for="log in logs">
          {{ log }}
        </div>
      </div>
    </template>
  </menu-layout>
</template>

<script setup lang="ts">
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router'
import MenuLayout from '../../layout/MenuLayout.vue'
import { Back } from '@element-plus/icons-vue'
import { Message } from '../../types/server'
import { listen } from '@tauri-apps/api/event'
import { reactive, computed, watch, nextTick } from 'vue'
import { U } from '@tauri-apps/api/event-30ea0228'
import { getServerLog, removeServerLog } from '../../api/server'

const router = useRouter()
const route = useRoute()
const logs = reactive<Array<string>>([])

const instanceId = computed(() => {
  const temp = route.query.instanceId
  if (temp) {
    return Number(temp)
  }

  return 0
})

let unlisten: U
watch(instanceId, async (val) => {
  logs.length = 0
  getServerLog({instanceId: instanceId.value}).then(async res => {
    unlisten = await listen<Message<string>>(`instance-${val}`, event => {
      logs.push(event.payload.content)
      if (event.payload.status === "ServerEula") {
        unlisten()
        // @ts-ignore
        ElMessage.error("请先同意用户协议")
        router.push({
          path: '/list/eula',
          query: {
            instanceId: instanceId.value
          }
        })
      }

      nextTick(() => {
        const logDiv = document.getElementById('logs') as HTMLDivElement
        logDiv.scrollTop = logDiv.scrollHeight
      })
    })
  })
}, {immediate: true})

onBeforeRouteLeave((to, from, next) => {
  removeServerLog({instanceId: instanceId.value})
    .then(res => {
    }).finally(() => {
      unlisten()
      next()
    })
})
</script>

<style scoped lang="scss">
@include b(log) {
  width: 100%;
  height: 100%;
  color: white;
  overflow: auto;
  background-color: #292c33;
}
</style>
