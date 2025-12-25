# FFmpeg Wrapper 技术文档

## 1. 项目概述
FFmpeg Wrapper 是一个基于 **Tauri 2.0** 和 **Vue 3** 开发的跨平台视频处理工具。它旨在为 FFmpeg 提供一个直观、易用的图形界面，支持硬件加速检测、批量任务处理和实时进度反馈。

## 2. 技术栈
- **前端框架**: Vue 3 (Composition API)
- **构建工具**: Vite
- **跨平台框架**: Tauri 2.0
- **后端语言**: Rust
- **核心引擎**: FFmpeg (需安装在系统 PATH 中)
- **通信协议**: Tauri IPC (Invoke & Events)

## 3. 核心功能实现

### 3.1 环境检测 (Rust)
程序启动时会通过 Rust 后端执行以下检测：
- **FFmpeg 可用性**: 执行 `ffmpeg -version` 和 `ffprobe -version` 验证环境。
- **硬件加速检测**: 解析 `ffmpeg -encoders` 的输出，识别系统支持的硬件加速器（如 `nvenc`, `videotoolbox`, `amf`, `qsv`）。

### 3.2 视频信息解析 (Rust + ffprobe)
当用户添加文件时，后端调用 `ffprobe` 以 JSON 格式获取视频元数据：
- 分辨率 (Width/Height)
- 时长 (Duration)
- 文件大小 (Size)
- 封装格式 (Format)

### 3.3 任务执行逻辑 (Rust)
任务执行采用异步方式，通过 `Command` 启动 FFmpeg 子进程。
- **参数构建**: 根据前端传递的 `TaskSettings` 结构体动态生成 FFmpeg 命令行参数。
- **进度追踪**: 实时捕获 FFmpeg 的 `stderr` 输出，通过正则表达式解析 `time=` 字段，计算当前进度百分比。
- **事件反馈**: 使用 Tauri 的 `Emitter` 接口将进度和状态实时推送到前端。

### 3.4 前端交互 (Vue)
- **任务管理**: 支持文件拖放（基于 Tauri 窗口事件）和文件选择对话框。
- **响应式配置**: 参数配置面板根据选中的任务动态显示信息，并根据后端检测到的 GPU 能力过滤编码器选项。
- **全局状态**: 实时显示 FFmpeg 准备状态、GPU 加速开关以及总处理进度。

## 4. 项目结构
```
├── src/                # Vue 前端代码
│   ├── App.vue         # 主界面逻辑与样式
│   └── main.js         # 入口文件
├── src-tauri/          # Rust 后端代码
│   ├── src/
│   │   ├── lib.rs      # 命令实现与 IPC 逻辑
│   │   └── main.rs     # 程序入口
│   ├── capabilities/   # 权限配置
│   └── tauri.conf.json # Tauri 配置文件
└── TECHNICAL.md        # 本文档
```

## 5. 关键数据结构 (Rust)
```rust
// 任务配置结构
pub struct TaskSettings {
    video: VideoSettings,
    audio: AudioSettings,
    advanced: AdvancedSettings,
    output: OutputSettings,
}

// 进度推送载荷
pub struct ProgressPayload {
    task_id: String,
    progress: f64,
    status: String,
}
```

## 6. 权限说明
项目在 `src-tauri/capabilities/default.json` 中配置了以下权限：
- `core:default`: 核心功能权限。
- `opener:default`: 允许打开外部资源。
- `dialog:default`: 允许使用原生文件选择对话框。

## 7. 开发与构建
- **开发模式**: `pnpm tauri dev`
- **生产构建**: `pnpm tauri build`

---
*文档更新日期: 2025-12-25*
