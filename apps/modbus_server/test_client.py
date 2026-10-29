"""
K10 ESP32-S3 Modbus/TCP 演示测试客户端
=====================================
运行前请确保：
1. 电脑 Wi-Fi 已连接到 K10 热点:
   - SSID: K10-Modbus
   - 密码: 12345678
2. 已安装 pymodbus:
   pip install pymodbus

运行方法:
   python apps/modbus_server/test_client.py
"""

import inspect
import sys
import time

try:
    from pymodbus.client import ModbusTcpClient
except ImportError:
    print("\n[错误] 未检测到 pymodbus 库！")
    print("请先在终端运行以下命令进行安装:")
    print("    pip install pymodbus\n")
    sys.exit(1)

# K10 SoftAP 默认固定 IP 和端口
SERVER_IP = "192.168.71.1"
SERVER_PORT = 502
SLAVE_ID = 1  # 对应单片机里的 Unit ID


def call_modbus(method, *args, **kwargs):
    """自动兼容不同版本 pymodbus 的从站 ID 参数 (device_id / slave / unit)"""
    sig = inspect.signature(method)
    if "device_id" in sig.parameters:
        kwargs["device_id"] = SLAVE_ID
    elif "slave" in sig.parameters:
        kwargs["slave"] = SLAVE_ID
    elif "unit" in sig.parameters:
        kwargs["unit"] = SLAVE_ID
    return method(*args, **kwargs)


def main():
    print("=" * 55)
    print("      K10 Modbus/TCP Python 交互测试客户端")
    print("=" * 55)
    print(f"正在连接 K10 ({SERVER_IP}:{SERVER_PORT}) ...")

    client = ModbusTcpClient(SERVER_IP, port=SERVER_PORT, timeout=3.0)
    if not client.connect():
        print(f"\n[连接失败] 无法连上 {SERVER_IP}:{SERVER_PORT}！")
        print("请检查：")
        print("1. 你的电脑是否已连接到 K10 的 Wi-Fi 热点: 'K10-Modbus'？")
        print("2. K10 是否已烧录运行 modbus_server？")
        return

    print(">>> 成功连上 K10 Modbus/TCP 服务端！\n")

    try:
        # ==============================================================
        # 演示 1: 连续读取输入寄存器（查看开机运行秒数与模拟温度）
        # ==============================================================
        print("-" * 55)
        print("【演示 1】读取输入寄存器 (Input Registers: 只读数据)")
        print("观察 K10 后台计数器，连续采样 5 秒:")
        print("-" * 55)

        for i in range(1, 6):
            # 读取从地址 0 开始的 2 个输入寄存器
            rr = call_modbus(client.read_input_registers, 0, count=2)
            if rr.isError():
                print(f"[{i}/5] 读取出错: {rr}")
            else:
                uptime_sec = rr.registers[0]
                temp_raw = rr.registers[1]
                temp_c = temp_raw / 10.0
                print(
                    f"[{i}/5] 开机运行时间: {uptime_sec:4d} 秒 | 模拟环境温度: {temp_c:.1f} ℃"
                )
            time.sleep(1)

        # ==============================================================
        # 演示 2: 读取保持寄存器（Holding Registers: 可读可写数据）
        # ==============================================================
        print("\n" + "-" * 55)
        print("【演示 2】读取当前保持寄存器 (Holding Registers)")
        print("-" * 55)

        rr = call_modbus(client.read_holding_registers, 0, count=2)
        if not rr.isError():
            print(f"当前寄存器 0 (设定温度): {rr.registers[0]}")
            print(f"当前寄存器 1 (屏幕背光): {rr.registers[1]}%")

        # ==============================================================
        # 演示 3: 远程向 K10 写入新数值
        # ==============================================================
        print("\n" + "-" * 55)
        print("【演示 3】远程写入新的设定值 (修改 Holding Registers)")
        print("-" * 55)

        new_temp_target = 30
        new_backlight = 95
        print(f"正在将 寄存器 0 改写为: {new_temp_target}")
        call_modbus(client.write_register, 0, new_temp_target)

        print(f"正在将 寄存器 1 改写为: {new_backlight}")
        call_modbus(client.write_register, 1, new_backlight)

        print("\n立即重新回读确认：")
        rr_after = call_modbus(client.read_holding_registers, 0, count=2)
        if not rr_after.isError():
            print(f"回读确认 -> 寄存器 0: {rr_after.registers[0]} (修改成功！)")
            print(f"回读确认 -> 寄存器 1: {rr_after.registers[1]}% (修改成功！)")

        # ==============================================================
        # 演示 4: 读写开关量线圈（Coil: 开/关状态）
        # ==============================================================
        print("\n" + "-" * 55)
        print("【演示 4】读写开关量 (Coils: 布尔值 开/关)")
        print("-" * 55)

        coils = call_modbus(client.read_coils, 0, count=1)
        if not coils.isError():
            print(f"当前线圈 0 (运行指示): {'[开启]' if coils.bits[0] else '[关闭]'}")

        print("正在向 K10 发送指令: 将线圈 0 翻转为 False (关)...")
        call_modbus(client.write_coil, 0, False)

        coils_after = call_modbus(client.read_coils, 0, count=1)
        if not coils_after.isError():
            print(f"回读确认 -> 线圈 0: {'[开启]' if coils_after.bits[0] else '[关闭]'}")

        # 改回 True
        call_modbus(client.write_coil, 0, True)
        print("已将线圈 0 恢复为 True (开)")

        print("\n" + "=" * 55)
        print("           所有 Modbus/TCP 演示操作圆满完成！")
        print("=" * 55)

    except Exception as e:
        print(f"\n[发生异常] 通信中断: {e}")
    finally:
        client.close()
        print("\n已断开与 K10 的 TCP 连接。")


if __name__ == "__main__":
    main()
