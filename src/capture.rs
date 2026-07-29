use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

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
            output.to_string_lossy().as_ref(),
        ]);
        command
    };

    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start recorder: {error}"))?;

    fs::write(pid_file, child.id().to_string()).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn stop_recording(pid_file: &Path) -> Result<(), String> {
    let pid = fs::read_to_string(pid_file)
        .map_err(|_| "no active recording pid file found".to_string())?
        .trim()
        .to_string();

    if cfg!(target_os = "windows") {
        run_status(
            Command::new("taskkill").args(["/PID", &pid, "/T", "/F"]),
            "taskkill",
        )?;
    } else {
        run_status(Command::new("kill").arg("-INT").arg(&pid), "kill")?;
    }

    let _ = fs::remove_file(pid_file);
    Ok(())
}

#[cfg(target_os = "windows")]
fn capture_screenshot_windows(output: &Path) -> Result<(), String> {
    let output = output.to_string_lossy();
    let script = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bitmap.Save('{output}', [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()
"#
    );

    run_status(
        Command::new("powershell").args(["-NoProfile", "-Command", &script]),
        "PowerShell screenshot",
    )
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
