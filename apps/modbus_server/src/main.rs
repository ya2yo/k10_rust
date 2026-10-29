use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::link_patches;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, EspWifi};
use rmodbus::server::context::ModbusContext;
use rmodbus::server::ModbusFrame;
use rmodbus::ModbusProto;

// 定义 Modbus 寄存器容量：16 个线圈，16 个离散输入，16 个输入寄存器，16 个保持寄存器
type AppModbusContext = ModbusContext<16, 16, 16, 16>;

fn main() -> anyhow::Result<()> {
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("=============================================");
    println!("        K10 Modbus/TCP Server (SoftAP)");
    println!("=============================================");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // 1. 初始化并启动 Wi-Fi SoftAP（热点模式）
    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;

    let ap_config = AccessPointConfiguration {
        ssid: "K10-Modbus".try_into().unwrap(),
        password: "12345678".try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        channel: 1,
        max_connections: 4,
        ..Default::default()
    };

    wifi.set_configuration(&Configuration::AccessPoint(ap_config))?;
    wifi.start()?;

    println!("[Wi-Fi AP] 热点已就绪！");
    println!("  - Wi-Fi 名称 (SSID): K10-Modbus");
    println!("  - Wi-Fi 密码       : 12345678");
    println!("  - 单片机固定 IP    : 192.168.71.1");
    println!("  - Modbus/TCP 端口  : 502");

    // 2. 初始化 Modbus 上下文（共享数据中心）
    let mut initial_ctx = AppModbusContext::new();

    // 预填一些初始数据方便观察
    // 保持寄存器 0: 目标温度设定值（比如 26 度）
    let _ = initial_ctx.set_holding(0, 26);
    // 保持寄存器 1: 屏幕背光设定值（比如 80%）
    let _ = initial_ctx.set_holding(1, 80);
    // 线圈 0: 运行状态（开=true）
    let _ = initial_ctx.set_coil(0, true);

    let ctx = Arc::new(Mutex::new(initial_ctx));

    // 3. 后台线程：模拟传感器与运行时间定时自增
    let bg_ctx = Arc::clone(&ctx);
    thread::spawn(move || {
        let mut seconds: u16 = 0;
        loop {
            thread::sleep(Duration::from_secs(1));
            seconds = seconds.wrapping_add(1);

            if let Ok(mut lock) = bg_ctx.lock() {
                // 输入寄存器 0: 开机秒数计数器（每秒自动 +1）
                let _ = lock.set_input(0, seconds);
                // 输入寄存器 1: 模拟当前温度（25.5℃ 放大 10 倍 = 255）
                let temp = 255 + (seconds % 5);
                let _ = lock.set_input(1, temp);
            }
        }
    });

    // 4. 启动 TCP 监听服务（绑定 502 端口）
    let listener = TcpListener::bind("0.0.0.0:502")?;
    println!("\n[Modbus Server] TCP 服务已在 0.0.0.0:502 启动，等待上位机连接...");

    // 5. 循环接受上位机网络连接
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let peer_addr = stream.peer_addr().ok();
                println!(">>> 客户端已连接: {:?}", peer_addr);

                let client_ctx = Arc::clone(&ctx);
                let mut rx_buf = [0u8; 256];
                let mut response = Vec::new();

                loop {
                    match stream.read(&mut rx_buf) {
                        Ok(0) => {
                            println!("<<< 客户端已断开: {:?}", peer_addr);
                            break;
                        }
                        Ok(_n) => {
                            // 默认从站地址 Unit ID 为 1，若客户端发 0/255 广播也支持
                            let mut frame = ModbusFrame::new(
                                1,
                                &rx_buf,
                                ModbusProto::TcpUdp,
                                &mut response,
                            );

                            if frame.parse().is_ok() {
                                if frame.processing_required {
                                    let mut lock = client_ctx.lock().unwrap();
                                    let res = if frame.readonly {
                                        frame.process_read(&*lock)
                                    } else {
                                        let write_res = frame.process_write(&mut *lock);
                                        // 打印被修改的提示
                                        println!(
                                            "[Modbus Write] 功能码: 0x{:02X}, 寄存器: {}, 写入成功",
                                            frame.func, frame.reg
                                        );
                                        write_res
                                    };

                                    if res.is_ok() {
                                        if let Ok(()) = frame.finalize_response() {
                                            if let Err(e) = stream.write_all(&response) {
                                                log::warn!("发送响应失败: {:?}", e);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("读取网络数据出错: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("接受连接失败: {:?}", e);
            }
        }
    }

    Ok(())
}
