interface ServerBrand {
  id: number,
  brand: string
}

interface ServerVersion {
  id: number,
  brand_id: number,
  version?: string
}

interface ServerResourceVo {
	id: number,
	brand_id?: number,
	brand?: string,
	version_id?: number,
	version?: string,
	download_url?: string,
  progress_percent?: number,
  content_length?: string,
  isDownloading?: boolean,
  isDeleteLoaind?: boolean
}

interface DownloadVo {
  id: number,
  progress_percent: number,
  content_length: string,
}

// 下载资源查询vo
interface ServerQueryVo {
  is_download: string,
  brand_id?: number,
  version_id?: number
}
interface ServerImportResourceVo {
  brandId?: number,
  versionId?: number,
  downloadUrl?: string
}

interface ServerImportMirrorVo {
  brandId?: number,
  versionId?: number,
  path?: string
}

interface SearchDialogForm {
  brand?: number,
  version?: number,
}

interface ServerRunVo {
  instanceId: number,
  serverName: string,
  itemId: number
}

// 服务器实例
interface ServerInstance {
  id: number,
  item_id: number,
  name: string,
  path: string,
  p_id?: number,
  remark: string,
  isStartLoading?: boolean,
  isStopLoading?: boolean,
  isOpenLoading?: boolean,
}

interface ServerInstanceVo {
  id: number,
  name: string,
  path: string,
  t_id?: number,
  p_id?: number,
  item_id: number,
  brand: string,
  version: string,
  remark: string,
  vm_options?: string,
  properties?: string
}

// emit消息体
interface Message<T> {
  status: string,
  content: T
}

// 服务器资源
interface ServeritemVo {
	id: number,
	brand_id?: number,
	version_id?: number,
	download_url?: string
}

export type {
  ServerBrand,
  ServerVersion,
  ServerResourceVo,
  DownloadVo,
  ServerQueryVo,
  ServerImportResourceVo,
  ServerImportMirrorVo,
  SearchDialogForm,
  ServerRunVo,
  ServerInstance,
  Message,
  ServerInstanceVo,
  ServeritemVo
}