# K10 Rust Workspace (UNIHIKER-S3)

基于 ESP-IDF Rust 标准库生态（`esp-idf-svc`）的多子项目（Cargo Workspace）架构工程。内置 `cargo xtask` 统一任务调度工具，支持快速构建、烧录及监视不同的子项目。

## 📁 目录结构

```text
k10_rust/
├── .cargo/
│   └── config.toml          # xtensa 编译目标与 xtask 快捷别名
├── apps/                    # 各子项目目录（每个子项目拥有独立的 Cargo.toml）
│   ├── hardware_test/       # K10 硬件外设与系统自检
│   └── modbus_server/       # Modbus/TCP 服务端
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
cargo xtask run hardware_test

# 运行 Modbus 服务端并指定串口
cargo xtask run modbus_server --port COM3
```
> **提示**：烧录前若设备未自动进入下载模式，可手动进入：
> - 按住板载 **BOOT** 键不放；
> - 短按一下 **RST** 键复位；
> - 松开 **BOOT** 键。

### 3. 一键创建新子项目
```powershell
cargo xtask new my_app
```
会以模板自动在 `apps/my_app` 下生成符合规范的 `Cargo.toml`、`build.rs` 与 `src/main.rs`。

### 4. 仅编译指定子项目
```powershell
cargo xtask build hardware_test
```

### 5. 仅打开串口监视器
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
