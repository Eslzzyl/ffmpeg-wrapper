use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::{Emitter, Window};

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskSettings {
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub advanced: AdvancedSettings,
    pub output: OutputSettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoSettings {
    pub codec: String,
    pub accel: String,
    pub mode: String,
    pub crf: i32,
    pub bitrate: String,
    pub preset: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioSettings {
    pub stream: String,
    pub codec: String,
    pub bitrate: String,
    pub sample_rate: String,
    pub channels: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdvancedSettings {
    pub resolution: String,
    pub custom_width: i32,
    pub custom_height: i32,
    pub trim_start: String,
    pub trim_end: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutputSettings {
    pub dir: String,
    pub suffix: String,
}

#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    pub task_id: String,
    pub progress: f64,
    pub status: String,
}

#[tauri::command]
pub async fn run_task(
    window: Window,
    task_id: String,
    input_path: String,
    duration: f64,
    settings: TaskSettings,
) -> Result<(), String> {
    let mut args = vec!["-i".to_string(), input_path.clone()];

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

            // Add sample rate if specified
            if !settings.audio.sample_rate.is_empty() {
                args.push("-ar".to_string());
                args.push(settings.audio.sample_rate.clone());
            }

            // Add channels if specified
            if !settings.audio.channels.is_empty() {
                args.push("-ac".to_string());
                args.push(settings.audio.channels.clone());
            }
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