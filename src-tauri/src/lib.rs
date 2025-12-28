mod ffmpeg;
mod video_info;
mod task;

pub use ffmpeg::{FFmpegInfo, GpuInfo};
pub use video_info::VideoInfo;
pub use task::{TaskSettings, VideoSettings, AudioSettings, AdvancedSettings, OutputSettings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            ffmpeg::check_ffmpeg,
            ffmpeg::check_gpu_accel,
            video_info::get_video_info,
            task::run_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
