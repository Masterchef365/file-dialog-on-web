use std::{
    future::Future,
    io::Cursor,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use egui::{ahash::HashSet, github_link_file_line, Color32, RichText};
use egui_file_dialog::{Disks, FileDialog, FileSystem, Metadata};
use zip::ZipArchive;

pub struct TemplateApp {
    dialog: Option<FileDialog>,
    loaded_file: Arc<Mutex<Option<Vec<u8>>>>,
    error: Option<String>,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            dialog: None, //,
            loaded_file: Default::default(),
            error: None,
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(bytes) = self.loaded_file.lock().unwrap().take() {
            match ZipArchive::new(Cursor::new(bytes)) {
                Err(e) => self.error = Some(e.to_string()),
                Ok(zip) => {
                    let wrapper = Arc::new(ZipWrapper(Mutex::new(zip)));
                    let mut dialog = FileDialog::from_filesystem(wrapper);
                    dialog.pick_file();
                    self.dialog = Some(dialog);
                }
            }
        }

        if let Some(dialog) = &mut self.dialog {
            dialog.update(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load zip file").clicked() {
                let loaded_file = self.loaded_file.clone();
                let ctx = ui.ctx().clone();
                execute_async(async move {
                    if let Some(file) = rfd::AsyncFileDialog::new().add_filter("zip", &["zip"]).pick_file().await {
                        let bytes = file.read().await;
                        *loaded_file.lock().unwrap() = Some(bytes);
                        ctx.request_repaint();
                    }
                });
            }

            if let Some(err) = &self.error {
                ui.label(RichText::new(err).color(Color32::RED));
            }

            ui.hyperlink("https://github.com/Masterchef365/file-dialog-on-web");
        });
    }
}

pub struct ZipWrapper(Mutex<ZipArchive<Cursor<Vec<u8>>>>);

impl FileSystem for ZipWrapper {
    fn is_dir(&self, path: &std::path::Path) -> bool {
        self.0
            .lock()
            .unwrap()
            .by_name(path.to_str().unwrap())
            .is_err()
    }

    fn is_file(&self, path: &std::path::Path) -> bool {
        self.0
            .lock()
            .unwrap()
            .by_name(path.to_str().unwrap())
            .is_ok()
    }

    fn metadata(&self, path: &std::path::Path) -> std::io::Result<Metadata> {
        Ok(Metadata::default())
    }

    fn read_dir(&self, base: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .file_names()
            .map(|fname| PathBuf::from(fname))
            .filter(|path| path.starts_with(base))
            .filter_map(|path| {
                path.strip_prefix(base)
                    .ok()
                    .and_then(|remainder| remainder.components().next().map(|part| base.join(part)))
            })
            .collect::<HashSet<PathBuf>>()
            .into_iter()
            .collect())
    }

    fn user_dirs(&self, canonicalize_paths: bool) -> Option<egui_file_dialog::UserDirectories> {
        None
    }

    fn get_disks(&self, canonicalize_paths: bool) -> egui_file_dialog::Disks {
        Disks { disks: vec![] }
    }

    fn create_dir(&self, path: &std::path::Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported",
        ))
    }

    fn is_path_hidden(&self, path: &std::path::Path) -> bool {
        false
    }

    fn load_text_file_preview(
        &self,
        path: &std::path::Path,
        max_chars: usize,
    ) -> std::io::Result<String> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported",
        ))
    }

    fn current_dir(&self) -> std::io::Result<PathBuf> {
        Ok("".into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_async<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute_async<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
