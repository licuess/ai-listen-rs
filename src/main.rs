mod capture;
mod sessions;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use capture::{capture_screenshot, start_recording, stop_recording};
use sessions::{SessionStore, slugify};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());
    let store = SessionStore::new(PathBuf::from("ai-listen-data"));

    match command.as_str() {
        "new" => {
            let title = collect_required(args, "missing session title")?;
            let session = store.create_session(&title)?;
            println!("created: {}", session.display());
        }
        "note" => {
            let title = args.next().ok_or("missing session title")?;
            let content = collect_required(args, "missing note content")?;
            let path = store.append_note(&title, &content)?;
            println!("saved note: {}", path.display());
        }
        "screenshot" => {
            let title = collect_required(args, "missing session title")?;
            let session = store.ensure_session(&title)?;
            let output = session.join(format!("screenshot-{}.png", timestamp()));
            capture_screenshot(&output)?;
            println!("saved screenshot: {}", output.display());
        }
        "record-start" => {
            let title = collect_required(args, "missing session title")?;
            let session = store.ensure_session(&title)?;
            let video = session.join(format!("recording-{}.mp4", timestamp()));
            let pid_file = session.join(".recording.pid");
            start_recording(&video, &pid_file)?;
            println!("recording: {}", video.display());
        }
        "record-stop" => {
            let title = collect_required(args, "missing session title")?;
            let session = store.ensure_session(&title)?;
            let pid_file = session.join(".recording.pid");
            stop_recording(&pid_file)?;
            println!("recording stopped");
        }
        "list" => {
            for session in store.list_sessions()? {
                println!("{}", session.display());
            }
        }
        "slug" => {
            let title = collect_required(args, "missing title")?;
            println!("{}", slugify(&title));
        }
        "help" | "--help" | "-h" => print_help(),
        unknown => return Err(format!("unknown command: {unknown}")),
    }

    Ok(())
}

fn collect_required(
    args: impl Iterator<Item = String>,
    message: &'static str,
) -> Result<String, String> {
    let value = args.collect::<Vec<_>>().join(" ");
    if value.trim().is_empty() {
        Err(message.to_string())
    } else {
        Ok(value)
    }
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

fn print_help() {
    println!(
        "\
AI Listen RS

Usage:
  ai-listen-rs new <title>
  ai-listen-rs note <title> <content>
  ai-listen-rs screenshot <title>
  ai-listen-rs record-start <title>
  ai-listen-rs record-stop <title>
  ai-listen-rs list

Examples:
  ai-listen-rs new \"Product sync\"
  ai-listen-rs note \"Product sync\" \"Follow up with the design review.\"
"
    );
}
