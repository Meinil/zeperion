use rbatis::executor::Executor;
use rbatis::html_sql;

use crate::entity::{ServerBrand, ServerItem, ServerInstance, ServerVersion};
use crate::vo::{ServerResourceVo, ServerQueryVo, ServerInstanceVo, ServerItemVo};

rbatis::crud!(ServerBrand {}); 
rbatis::crud!(ServerItem {});
rbatis::crud!(ServerInstance {}); 
rbatis::crud!(ServerVersion {}); 


#[html_sql("mapper/server_brand.html")]
async fn query_server_page(rb: &mut dyn Executor, query_vo: ServerQueryVo) -> Vec<ServerResourceVo> {
  impled!()
}

#[html_sql("mapper/server_brand.html")]
async fn query_server_brand_by_id(rb: &mut dyn Executor, id: u32) -> ServerBrand {
  impled!()
}

#[html_sql("mapper/server_brand.html")]
async fn insert_server_brand(rb: &mut dyn Executor, brand: &str) -> Vec<ServerBrand> {
  impled!()
}

#[html_sql("mapper/server_brand.html")]
async fn query_server_brand_by_brand(rb: &mut dyn Executor, brand: &str) -> Option<ServerBrand> {
  impled!()
}

#[html_sql("mapper/server_item.html")]
async fn update_server_item_download(rb: &mut dyn Executor, id: u32, is_download: &str) -> Vec<ServerItem>{
	impled!()
}

#[html_sql("mapper/server_item.html")]
async fn update_server_item(rb: &mut dyn Executor, item: &ServerItemVo) -> Vec<ServerItem>{
	impled!()
}

#[html_sql("mapper/server_item.html")]
async fn insert_server_item(rb: &mut dyn Executor, brand_id: u32, version_id: u32, download_url: Option<String>, is_download: String) -> Vec<ServerItem>{
	impled!()
}

#[html_sql("mapper/server_item.html")]
async fn query_server_item(rb: &mut dyn Executor, brand_id: u32, version_id: u32) -> Option<ServerItem> {
	impled!()
}

#[html_sql("mapper/server_item.html")]
async fn query_item_by_id(rb: &mut dyn Executor, id: u32) -> ServerItem {
	impled!()
}

#[html_sql("mapper/server_item.html")]
async fn remove_server_item_by_id(rb: &mut dyn Executor, id: u32) -> Vec<ServerItem>{
	impled!()
}

#[html_sql("mapper/server_instance.html")]
async fn query_instance_vo_by_id(rb: &mut dyn Executor, id: u32) -> ServerInstanceVo{
	impled!()
}

#[html_sql("mapper/server_instance.html")]
async fn query_instance_by_id(rb: &mut dyn Executor, id: &u32) -> ServerInstance {
	impled!()
}

// 更新进程id
#[html_sql("mapper/server_instance.html")]
async fn update_server_instance_p_id(rb: &mut dyn Executor, id: &u32, p_id: Option<u32>, t_id: Option<u32>) -> Vec<ServerInstance> {
	impled!()
}

// 更新日志进程id
#[html_sql("mapper/server_instance.html")]
async fn update_server_instance_t_id(rb: &mut dyn Executor, id: &u32, t_id: Option<u32>) -> Vec<ServerInstance> {
	impled!()
}

// 更新instance
#[html_sql("mapper/server_instance.html")]
async fn update_server_instance_by_id(rb: &mut dyn Executor, id: &u32, name: String, remark: Option<String>, vm_options: Option<String>) -> Vec<ServerInstance> {
	impled!()
}

#[html_sql("mapper/server_instance.html")]
async fn remove_instance_by_id(rb: &mut dyn Executor, id: u32) -> Vec<ServerInstance> {
	impled!()
}

#[html_sql("mapper/server_instance.html")]
async fn query_instance_by_item_id(rb: &mut dyn Executor, item_id: u32) -> Vec<ServerInstance> {
	impled!()
}

#[html_sql("mapper/server_instance.html")]
async fn query_instance_p_id_not_null(rb: &mut dyn Executor) -> Vec<ServerInstance> {
	impled!()
}

#[html_sql("mapper/server_version.html")]
async fn query_version_by_id(rb: &mut dyn Executor, id: u32) -> ServerVersion {
	impled!()
}

#[html_sql("mapper/server_version.html")]
async fn query_version_by_brand_id(rb: &mut dyn Executor, brand_id: &u32, version: &str) -> Option<ServerVersion> {
	impled!()
}

#[html_sql("mapper/server_version.html")]
async fn insert_server_version(rb: &mut dyn Executor, brand_id: &u32, version: &str) -> Vec<ServerVersion> {
	impled!()
}