use std::io::{BufReader, BufRead};
use std::process::{Command, Stdio};
use libc::kill;

use regex::Regex;
use tauri::Window;

use crate::config::{CONFIG, RB, ZeperionError, Message, MessageStatus};
use crate::entity::ServerInstance;
use crate::mapper;

use lazy_static::lazy_static;

lazy_static! {
	static ref SERVER_START_SUCCESS : Regex = Regex::new(
		r#"Done \([0-9\.]+s\)! For help, type "help""#
		).unwrap();
}

pub async fn run_all_server(window: Window) -> Result<(), ZeperionError> {
    let mut rb = RB.acquire().await?;
    let instances = mapper::query_instance_p_id_not_null(&mut rb).await?;
    for instance in instances.into_iter() {
        let win = window.clone();
        run_one_server(win, instance);
    }
    Ok(())
}

// 杀死所有已启动的进程
pub async fn kill_all_server() -> Result<(), ZeperionError> {
    let mut rb = RB.acquire().await?;
    let instances = mapper::query_instance_p_id_not_null(&mut rb).await?;
    for instance in instances.into_iter() {
        match instance.p_id {
            Some(p_id) => {
                kill_process_alive(p_id as i32)?;
                mapper::update_server_instance_p_id(&mut rb, &instance.id.unwrap(), None, instance.t_id).await?;
            },
            None => {}
        }
        
        match instance.t_id {
            Some(t_id) => {
                kill_process_alive(t_id as i32)?;
                mapper::update_server_instance_t_id(&mut rb, &instance.id.unwrap(), None).await?;
            },
            None => {}
        }
    }
    Ok(())
}

async fn run_server(window: Window, instance: ServerInstance) -> Result<(), ZeperionError>{

    let mut rb = RB.acquire().await?;

    // 验证进程是否还依旧存活
    let result = is_process_alive(instance.p_id.unwrap() as i32);
    if !result {
        log::info!("该进程已死: {:?}", instance);
        // 进程不存活,更新pid为null
        mapper::update_server_instance_p_id(&mut rb, &instance.id.unwrap(), None, None).await?;
        return Ok(());
    }

	let mut child = Command::new("tail")
		.arg("-f")
		.arg(format!("{}/{}/logs/latest.log", CONFIG.get_servers_path(), instance.path))
		.stdout(Stdio::piped())
		.spawn()?;

    // 更新日志id
    log::info!("更新实例{}的日志{}", instance.id.unwrap(), child.id());
    mapper::update_server_instance_t_id(&mut rb, &instance.id.unwrap(), Some(child.id())).await?;

	let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let result = line?;
        let event_name = format!("instance-{}", instance.id.unwrap());
        
        let status = if result.contains("You need to agree to the EULA in order to run the server. Go to eula.txt for more info.") {
            println!("触发了eula: {}", result);
            child.kill()?;
            MessageStatus::ServerEula
        } else if SERVER_START_SUCCESS.is_match(&result) {
            MessageStatus::ServerSuccess
        } else if result.contains("Failed to start the minecraft server") {
            println!("触发了启动失败: {}", result);
            child.kill()?;
            MessageStatus::ServerFail
        } else {
            MessageStatus::ServerStarting
        };

        let message = Message{
            status,
            content: result
        };
        println!("{:?}", message);
        window.emit(&event_name, message)?;
    }

    child.wait()?;
    println!("执行完毕");

	Ok(())
}

pub fn run_one_server(window: Window, instance: ServerInstance) {
    tauri::async_runtime::spawn(async move {
        let result = run_server(window, instance).await;
        log::info!("服务器实例日志采集结果: {:?}", result);
    });
}

// 杀死一个已知进程id的进程
pub fn kill_process_alive(p_id: i32) -> Result<bool, ZeperionError> {
    if !is_process_alive(p_id) {
        return Ok(true);
    }

    let result = unsafe { libc::kill(p_id as i32, libc::SIGKILL) };
	if result != 0 {
		return Err(ZeperionError::Error("杀死进程失败".to_string()));
	}

    Ok(true)
}

fn is_process_alive(pid: i32) -> bool {
    // 发送 0 信号给指定 ID 的进程
    let result = unsafe { kill(pid, 0) };
    println!("进程是否存活: {}", result);
    // 如果进程存活，则 kill() 函数返回 0：
    // 若进程不存在，则 kill() 函数返回 ESRCH 错误。
    // EPERM 错误表示进程存在，但是我们没有足够的权限发送信号。
    result == 0 || result == libc::EPERM
}