use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FFmpegInfo {
    pub version: String,
    pub is_available: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GpuInfo {
    pub has_nvenc: bool,
    pub has_videotoolbox: bool,
    pub has_amf: bool,
    pub has_qsv: bool,
}

#[tauri::command]
pub fn check_ffmpeg() -> FFmpegInfo {
    let ffmpeg_output = std::process::Command::new("ffmpeg").arg("-version").output();

    let ffprobe_output = std::process::Command::new("ffprobe").arg("-version").output();

    match (ffmpeg_output, ffprobe_output) {
        (Ok(out), Ok(_)) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let version = stdout.lines().next().unwrap_or("Unknown").to_string();
            FFmpegInfo {
                version,
                is_available: true,
            }
        }
        _ => FFmpegInfo {
            version: "Not found".to_string(),
            is_available: false,
        },
    }
}

#[tauri::command]
pub fn check_gpu_accel() -> GpuInfo {
    let output = std::process::Command::new("ffmpeg").arg("-encoders").output();

    let mut info = GpuInfo {
        has_nvenc: false,
        has_videotoolbox: false,
        has_amf: false,
        has_qsv: false,
    };

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        info.has_nvenc = stdout.contains("nvenc");
        info.has_videotoolbox = stdout.contains("videotoolbox");
        info.has_amf = stdout.contains("amf");
        info.has_qsv = stdout.contains("qsv");
    }

    info
}