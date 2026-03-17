mod camoufox;
mod fingerprint_presets;
mod instances;
mod settings;

#[tauri::command]
async fn check_camoufox(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(camoufox::get_camoufox_binary(&app).await.is_some())
}

#[tauri::command]
async fn fetch_camoufox(app: tauri::AppHandle) -> Result<(), String> {
    camoufox::download_and_extract(&app).await
}

#[tauri::command]
async fn list_instances(app: tauri::AppHandle) -> Result<Vec<instances::InstanceConfig>, String> {
    instances::list_instances(&app).await
}

#[tauri::command]
async fn create_instance(app: tauri::AppHandle, name: String, proxy: Option<String>, persist_data: bool) -> Result<instances::InstanceConfig, String> {
    instances::create_instance(&app, name, proxy, persist_data).await
}

#[tauri::command]
async fn toggle_persistence(app: tauri::AppHandle, id: String, enabled: bool) -> Result<(), String> {
    instances::toggle_persistence(&app, id, enabled).await
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> settings::AppSettings {
    settings::load_settings(&app).await
}

#[tauri::command]
async fn update_settings(app: tauri::AppHandle, settings: settings::AppSettings) -> Result<(), String> {
    settings::save_settings(&app, &settings).await
}

#[tauri::command]
async fn delete_instance(app: tauri::AppHandle, id: String) -> Result<(), String> {
    instances::delete_instance(&app, id).await
}

#[tauri::command]
async fn launch_instance(app: tauri::AppHandle, id: String) -> Result<(), String> {
    instances::launch_instance(&app, id).await
}

#[tauri::command]
async fn update_instance_settings(app: tauri::AppHandle, id: String, fingerprint: instances::FingerprintConfig) -> Result<(), String> {
    instances::update_instance_settings(&app, id, fingerprint).await
}

#[tauri::command]
async fn get_fingerprint_presets() -> std::collections::HashMap<String, Vec<fingerprint_presets::Preset>> {
    fingerprint_presets::get_presets().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_camoufox, 
            fetch_camoufox,
            list_instances,
            create_instance,
            delete_instance,
            launch_instance,
            toggle_persistence,
            get_settings,
            update_settings,
            update_instance_settings,
            get_fingerprint_presets
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
