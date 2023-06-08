import { invoke } from '@tauri-apps/api'

type Args = Record<string, unknown> | undefined

interface ApiParam {
  url: string,
  args?: Args
}

function invokeRust<T>(params: ApiParam) {
  // return invoke<T>(params.url, params.args)
  return new Promise<T>((resolve, reject) => {
    invoke<T>(params.url, params.args).then(res => {
      resolve(res)
    }).catch(err => {
      // @ts-ignore
      ElMessage.error(JSON.stringify(err))
      console.log(JSON.stringify(err))
      reject(err)
    })
  })
}

export type {
  Args
}

export {
  invokeRust
}
