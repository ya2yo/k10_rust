# K10 Rust Workspace (UNIHIKER-S3)

基于 ESP-IDF Rust 标准库生态（`esp-idf-svc`）的多子项目（Cargo Workspace）架构工程。内置 `cargo xtask` 统一任务调度工具，支持快速构建、烧录及监视不同的子项目。

## 📁 目录结构

```text
k10_rust/
├── .cargo/
│   └── config.toml          # xtensa 编译目标与 xtask 快捷别名
├── apps/                    # 各子项目目录（每个子项目拥有独立的 Cargo.toml）
│   ├── k10_test/            # K10 硬件外设与系统自检（CPU、Flash 16MB、PSRAM 8MB）
│   └── modbus_server/       # Modbus/TCP 服务端（SoftAP 模式）及 Python 测试客户端
├── xtask/                   # 宿主机任务执行器 (cargo xtask)
├── sdkconfig.defaults       # K10 (ESP32-S3-N16R8) 硬件底层配置
├── Cargo.toml               # Workspace 根配置文件
└── rust-toolchain.toml      # ESP 工具链声明
```

---

## 📋 工具链准备

因为 ESP32-S3 基于 Xtensa 架构，且使用 `std` + ESP-IDF 方案，需准备 Xtensa Rust 工具链与烧录工具：

1. **安装辅助工具**：
   ```powershell
   cargo install espup
   cargo install ldproxy
   cargo install espflash
   ```

2. **安装并激活 ESP Rust 工具链**：
   ```powershell
   espup install
   . $HOME/export-esp.ps1
   ```

---

## 🚀 使用 cargo xtask 管理与运行子项目

已配置 Cargo 别名，在项目根目录即可直接调用 `cargo xtask`：

### 1. 列出所有子项目
```powershell
cargo xtask list
```

### 2. 编译并烧录运行指定子项目（带串口监视）
```powershell
# 运行硬件自检程序
cargo xtask run k10_test

# 运行 Modbus 服务端并自动选择端口
cargo xtask run modbus_server
```

> **提示**：
> - **自动烧录**：K10（ESP32-S3 原生 USB-Serial-JTAG）支持全自动软复位与下载，执行 `cargo xtask run` 时无需按任何物理按键。
> - **手动进入下载模式（异常应急）**：若因固件死锁导致自动复位失败，可手动进入下载模式：
>   1. 按住板子**【背面】**的 **BOOT** 键不放（注意正面的两个按键是 A/B 键）；
>   2. 短按一下侧面的 **RST** 键复位；
>   3. 松开 **BOOT** 键。

### 3. Modbus/TCP 服务端交互验证
烧录 `modbus_server` 后，K10 会自动启动 Wi-Fi 热点：
- **热点 SSID**：`K10-Modbus`
- **Wi-Fi 密码**：`12345678`
- **服务端 IP 与端口**：`192.168.71.1:502`

电脑连接该 Wi-Fi 热点后，使用子项目自带的 Python 客户端进行功能测试：
```powershell
# 安装依赖（已安装可跳过）
pip install pymodbus

# 运行测试客户端
python apps/modbus_server/test_client.py
```

### 4. 一键创建新子项目
```powershell
cargo xtask new my_app
```
会以模板自动在 `apps/my_app` 下生成符合规范的 `Cargo.toml`、`build.rs` 与 `src/main.rs`。

### 5. 仅编译指定子项目
```powershell
cargo xtask build k10_test
```

### 6. 仅打开串口监视器
```powershell
cargo xtask monitor
```

---

## ➕ 如何新增子项目？

最简单的方法是使用一键命令：
```powershell
cargo xtask new <子项目名称>
```

创建后无需修改任何配置，立即生效：
```powershell
cargo xtask build <子项目名称>
cargo xtask run <子项目名称>
```

---

## 💡 IDE 开发建议 (VS Code)

由于根目录 `Cargo.toml` 中显式排除了 `xtask`，建议在 `.vscode/settings.json` 中配置多项目联动，使 `rust-analyzer` 能同时解析单片机子项目与任务执行器代码：

```json
{
  "rust-analyzer.linkedProjects": [
    "Cargo.toml",
    "xtask/Cargo.toml"
  ]
}
```
