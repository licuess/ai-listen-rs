# AI Listen RS

AI Listen RS is a Rust-first, cross-platform prototype for an AI meeting/listening note app.

The current version includes a dependency-free CLI foundation and a Tauri desktop shell.

CLI supports:

- Notes saved as Markdown
- Screenshot capture hooks
- Screen recording start/stop hooks
- Session folders for meeting assets
- A clean place to plug in speech-to-text and AI summarization later

Desktop shell supports:

- Meeting/session sidebar
- Markdown note editor
- Screenshot and recording buttons
- Microphone recording button
- Audio input device selector
- Microphone input test button
- Speech transcription hook
- Background transcription task queue
- Background transcription progress and partial text preview
- Local search across titles and notes
- Persistent local index across notes, media filenames, and transcript text with snippets, highlights, and updated timestamps
- OpenAI-backed summary hook with offline fallback

## Quick Start

CLI:

```powershell
cargo run -- new "Product sync"
cargo run -- note "Product sync" "Decided to ship the Rust desktop prototype first."
cargo run -- screenshot "Product sync"
cargo run -- record-start "Product sync"
cargo run -- record-stop "Product sync"
cargo run -- list
```

Desktop:

```powershell
npm install
npm run tauri:dev
```

AI features:

```powershell
$env:OPENAI_API_KEY = "your_api_key"
$env:OPENAI_TRANSCRIBE_MODEL = "gpt-4o-transcribe"
$env:OPENAI_SUMMARY_MODEL = "gpt-5.5"
npm run tauri:dev
```

Data is stored under:

```text
./ai-listen-data/
```

## Cross-Platform Capture Strategy

Screenshot:

- Windows: PowerShell + .NET `System.Drawing`
- macOS: `screencapture`
- Linux: tries `grim`, `gnome-screenshot`, then ImageMagick `import`

Screen recording:

- Windows: `ffmpeg` with `gdigrab`
- macOS: `screencapture -v`
- Linux: tries `wf-recorder`, then `ffmpeg`

Recording requires the relevant system capture command to be installed and screen-recording permissions granted by the OS.

Audio recording:

- Windows: `ffmpeg` with DirectShow `audio=default`
- macOS: `ffmpeg` with AVFoundation `:0`
- Linux: `ffmpeg` with PulseAudio `default`

Speech transcription uploads the latest session audio file to the configured OpenAI transcription model. Summary generation uses the configured OpenAI summary model when `OPENAI_API_KEY` is set; otherwise the app returns a local summary preview.

Transcription runs as a background task. The UI starts a task, polls its status, and refreshes the current session when the transcript is written back to `notes.md`.

The task panel shows progress and partial text while transcription is running. The microphone test button records a short sample through the selected device and reports whether bytes were captured.

The local search index is persisted at:

```text
./ai-listen-data/.index.json
```

It is rebuilt when sessions, notes, and transcripts change, and it includes transcript file contents for full-text search. Search results include a snippet, highlighted query terms, and the session updated timestamp.

## AI Roadmap

The app shell is ready for these next modules:

- Audio input capture
- Streaming speech-to-text partial updates
- Speaker diarization
- Speaker-aware summaries
- Richer persistent index ranking and snippets
- Production bundle signing and installer polish

## Project Layout

```text
src/            Dependency-free Rust CLI
src-tauri/      Tauri desktop backend and commands
ui/             Static desktop frontend loaded by Tauri
ai-listen-data/ Local meeting data
```

## Verification Notes

- `cargo test` passes for the CLI crate.
- `cargo check` passes for the Tauri backend.
- `npm run tauri:dev` starts `ai-listen-rs-desktop`, confirming the local WebView runtime path.
- `npx tauri build --no-bundle` builds the release executable successfully at `src-tauri/target/release/ai-listen-rs-desktop.exe`.
- Full `npm run tauri:build` now gets past the previous Windows/MSVC compiler access violation; MSI bundling requires WiX. Preinstall WiX or cache `wix314-binaries.zip` if the automatic download times out.
