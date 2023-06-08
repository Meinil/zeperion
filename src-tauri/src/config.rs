use lazy_static::lazy_static;
use rbatis::{Rbatis, rbdc};
use rbdc_sqlite::driver::SqliteDriver;
use serde::{Serialize, Deserialize};
use std::backtrace::Backtrace;
use std::{env, io, fmt};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use thiserror::Error;

// 自定义错误处理
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ZeperionError {
    #[error("{0}请求时出现网络问题")]
    NetWork(String),
	#[error("{0}解析错误")]
    Parse(String),
	#[error("数据库错误{0}")]
	DATABASE(String),
	#[error("文件错误{0}")]
	FILE(String),
	#[error("错误{0}")]
	Error(String)
}

impl From<rbdc::Error> for ZeperionError{
    fn from(value: rbdc::Error) -> Self {
		eprintln!("{:?}", Backtrace::capture());
        ZeperionError::DATABASE(value.to_string())
    }
}

impl From<serde_json::Error> for ZeperionError{
    fn from(value: serde_json::Error) -> Self {
		ZeperionError::Error(value.to_string())
    }
}

impl From<io::Error> for ZeperionError{
    fn from(value: io::Error) -> Self {
		ZeperionError::Error(value.to_string())
    }
}

impl From<tauri::Error> for ZeperionError{
    fn from(value: tauri::Error) -> Self {
		ZeperionError::Error(value.to_string())
    }
}

// 全局配置
#[derive(Clone, Debug, Serialize, Deserialize, )]
pub struct Config {
	java_home: Option<String>,
	cache_path: Option<String>
}

enum FileType {
	DIR
	// FILE
}

impl Config {
	pub fn new() -> Config {
		let config_file = Self::get_config_path() + "/app_config.json";
		let b = Path::new(&config_file).exists();
		if !b {
			let mut file = File::create(&config_file).unwrap();
			file.write("{}".as_bytes()).unwrap();
		}
		let mut file =  File::open(config_file).unwrap();
		let mut contents = String::new();
		file.read_to_string(&mut contents).unwrap();
		serde_json::from_str(&contents).unwrap()
    }

	// 获取全局的java路径
	pub fn get_java_home(&self) -> Option<String> {
		match &self.java_home {
			Some(path) => Some(path.to_string()),
			None => match env::var("JAVA_HOME") {
				Ok(java_home) => Some(java_home),
				Err(_) => None
			}
		}
	}

	// 获取缓存的路径
	pub fn get_cache_path(&self) -> String {
		match &self.cache_path {
			Some(path) => path.to_string(),
			None => Self::get_config_path()
		}
	}

	// 获取服务器的存放地址
	pub fn get_servers_path(&self) -> String {
		let cache_path = self.get_cache_path();
		let servers_path = cache_path + "/servers";
		Config::exists(servers_path, FileType::DIR)
	}

	// 获取镜像存放地址
	pub fn get_mirrors_path(&self) -> String {
		let cache_path = self.get_cache_path();
		let servers_path = cache_path + "/mirrors";
		Config::exists(servers_path, FileType::DIR)
	}

	// 获取app的名称
	pub fn get_app_name() -> String {
		String::from("zeperion")
	}
	
	// 获取app配置文件路径
	pub fn get_app_config() -> String {
		Self::get_config_path() + "/app_config.json"
	}

	// 获取配置文件的路径
	pub fn get_config_path() -> String {
		let config_path = env::var("HOME").unwrap() + "/." + &Self::get_app_name();
		Config::exists(config_path, FileType::DIR)
	}

	// 判断文件或目录是否存在不存在则创建
	fn exists(path: String, file_type: FileType) -> String{
		let b = Path::new(&path).exists();
		if !b {
			match file_type {
				FileType::DIR => fs::create_dir(&path).unwrap()
				// FileType::FILE => {File::create(&path).unwrap(); ()}
			}
		}
		path
	}
}

lazy_static! {
    pub static ref CONFIG: Config = {
        Config::new()
    };

	pub static ref RB: Rbatis = {
		let rb = Rbatis::new();
		let path = format!("{}/{}.db", CONFIG.get_cache_path(), Config::get_app_name());
		let b = Path::new(&path).exists();
		if !b {
			println!("{}", path);
			let mut file = File::create(&path).unwrap();
			let bytes = include_bytes!("../mapper/zeperion.db");
			file.write_all(bytes).unwrap();
		}
		rb.init(SqliteDriver {}, &format!("sqlite://{}", path)).unwrap();
		rb
	};
}
#[derive(Clone, Error, Debug, Serialize, Deserialize)]
pub enum MessageStatus {
	// 下载推送消息
	Download,
	// 服务器正在启动
	ServerStarting,
	// 用户还未同意eula协议
	ServerEula,
	// 服务器启动成功
	ServerSuccess,
	// 服务器启动失败
	ServerFail,
}
impl fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MessageStatus::Download => write!(f, "Download"),
			MessageStatus::ServerStarting => write!(f, "ServerStarting"),
			MessageStatus::ServerEula => write!(f, "ServerEula"),
			MessageStatus::ServerSuccess => write!(f, "ServerSuccess"),
			MessageStatus::ServerFail => write!(f, "ServerFail")
		}
    }
}


#[derive(Clone, Error, Debug, Serialize, Deserialize)]
pub struct Message<T> {
	pub status: MessageStatus,
	pub content: T
}