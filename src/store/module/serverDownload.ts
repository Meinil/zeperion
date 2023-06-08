import { defineStore } from 'pinia'
import { Modules } from '../module'

interface ServerVersion {
  id: string,
  name: string
}
interface ServerType {
  id: string,
  name: string
}
interface ServerItem {
  type: ServerType,       // 服务器类型
  version: string,        // 服务器版本
  download: string,       // 下载链接
}
export default defineStore(Modules.SERVER_DOWNLOAD, {
  state: () => {
    return {
      types: new Array<ServerType>(),
      versions: [{
        id: '1',
        name: '1.19'
      }],
      servers: [{
        id: '1',
        name: 'spigot',
        version: '1.1911',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      },{
        id: '1',
        name: 'spigot',
        version: '1.19',
        download: 'https://www.baidu.com'
      }]
    }
  }
})