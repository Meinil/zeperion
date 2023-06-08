use serde::{Serialize, Deserialize};

use crate::vo::ServerCreateVo;

// 服务器类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerBrand {
	pub id: u32,
	pub brand: String,
	pub is_deleted: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerItem {
	pub id: u32,
	pub brand_id: u32,
	pub version_id: u32,
	pub download_url: Option<String>,
	pub is_download: String,
	pub is_deleted: String
}

// 服务器实例
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerInstance {
	pub id: Option<u32>,
	pub item_id: u32,
	pub name: String,
	pub path: String,
	pub p_id: Option<u32>,
	pub t_id: Option<u32>,
	pub remark: Option<String>,
	pub vm_options: Option<String>,
	pub is_deleted: String
}


impl ServerInstance {
	pub fn form_server_create(create_vo: ServerCreateVo) -> Self {
		ServerInstance { 
			id: None, 
			item_id: create_vo.item_id, 
			name: create_vo.name, 
			path: create_vo.path, 
			p_id: None, 
			t_id: None,
			remark: create_vo.remark, 
			vm_options: None,
			is_deleted: String::from("0")
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerVersion {
	pub id: u32,
	pub brand_id: u32,
	pub version: String,
	pub is_deleted: String
}
