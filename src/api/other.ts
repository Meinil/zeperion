import { invokeRust, Args} from "./index"

// 关闭加载动画
export const closeSplashscreen = () => invokeRust({
  url: 'close_splashscreen'
})

// 打开文件管理器
export const openFileManager = (args?: Args) => invokeRust({
  url: 'open_file_manager',
  args
})