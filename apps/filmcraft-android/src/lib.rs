//! FilmCraft on Android.
//!
//! The same engine and egui UI as the desktop app, built as a `cdylib` that Android's
//! `NativeActivity` loads, run by eframe on Vulkan (GLES fallback). What the desktop shell gets
//! from its OS, this crate gets from Android:
//!
//! | Desktop | Android |
//! |---|---|
//! | data dir (auto-save, recovery, prefs, logs) | the app's internal storage |
//! | file dialogs (rfd) | the app's shared folder `Android/data/ai.storyteller.filmcraft/files/` ([`folders`]) |
//! | cpal audio output | the same cpal output, on AAudio ([`audio`], shared with the desktop app) |
//! | stderr / log file | logcat (tag `filmcraft`) |
//!
//! Android has no synchronous file dialog a native activity can show, so media are exchanged
//! through the app's own folder on shared storage, which a computer (USB) and most file managers
//! can reach without any storage permission: **Import** brings in every matching file from
//! `Import/`, saves and exports go to `Projects/` and `Exports/`.
//!
//! Everything here is Android-only; on other targets the crate is empty.
#![cfg(target_os = "android")]

#[path = "../../filmcraft/src/audio.rs"]
mod audio;
pub mod folders;

use std::path::PathBuf;

use filmcraft_engine::Session;
use filmcraft_engine::autosave::AutosaveConfig;
use filmcraft_ui_egui::FilmcraftApp;
use winit::platform::android::activity::AndroidApp;

/// Called by the NativeActivity glue (android-activity) on its own thread once the activity starts.
#[allow(unsafe_code)] // `no_mangle`: the glue finds the entry point by this exact symbol name.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("filmcraft"));
    log::info!("FilmCraft {} (android {})", env!("CARGO_PKG_VERSION"), std::env::consts::ARCH);

    let data_dir = app.internal_data_path();
    let folders = folders::Folders::new(app.external_data_path());
    // Panics anywhere go to <internal data>/Logs/crash-<day>.log; the UI pass and frame workers
    // catch them and keep running (see filmcraft_ui_egui::crash).
    filmcraft_ui_egui::crash::install(data_dir.as_ref().map(|d| d.join("Logs")));

    let options = eframe::NativeOptions { android_app: Some(app), ..Default::default() };
    let creator_dir = data_dir.clone();
    let result = eframe::run_native("FilmCraft", options, Box::new(move |cc| Ok(Box::new(create_app(cc, creator_dir, folders)))));
    if let Err(e) = result {
        let msg = format!("FilmCraft could not start its window: {e}");
        log::error!("{msg}");
        filmcraft_ui_egui::crash::record(&msg);
    }
}

fn create_app(cc: &eframe::CreationContext<'_>, data_dir: Option<PathBuf>, folders: folders::Folders) -> FilmcraftApp {
    let mut session = Session::default();
    if let Some(dir) = data_dir {
        if let Err(e) = session.start_autosave(AutosaveConfig::new(dir)) {
            log::warn!("auto-save and crash recovery unavailable: {e}");
        }
    }
    // A first start shows something to edit; later starts follow Settings ▸ General ▸ At Startup.
    if session.prefs.general.at_startup == "openMostRecent" {
        let recent = session.prefs.general.recent_projects.iter().find(|p| std::path::Path::new(p).exists()).cloned();
        let opened = recent.is_some_and(|p| match session.execute("file.open", serde_json::json!({"path": p})) {
            Ok(_) => true,
            Err(e) => {
                log::error!("{e}");
                false
            }
        });
        if !opened {
            let _ = session.execute("file.openDemoProject", serde_json::json!({}));
        }
    } else if session.prefs.general.at_startup != "emptyProject" {
        let _ = session.execute("file.openDemoProject", serde_json::json!({}));
    }

    let mut app = FilmcraftApp::new(session);
    if let Some(rs) = cc.wgpu_render_state.clone() {
        app.set_wgpu(rs);
    }
    app.audio = Some(Box::new(audio::CpalOut::new()));
    folders.install_hooks(&mut app);
    app.hooks.system_languages = Some(Box::new(|| sys_locale::get_locales().collect()));
    app.status(folders.welcome());
    app
}
