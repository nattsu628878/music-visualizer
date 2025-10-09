use std::process::Command;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn convert_video(input_path: String, output_format: String) -> Result<String, String> {
    // Check if FFmpeg is installed
    let ffmpeg_check = Command::new("ffmpeg")
        .arg("-version")
        .output();

    if ffmpeg_check.is_err() {
        return Err("FFmpeg is not installed. Please install FFmpeg first.".to_string());
    }

    // Determine output path
    let output_path = input_path.replace(".webm", &format!(".{}", output_format));

    // Run FFmpeg conversion
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(&input_path)
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("medium")
        .arg("-crf")
        .arg("23")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("192k")
        .arg("-y") // Overwrite output file if exists
        .arg(&output_path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(output_path)
            } else {
                let error_msg = String::from_utf8_lossy(&result.stderr);
                Err(format!("FFmpeg conversion failed: {}", error_msg))
            }
        }
        Err(e) => Err(format!("Failed to execute FFmpeg: {}", e)),
    }
}

#[tauri::command]
async fn check_ffmpeg_installed() -> Result<bool, String> {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, convert_video, check_ffmpeg_installed])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
