# ohmkit-tauri

电阻工具箱 — Tauri 跨平台 GUI 版。

基于 [ohmkit](https://github.com/liruixi-4869/ohmkit) C CLI 版的分数引擎，用 Rust 重写核心逻辑，HTML/CSS 搭建界面，Tauri 打包为桌面应用。

## 功能

- **串并联等效电阻** — 精确分数运算，无浮点误差，支持 `+` 串联、`||` 并联、括号嵌套、分数输入 `3/7`
- **Δ-Y 变换** — 三角形 ↔ 星形互相转换
- **桥式电路求解** — 自动 Δ→Y 变换，逐步展示中间结果和电路图
- **色环电阻编码** — 阻值 → 色环

## 下载

从 [Releases](https://github.com/liruixi-4869/ohmkit-tauri/releases) 下载对应系统版本：

| 平台 | 文件 |
|------|------|
| Linux | `ohmkit-tauri-linux.tar.gz` |
| Windows | `ohmkit-tauri-windows.zip` |
| macOS | `ohmkit-tauri-macos.tar.gz` |

## 开发

```bash
# 安装 Rust: https://rustup.rs
# Linux 额外依赖:
sudo pacman -S webkit2gtk-4.1

git clone https://github.com/liruixi-4869/ohmkit-tauri.git
cd ohmkit-tauri
cargo run
```

## 协议

MIT
