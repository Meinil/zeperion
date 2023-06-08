#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use tauri::async_runtime::block_on;
use zeperion::utils::server;

pub mod commands;

fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
    tauri::Builder::default()
    	//新增关闭提示的逻辑
        .on_window_event(|event|{
            match event.event() {
                tauri::WindowEvent::CloseRequested { api, .. } =>{
                    //阻止默认关闭
                    api.prevent_close();
                    let window = event.window().clone();
                    tauri::api::dialog::confirm(Some(&event.window()), "关闭应用", "如果关闭了Zeperion，则所有服务器都会停止运行", move| answer|{
                        if answer {
                            // 停止所有进程
                            let result = block_on(server::kill_all_server());
                            match result {
                                Ok(()) => {
                                    window.close().unwrap();
                                }
                                Err(_) => {
                                    // tauri::api::dialog::message(Some(&event.window()), "提示", "关闭所有应用失败")
                                }
                            }
                        }
                    })
                },
                _ => {}//todo
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::close_splashscreen,
            commands::query_brand_all,
            commands::query_server, 
            commands::update_server_item,
            commands::download,
            commands::query_version_by_brand_id,
            commands::query_global_config,
            commands::update_global_config,
            commands::run_server,
            commands::stop_server,
            commands::get_server_log,
            commands::remove_server_log,
            commands::create_server,
            commands::query_instance_all,
            commands::query_instance_by_id,
            commands::update_server_instance_by_id, 
            commands::set_eula,
            commands::open_file_manager,
            commands::import_resource,
            commands::import_mirror,
            commands::remove_resource,
            commands::remove_mirror,
            commands::remove_instance,
            commands::add_server_brand,
            commands::add_server_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}