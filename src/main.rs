#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf, sync::Mutex};
use tauri::Manager;

#[derive(Clone, Serialize, Deserialize)]
struct Profile {
    id: String,
    name: String,
}

fn app_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

fn profile_registry(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_dir(app)?.join("profiles.json"))
}

fn replace_file(temp: &PathBuf, path: &PathBuf, backup: &PathBuf) -> Result<(), String> {
    if path.exists() {
        fs::copy(path, backup).map_err(|e| e.to_string())?;
    }
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    if let Err(error) = fs::rename(temp, path) {
        #[cfg(windows)]
        if backup.exists() { let _ = fs::copy(backup, path); }
        return Err(error.to_string());
    }
    Ok(())
}

fn read_profiles(app: &tauri::AppHandle) -> Result<Vec<Profile>, String> {
    let path = profile_registry(app)?;
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("Registro de usuarios inválido: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Keep the original single-user vault accessible without moving or replacing it.
            let legacy = app_dir(app)?.join("workspace.umbral");
            if legacy.exists() {
                let profiles = vec![Profile { id: "default".into(), name: "Espacio anterior".into() }];
                write_profiles(app, &profiles)?;
                Ok(profiles)
            } else {
                Ok(Vec::new())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn write_profiles(app: &tauri::AppHandle, profiles: &[Profile]) -> Result<(), String> {
    let path = profile_registry(app)?;
    let temp = path.with_extension("json.tmp");
    let text = serde_json::to_vec_pretty(profiles).map_err(|e| e.to_string())?;
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp).map_err(|e| e.to_string())?;
    file.write_all(&text).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    replace_file(&temp, &path, &path.with_extension("previous.json"))
}

fn valid_profile_id(id: &str) -> bool {
    id == "default" || (id.len() == 36 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'))
}

fn vault_path(app: &tauri::AppHandle, profile_id: &str) -> Result<PathBuf, String> {
    if !valid_profile_id(profile_id) {
        return Err("Perfil inválido".into());
    }
    let dir = app_dir(app)?;
    Ok(if profile_id == "default" {
        dir.join("workspace.umbral")
    } else {
        dir.join(format!("workspace-{profile_id}.umbral"))
    })
}

#[tauri::command]
fn list_profiles(app: tauri::AppHandle, state: tauri::State<'_, Mutex<()>>) -> Result<Vec<Profile>, String> {
    let _guard = state.lock().map_err(|e| e.to_string())?;
    read_profiles(&app)
}

#[tauri::command]
fn create_profile(app: tauri::AppHandle, state: tauri::State<'_, Mutex<()>>, name: String) -> Result<Profile, String> {
    let _guard = state.lock().map_err(|e| e.to_string())?;
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 50 {
        return Err("El nombre debe tener entre 1 y 50 caracteres".into());
    }
    let mut profiles = read_profiles(&app)?;
    if profiles.iter().any(|p| p.name.eq_ignore_ascii_case(name)) {
        return Err("Ya existe un usuario con ese nombre".into());
    }
    let profile = Profile { id: uuid::Uuid::new_v4().to_string(), name: name.to_string() };
    profiles.push(profile.clone());
    write_profiles(&app, &profiles)?;
    Ok(profile)
}

#[tauri::command]
fn load_vault(app: tauri::AppHandle, state: tauri::State<'_, Mutex<()>>, profile_id: String) -> Result<Option<String>, String> {
    let _guard = state.lock().map_err(|e| e.to_string())?;
    if !read_profiles(&app)?.iter().any(|p| p.id == profile_id) {
        return Err("Usuario no encontrado".into());
    }
    match fs::read_to_string(vault_path(&app, &profile_id)?) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_vault(app: tauri::AppHandle, state: tauri::State<'_, Mutex<()>>, profile_id: String, content: String) -> Result<(), String> {
    let _guard = state.lock().map_err(|e| e.to_string())?;
    if !read_profiles(&app)?.iter().any(|p| p.id == profile_id) {
        return Err("Usuario no encontrado".into());
    }
    if content.len() > 40_000_000 { return Err("Espacio demasiado grande".into()); }
    let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if parsed["format"] != "umbral-v1" { return Err("Formato incorrecto".into()); }
    let path = vault_path(&app, &profile_id)?;
    let temp = path.with_extension("tmp");
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    replace_file(&temp, &path, &path.with_extension("previous.umbral"))?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(()))
        .invoke_handler(tauri::generate_handler![list_profiles, create_profile, load_vault, save_vault])
        .run(tauri::generate_context!())
        .expect("No se pudo iniciar Umbral");
}
