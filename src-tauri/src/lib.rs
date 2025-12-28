use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::{Emitter, Window};

#[derive(Serialize, Deserialize)]
pub struct FFmpegInfo {
    version: String,
    is_available: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GpuInfo {
    has_nvenc: bool,
    has_videotoolbox: bool,
    has_amf: bool,
    has_qsv: bool,
}

#[tauri::command]
fn check_ffmpeg() -> FFmpegInfo {
    let ffmpeg_output = Command::new("ffmpeg").arg("-version").output();

    let ffprobe_output = Command::new("ffprobe").arg("-version").output();

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
fn check_gpu_accel() -> GpuInfo {
    let output = Command::new("ffmpeg").arg("-encoders").output();

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

#[derive(Serialize, Deserialize)]
pub struct VideoInfo {
    width: i32,
    height: i32,
    duration: f64,
    size: u64,
    format: String,
    video_codec: String,
    video_bitrate: String,
    audio_codec: String,
    audio_bitrate: String,
    frame_rate: String,
}

#[tauri::command]
async fn get_video_info(path: String) -> Result<VideoInfo, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=width,height,duration,codec_name,bit_rate,r_frame_rate,codec_type:format=size,format_name,bit_rate",
            "-of",
            "json",
            &path,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

    let streams = json["streams"].as_array();
    let format = &json["format"];

    let mut video_info = VideoInfo {
        width: 0,
        height: 0,
        duration: 0.0,
        size: format["size"].as_str().unwrap_or("0").parse().unwrap_or(0),
        format: format["format_name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        video_codec: "none".to_string(),
        video_bitrate: "0".to_string(),
        audio_codec: "none".to_string(),
        audio_bitrate: "0".to_string(),
        frame_rate: "0".to_string(),
    };

    if let Some(streams) = streams {
        for s in streams {
            match s["codec_type"].as_str() {
                Some("video") if video_info.video_codec == "none" => {
                    video_info.width = s["width"].as_i64().unwrap_or(0) as i32;
                    video_info.height = s["height"].as_i64().unwrap_or(0) as i32;
                    video_info.video_codec =
                        s["codec_name"].as_str().unwrap_or("unknown").to_string();
                    video_info.video_bitrate = s["bit_rate"].as_str().unwrap_or("0").to_string();
                    video_info.frame_rate = s["r_frame_rate"].as_str().unwrap_or("0").to_string();
                    if video_info.duration == 0.0 {
                        video_info.duration =
                            s["duration"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    }
                }
                Some("audio") if video_info.audio_codec == "none" => {
                    video_info.audio_codec =
                        s["codec_name"].as_str().unwrap_or("unknown").to_string();
                    video_info.audio_bitrate = s["bit_rate"].as_str().unwrap_or("0").to_string();
                }
                _ => {}
            }
        }
    }

    if video_info.duration == 0.0 {
        video_info.duration = format["duration"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);
    }

    Ok(video_info)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskSettings {
    video: VideoSettings,
    audio: AudioSettings,
    advanced: AdvancedSettings,
    output: OutputSettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoSettings {
    codec: String,
    accel: String,
    mode: String,
    crf: i32,
    bitrate: String,
    preset: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioSettings {
    stream: String,
    codec: String,
    bitrate: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdvancedSettings {
    resolution: String,
    custom_width: i32,
    custom_height: i32,
    trim_start: String,
    trim_end: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutputSettings {
    dir: String,
    suffix: String,
}

#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    task_id: String,
    progress: f64,
    status: String,
}

#[tauri::command]
async fn run_task(
    window: Window,
    task_id: String,
    input_path: String,
    duration: f64,
    settings: TaskSettings,
) -> Result<(), String> {
    let mut args = vec!["-i".to_string(), input_path.clone()];
    // ... (rest of the args construction)

    // Video settings
    let codec = match settings.video.accel.as_str() {
        "nvenc" => {
            if settings.video.codec == "libx264" {
                "h264_nvenc"
            } else {
                "hevc_nvenc"
            }
        }
        "videotoolbox" => {
            if settings.video.codec == "libx264" {
                "h264_videotoolbox"
            } else {
                "hevc_videotoolbox"
            }
        }
        "amf" => {
            if settings.video.codec == "libx264" {
                "h264_amf"
            } else {
                "hevc_amf"
            }
        }
        "qsv" => {
            if settings.video.codec == "libx264" {
                "h264_qsv"
            } else {
                "hevc_qsv"
            }
        }
        _ => &settings.video.codec,
    };
    args.push("-c:v".to_string());
    args.push(codec.to_string());

    if settings.video.mode == "crf" {
        args.push("-crf".to_string());
        args.push(settings.video.crf.to_string());
    } else {
        args.push("-b:v".to_string());
        args.push(settings.video.bitrate.clone());
    }

    args.push("-preset".to_string());
    args.push(settings.video.preset.clone());

    // Audio settings
    match settings.audio.stream.as_str() {
        "copy" => {
            args.push("-c:a".to_string());
            args.push("copy".to_string());
        }
        "none" => {
            args.push("-an".to_string());
        }
        "encode" => {
            args.push("-c:a".to_string());
            args.push(settings.audio.codec.clone());
            args.push("-b:a".to_string());
            args.push(settings.audio.bitrate.clone());
        }
        _ => {}
    }

    // Advanced settings
    if settings.advanced.resolution != "original" {
        args.push("-vf".to_string());
        let scale_filter = if settings.advanced.resolution == "custom" {
            format!("scale={}:{}", settings.advanced.custom_width, settings.advanced.custom_height)
        } else {
            format!("scale={}", settings.advanced.resolution)
        };
        args.push(scale_filter);
    }

    if !settings.advanced.trim_start.is_empty() {
        args.push("-ss".to_string());
        args.push(settings.advanced.trim_start.clone());
    }
    if !settings.advanced.trim_end.is_empty() {
        args.push("-to".to_string());
        args.push(settings.advanced.trim_end.clone());
    }

    // Output path
    let input_file = std::path::Path::new(&input_path);
    let parent = if settings.output.dir.is_empty() || settings.output.dir == "original" {
        input_file
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
    } else {
        std::path::Path::new(&settings.output.dir)
    };

    let stem = input_file.file_stem().unwrap_or_default().to_string_lossy();
    let ext = input_file.extension().unwrap_or_default().to_string_lossy();
    let output_name = format!("{}{}.{}", stem, settings.output.suffix, ext);
    let output_path = parent.join(output_name);

    args.push("-y".to_string()); // Overwrite
    args.push(output_path.to_string_lossy().to_string());

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    let reader = BufReader::new(stderr);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("time=") {
                // Extract time=00:00:05.00
                if let Some(time_part) = line.split("time=").nth(1) {
                    let time_str = time_part.split_whitespace().next().unwrap_or("");
                    // Parse HH:MM:SS.MS
                    let parts: Vec<&str> = time_str.split(':').collect();
                    if parts.len() == 3 {
                        let h: f64 = parts[0].parse().unwrap_or(0.0);
                        let m: f64 = parts[1].parse().unwrap_or(0.0);
                        let s: f64 = parts[2].parse().unwrap_or(0.0);
                        let current_seconds = h * 3600.0 + m * 60.0 + s;

                        if duration > 0.0 {
                            let progress = (current_seconds / duration * 100.0).min(99.9);
                            window
                                .emit(
                                    "task-progress",
                                    ProgressPayload {
                                        task_id: task_id.clone(),
                                        progress,
                                        status: "processing".to_string(),
                                    },
                                )
                                .unwrap();
                        }
                    }
                }
            }
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        window
            .emit(
                "task-progress",
                ProgressPayload {
                    task_id: task_id.clone(),
                    progress: 100.0,
                    status: "done".to_string(),
                },
            )
            .unwrap();
        Ok(())
    } else {
        Err("FFmpeg process failed".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg,
            check_gpu_accel,
            get_video_info,
            run_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
