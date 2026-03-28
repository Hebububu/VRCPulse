use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub indicator: String,
    pub description: String,
    pub updated_at: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! VRCPulse Desktop is running.", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
