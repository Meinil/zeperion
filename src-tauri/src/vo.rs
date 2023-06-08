use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerResourceVo {
	pub id: u32,
	pub brand_id: u32,
	pub brand: String,
	pub version_id: u32,
	pub version: String,
	pub download_url: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerQueryVo {
	pub is_download: String,
	pub brand_id: Option<u32>,
	pub version_id: Option<u32>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVo {
	pub java_home: Option<String>,
	pub cache_path: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCreateVo {
	pub item_id: u32,
	pub name: String,
	pub path: String,
	pub remark: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerInstanceVo {
	pub id: u32,
	pub name: String,
	pub path: String,
	pub t_id: Option<u32>,
	pub p_id: Option<u32>,
	pub item_id: u32,
	pub brand: String,
	pub version: String,
	pub remark: Option<String>,
	pub vm_options: Option<String>,
	pub properties: Option<String>
}

// 本地导入资源
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerImportResourceVo {
	pub brand_id: u32,
	pub version_id: u32,
	pub download_url: String
}

// 本地导入jar
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerImportMirrorVo {
	pub brand_id: u32,
	pub version_id: u32,
	pub path: String
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerItemVo {
	pub id: u32,
	pub brand_id: u32,
	pub version_id: u32,
	pub download_url: Option<String>
}
