use std::fs::{OpenOptions, self};
use std::io::{Write, Read, BufReader, BufRead};
use std::path::Path;
use std::process::{Command, Stdio};

use regex::Regex;
use tauri::Window;
use zeperion::config::{ZeperionError, RB, CONFIG, Config, Message, MessageStatus};
use zeperion::mapper;
use zeperion::vo::{ServerQueryVo, ConfigVo, ServerResourceVo, ServerCreateVo, ServerImportResourceVo, ServerImportMirrorVo, ServerInstanceVo, ServerItemVo};
use zeperion::entity::{ServerBrand, ServerVersion, ServerInstance};
use zeperion::utils::{download, server};
use tauri::Manager;
use lazy_static::lazy_static;

lazy_static! {
	static ref SERVER_START_SUCCESS : Regex = Regex::new(
		r#"Done \([0-9\.]+s\)! For help, type "help""#
		).unwrap();
}

// 关闭启动动画
#[tauri::command]
pub async fn close_splashscreen(window: tauri::Window) {
  if let Some(splashscreen) = window.get_window("splashscreen") {
    splashscreen.close().unwrap();
  }
  window.get_window("main").unwrap().show().unwrap();
}

// 服务器查询
#[tauri::command]
pub async fn query_server(query_vo: ServerQueryVo) -> Result<Vec<ServerResourceVo> , ZeperionError>{
	let mut rb = RB.acquire().await.unwrap();
	let result = mapper::query_server_page(&mut rb, query_vo).await?;
	Ok(result)
}

// 查询所有服务器类型
#[tauri::command]
pub async fn query_brand_all() -> Result<Vec<ServerBrand> , ZeperionError>{
	let mut rb = RB.acquire().await.unwrap();
	let result = ServerBrand::select_all(&mut rb)
		.await
		.map_err(|_| ZeperionError::DATABASE(String::from("查询服务器类型失败")));
	result
}

// 下载服务器
#[tauri::command]
pub async fn download(window: Window, item: ServerResourceVo) -> Result<(), ZeperionError> {
    download::download(window, item)
        .await
        .map_err(|e| ZeperionError::Error(e.to_string()))
}

// 更新服务器资源
#[tauri::command]
pub async fn update_server_item(item: ServerItemVo) -> Result<() , ZeperionError>{
	let mut rb = RB.acquire().await.unwrap();
	mapper::update_server_item(&mut rb, &item).await?;
	Ok(())
}

// 根据brand_id 查询所有版本
#[tauri::command]
pub async fn query_version_by_brand_id(brand_id: u32) -> Result<Vec<ServerVersion>, ZeperionError> {
	let mut rb = RB.acquire().await?;
    Ok(ServerVersion::select_by_column(&mut rb, "brand_id", brand_id).await?)
}

// 查询全局配置
#[tauri::command]
pub fn query_global_config() -> ConfigVo {
	ConfigVo { java_home: CONFIG.get_java_home(), cache_path: CONFIG.get_cache_path() }
}

// 更新全局配置
#[tauri::command]
pub fn update_global_config(config: ConfigVo) -> Result<(), ZeperionError>{
	let result = serde_json::to_string(&config)?;
	let path = Config::get_app_config();
	let mut file = OpenOptions::new()
        .write(true)
		.truncate(true)
		.open(&path)?;
	file.write_all(result.as_bytes())?;
	Ok(())
}

// 创建服务器实例
#[tauri::command]
pub async fn create_server(create_vo: ServerCreateVo) -> Result<(), ZeperionError> {
	let instance = ServerInstance::form_server_create(create_vo);
	let mut rb = RB.acquire().await?;
	ServerInstance::insert(&mut rb, &instance).await?;

	// 创建该服务实例的存储路径
	let path = format!("{}/{}", CONFIG.get_servers_path(), instance.path);
	let b = Path::new(&path).exists();
	if !b {
		fs::create_dir(&path)?;
	}
	Ok(())
}

// 查询所有服务器实例
#[tauri::command]
pub async fn query_instance_all() -> Result<Vec<ServerInstance>, ZeperionError> {
	let mut rb = RB.acquire().await?;
	let result = ServerInstance::select_by_column(&mut rb, "is_deleted", "0").await?;
	Ok(result)
}

// 根据id查询服务器实例
#[tauri::command]
pub async fn query_instance_by_id(instance_id: u32) -> Result<ServerInstanceVo, ZeperionError> {
	let mut rb = RB.acquire().await?;
	let mut result = mapper::query_instance_vo_by_id(&mut rb, instance_id).await?;

	let mut rb = RB.acquire().await?;

	let instance = mapper::query_instance_vo_by_id(&mut rb, instance_id).await?;
	let path = format!("{}/{}/server.properties", CONFIG.get_servers_path(), instance.path);

	// 读取该实例的properties文件
	let mut file = OpenOptions::new()
        .read(true)
		.open(&path)?;
	let mut content = String::default();
	file.read_to_string(&mut content)?;
	log::info!("实例{} properties文件: {}", instance_id, content);

	result.properties = Some(content);
	Ok(result)
}

// 更新服务器实例
#[tauri::command]
pub async fn update_server_instance_by_id(instance: ServerInstanceVo) -> Result<(), ZeperionError> {
	let properties = match instance.properties {
		Some(p) => p,
		None => return Err(ZeperionError::Error("请填写正确的properties".to_string()))
	};

	let path = format!("{}/{}/server.properties", CONFIG.get_servers_path(), instance.path);

	// 将properties文件写入
	let mut file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.open(&path)?;
	file.write(properties.as_bytes())?;

	// 更新instance
	let mut rb = RB.acquire().await?;
	mapper::update_server_instance_by_id(&mut rb, &instance.id, instance.name, instance.remark, instance.vm_options).await?;
	Ok(())
}

// 启动服务器
#[cfg(any(target_os="linux", target_os="macos"))]
#[tauri::command]
pub async fn run_server(window: Window, instance_id: u32) -> Result<u32, ZeperionError>{
	let java_home = CONFIG.get_java_home();
	if java_home.is_none() {
		return Err(ZeperionError::Error("请先设置JAVA_HOME环境变量,或者在全局设置里面设置".to_string()));
	}

	let mut rb = RB.acquire().await?;
	let instance = mapper::query_instance_vo_by_id(&mut rb, instance_id).await?;
	let vm_options = match instance.vm_options {
		Some(vm) => vm,
		None => "".to_string()
	};

	let mut child = Command::new(java_home.unwrap() + "/bin" + "/java")
		.arg("-jar")
		.arg(vm_options)
		.arg(format!("{}/{}-{}.jar", CONFIG.get_mirrors_path(), instance.brand, instance.version))
		.arg("--nogui")
		.current_dir(format!("{}/{}", CONFIG.get_servers_path(), instance.path))
		.stdout(Stdio::piped())
		.spawn()?;

	// 更新进程id
	let p_id = child.id();
	mapper::update_server_instance_p_id(&mut rb, &instance_id, Some(p_id), None).await?;
	tauri::async_runtime::spawn(async move{
		let stdout = child.stdout.take().unwrap();
		let reader = BufReader::new(stdout);

		let mut is_break: bool = false;
		for line in reader.lines() {
			let result = line.unwrap();
			let event_name = format!("instance-{}", instance.id);
			let status = if result.contains("You need to agree to the EULA in order to run the server. Go to eula.txt for more info.") {
				child.kill().unwrap();
				MessageStatus::ServerEula
			} else if SERVER_START_SUCCESS.is_match(&result) {
				// 服务器启动成功不再响应日志,之后的日志由tail -f打印
				is_break = true;
				MessageStatus::ServerSuccess
			} else if result.contains("Failed to start the minecraft server") || result.contains("java.lang.RuntimeException: net.minecraft.util.SessionLock$ExceptionWorldConflict"){
				let mut rb = RB.acquire().await.unwrap();
				mapper::update_server_instance_p_id(&mut rb, &instance.id, None, None).await.unwrap();
				child.kill().unwrap();
				log::info!("启动失败: {}", instance.id);
				MessageStatus::ServerFail
			} else {
				MessageStatus::ServerStarting
			};

			let message = Message{
				status,
				content: result
			};
			println!("{:?}", message);
			window.emit(&event_name, message).unwrap();
			if is_break {
				break;
			}
		}

		log::info!("启动成功: instance_id: {}", instance.id);
	});

	Ok(p_id)
}

// 停止服务器
#[tauri::command]
pub async fn stop_server(p_id: u32, instance_id: u32) -> Result<bool, ZeperionError> {
	// 杀死该游戏进程
	server::kill_process_alive(p_id as i32)?;

	// 清空服务器进程id信息
	let mut rb = RB.acquire().await?;
	mapper::update_server_instance_p_id(&mut rb, &instance_id, None, None).await?;
	Ok(true)
}

// 获取服务器的日志
#[tauri::command]
pub async fn get_server_log(window: Window, instance_id: u32) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let instance = mapper::query_instance_by_id(&mut rb, &instance_id).await?;
	server::run_one_server(window, instance);
	Ok(())
}

// 移除服务器日志
#[tauri::command]
pub async fn remove_server_log(instance_id: u32) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let instance = mapper::query_instance_by_id(&mut rb, &instance_id).await?;

	if instance.t_id.is_some() {
		server::kill_process_alive(instance.t_id.unwrap() as i32)?;
		log::info!("杀死{}实例日志进程成功", instance_id);
	} else {
		log::warn!("该{}实例日志进程不存在", instance_id);
	}

	mapper::update_server_instance_t_id(&mut rb, &instance_id, None).await?;
	// mapper::update_server_instance_p_id(&mut rb, &instance_id, instance.p_id, None).await?;
	log::info!("更新实例{}的日志成功", instance_id);
	Ok(())
}

// 设置开服协议
#[tauri::command]
pub async fn set_eula(is_agree: bool, instance_id: u32) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;

	let instance = mapper::query_instance_vo_by_id(&mut rb, instance_id).await?;
	let path = format!("{}/{}/eula.txt", CONFIG.get_servers_path(), instance.path);
	let mut file = OpenOptions::new()
        .write(true)
		.truncate(true)
		.open(&path)?;
	file.write(format!("eula={}", is_agree).as_bytes())?;
	Ok(())
}


// 打开服务器实例文件管理器
#[tauri::command]
pub async fn open_file_manager(instance_id: u32) -> Result<bool, ZeperionError> {
	let mut rb = RB.acquire().await?;

	let instance = mapper::query_instance_vo_by_id(&mut rb, instance_id).await?;
	let path = format!("{}/{}", CONFIG.get_servers_path(), instance.path);

	open::that(path)?;

	Ok(true)
}

// 保存服务器
#[tauri::command]
pub async fn import_resource(import_resource_vo: ServerImportResourceVo) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let item = mapper::query_server_item(&mut rb, import_resource_vo.brand_id, import_resource_vo.version_id).await?;
	match item {
		Some(_) => return Err(ZeperionError::DATABASE(String::from("该资源已经存在"))),
		None => {}
	}
	mapper::insert_server_item(&mut rb, import_resource_vo.brand_id, import_resource_vo.version_id, Some(import_resource_vo.download_url), String::from("0")).await?;
	Ok(())
}

// 从本地导入镜像
#[tauri::command]
pub async fn import_mirror(mirror_vo: ServerImportMirrorVo) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire_begin().await?;

	// 验证当前实例是否存在
	let item = mapper::query_server_item(&mut rb, mirror_vo.brand_id, mirror_vo.version_id).await?;
	match item {
		Some(i) => {
			if i.is_download == "1" { 
				return Err(ZeperionError::DATABASE(String::from("该资源已经存在"))) 
			} else {
				mapper::update_server_item_download(&mut rb, i.id, "1").await?;
			}
		},
		None => {
			mapper::insert_server_item(&mut rb, mirror_vo.brand_id, mirror_vo.version_id, None, String::from("1")).await?;
		}
	}

	// 查询服务器类型
	let brand = mapper::query_server_brand_by_id(&mut rb, mirror_vo.brand_id).await?;
	// 查询服务器版本
	let version = mapper::query_version_by_id(&mut rb, mirror_vo.version_id).await?;
	let to_path = format!("{}/{}-{}.jar", CONFIG.get_mirrors_path(), brand.brand, version.version);
	fs::copy(mirror_vo.path, to_path)?;
	rb.commit().await?;
	Ok(())
}

// 删除资源
#[tauri::command]
pub async fn remove_resource(item_id: u32) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let item = mapper::query_item_by_id(&mut rb, item_id).await?;
	if item.is_download == "1" {
		return Err(ZeperionError::Error(String::from("当前资源已被下载,请先删除对应镜像")))
	}
	mapper::remove_server_item_by_id(&mut rb, item_id).await?;
	Ok(())
}

// 删除镜像
#[tauri::command]
pub async fn remove_mirror(item_id: u32) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let instances = mapper::query_instance_by_item_id(&mut rb, item_id).await?;
	if instances.len() > 0 {
		return Err(ZeperionError::Error(String::from("当前镜像还有服务器使用,请先删除正在使用此镜像的服务器")))
	}

	// 查询服务器类型
	let item = mapper::query_item_by_id(&mut rb, item_id).await?;
	let brand = mapper::query_server_brand_by_id(&mut rb, item.brand_id).await?;
	// 查询服务器版本
	let version = mapper::query_version_by_id(&mut rb, item.version_id).await?;
	let path = format!("{}/{}-{}.jar", CONFIG.get_mirrors_path(), brand.brand, version.version);
	fs::remove_file(path)?;
	// 设置状态为未下载
	mapper::update_server_item_download(&mut rb, item_id, "0").await?;
	Ok(())
}

// 删除服务器实例
#[tauri::command]
pub async fn remove_instance(instance_id: u32) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let instance = mapper::query_instance_vo_by_id(&mut rb, instance_id).await?;
	match instance.p_id {
		Some(_) => {
			Err(ZeperionError::Error(String::from("该实例还在运行中,请先停止")))
		},
		None => {
			mapper::remove_instance_by_id(&mut rb, instance_id).await?;
			Ok(())
		}
	}
}

// 添加服务器类型
#[tauri::command]
pub async fn add_server_brand(brand: String) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let server_brand = mapper::query_server_brand_by_brand(&mut rb, &brand).await?;
	match server_brand {
		Some(_) => {
			Err(ZeperionError::Error(String::from("该服务器类型已经存在")))
		},
		None => {
			mapper::insert_server_brand(&mut rb, &brand).await?;
			Ok(())
		}
	}
}

// 添加服务器版本
#[tauri::command]
pub async fn add_server_version(brand_id: u32, version: String) -> Result<(), ZeperionError>{
	let mut rb = RB.acquire().await?;
	let server_version = mapper::query_version_by_brand_id(&mut rb, &brand_id, &version).await?;
	match server_version {
		Some(_) => {
			Err(ZeperionError::Error(String::from("该服务器版本已经存在")))
		},
		None => {
			mapper::insert_server_version(&mut rb, &brand_id, &version).await?;
			Ok(())
		}
	}
}