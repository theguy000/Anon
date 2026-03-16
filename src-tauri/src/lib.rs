mod camoufox;
mod instances;

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
async fn create_instance(app: tauri::AppHandle, name: String, proxy: Option<String>) -> Result<instances::InstanceConfig, String> {
    instances::create_instance(&app, name, proxy).await
}

#[tauri::command]
async fn delete_instance(app: tauri::AppHandle, id: String) -> Result<(), String> {
    instances::delete_instance(&app, id).await
}

#[tauri::command]
async fn launch_instance(app: tauri::AppHandle, id: String) -> Result<(), String> {
    instances::launch_instance(&app, id).await
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
            launch_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
