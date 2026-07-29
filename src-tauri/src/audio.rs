use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioTestResult {
    pub ok: bool,
    pub message: String,
    pub sample_path: String,
    pub bytes: u64,
    pub mean_db: Option<f32>,
    pub max_db: Option<f32>,
    pub level_percent: u8,
}

pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    if cfg!(target_os = "windows") {
        list_windows_audio_devices()
    } else if cfg!(target_os = "macos") {
        list_macos_audio_devices()
    } else {
        list_linux_audio_devices()
    }
}

pub fn start_audio_recording(
    output: &Path,
    pid_file: &Path,
    device_id: Option<&str>,
) -> Result<(), String> {
    let mut command = audio_command(output, device_id, None);

    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start audio recording: {error}"))?;

    fs::write(pid_file, child.id().to_string()).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn test_audio_input(device_id: Option<&str>) -> Result<AudioTestResult, String> {
    let output = std::env::temp_dir().join(format!("ai-listen-audio-test-{}.wav", timestamp()));
    let mut command = audio_command(&output, device_id, Some("2"));

    let status = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to test audio input: {error}"))?;

    if !status.success() {
        return Err(format!("audio test exited with status {status}"));
    }

    let bytes = fs::metadata(&output)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let (mean_db, max_db) = analyze_audio_level(&output);
    let level_percent = max_db.map(level_from_db).unwrap_or(0);
    Ok(AudioTestResult {
        ok: bytes > 44 && level_percent > 0,
        message: if bytes > 44 {
            format!("麦克风测试完成，输入电平约 {level_percent}%")
        } else {
            "麦克风测试完成，但样本几乎为空".to_string()
        },
        sample_path: output.to_string_lossy().to_string(),
        bytes,
        mean_db,
        max_db,
        level_percent,
    })
}

fn audio_command(output: &Path, device_id: Option<&str>, duration: Option<&str>) -> Command {
    let mut command = Command::new("ffmpeg");

    if cfg!(target_os = "windows") {
        let input = device_id
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("audio={value}"))
            .unwrap_or_else(|| "audio=default".to_string());
        command.args(["-y", "-f", "dshow", "-i", &input]);
    } else if cfg!(target_os = "macos") {
        let input = device_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(":0");
        command.args(["-y", "-f", "avfoundation", "-i", input]);
    } else {
        let input = device_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("default");
        command.args(["-y", "-f", "pulse", "-i", input]);
    }

    if let Some(duration) = duration {
        command.args(["-t", duration]);
    }

    command.arg(output);

    command
}

fn analyze_audio_level(path: &Path) -> (Option<f32>, Option<f32>) {
    let null_target = if cfg!(target_os = "windows") {
        "NUL"
    } else {
        "/dev/null"
    };
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(path)
        .args(["-af", "volumedetect", "-f", "null", null_target])
        .output();
    let Ok(output) = output else {
        return (None, None);
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    (
        parse_db(&stderr, "mean_volume:"),
        parse_db(&stderr, "max_volume:"),
    )
}

fn parse_db(text: &str, label: &str) -> Option<f32> {
    text.lines().find_map(|line| {
        let index = line.find(label)? + label.len();
        let value = line[index..].trim().split_whitespace().next()?;
        value.parse::<f32>().ok()
    })
}

fn level_from_db(db: f32) -> u8 {
    if db <= -60.0 {
        0
    } else {
        (((db + 60.0) / 60.0) * 100.0).round().clamp(0.0, 100.0) as u8
    }
}

fn list_windows_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .output()
        .map_err(|error| format!("failed to list DirectShow devices with ffmpeg: {error}"))?;
    let text = String::from_utf8_lossy(&output.stderr);
    let mut devices = Vec::new();

    for line in text.lines() {
        if !line.contains("(audio)") {
            continue;
        }
        if let Some(name) = quoted_value(line) {
            devices.push(AudioDevice {
                id: name.clone(),
                name,
                is_default: false,
            });
        }
    }

    with_default_device(devices)
}

fn list_macos_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-f",
            "avfoundation",
            "-list_devices",
            "true",
            "-i",
            "",
        ])
        .output()
        .map_err(|error| format!("failed to list AVFoundation devices with ffmpeg: {error}"))?;
    let text = String::from_utf8_lossy(&output.stderr);
    let mut devices = Vec::new();
    let mut in_audio = false;

    for line in text.lines() {
        if line.contains("AVFoundation audio devices") {
            in_audio = true;
            continue;
        }
        if !in_audio {
            continue;
        }
        let Some((id, name)) = bracket_device(line) else {
            continue;
        };
        devices.push(AudioDevice {
            id: format!(":{id}"),
            name,
            is_default: false,
        });
    }

    with_default_device(devices)
}

fn list_linux_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let output = Command::new("pactl")
        .args(["list", "short", "sources"])
        .output();
    let Ok(output) = output else {
        return with_default_device(Vec::new());
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let devices = text
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let _index = parts.next()?;
            let id = parts.next()?.to_string();
            Some(AudioDevice {
                name: id.clone(),
                id,
                is_default: false,
            })
        })
        .collect();

    with_default_device(devices)
}

fn with_default_device(mut devices: Vec<AudioDevice>) -> Result<Vec<AudioDevice>, String> {
    devices.insert(
        0,
        AudioDevice {
            id: String::new(),
            name: "系统默认输入".to_string(),
            is_default: true,
        },
    );
    Ok(devices)
}

fn quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

fn bracket_device(line: &str) -> Option<(String, String)> {
    let start = line.find('[')? + 1;
    let end = line[start..].find(']')? + start;
    let id = line[start..end].trim().to_string();
    let name = line[end + 1..].trim().to_string();
    if id.is_empty() || name.is_empty() {
        None
    } else {
        Some((id, name))
    }
}

pub fn stop_audio_recording(pid_file: &Path) -> Result<(), String> {
    let pid = fs::read_to_string(pid_file)
        .map_err(|_| "no active audio recording pid file found".to_string())?
        .trim()
        .to_string();

    // 先清理 PID 文件
    let _ = fs::remove_file(pid_file);

    if cfg!(target_os = "windows") {
        // taskkill 失败不报错（进程可能已退出）
        let _ = Command::new("taskkill")
            .args(["/PID", &pid, "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        let _ = Command::new("kill")
            .arg("-INT")
            .arg(&pid)
            .stdin(Stdio::null())
            .status();
    }

    Ok(())
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    millis.to_string()
}
