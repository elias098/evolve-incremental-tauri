// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicI32, Ordering},
};

use chrono::Utc;
use tauri::State;
use url::Url;

struct ExternalLabel(AtomicI32);

fn main() {
    tauri::Builder::default()
        .manage(ExternalLabel(AtomicI32::new(0)))
        .invoke_handler(tauri::generate_handler![
            save,
            save_script_settings,
            log_action,
            clear_log,
            open_window
        ])
        .setup(|app| {
            let documents = tauri::api::path::document_dir().unwrap();
            let scripts = documents.join("evolve-incremental/scripts");

            let scripts_text = fs::read_dir(scripts)
                .unwrap()
                .map(|file| file.as_ref().unwrap().path())
                .filter(|file| {
                    if let Some(extension) = file.extension() {
                        extension == "js"
                    } else {
                        false
                    }
                })
                .map(|path| {
                    format!(
                        "(function () {{ {} }})();",
                        fs::read_to_string(path).unwrap()
                    )
                })
                .collect::<String>();

            tauri::WindowBuilder::new(
                app,
                "evolve-incremental",
                tauri::WindowUrl::External(
                    Url::parse("https://pmotschmann.github.io/Evolve/").unwrap(),
                ),
            )
            .title("evolve-incremental")
            .fullscreen(true)
            .additional_browser_args(
                "--disable-raf-throttling --disable-backgrounding-occluded-windows --disable-background-timer-throttling --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
            )
            .initialization_script(&format!(
                r#"
                {}

                setTimeout(() => {{ {} }}, 1000);
                "#,
                include_str!("initialization.js"),
                scripts_text
            ))
            .build()?;
            // .open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}

fn folder_path() -> PathBuf {
    tauri::api::path::document_dir()
        .unwrap()
        .join("evolve-incremental")
}

#[tauri::command]
fn save(file_name: &str, save_data: &str) -> Result<(), String> {
    let save_path = folder_path().join(format!("saves/{}.txt", file_name));

    fs::write(save_path, save_data).map_err(|err| format!("Failed to save the game: {}", err))
}

#[tauri::command]
fn save_script_settings(script_settings: &str) -> Result<(), String> {
    let script_settings_path = folder_path().join(format!(
        "script-settings/script-settings-{}.txt",
        Utc::now()
    ));
    let latest_script_settings_path = folder_path().join("script-settings/latest.txt");

    if !latest_script_settings_path.exists()
        || script_settings != fs::read_to_string(&latest_script_settings_path).unwrap()
    {
        match (
            fs::write(&latest_script_settings_path, script_settings).map_err(|err| {
                format!(
                    "Failed to save script settings to {}: {}",
                    latest_script_settings_path.to_string_lossy(),
                    err
                )
            }),
            fs::write(&script_settings_path, script_settings).map_err(|err| {
                format!(
                    "Failed to save script settings to {}: {}",
                    script_settings_path.to_string_lossy(),
                    err
                )
            }),
        ) {
            (Err(err1), Err(err2)) => Err(format!("{}\n{}", err1, err2)),
            (Err(err), _) | (_, Err(err)) => Err(err),
            (Ok(_), Ok(_)) => Ok(()),
        }
    } else {
        Ok(())
    }
}

#[tauri::command]
fn log_action(file_name: &str, action: &str) -> Result<(), String> {
    let log_path = folder_path().join(format!("logs/{}.txt", file_name));

    fs::File::options()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut file| writeln!(file, "{}", action))
        .map_err(|err| format!("Failed to log action: {}", err))
}

#[tauri::command]
fn clear_log(file_name: &str) -> Result<(), String> {
    let log_path = folder_path().join(format!("logs/{}.txt", file_name));

    fs::write(log_path, "").map_err(|err| format!("Failed to log action: {}", err))
}

#[tauri::command]
async fn open_window(
    handle: tauri::AppHandle,
    id: State<'_, ExternalLabel>,
    href: &str,
) -> Result<(), String> {
    let label = id.0.fetch_add(1, Ordering::Relaxed).to_string();
    tauri::WindowBuilder::new(
        &handle,
        label,
        tauri::WindowUrl::External(Url::parse(href).map_err(|err| err.to_string())?),
    )
    .title(href)
    .fullscreen(true)
    // .initialization_script(
    //     r#"
    //     document.addEventListener(
    //         "click",
    //         (event) => {
    //           if (
    //             event.target instanceof HTMLAnchorElement &&
    //             event.target.tagName !== "a" &&
    //             event.target.href !== ""
    //           ) {
    //             event.stopPropagation();
    //             invoke("open_window", { href: event.target.href }).catch((err) =>
    //               message(err)
    //             );
    //           }
    //         },
    //         true
    //       );
    //     "#,
    // )
    .build()
    .map_err(|err| err.to_string())?;
    Ok(())
}
