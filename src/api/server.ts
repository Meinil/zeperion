import { invokeRust, Args} from "./index"
import { ServerResourceVo, ServerBrand, ServerVersion, ServerInstanceVo, ServerInstance, ServerImportResourceVo, ServerImportMirrorVo, ServeritemVo } from "../types/server"

// 查询所有服务器类型
export const queryBrandAll = () => invokeRust<Array<ServerBrand>>({
  url: 'query_brand_all'
})

// 根据服务器类型id查询对应服务器版本列表
export const queryVersionByBrandId = (args?: Args) => invokeRust<Array<ServerVersion>>({
  url: 'query_version_by_brand_id',
  args
})

// 查询服务器列表
export const queryServer = (args?: Args) => invokeRust<Array<ServerResourceVo>>({
  url: 'query_server',
  args
})

// 查询服务器列表
export const updateServerItem = (args?: {item: ServeritemVo}) => invokeRust<Array<ServerResourceVo>>({
  url: 'update_server_item',
  args
})

// 下载服务器
export const download = (args?: Args) => invokeRust({
  url: 'download',
  args
})

// 创建一个服务器
export const createServer = (args?: Args) => invokeRust({
  url: 'create_server',
  args
})

// 查询所有服务器实例
export const queryInstanceAll = (args?: Args) => invokeRust<Array<ServerInstance>>({
  url: 'query_instance_all',
  args
})

// 查询所有服务器实例
export const queryInstanceById = (args?: { instanceId: number }) => invokeRust<ServerInstanceVo>({
  url: 'query_instance_by_id',
  args
})

// 更新服务器实例
export const updateServerInstanceById = (args?: {instance: ServerInstanceVo}) => invokeRust({
  url: 'update_server_instance_by_id',
  args
})

// 运行一个服务器
export const runServer = (args?: Args) => invokeRust<number>({
  url: 'run_server',
  args
})

// 运行一个服务器
export const stopServer = (args: {pId: number | undefined, instanceId: number}) => invokeRust<boolean>({
  url: 'stop_server',
  args
})

// 设置开服协议
export const setEula = (args: {isAgree: boolean, instanceId: number}) => invokeRust({
  url: 'set_eula',
  args
})

// 保存服务器
export const importResource = (args: {importResourceVo: ServerImportResourceVo}) => invokeRust({
  url: 'import_resource',
  args
})

// 导入镜像
export const importMirror = (args: {mirrorVo: ServerImportMirrorVo}) => invokeRust({
  url: 'import_mirror',
  args
})

// 删除镜像
export const removeResource = (args: {itemId: number}) => invokeRust({
  url: 'remove_resource',
  args
})

// 删除镜像
export const removeMirror = (args: {itemId: number}) => invokeRust({
  url: 'remove_mirror',
  args
})

// 删除服务器实例
export const removeInstance = (args: {instanceId: number}) => invokeRust({
  url: 'remove_instance',
  args
})

// 添加服务器类型
export const addServerBrand = (args: {brand: string}) => invokeRust({
  url: 'add_server_brand',
  args
})

// 添加服务器版本
export const addServerVersion = (args: {brandId?: number, version?: string}) => invokeRust({
  url: 'add_server_version',
  args
})

// 触发推送日志
export const getServerLog = (args: {instanceId: number}) => invokeRust({
  url: 'get_server_log',
  args
})

// 结束推送日志
export const removeServerLog = (args: {instanceId: number}) => invokeRust({
  url: 'remove_server_log',
  args
})
