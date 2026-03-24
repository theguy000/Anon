mod auto_fingerprint;
mod camoufox;
mod fingerprint_presets;
mod fingerprint_validator;
mod instances;
mod process_manager;
mod proxy_tester;
mod session_manager;
mod settings;

use tauri::Emitter;

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
async fn create_instance(
    app: tauri::AppHandle,
    name: String,
    proxy: Option<String>,
    persist_data: bool,
) -> Result<instances::InstanceConfig, String> {
    instances::create_instance(&app, name, proxy, persist_data).await
}

#[tauri::command]
async fn toggle_persistence(
    app: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    instances::toggle_persistence(&app, id, enabled).await
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> settings::AppSettings {
    settings::load_settings(&app).await
}

#[tauri::command]
async fn update_settings(
    app: tauri::AppHandle,
    settings: settings::AppSettings,
) -> Result<(), String> {
    settings::save_settings(&app, &settings).await
}

#[tauri::command]
async fn delete_instance(app: tauri::AppHandle, id: String) -> Result<(), String> {
    // Prevent deleting a running instance
    if process_manager::is_running(&id) {
        return Err("Cannot delete a running instance. Stop it first.".to_string());
    }
    instances::delete_instance(&app, id).await
}

#[tauri::command]
async fn launch_instance(app: tauri::AppHandle, id: String, startup_url: Option<String>) -> Result<u32, String> {
    instances::launch_instance(&app, id, startup_url).await
}

#[tauri::command]
async fn update_instance_settings(
    app: tauri::AppHandle,
    id: String,
    fingerprint: instances::FingerprintConfig,
) -> Result<Vec<fingerprint_validator::FingerprintConflict>, String> {
    instances::update_instance_settings(&app, id, fingerprint).await
}

#[tauri::command]
async fn update_instance_proxy(
    app: tauri::AppHandle,
    id: String,
    proxy_config: Option<instances::ProxyConfig>,
) -> Result<(), String> {
    instances::update_instance_proxy(&app, id, proxy_config).await
}

#[tauri::command]
async fn get_fingerprint_presets(
) -> std::collections::HashMap<String, Vec<fingerprint_presets::Preset>> {
    fingerprint_presets::get_presets().clone()
}

// ── New process management commands ──────────────────────────────────────

#[tauri::command]
async fn get_running_instances() -> Vec<process_manager::RunningInstance> {
    process_manager::get_running_instances()
}

#[tauri::command]
async fn is_instance_running(id: String) -> bool {
    process_manager::is_running(&id)
}

#[tauri::command]
async fn stop_instance(id: String) -> Result<(), String> {
    process_manager::stop_instance(&id)
}

/// Called by the frontend when the user picks an action from the exit
/// confirmation dialog.  
/// `action` is one of: `"stop_all"`, `"force_close"`, `"cancel"`.
#[tauri::command]
async fn confirm_close_action(app: tauri::AppHandle, action: String) -> Result<(), String> {
    match action.as_str() {
        "stop_all" => {
            // Kill every running instance, then quit
            let running = process_manager::get_running_instances();
            for inst in &running {
                let _ = process_manager::stop_instance(&inst.id);
            }
            // Give processes a moment to exit
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            app.exit(0);
            Ok(())
        }
        "force_close" => {
            // Quit immediately without stopping instances
            app.exit(0);
            Ok(())
        }
        "cancel" => {
            // Do nothing — the user cancelled the close
            Ok(())
        }
        _ => Err(format!("Unknown action: {}", action)),
    }
}

// ── New feature commands ─────────────────────────────────────────────────

#[tauri::command]
async fn rename_instance(app: tauri::AppHandle, id: String, new_name: String) -> Result<(), String> {
    instances::rename_instance(&app, id, new_name).await
}

#[tauri::command]
async fn update_instance_tags(app: tauri::AppHandle, id: String, tags: Vec<String>) -> Result<(), String> {
    instances::update_instance_tags(&app, id, tags).await
}

#[tauri::command]
async fn update_instance_notes(app: tauri::AppHandle, id: String, notes: Option<String>) -> Result<(), String> {
    instances::update_instance_notes(&app, id, notes).await
}

#[tauri::command]
async fn export_instance(app: tauri::AppHandle, id: String) -> Result<String, String> {
    instances::export_instance(&app, id).await
}

#[tauri::command]
async fn export_all_instances(app: tauri::AppHandle) -> Result<String, String> {
    instances::export_all_instances(&app).await
}

#[tauri::command]
async fn import_instances(app: tauri::AppHandle, json: String) -> Result<Vec<instances::InstanceConfig>, String> {
    instances::import_instances(&app, json).await
}

#[tauri::command]
async fn test_proxy(proxy_config: instances::ProxyConfig) -> Result<proxy_tester::ProxyTestResult, String> {
    proxy_tester::test_proxy(proxy_config).await
}

#[tauri::command]
async fn update_proxy_pool(app: tauri::AppHandle, id: String, pool: Vec<instances::ProxyConfig>, mode: Option<String>) -> Result<(), String> {
    instances::update_proxy_pool(&app, id, pool, mode).await
}

#[tauri::command]
async fn update_fingerprint_pool(app: tauri::AppHandle, id: String, pool: Vec<instances::FingerprintConfig>, mode: Option<String>) -> Result<(), String> {
    instances::update_fingerprint_pool(&app, id, pool, mode).await
}

#[tauri::command]
async fn get_session_info(app: tauri::AppHandle, id: String) -> Result<session_manager::SessionInfo, String> {
    session_manager::get_session_info(&app, id).await
}

#[tauri::command]
async fn clear_session_data(app: tauri::AppHandle, id: String, types: Vec<String>) -> Result<(), String> {
    session_manager::clear_session_data(&app, id, types).await
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
            update_instance_proxy,
            get_fingerprint_presets,
            get_running_instances,
            is_instance_running,
            stop_instance,
            confirm_close_action,
            rename_instance,
            update_instance_tags,
            update_instance_notes,
            export_instance,
            export_all_instances,
            import_instances,
            test_proxy,
            update_proxy_pool,
            update_fingerprint_pool,
            get_session_info,
            clear_session_data
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // If there are running instances, prevent close and ask the user
                if process_manager::has_running_instances() {
                    api.prevent_close();
                    let _ = window.emit("close-requested", ());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
