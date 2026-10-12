//! The app's shared folders, standing in for file dialogs.
//!
//! `Android/data/ai.storyteller.filmcraft/files/` on shared storage holds three folders:
//!
//! - `Import/`: copy media here (USB, a file manager); File ▸ Import brings in every file whose
//!   extension the dialog asks for.
//! - `Projects/`: Save / Save As write here; Open picks the most recently changed project.
//! - `Exports/`: exports and exported frames.
//!
//! Nothing here needs a storage permission. Every operation is fallible and only logged: a folder
//! that cannot be created or read means an empty pick, never a crash.

use std::path::{Path, PathBuf};

use filmcraft_ui_egui::FilmcraftApp;

#[derive(Clone, Debug)]
pub struct Folders {
    /// The app's external files dir; `None` when shared storage is unavailable.
    root: Option<PathBuf>,
}

impl Folders {
    pub const IMPORT: &'static str = "Import";
    pub const PROJECTS: &'static str = "Projects";
    pub const EXPORTS: &'static str = "Exports";

    /// Creates the three folders under `root` (the app's external files dir).
    pub fn new(root: Option<PathBuf>) -> Self {
        if let Some(r) = &root {
            for name in [Self::IMPORT, Self::PROJECTS, Self::EXPORTS] {
                if let Err(e) = std::fs::create_dir_all(r.join(name)) {
                    log::warn!("cannot create {}: {e}", r.join(name).display());
                }
            }
        }
        Self { root }
    }

    fn dir(&self, name: &str) -> Option<PathBuf> {
        self.root.as_ref().map(|r| r.join(name))
    }

    /// The status-bar line shown at start-up: where media go.
    pub fn welcome(&self) -> String {
        match self.dir(Self::IMPORT) {
            Some(d) => format!("Import: put media in {} (File ▸ Import adds them)", d.display()),
            None => "Shared storage is unavailable: importing media is not possible".to_string(),
        }
    }

    /// File-dialog hooks backed by the folders.
    pub fn install_hooks(&self, app: &mut FilmcraftApp) {
        let f = self.clone();
        app.hooks.pick_files = Some(Box::new(move |exts: &[&str]| f.dir(Self::IMPORT).map(|d| files_with(&d, exts)).unwrap_or_default()));
        let f = self.clone();
        app.hooks.pick_file_for_relink = Some(Box::new(move |exts: &[&str], _hint| f.dir(Self::IMPORT).and_then(|d| files_with(&d, exts).into_iter().next())));
        let f = self.clone();
        app.hooks.pick_save = Some(Box::new(move |name: &str| f.dir(Self::PROJECTS).map(|d| path_string(&d.join(file_name(name))))));
        let f = self.clone();
        app.hooks.pick_save_as =
            Some(Box::new(move |_filter: &str, _exts: &[&str], name: &str| f.dir(Self::EXPORTS).map(|d| path_string(&d.join(file_name(name))))));
        let f = self.clone();
        app.hooks.pick_open_project = Some(Box::new(move || f.dir(Self::PROJECTS).and_then(|d| newest(&files_with(&d, &["fcproj"])))));
        let f = self.clone();
        app.hooks.pick_open_file = Some(Box::new(move |_filter: &str, exts: &[&str]| f.dir(Self::IMPORT).and_then(|d| newest(&files_with(&d, exts)))));
        let f = self.clone();
        app.hooks.pick_folder = Some(Box::new(move || f.dir(Self::EXPORTS).map(|d| path_string(&d))));
        let f = self.clone();
        app.hooks.pick_folder_at = Some(Box::new(move |_dir: &str| f.dir(Self::EXPORTS).map(|d| path_string(&d))));
    }
}

/// The last component of a suggested name, which may be a full path; never empty.
fn file_name(suggested: &str) -> String {
    let name = Path::new(suggested).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if name.is_empty() { "Untitled".to_string() } else { name }
}

fn path_string(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Regular files in `dir` whose extension is one of `exts` (any case, with or without a leading
/// dot; no extensions: every file), sorted by name.
pub fn files_with(dir: &Path, exts: &[&str]) -> Vec<String> {
    let wanted: Vec<String> = exts.iter().map(|e| e.trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty() && e != "*").collect();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("cannot read {}: {e}", dir.display());
            return Vec::new();
        }
    };
    let mut out: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| wanted.is_empty() || p.extension().is_some_and(|x| wanted.contains(&x.to_string_lossy().to_ascii_lowercase())))
        .map(|p| path_string(&p))
        .collect();
    out.sort();
    out
}

/// The most recently modified of `paths`.
fn newest(paths: &[String]) -> Option<String> {
    paths.iter().max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok()).cloned()
}
