use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VideoInfo {
    pub width: i32,
    pub height: i32,
    pub duration: f64,
    pub size: u64,
    pub format: String,
    pub video_codec: String,
    pub video_bitrate: String,
    pub audio_codec: String,
    pub audio_bitrate: String,
    pub audio_sample_rate: String,
    pub audio_channels: String,
    pub frame_rate: String,
}

#[tauri::command]
pub async fn get_video_info(path: String) -> Result<VideoInfo, String> {
    let output = std::process::Command::new("ffprobe")
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
        audio_sample_rate: "0".to_string(),
        audio_channels: "0".to_string(),
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
                    video_info.audio_sample_rate = s["sample_rate"].as_str().unwrap_or("0").to_string();
                    video_info.audio_channels = s["channels"].as_str().unwrap_or("0").to_string();
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