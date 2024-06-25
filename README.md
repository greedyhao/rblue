# rblue

## 项目描述
纯 rust 实现蓝牙协议栈

- 搭建模拟通信环境，实现 hci 之上的部分
- hci 之下对接到实际硬件，测试实际收发数据的能力（预计使用 esp32）
- hci 之下的实现

## 调试信息
log 使用的是 env_logger，log 等级可以参考下面进行设置

总共有下面这些等级

- error
- warn
- info
- debug
- trace
- off

powershell

```
$env:RUST_LOG="debug"
```
