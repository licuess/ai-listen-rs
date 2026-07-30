use std::fs;
use std::path::Path;
use std::process::{Command, Stdio, ChildStdin};
use std::sync::Mutex;
use std::sync::LazyLock;

// 保存录屏进程的 stdin 管道（用于优雅停止）
static RECORD_STDIN: LazyLock<Mutex<Option<ChildStdin>>> = LazyLock::new(|| Mutex::new(None));

pub fn capture_screenshot(output: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        capture_screenshot_windows(output)
    } else if cfg!(target_os = "macos") {
        run_status(Command::new("screencapture").arg(output), "screencapture")
    } else {
        capture_screenshot_linux(output)
    }
}

pub fn start_recording(output: &Path, pid_file: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = Command::new("ffmpeg");
        command.args([
            "-y",
            "-f",
            "gdigrab",
            "-framerate",
            "30",
            "-i",
            "desktop",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
            output.to_string_lossy().as_ref(),
        ]);
        command
    } else if cfg!(target_os = "macos") {
        let mut command = Command::new("screencapture");
        command.args(["-v", output.to_string_lossy().as_ref()]);
        command
    } else if command_exists("wf-recorder") {
        let mut command = Command::new("wf-recorder");
        command.args(["-f", output.to_string_lossy().as_ref()]);
        command
    } else {
        let mut command = Command::new("ffmpeg");
        command.args([
            "-y",
            "-f",
            "x11grab",
            "-framerate",
            "30",
            "-i",
            ":0.0",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
            output.to_string_lossy().as_ref(),
        ]);
        command
    };

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start recorder: {error}"))?;

    // 保存 stdin 管道用于后续优雅停止
    if let Some(stdin) = child.stdin.take() {
        *RECORD_STDIN.lock().unwrap() = Some(stdin);
    }

    fs::write(pid_file, child.id().to_string()).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn stop_recording(pid_file: &Path) -> Result<(), String> {
    let pid = fs::read_to_string(pid_file)
        .map_err(|_| "no active recording pid file found".to_string())?
        .trim()
        .to_string();

    // 清理 PID 文件
    let _ = fs::remove_file(pid_file);

    // 通过 stdin 发送 "q" 优雅停止 ffmpeg（确保 mp4 文件正确关闭）
    {
        use std::io::Write;
        let mut stdin_lock = RECORD_STDIN.lock().unwrap();
        if let Some(mut stdin) = stdin_lock.take() {
            let _ = stdin.write_all(b"q");
            let _ = stdin.flush();
        }
    }

    // 等待 ffmpeg 自行退出并写完文件头
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // 如果仍然在运行，强制结束（兜底）
    if cfg!(target_os = "windows") {
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

#[cfg(target_os = "windows")]
fn capture_screenshot_windows(output: &Path) -> Result<(), String> {
    // 优先尝试 ffmpeg，如果不可用则回退到 PowerShell
    if command_exists("ffmpeg") {
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "gdigrab",
                "-frames:v",
                "1",
                "-i",
                "desktop",
            ])
            .arg(output)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| format!("failed to run ffmpeg screenshot: {error}"))?;

        if status.success() {
            return Ok(());
        }
    }

    // 回退到 PowerShell + .NET 截图（通过环境变量传路径，避免转义问题）
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$screen = [System.Windows.Forms.Screen]::PrimaryScreen
$bitmap = New-Object System.Drawing.Bitmap($screen.Bounds.Width, $screen.Bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($screen.Bounds.Location, [System.Drawing.Point]::Empty, $screen.Bounds.Size)
$bitmap.Save($env:AI_LISTEN_OUTPUT)
$graphics.Dispose()
$bitmap.Dispose()
"#;

    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .env("AI_LISTEN_OUTPUT", output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run powershell screenshot: {error}"))?;

    if !status.success() {
        return Err("screenshot capture failed".to_string());
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn capture_screenshot_windows(_output: &Path) -> Result<(), String> {
    Err("Windows screenshot is unavailable on this platform".to_string())
}

#[cfg(target_os = "linux")]
fn capture_screenshot_linux(output: &Path) -> Result<(), String> {
    if command_exists("grim") {
        run_status(Command::new("grim").arg(output), "grim")
    } else if command_exists("gnome-screenshot") {
        run_status(
            Command::new("gnome-screenshot").args(["-f", output.to_string_lossy().as_ref()]),
            "gnome-screenshot",
        )
    } else {
        run_status(
            Command::new("import")
                .arg("-window")
                .arg("root")
                .arg(output),
            "import",
        )
    }
}

#[cfg(not(target_os = "linux"))]
fn capture_screenshot_linux(_output: &Path) -> Result<(), String> {
    Err("Linux screenshot is unavailable on this platform".to_string())
}

fn run_status(command: &mut Command, label: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("failed to run {label}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} exited with status {status}"))
    }
}

fn command_exists(name: &str) -> bool {
    let mut checker = if cfg!(target_os = "windows") {
        let mut command = Command::new("where");
        command.arg(name);
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c").arg(format!("command -v {name}"));
        command
    };

    checker
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
