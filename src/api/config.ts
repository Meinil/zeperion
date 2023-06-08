import { invokeRust,Args } from "./index"
import { ConfigVo } from "../types/config"

// 查询所有服务器类型
export const queryGlobalConfig = () => invokeRust<ConfigVo>({
  url: 'query_global_config'
})

// 更新全局配置文件
export const updateGlobalConfig = (args: Args) => invokeRust<ConfigVo>({
  url: 'update_global_config',
  args
})
